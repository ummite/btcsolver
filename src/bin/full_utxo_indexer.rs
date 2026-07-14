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
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read, Write};
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
        /// Checkpoint interval in files (default: 200)
        #[arg(long, default_value = "200")]
        checkpoint_interval: u32,
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
        Commands::Build { blocks_dir, db_path, obf_key, start_file, checkpoint_interval } => {
            cmd_build(&blocks_dir, &db_path, &obf_key, start_file, checkpoint_interval)
        }
        Commands::Query { key, db_path } => cmd_query(&key, &db_path),
        Commands::Stats { db_path } => cmd_stats(&db_path),
    }
}

// ─── Optimized UTXO Tracker ──────────────────────────────────────────────
//
// Key design: maintain BOTH utxo_set AND script_index incrementally.
// Spends use HashSet lookup (O(1)) instead of retain().
// At checkpoint: only filter dirty scripts in script_index (single pass).

struct UtxoTracker {
    /// Current unspent outputs: outpoint -> (script_bytes, value)
    utxo_set: std::collections::HashMap<OutPoint, (Vec<u8>, u64)>,

    /// Script -> list of UTXOs (may contain stale spent entries between checkpoints)
    script_index: std::collections::HashMap<Vec<u8>, Vec<([u8; 32], u32, u64)>>,

    /// Outpoints spent since last checkpoint
    spent_outpoints: HashSet<OutPoint>,

    /// Scripts affected by spends (need filtering at checkpoint)
    dirty_scripts: HashSet<Vec<u8>>,
}

impl UtxoTracker {
    fn new() -> Self {
        Self {
            utxo_set: std::collections::HashMap::new(),
            script_index: std::collections::HashMap::new(),
            spent_outpoints: HashSet::new(),
            dirty_scripts: HashSet::new(),
        }
    }

    /// Process a spend — O(1), no retain!
    fn spend(&mut self, prev_txid: [u8; 32], prev_vout: u32) {
        if let Some((script, _value)) = self.utxo_set.remove(&(prev_txid, prev_vout)) {
            self.spent_outpoints.insert((prev_txid, prev_vout));
            self.dirty_scripts.insert(script);
        }
    }

    /// Add a new output — O(1)
    fn add_output(&mut self, txid: [u8; 32], vout: u32, script: Vec<u8>, value: u64) {
        self.utxo_set.insert((txid, vout), (script.clone(), value));
        self.script_index.entry(script).or_default().push((txid, vout, value));
    }

    fn len(&self) -> usize {
        self.utxo_set.len()
    }

    /// Export script_index to a binary snapshot file (lock-free read)
    /// Format: [num_scripts:u32][script_len:u16][script_bytes][num_utxos:u32][txid:32][vout:u32][value:u64]...
    fn export_snapshot(&self, db_path: &str) -> Result<()> {
        let snapshot_path = db_path.replace(".redb", ".snapshot");
        let mut f = std::io::BufWriter::new(std::fs::File::create(&snapshot_path)?);

        // Write number of scripts
        let num_scripts = self.script_index.len() as u32;
        f.write_all(&num_scripts.to_le_bytes())?;

        for (script, entries) in &self.script_index {
            let script_len = script.len() as u16;
            f.write_all(&script_len.to_le_bytes())?;
            f.write_all(script)?;

            let num_entries = entries.len() as u32;
            f.write_all(&num_entries.to_le_bytes())?;

            for (txid, vout, value) in entries {
                f.write_all(txid)?;
                f.write_all(&vout.to_le_bytes())?;
                f.write_all(&value.to_le_bytes())?;
            }
        }
        f.flush()?;

        Ok(())
    }

