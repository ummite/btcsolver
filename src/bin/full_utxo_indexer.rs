use anyhow::Result;
use bitcoin::blockdata::block::Block;
use bitcoin::key::{CompressedPublicKey, PrivateKey, UntweakedPublicKey};
use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::{Address, Network, ScriptBuf, Txid};
use bip39::Mnemonic;
use bitcoin_hashes::Hash;
use clap::{Parser, Subcommand};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::str::FromStr;
use std::time::Instant;

type OutPoint = ([u8; 32], u32);

// redb table definitions
const META_TABLE: TableDefinition<&str, u64> = TableDefinition::new("meta");
const UTXO_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("utxos");
const SCRIPT_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("by_script");

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the full UTXO index from block files
    Build {
        #[arg(short, long, default_value = "Y:\\Bitcoin\\blocks")]
        blocks_dir: String,
        #[arg(short, long, default_value = "utxo-index.redb")]
        db_path: String,
        #[arg(short, long, default_value = "b3a2cd522df3a049")]
        obf_key: String,
        /// Start from a specific file number (resume)
        #[arg(short, long, default_value = "0")]
        start_file: u32,
    },
    /// Query balance for any private key from the index
    Query {
        #[arg(short, long)]
        key: String,
        #[arg(short, long, default_value = "utxo-index.redb")]
        db_path: String,
    },
    /// Show index statistics
    Stats {
        #[arg(short, long, default_value = "utxo-index.redb")]
        db_path: String,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Build { blocks_dir, db_path, obf_key, start_file } => {
            cmd_build(&blocks_dir, &db_path, &obf_key, start_file)
        }
        Commands::Query { key, db_path } => cmd_query(&key, &db_path),
        Commands::Stats { db_path } => cmd_stats(&db_path),
    }
}

// ─── Build ────────────────────────────────────────────────────────────────

fn cmd_build(blocks_dir: &str, db_path: &str, obf_key_hex: &str, start_file: u32) -> Result<()> {
    let obf_key = parse_obf_key(obf_key_hex)?;
    let block_files = collect_block_files(blocks_dir)?;
    let total_files = block_files.len();

    let db = Database::create(db_path)?;

    // Check existing progress
    let existing_progress = {
        let read_tx = db.begin_read()?;
        let meta = read_tx.open_table(META_TABLE)?;
        meta.get("last_file")?.map(|v| v.value() as u32)
    };

    let skip_until = existing_progress.unwrap_or(0).max(start_file);
    let files_to_process = total_files.saturating_sub(skip_until as usize);

    println!("UTXO Indexer - Build");
    println!("  Blocks dir: {}", blocks_dir);
    println!("  Database: {}", db_path);
    println!("  Files: {} (skip {}-{}, process {})", total_files, 0, skip_until, files_to_process);

    if files_to_process <= 0 {
        println!("  Already up to date!");
        return Ok(());
    }

    let mut utxo_set: HashMap<OutPoint, (Vec<u8>, u64)> = HashMap::new();
    let mut script_index: HashMap<Vec<u8>, Vec<([u8; 32], u32, u64)>> = HashMap::new();

    // Reload from DB if resuming
    if existing_progress.is_some() {
        println!("  Reloading existing UTXO set...");
        let read_tx = db.begin_read()?;
        let table = read_tx.open_table(UTXO_TABLE)?;
        for entry in table.iter()? {
            let (k, v) = entry?;
            let kbuf = k.value();
            let vbuf = v.value();
            if kbuf.len() == 36 && vbuf.len() >= 10 {
                let txid: [u8; 32] = kbuf[..32].try_into().unwrap();
                let vout = u32::from_le_bytes(kbuf[32..36].try_into().unwrap());
                let script_len = u16::from_le_bytes(vbuf[..2].try_into().unwrap()) as usize;
                let script = vbuf[2..2 + script_len].to_vec();
                let value = u64::from_le_bytes(vbuf[2 + script_len..2 + script_len + 8].try_into().unwrap());
                utxo_set.insert((txid, vout), (script.clone(), value));
                script_index.entry(script).or_default().push((txid, vout, value));
            }
        }
        println!("  Loaded {} UTXOs", utxo_set.len());
    }

    let start = Instant::now();
    let mut total_blocks = 0u64;
    let mut total_txs = 0u64;

    for (file_idx, block_file) in block_files.iter().enumerate() {
        if file_idx < skip_until as usize {
            continue;
        }

        if file_idx % 50 == 0 {
            let elapsed = start.elapsed();
            let processed = file_idx.saturating_sub(skip_until as usize);
            let eta = if processed > 0 {
                let per_file = elapsed.as_secs_f64() / processed as f64;
                let remaining = (total_files - file_idx) as f64 * per_file;
                format!("{:.0}s remaining", remaining)
            } else {
                "calculating...".to_string()
            };
            eprintln!("[{}/{}] {} blocks | {} txs | {} UTXOs | {:?} | {}",
                file_idx, total_files, total_blocks, total_txs, utxo_set.len(),
                start.elapsed(), eta);
        }

        if file_idx > skip_until as usize && (file_idx - skip_until as usize) % 100 == 0 {
            eprintln!("  Saving checkpoint...");
            save_checkpoint(&db, file_idx as u32, &utxo_set, &script_index)?;
        }

        scan_block_file_full(&block_file, obf_key, &mut utxo_set, &mut script_index,
            &mut total_blocks, &mut total_txs)?;
    }

    eprintln!("  Saving final index...");
    save_checkpoint(&db, total_files as u32, &utxo_set, &script_index)?;

    let elapsed = start.elapsed();
    println!("\nDone in {:?}!", elapsed);
    println!("  Blocks: {} | Transactions: {} | UTXOs: {}",
        total_blocks, total_txs, utxo_set.len());

    Ok(())
}

