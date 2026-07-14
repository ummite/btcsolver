use anyhow::Result;
use bitcoin::blockdata::block::Block;
use bitcoin_hashes::Hash;
use bitcoin::key::{CompressedPublicKey, PrivateKey, UntweakedPublicKey};
use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::{Address, Network, ScriptBuf, Txid};
use bip39::Mnemonic;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::str::FromStr;
use std::time::Instant;

type ScriptKey = Vec<u8>;
type UtxoEntry = (Txid, u32, u64);

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check balance from UTXO cache file
    Cache {
        #[arg(short, long, default_value = "utxo-cache.bin")]
        cache: String,
        #[arg(short, long)]
        key: String,
    },
    /// Scan blockchain directly for a single key
    Scan {
        #[arg(short, long)]
        key: String,
        #[arg(short, long, default_value = "Y:\\Bitcoin\\blocks")]
        blocks_dir: String,
        #[arg(short, long, default_value = "b3a2cd522df3a049")]
        obf_key: String,
    },
    /// Build a UTXO cache for custom keys
    Build {
        #[arg(short, long, num_args = 1..)]
        keys: Vec<String>,
        #[arg(short, long, default_value = "utxo-cache.bin")]
        output: String,
        #[arg(short, long, default_value = "Y:\\Bitcoin\\blocks")]
        blocks_dir: String,
        #[arg(short, long, default_value = "b3a2cd522df3a049")]
        obf_key: String,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Cache { cache, key } => cmd_cache(&cache, &key),
        Commands::Scan { key, blocks_dir, obf_key } => cmd_scan(&key, &blocks_dir, &obf_key),
        Commands::Build { keys, output, blocks_dir, obf_key } => cmd_build(&keys, &output, &blocks_dir, &obf_key),
    }
}

// ─── Cache mode ──────────────────────────────────────────────────────────

fn cmd_cache(cache_path: &str, key_input: &str) -> Result<()> {
    let cache = load_cache(cache_path)?;
    let scripts = derive_scripts(key_input)?;

    println!("Cache: {} ({} blocks, {} scripts indexed)",
        cache_path, cache.total_blocks, cache.scripts.len());
    println!("Key: {}\n", format_key_display(key_input));

    let mut found = false;
    for (script, addr_str, addr_type) in &scripts {
        if let Some(entries) = cache.utxos.get(script.as_bytes()) {
            found = true;
            let total_sats: u64 = entries.iter().map(|e| e.2).sum();
            let btc = total_sats as f64 / 100_000_000.0;
            println!("  [{}] {}", addr_type, addr_str);
            println!("    OK {} UTXO(s) | {} sats ({:.8} BTC)", entries.len(), total_sats, btc);
            for (txid, vout, val) in entries {
                println!("      - {}#{} = {} sats ({:.8} BTC)",
                    txid, vout, val, *val as f64 / 100_000_000.0);
            }
        } else {
            println!("  [{}] {} -> 0 BTC", addr_type, addr_str);
        }
    }
    if !found {
        println!("\n  No balance found in cache.");
    }
    Ok(())
}

// ─── Scan mode ───────────────────────────────────────────────────────────

fn cmd_scan(key_input: &str, blocks_dir: &str, obf_key_hex: &str) -> Result<()> {
    let scripts = derive_scripts(key_input)?;
    let target_set: HashMap<ScriptKey, String> = scripts
        .iter()
        .map(|(s, addr, at)| (s.as_bytes().to_vec(), format!("[{}] {}", at, addr)))
        .collect();

    let obf_key = parse_obf_key(obf_key_hex)?;
    run_scan(&blocks_dir, obf_key, &target_set, &scripts, key_input)
}

// ─── Build mode ──────────────────────────────────────────────────────────