    /// Save checkpoint — opens DB, writes, exports binary snapshot, releases lock
    fn save_checkpoint(&mut self, db_path: &str, last_file: u32) -> Result<()> {
        // Export binary snapshot for lock-free reading by brute-force
        self.export_snapshot(db_path)?;

        let db = if std::path::Path::new(db_path).exists() {
            Database::open(db_path)?
        } else {
            Database::create(db_path)?
        };
        // Filter dirty scripts — remove spent entries (single pass per dirty script)
        for script in &self.dirty_scripts {
            if let Some(entries) = self.script_index.get_mut(script) {
                entries.retain(|(t, v, _)| !self.spent_outpoints.contains(&(*t, *v)));
                if entries.is_empty() {
                    self.script_index.remove(script);
                }
            }
        }

        let write_tx = db.begin_write()?;

        // Meta
        {
            let mut meta = write_tx.open_table(META_TABLE)?;
            meta.insert("last_file", last_file as u64)?;
            meta.insert("utxo_count", self.utxo_set.len() as u64)?;
        }

        // UTXO table
        {
            let mut table = write_tx.open_table(UTXO_TABLE)?;
            for ((txid, vout), (script, value)) in &self.utxo_set {
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

        // Script table — use maintained index (already filtered)
        {
            let mut table = write_tx.open_table(SCRIPT_TABLE)?;
            for (script, entries) in &self.script_index {
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

        // Clear batch state
        self.spent_outpoints.clear();
        self.dirty_scripts.clear();

        Ok(())
    }

    /// Load from database for resume (opens and closes DB)
    fn load_from_db(db_path: &str) -> Result<Self> {
        let mut tracker = Self::new();
        let db = Database::open(db_path)?;
        let read_tx = db.begin_read()?;

        if let Ok(table) = read_tx.open_table(UTXO_TABLE) {
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
                    tracker.utxo_set.insert((txid, vout), (script.clone(), value));
                    tracker.script_index.entry(script).or_default()
                        .push((txid, vout, value));
                }
            }
        }

        Ok(tracker)
    }
}

fn cmd_build(blocks_dir: &str, db_path: &str, obf_key_hex: &str, start_file: u32, checkpoint_interval: u32) -> Result<()> {
    let obf_key = parse_obf_key(obf_key_hex)?;
    let block_files = collect_block_files(blocks_dir)?;
    let total_files = block_files.len();

    // Check existing progress (open DB briefly, then release lock)
    let existing_progress = if std::path::Path::new(db_path).exists() {
        if let Ok(db) = Database::open(db_path) {
            if let Ok(read_tx) = db.begin_read() {
                if let Ok(meta) = read_tx.open_table(META_TABLE) {
                    meta.get("last_file").ok().map(|v| v.map(|x| x.value() as u32).unwrap_or(0))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    // DB handle dropped here — lock released!

    let skip_until = existing_progress.unwrap_or(0).max(start_file);
    let files_to_process = total_files.saturating_sub(skip_until as usize);

    println!("UTXO Indexer - Build (v4 - lock-free between checkpoints)");
    println!("  Blocks dir: {}", blocks_dir);
    println!("  Database: {}", db_path);
    println!("  Files: {} (skip {}-{}, process {})", total_files, 0, skip_until, files_to_process);
    println!("  Checkpoint interval: {} files", checkpoint_interval);
    println!("  DB lock: released between checkpoints (brute-force can run in parallel)");

    if files_to_process <= 0 {
        println!("  Already up to date!");
        return Ok(());
    }

    // Load or create tracker (opens DB briefly, then releases)
    let mut tracker = if existing_progress.is_some() {
        println!("  Reloading existing UTXO set...");
        let t = UtxoTracker::load_from_db(db_path)?;
        println!("  Loaded {} UTXOs, {} scripts", t.len(), t.script_index.len());
        t
    } else {
        UtxoTracker::new()
    };

    let start = Instant::now();
    let mut total_blocks = 0u64;
    let mut total_txs = 0u64;

    for (file_idx, block_file) in block_files.iter().enumerate() {
        if file_idx < skip_until as usize {
            continue;
        }

        let file_start = Instant::now();
        scan_block_file_full(&block_file, obf_key, &mut tracker, &mut total_blocks, &mut total_txs)?;
        let file_elapsed = file_start.elapsed();
        let now_utxos = tracker.len();

        // Progress every 10 files
        if file_idx % 10 == 0 {
            let elapsed = start.elapsed();
            let processed = file_idx.saturating_sub(skip_until as usize);
            let eta = if processed > 0 {
                let per_file = elapsed.as_secs_f64() / processed as f64;
                let remaining = (total_files - file_idx) as f64 * per_file;
                let mins = remaining / 60.0;
                format!("{:.0}min remaining", mins)
            } else {
                "calculating...".to_string()
            };
            let tx_rate = if elapsed.as_secs() > 0 {
                total_txs as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            };
            eprintln!("[{}/{}] {} blocks | {} txs ({:.0}k/s) | {} UTXOs | {} scripts | file:{:?} | {:?} total | {}",
                file_idx, total_files, total_blocks, total_txs, tx_rate / 1000.0, now_utxos,
                tracker.script_index.len(), file_elapsed, start.elapsed(), eta);
        }

        // Checkpoint (opens DB, writes, releases lock)
        if file_idx > skip_until as usize && (file_idx - skip_until as usize) % checkpoint_interval as usize == 0 {
            eprintln!("  >>> CHECKPOINT (file {})...", file_idx);
            let cp_start = Instant::now();
            tracker.save_checkpoint(db_path, file_idx as u32)?;
            eprintln!("  >>> CHECKPOINT done in {:?} ({} scripts) | LOCK RELEASED", cp_start.elapsed(), tracker.script_index.len());
        }
    }

    eprintln!("  >>> FINAL CHECKPOINT...");
    let cp_start = Instant::now();
    tracker.save_checkpoint(db_path, total_files as u32)?;
    eprintln!("  >>> FINAL CHECKPOINT done in {:?}", cp_start.elapsed());

    let elapsed = start.elapsed();
    println!("\nDone in {:?}!", elapsed);
    println!("  Blocks: {} | Transactions: {} | UTXOs: {} | Scripts: {}",
        total_blocks, total_txs, tracker.len(), tracker.script_index.len());

    Ok(())
}

fn scan_block_file_full(
    path: &Path,
    obf_key: [u8; 8],
    tracker: &mut UtxoTracker,
    total_blocks: &mut u64,
    total_txs: &mut u64,
) -> Result<()> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    // Optimized XOR deobfuscation
    for chunk in buf.chunks_mut(obf_key.len()) {
        for (b, k) in chunk.iter_mut().zip(obf_key.iter()) {
            *b ^= k;
        }
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

                // Process spends (O(1))
                for input in tx.input.iter() {
                    let prev_txid: [u8; 32] = input.previous_output.txid.to_byte_array();
                    let prev_vout = input.previous_output.vout;
                    tracker.spend(prev_txid, prev_vout);
                }

                // Add new outputs (O(1))
                for (vout_idx, txout) in tx.output.iter().enumerate() {
                    let script = txout.script_pubkey.as_bytes().to_vec();
                    let value = txout.value.to_sat();
                    tracker.add_output(txid_bytes, vout_idx as u32, script, value);
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
        if let Ok(meta) = read_tx.open_table(META_TABLE) {
            meta.get("last_file").ok().map(|v| v.map(|x| x.value()).unwrap_or(0))
        } else {
            None
        }
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

    let (last_file, utxo_count) = if let Ok(meta) = read_tx.open_table(META_TABLE) {
        let lf = meta.get("last_file").ok().map(|v| v.map(|x| x.value()));
        let uc = meta.get("utxo_count").ok().map(|v| v.map(|x| x.value()));
        (lf.flatten(), uc.flatten())
    } else {
        (None, None)
    };

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