fn save_checkpoint(
    db: &Database,
    last_file: u32,
    utxo_set: &HashMap<OutPoint, (Vec<u8>, u64)>,
    script_index: &HashMap<Vec<u8>, Vec<([u8; 32], u32, u64)>>,
) -> Result<()> {
    let write_tx = db.begin_write()?;

    {
        let mut meta = write_tx.open_table(META_TABLE)?;
        meta.insert("last_file", last_file as u64)?;
        meta.insert("utxo_count", utxo_set.len() as u64)?;
    }

    {
        let mut table = write_tx.open_table(UTXO_TABLE)?;
        for ((txid, vout), (script, value)) in utxo_set {
            let mut key = Vec::with_capacity(36);
            key.extend_from_slice(txid);
            key.extend_from_slice(&vout.to_le_bytes());

            let mut val = Vec::with_capacity(2 + script.len() + 8);
            val.extend_from_slice(&(script.len() as u16).to_le_bytes());
            val.extend_from_slice(script);
            val.extend_from_slice(&value.to_le_bytes());

            table.insert(&*key, &*val)?;
        }
    }

    {
        let mut table = write_tx.open_table(SCRIPT_TABLE)?;
        for (script, entries) in script_index {
            let mut val = Vec::with_capacity(4 + 44 * entries.len());
            val.extend_from_slice(&(entries.len() as u32).to_le_bytes());
            for (txid, vout, value) in entries {
                val.extend_from_slice(txid);
                val.extend_from_slice(&vout.to_le_bytes());
                val.extend_from_slice(&value.to_le_bytes());
            }
            table.insert(script.as_slice(), &*val)?;
        }
    }

    write_tx.commit()?;
    Ok(())
}