fn cmd_build(keys: &[String], output: &str, blocks_dir: &str, obf_key_hex: &str) -> Result<()> {
    let mut all_scripts: Vec<(ScriptBuf, String)> = Vec::new();

    for key_input in keys {
        let scripts = derive_scripts(key_input)?;
        for (script, addr_str, addr_type) in scripts {
            let label = format!("{} [{}] {}", format_key_display(key_input), addr_type, addr_str);
            all_scripts.push((script, label));
        }
    }

    let target_set: HashMap<ScriptKey, String> = all_scripts
        .iter()
        .map(|(s, l)| (s.as_bytes().to_vec(), l.clone()))
        .collect();

    let obf_key = parse_obf_key(obf_key_hex)?;

    let mut utxo_map: HashMap<ScriptKey, Vec<UtxoEntry>> = HashMap::new();
    let mut target_outpoints: HashMap<(Txid, u32), ScriptKey> = HashMap::new();
    let start = Instant::now();
    let mut total_blocks = 0u64;
    let mut total_txs = 0u64;

    let block_files = collect_block_files(blocks_dir)?;

    println!("Building cache for {} key(s) ({} addresses)", keys.len(), all_scripts.len());
    println!("Files: {}\n", block_files.len());

    for (file_idx, block_file) in block_files.iter().enumerate() {
        if file_idx % 50 == 0 {
            let elapsed = start.elapsed();
            let target_utxos: usize = utxo_map.values().map(|v| v.len()).sum();
            eprintln!("[{}/{}] {} blocks | {} txs | {} UTXOs | {:?} elapsed",
                file_idx, block_files.len(), total_blocks, total_txs, target_utxos, elapsed);
        }
        scan_block_file(&block_file, obf_key, &target_set,
            &mut utxo_map, &mut target_outpoints,
            &mut total_blocks, &mut total_txs)?;
    }

    let elapsed = start.elapsed();
    println!("\nScan done in {:?}!", elapsed);

    for key_input in keys {
        let scripts = derive_scripts(key_input)?;
        print_results(&scripts, &utxo_map);
    }

    println!("Saving cache to {}...", output);
    save_cache(output, &utxo_map, &all_scripts, total_blocks)?;
    println!("  Cache saved!");

    Ok(())
}

// ─── Shared scan runner ─────────────────────────────────────────────────

fn run_scan(
    blocks_dir: &str,
    obf_key: [u8; 8],
    target_set: &HashMap<ScriptKey, String>,
    scripts: &[(ScriptBuf, String, String)],
    key_input: &str,
) -> Result<()> {
    let mut utxo_map: HashMap<ScriptKey, Vec<UtxoEntry>> = HashMap::new();
    let mut target_outpoints: HashMap<(Txid, u32), ScriptKey> = HashMap::new();
    let start = Instant::now();
    let mut total_blocks = 0u64;
    let mut total_txs = 0u64;

    let block_files = collect_block_files(blocks_dir)?;

    println!("Direct scan for 1 key ({} addresses)", scripts.len());
    println!("Files: {} | Key: {}\n", block_files.len(), format_key_display(key_input));

    for (file_idx, block_file) in block_files.iter().enumerate() {
        if file_idx % 100 == 0 {
            let elapsed = start.elapsed();
            let target_utxos: usize = utxo_map.values().map(|v| v.len()).sum();
            eprintln!("[{}/{}] {} blocks | {} txs | {} UTXOs | {:?} elapsed",
                file_idx, block_files.len(), total_blocks, total_txs, target_utxos, elapsed);
        }
        scan_block_file(&block_file, obf_key, target_set,
            &mut utxo_map, &mut target_outpoints,
            &mut total_blocks, &mut total_txs)?;
    }

    let elapsed = start.elapsed();
    println!("\nScan done in {:?}!", elapsed);
    println!("Blocks: {} | Transactions: {}\n", total_blocks, total_txs);
    print_results(scripts, &utxo_map);
    Ok(())
}

// ─── Core block scanning ─────────────────────────────────────────────────

fn scan_block_file(
    path: &Path,
    obf_key: [u8; 8],
    target_set: &HashMap<ScriptKey, String>,
    utxo_map: &mut HashMap<ScriptKey, Vec<UtxoEntry>>,
    target_outpoints: &mut HashMap<(Txid, u32), ScriptKey>,
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

                for input in tx.input.iter() {
                    let prev = &input.previous_output;
                    if let Some(script_key) = target_outpoints.remove(&(prev.txid, prev.vout)) {
                        if let Some(utxos) = utxo_map.get_mut(&script_key) {
                            utxos.retain(|(tid, vout, _)| (*tid, *vout) != (prev.txid, prev.vout));
                            if utxos.is_empty() {
                                utxo_map.remove(&script_key);
                            }
                        }
                    }
                }

                for (vout_idx, txout) in tx.output.iter().enumerate() {
                    let script_bytes = txout.script_pubkey.as_bytes().to_vec();
                    if target_set.contains_key(&script_bytes) {
                        utxo_map
                            .entry(script_bytes.clone())
                            .or_default()
                            .push((txid, vout_idx as u32, txout.value.to_sat()));
                        target_outpoints.insert((txid, vout_idx as u32), script_bytes);
                    }
                }
            }
        }

        offset += 8 + block_size;
    }

    Ok(())
}

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