fn scan_block_file_full(
    path: &Path,
    obf_key: [u8; 8],
    utxo_set: &mut HashMap<OutPoint, (Vec<u8>, u64)>,
    script_index: &mut HashMap<Vec<u8>, Vec<([u8; 32], u32, u64)>>,
    total_blocks: &mut u64,
    total_txs: &mut u64,
) -> Result<()> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    for (i, b) in buf.iter_mut().enumerate() {
        *b ^= obf_key[i % 8];
    }

    let data = buf;
    let mut offset = 0usize;

    while offset + 8 <= data.len() {
        let magic = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]);
        if magic != 0xd9b4bef9 {
            offset += 1;
            continue;
        }

        let block_size = u32::from_le_bytes([
            data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
        ]) as usize;

        if offset + 8 + block_size > data.len() {
            break;
        }

        let block_data = &data[offset + 8..offset + 8 + block_size];

        if let Ok(block) = bitcoin::consensus::encode::deserialize::<Block>(block_data) {
            *total_blocks += 1;

            for tx in block.txdata.iter() {
                *total_txs += 1;
                let txid = tx.compute_txid();
                let txid_bytes = txid.to_byte_array();

                for input in tx.input.iter() {
                    let prev_txid: [u8; 32] = input.previous_output.txid.to_byte_array();
                    let prev_vout = input.previous_output.vout;
                    if let Some((script, _value)) = utxo_set.remove(&(prev_txid, prev_vout)) {
                        if let Some(entries) = script_index.get_mut(&script) {
                            entries.retain(|(t, v, _)| (*t, *v) != (prev_txid, prev_vout));
                            if entries.is_empty() {
                                script_index.remove(&script);
                            }
                        }
                    }
                }

                for (vout_idx, txout) in tx.output.iter().enumerate() {
                    let script = txout.script_pubkey.as_bytes().to_vec();
                    let value = txout.value.to_sat();
                    utxo_set.insert((txid_bytes, vout_idx as u32), (script.clone(), value));
                    script_index.entry(script).or_default().push((txid_bytes, vout_idx as u32, value));
                }
            }
        }

        offset += 8 + block_size;
    }

    Ok(())
}

// ─── Query ────────────────────────────────────────────────────────────────

fn cmd_query(key_input: &str, db_path: &str) -> Result<()> {
    let scripts = derive_scripts(key_input)?;

    let db = Database::open(db_path)?;
    let read_tx = db.begin_read()?;
    let by_script = read_tx.open_table(SCRIPT_TABLE)?;

    let last_file = {
        let meta = read_tx.open_table(META_TABLE)?;
        meta.get("last_file")?.map(|v| v.value())
    };

    println!("UTXO Query");
    println!("  Database: {} (file {})", db_path, last_file.unwrap_or(0));
    println!("  Key: {}\n", format_key_display(key_input));

    let mut any_balance = false;

    for (script, addr_str, addr_type) in &scripts {
        let script_bytes = script.as_bytes().to_vec();
        let mut total_sats = 0u64;
        let mut utxo_count = 0u32;
        let mut details: Vec<(String, u32, u64)> = Vec::new();

        if let Ok(Some(val)) = by_script.get(script_bytes.as_slice()) {
            let vbuf = val.value();
            if vbuf.len() >= 4 {
                let count = u32::from_le_bytes(vbuf[..4].try_into().unwrap());
                let mut pos = 4usize;
                for _ in 0..count {
                    if pos + 44 > vbuf.len() { break; }
                    let txid_bytes: [u8; 32] = vbuf[pos..pos+32].try_into().unwrap();
                    let vout = u32::from_le_bytes(vbuf[pos+32..pos+36].try_into().unwrap());
                    let value = u64::from_le_bytes(vbuf[pos+36..pos+44].try_into().unwrap());
                    pos += 44;

                    total_sats += value;
                    utxo_count += 1;
                    details.push((Txid::from_byte_array(txid_bytes).to_string(), vout, value));
                }
            }
        }

        println!("  [{}] {}", addr_type, addr_str);
        if utxo_count > 0 {
            let btc = total_sats as f64 / 100_000_000.0;
            println!("    OK {} UTXO(s) | {} sats ({:.8} BTC)", utxo_count, total_sats, btc);
            for (txid, vout, val) in &details {
                println!("      - {}#{} = {} sats ({:.8} BTC)",
                    txid, vout, val, *val as f64 / 100_000_000.0);
            }
            any_balance = true;
        } else {
            println!("    -> 0 BTC");
        }
    }

    if !any_balance {
        println!("\n  No balance found.");
    }

    Ok(())
}

// ─── Stats ────────────────────────────────────────────────────────────────

fn cmd_stats(db_path: &str) -> Result<()> {
    let db = Database::open(db_path)?;
    let read_tx = db.begin_read()?;
    let meta = read_tx.open_table(META_TABLE)?;
    let last_file = meta.get("last_file")?.map(|v| v.value());
    let utxo_count = meta.get("utxo_count")?.map(|v| v.value());

    println!("UTXO Index Statistics");
    println!("  Database: {}", db_path);
    println!("  Last file indexed: {:?}", last_file);
    println!("  UTXOs in index: {:?}", utxo_count);
    if let Ok(m) = std::fs::metadata(db_path) {
        println!("  DB size: {:.1} MB", m.len() as f64 / 1_048_576.0);
    }

    Ok(())
}

// ─── Utilities ────────────────────────────────────────────────────────────

fn collect_block_files(blocks_dir: &str) -> Result<Vec<std::path::PathBuf>> {
    let blocks_path = Path::new(blocks_dir);
    let mut block_files: Vec<_> = std::fs::read_dir(blocks_path)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let binding = e.file_name();
            let name = binding.to_string_lossy();
            name.starts_with("blk") && name.ends_with(".dat")
        })
        .map(|e| e.path())
        .collect();
    block_files.sort();
    Ok(block_files)
}

fn derive_scripts(key_input: &str) -> Result<Vec<(ScriptBuf, String, String)>> {
    let network = Network::Bitcoin;
    let secp = Secp256k1::<All>::new();
    let mut results = Vec::new();

    let (pk, pubkey) = parse_key(key_input, network, &secp)?;
    let compressed = CompressedPublicKey::from_private_key(&secp, &pk)?;
    let xonly: UntweakedPublicKey = compressed.into();

    let p2pkh = Address::p2pkh(pubkey, network);
    let p2wpkh = Address::p2wpkh(&compressed, network);
    let p2sh_wpkh = Address::p2shwpkh(&compressed, network);
    let p2tr = Address::p2tr(&secp, xonly, None, network);

    results.push((p2pkh.script_pubkey(), p2pkh.to_string(), "legacy".to_string()));
    results.push((p2wpkh.script_pubkey(), p2wpkh.to_string(), "segwit".to_string()));
    results.push((p2sh_wpkh.script_pubkey(), p2sh_wpkh.to_string(), "wrapped".to_string()));
    results.push((p2tr.script_pubkey(), p2tr.to_string(), "taproot".to_string()));

    Ok(results)
}

fn parse_key(input: &str, network: Network, secp: &Secp256k1<All>) -> Result<(PrivateKey, bitcoin::key::PublicKey)> {
    if let Ok(pk) = PrivateKey::from_str(input) {
        let pk = PrivateKey { inner: pk.inner, network: network.into(), compressed: true };
        return Ok((pk, pk.public_key(secp)));
    }
    if let Ok(bytes) = hex::decode(input) {
        if bytes.len() == 32 {
            let inner = bitcoin::secp256k1::SecretKey::from_slice(&bytes)?;
            let pk = PrivateKey { inner, network: network.into(), compressed: true };
            return Ok((pk, pk.public_key(secp)));
        }
    }
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.len() >= 12 {
        let phrase = input.trim();
        if let Ok(mnemonic) = Mnemonic::parse(phrase) {
            if mnemonic.words().count() >= 12 {
                let seed: [u8; 64] = {
                    let mut out = [0u8; 64];
                    pbkdf2::pbkdf2_hmac::<sha2::Sha512>(phrase.as_bytes(), b"mnemonic", 2048, &mut out);
                    out
                };
                let xpriv = Xpriv::new_master(network, &seed)?;
                let path = DerivationPath::from_str("m/44'/0'/0'/0/0")?;
                let derived = xpriv.derive_priv(secp, &path)?;
                let pk = PrivateKey { inner: derived.private_key, network: network.into(), compressed: true };
                return Ok((pk, pk.public_key(secp)));
            }
        }
    }
    anyhow::bail!("Cannot parse key. Use WIF, hex (32 bytes), or BIP39 (12+ words)");
}

fn format_key_display(key: &str) -> String {
    let words: Vec<&str> = key.split_whitespace().collect();
    if words.len() > 4 {
        let first: Vec<&str> = words.iter().take(3).copied().collect();
        format!("{} ... ({} words)", first.join(" "), words.len())
    } else {
        let s = key.to_string();
        if s.len() > 20 { format!("{}...{}", &s[..10], &s[s.len()-6..]) } else { s }
    }
}

fn parse_obf_key(hex_str: &str) -> Result<[u8; 8]> {
    let bytes = hex::decode(hex_str)?;
    anyhow::ensure!(bytes.len() == 8);
    Ok(bytes.try_into().unwrap())
}