// ─── Key derivation ──────────────────────────────────────────────────────

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
    // Try WIF
    if let Ok(pk) = PrivateKey::from_str(input) {
        let pk = PrivateKey { inner: pk.inner, network: network.into(), compressed: true };
        let pubkey = pk.public_key(secp);
        return Ok((pk, pubkey));
    }

    // Try raw hex (32 bytes)
    if let Ok(bytes) = hex::decode(input) {
        if bytes.len() == 32 {
            let inner = bitcoin::secp256k1::SecretKey::from_slice(&bytes)?;
            let pk = PrivateKey { inner, network: network.into(), compressed: true };
            let pubkey = pk.public_key(secp);
            return Ok((pk, pubkey));
        }
    }

    // Try BIP39 mnemonic
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.len() >= 12 {
        let phrase = input.trim();
        if let Ok(mnemonic) = Mnemonic::parse(phrase) {
            if mnemonic.words().count() >= 12 {
                let seed: [u8; 64] = {
                    let salt = "mnemonic";
                    let mut out = [0u8; 64];
                    pbkdf2::pbkdf2_hmac::<sha2::Sha512>(
                        phrase.as_bytes(), salt.as_bytes(), 2048, &mut out,
                    );
                    out
                };

                let xpriv = Xpriv::new_master(network, &seed)?;
                let path = DerivationPath::from_str("m/44'/0'/0'/0/0")?;
                let derived = xpriv.derive_priv(secp, &path)?;

                let pk = PrivateKey {
                    inner: derived.private_key,
                    network: network.into(),
                    compressed: true,
                };
                let pubkey = pk.public_key(secp);
                return Ok((pk, pubkey));
            }
        }
    }

    anyhow::bail!("Cannot parse key '{}'. Use WIF, hex (32 bytes), or BIP39 phrase (12+ words)", input);
}

// ─── Display ─────────────────────────────────────────────────────────────

fn print_results(scripts: &[(ScriptBuf, String, String)], utxo_map: &HashMap<ScriptKey, Vec<UtxoEntry>>) {
    println!("\nRESULTS:");
    let mut any_balance = false;

    for (script, addr_str, addr_type) in scripts {
        if let Some(utxos) = utxo_map.get(script.as_bytes()) {
            if !utxos.is_empty() {
                let total_sats: u64 = utxos.iter().map(|(_, _, v)| *v).sum();
                let btc = total_sats as f64 / 100_000_000.0;
                println!("  OK [{}] {}", addr_type, addr_str);
                println!("     {} UTXO(s) | {} sats ({:.8} BTC)", utxos.len(), total_sats, btc);
                for (txid, vout, val) in utxos {
                    println!("       - {}#{} = {} sats ({:.8} BTC)",
                        txid, vout, val, *val as f64 / 100_000_000.0);
                }
                any_balance = true;
            }
        } else {
            println!("  X [{}] {} -> 0 BTC", addr_type, addr_str);
        }
    }

    if !any_balance {
        println!("\n  No balance found.");
    }
    println!();
}

fn format_key_display(key: &str) -> String {
    let words: Vec<&str> = key.split_whitespace().collect();
    if words.len() > 4 {
        let first: Vec<&str> = words.iter().take(3).copied().collect();
        format!("{} ... ({} words)", first.join(" "), words.len())
    } else {
        let s = key.to_string();
        if s.len() > 20 {
            format!("{}...{}", &s[..10], &s[s.len()-6..])
        } else {
            s
        }
    }
}

fn parse_obf_key(hex_str: &str) -> Result<[u8; 8]> {
    let bytes = hex::decode(hex_str)?;
    if bytes.len() != 8 {
        anyhow::bail!("Obfuscation key: expected 8 bytes, got {}", bytes.len());
    }
    Ok(bytes.try_into().unwrap())
}

// ─── Cache I/O ───────────────────────────────────────────────────────────

struct UtxoCache {
    total_blocks: u64,
    scripts: HashMap<ScriptKey, String>,
    utxos: HashMap<ScriptKey, Vec<UtxoEntry>>,
}

fn load_cache(path: &str) -> Result<UtxoCache> {
    let mut file = File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let mut pos = 0;

    let magic = read_u32(&data, &mut pos);
    if magic != 0x55545842 {
        anyhow::bail!("Invalid cache: bad magic 0x{:08X}", magic);
    }
    let version = read_u32(&data, &mut pos);
    if version != 1 {
        anyhow::bail!("Unsupported cache version: {}", version);
    }
    let tb_bytes: [u8; 8] = read_array(&data, &mut pos, 8)?;
    let total_blocks = u64::from_le_bytes(tb_bytes);

    let num_scripts = read_u32(&data, &mut pos) as usize;
    let mut scripts = HashMap::new();
    for _ in 0..num_scripts {
        let script_len = read_u16(&data, &mut pos) as usize;
        let script = read_bytes(&data, &mut pos, script_len).to_vec();
        let label_len = read_u32(&data, &mut pos) as usize;
        let label_bytes = read_bytes(&data, &mut pos, label_len);
        let label = String::from_utf8_lossy(label_bytes).to_string();
        scripts.insert(script, label);
    }

    let total_entries = read_u32(&data, &mut pos);
    let mut utxos: HashMap<ScriptKey, Vec<UtxoEntry>> = HashMap::new();
    let mut entries_read = 0u32;

    while entries_read < total_entries && pos < data.len() {
        let script_len = read_u16(&data, &mut pos) as usize;
        let script = read_bytes(&data, &mut pos, script_len).to_vec();
        let count = read_u32(&data, &mut pos) as usize;

        for _ in 0..count {
            let txid_bytes: [u8; 32] = read_array(&data, &mut pos, 32)?;
            let txid = Txid::from_byte_array(txid_bytes);
            let vout = read_u32(&data, &mut pos);
            let val_bytes: [u8; 8] = read_array(&data, &mut pos, 8)?;
            let value = u64::from_le_bytes(val_bytes);
            utxos.entry(script.clone()).or_default().push((txid, vout, value));
            entries_read += 1;
        }
    }

    Ok(UtxoCache { total_blocks, scripts, utxos })
}

fn save_cache(
    path: &str,
    utxo_map: &HashMap<ScriptKey, Vec<UtxoEntry>>,
    target_scripts: &[(ScriptBuf, String)],
    total_blocks: u64,
) -> Result<()> {
    let mut file = File::create(path)?;

    file.write_all(&0x55545842u32.to_le_bytes())?;
    file.write_all(&1u32.to_le_bytes())?;
    file.write_all(&total_blocks.to_le_bytes())?;

    let num_scripts = target_scripts.len() as u32;
    file.write_all(&num_scripts.to_le_bytes())?;
    for (script, label) in target_scripts {
        let sb = script.as_bytes();
        file.write_all(&(sb.len() as u16).to_le_bytes())?;
        file.write_all(sb)?;
        let lb = label.as_bytes();
        file.write_all(&(lb.len() as u32).to_le_bytes())?;
        file.write_all(lb)?;
    }

    let total_entries: u32 = utxo_map.values().map(|v| v.len() as u32).sum();
    file.write_all(&total_entries.to_le_bytes())?;
    for (script, entries) in utxo_map {
        file.write_all(&(script.len() as u16).to_le_bytes())?;
        file.write_all(script)?;
        file.write_all(&(entries.len() as u32).to_le_bytes())?;
        for (txid, vout, value) in entries {
            file.write_all(txid.as_ref())?;
            file.write_all(&vout.to_le_bytes())?;
            file.write_all(&value.to_le_bytes())?;
        }
    }

    Ok(())
}

fn read_u32(data: &[u8], pos: &mut usize) -> u32 {
    let val = u32::from_le_bytes([data[*pos], data[*pos+1], data[*pos+2], data[*pos+3]]);
    *pos += 4;
    val
}

fn read_u16(data: &[u8], pos: &mut usize) -> u16 {
    let val = u16::from_le_bytes([data[*pos], data[*pos+1]]);
    *pos += 2;
    val
}

fn read_bytes<'a>(data: &'a [u8], pos: &mut usize, len: usize) -> &'a [u8] {
    let slice = &data[*pos..*pos + len];
    *pos += len;
    slice
}

fn read_array<'a, const N: usize>(data: &'a [u8], pos: &mut usize, len: usize) -> Result<[u8; N]> {
    anyhow::ensure!(len == N, "Expected {} bytes, got {}", N, len);
    let arr: [u8; N] = read_bytes(data, pos, len).try_into().unwrap();
    Ok(arr)
}
