use anyhow::Result;
use bitcoin::key::{CompressedPublicKey, PrivateKey, UntweakedPublicKey};
use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::{Network, Txid};
use bitcoin_hashes::Hash;
use clap::Parser;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

/// BTCSolver - Brute-force de cl^es priv^ees (CPU multi-core + GPU CUDA)
#[derive(Parser)]
struct Cli {
    /// Path to UTXO index database
    #[arg(short, long, default_value = "utxo-index.redb")]
    db_path: String,

    /// Number of CPU threads (0 = auto-detect)
    #[arg(short, long, default_value = "0")]
    threads: usize,

    /// Start key (hex, 32 bytes). Default: 0000...0001
    #[arg(short, long)]
    start: Option<String>,

    /// Number of keys to test
    #[arg(short, long, default_value = "0")]
    count: u64, // 0 = unlimited

    /// Generate random keys instead of sequential
    #[arg(long)]
    random: bool,

    /// Use GPU (CUDA) for key derivation (requires NVIDIA GPU)
    #[arg(long)]
    use_gpu: bool,

    /// GPU device IDs (comma-separated). Default: all available
    #[arg(long)]
    gpus: Option<String>,

    /// Batch size per GPU/CPU thread
    #[arg(long, default_value = "256000")]
    batch_size: usize,

    /// Only test specific address types (comma-separated: legacy,segwit,wrapped,taproot)
    #[arg(long, default_value = "legacy,segwit,wrapped,taproot")]
    addr_types: String,

    /// Stop immediately when a balance > 0 is found
    #[arg(long)]
    stop_on_match: bool,

    /// Output file for found keys (JSON). Default: found-keys.json
    #[arg(long, default_value = "found-keys.json")]
    output_file: String,

    /// Max retries when DB is locked (0 = infinite). Default: 120
    #[arg(long, default_value = "120")]
    db_retries: u32,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Parse address types
    let addr_types: Vec<String> = cli.addr_types.split(',').map(|s| s.trim().to_string()).collect();

    // Load UTXO index into RAM (with retry if indexer holds lock)
    println!("Loading UTXO index from {}...", cli.db_path);
    let load_start = Instant::now();
    let (utxo_index, last_file) = load_index_with_retry(&cli.db_path, cli.db_retries, 10)?;
    let load_time = load_start.elapsed();

    println!("Index loaded in {:?}!", load_time);
    println!("  Scripts indexed: {}", utxo_index.len());
    println!("  Blockchain coverage: file {}", last_file.unwrap_or(0));
    println!("  CPU threads: {}", if cli.threads == 0 {
        num_cpus::get()
    } else {
        cli.threads
    });
    if cli.use_gpu {
        println!("  GPU acceleration: ENABLED (CUDA)");
    }
    println!();

    // Determine start key
    let start_key: [u8; 32] = if let Some(hex_str) = &cli.start {
        let bytes = hex::decode(hex_str).map_err(|e| anyhow::anyhow!("Invalid hex: {}", e))?;
        bytes.try_into().map_err(|_| anyhow::anyhow!("Key must be 32 bytes (64 hex chars)"))?
    } else {
        let mut k = [0u8; 32];
        k[31] = 1;
        k
    };

    // Shared state
    let results = Arc::new(Mutex::new(Vec::new()));
    let keys_tested = Arc::new(AtomicU64::new(0));
    let stop_flag = Arc::new(AtomicBool::new(false));
    let found_keys_file = cli.output_file.clone();

    let num_threads = if cli.threads == 0 { num_cpus::get() } else { cli.threads };
    let batch_size = cli.batch_size;

    println!("Starting brute-force...");
    if cli.random {
        println!("  Mode: RANDOM keys (cryptographic RNG via getrandom)");
    } else {
        println!("  Mode: SEQUENTIAL keys from {}", hex::encode(&start_key));
    }
    if cli.count > 0 {
        println!("  Keys to test: {}", cli.count);
    } else if cli.stop_on_match {
        println!("  Keys to test: unlimited (stops on first balance)");
    } else {
        println!("  Keys to test: unlimited (Ctrl+C to stop)");
    }
    println!("  Batch size: {}", batch_size);
    println!("  Address types: {}", cli.addr_types);
    if cli.stop_on_match {
        println!("  Auto-stop: ENABLED on first balance");
        println!("  Output file: {}", cli.output_file);
    }
    println!();

    let start = Instant::now();

    if cli.use_gpu {
        // TODO: GPU path - CUDA kernel launch
        // For now, fall back to CPU with a warning
        eprintln!("WARNING: GPU mode not yet implemented. Using CPU only.");
        run_cpu_bruteforce(
            utxo_index.clone(),
            addr_types.clone(),
            cli.count,
            num_threads,
            batch_size,
            &results,
            &keys_tested,
            &stop_flag,
            &found_keys_file,
            &start,
            cli.random,
            cli.stop_on_match,
        );
    } else {
        run_cpu_bruteforce(
            utxo_index.clone(),
            addr_types.clone(),
            cli.count,
            num_threads,
            batch_size,
            &results,
            &keys_tested,
            &stop_flag,
            &found_keys_file,
            &start,
            cli.random,
            cli.stop_on_match,
        );
    }

    let elapsed = start.elapsed();
    let total = keys_tested.load(Ordering::Relaxed);
    let found = results.lock().unwrap().len();

    println!("\n{}", "=".repeat(60));
    println!("RESULTS");
    println!("{}", "=".repeat(60));
    println!("  Time: {:?}", elapsed);
    println!("  Keys tested: {} ({:.0} keys/sec)",
        total, total as f64 / elapsed.as_secs_f64());
    println!("  Balances found: {}", found);

    if found > 0 {
        for r in results.lock().unwrap().iter() {
            println!("  KEY: {} -> {:.8} BTC ({:.0} sats)",
                r.key_hex, r.btc, r.sats);
            for addr in &r.addresses {
                println!("    {}", addr);
            }
        }
    }

    Ok(())
}

fn run_cpu_bruteforce(
    utxo_index: HashMap<Vec<u8>, Vec<(Txid, u32, u64)>>,
    addr_types: Vec<String>,
    max_keys: u64,
    num_threads: usize,
    batch_size: usize,
    results: &Arc<Mutex<Vec<BalanceResult>>>,
    keys_tested: &Arc<AtomicU64>,
    stop_flag: &Arc<AtomicBool>,
    output_file: &str,
    start: &Instant,
    random: bool,
    stop_on_match: bool,
) {
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let utxo_index = utxo_index.clone();
        let addr_types = addr_types.to_vec();
        let results = Arc::clone(results);
        let keys_tested = Arc::clone(keys_tested);
        let stop_flag = Arc::clone(stop_flag);
        let output_file = output_file.to_string();
        let start_time = *start;

        // Seed each thread's RNG with true entropy from getrandom
        let seed = {
            let mut s = [0u8; 32];
            getrandom::getrandom(&mut s).expect("Failed to get random bytes for seeding");
            // XOR thread_id into seed for thread uniqueness
            let tid_bytes = (thread_id as u64).to_le_bytes();
            for i in 0..8 {
                s[i] ^= tid_bytes[i];
            }
            s
        };

        let handle = thread::spawn(move || {
            let secp = Secp256k1::<All>::new();
            let network = Network::Bitcoin;

            // Each thread gets its own RNG instance
            let mut rng = if random {
                Some(StdRng::from_seed(seed))
            } else {
                None
            };

            let mut key = [0u8; 32];
            key[31] = 1; // Sequential start
            let mut local_count = 0u64;

            loop {
                // Check stop flag at batch boundary
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }

                for _ in 0..batch_size {
                    if max_keys > 0 && keys_tested.load(Ordering::Relaxed) >= max_keys {
                        return;
                    }

                    // Generate next key: random or sequential
                    if random {
                        let r = rng.as_mut().unwrap();
                        let mut candidate = [0u8; 32];
                        r.fill(&mut candidate);
                        key = candidate;
                    } else {
                        increment_key(&mut key);
                    }

                    // Skip invalid keys (must be < secp256k1 order)
                    if !is_valid_private_key(&key) {
                        continue;
                    }

                    // Derive private key
                    let secp_key = match bitcoin::secp256k1::SecretKey::from_slice(&key) {
                        Ok(k) => k,
                        Err(_) => continue,
                    };

                    let pk = PrivateKey {
                        inner: secp_key,
                        network: network.into(),
                        compressed: true,
                    };

                    let pubkey = pk.public_key(&secp);
                    let compressed = match CompressedPublicKey::from_private_key(&secp, &pk) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };
                    let xonly: UntweakedPublicKey = compressed.into();
                    let key_hex = hex::encode(&key);

                    // Derive addresses based on requested types
                    let mut total_sats = 0u64;
                    let mut matched_addrs: Vec<String> = Vec::new();

                    if addr_types.contains(&"legacy".to_string()) {
                        let addr = bitcoin::Address::p2pkh(pubkey, network);
                        if let Some(utxos) = utxo_index.get(addr.script_pubkey().as_bytes()) {
                            for (_, _, val) in utxos {
                                total_sats += val;
                            }
                            matched_addrs.push(format!("{} [legacy]", addr));
                        }
                    }

                    if addr_types.contains(&"segwit".to_string()) {
                        let addr = bitcoin::Address::p2wpkh(&compressed, network);
                        if let Some(utxos) = utxo_index.get(addr.script_pubkey().as_bytes()) {
                            for (_, _, val) in utxos {
                                total_sats += val;
                            }
                            matched_addrs.push(format!("{} [segwit]", addr));
                        }
                    }

                    if addr_types.contains(&"wrapped".to_string()) {
                        let addr = bitcoin::Address::p2shwpkh(&compressed, network);
                        if let Some(utxos) = utxo_index.get(addr.script_pubkey().as_bytes()) {
                            for (_, _, val) in utxos {
                                total_sats += val;
                            }
                            matched_addrs.push(format!("{} [wrapped]", addr));
                        }
                    }

                    if addr_types.contains(&"taproot".to_string()) {
                        let addr = bitcoin::Address::p2tr(&secp, xonly, None, network);
                        if let Some(utxos) = utxo_index.get(addr.script_pubkey().as_bytes()) {
                            for (_, _, val) in utxos {
                                total_sats += val;
                            }
                            matched_addrs.push(format!("{} [taproot]", addr));
                        }
                    }

                    if total_sats > 0 {
                        let wif = {
                            let wif_key = PrivateKey {
                                inner: secp_key,
                                network: network.into(),
                                compressed: true,
                            };
                            Some(wif_key.to_wif())
                        };

                        let result = BalanceResult {
                            key_hex: key_hex.clone(),
                            wif,
                            sats: total_sats,
                            btc: total_sats as f64 / 100_000_000.0,
                            addresses: matched_addrs.clone(),
                            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        };

                        results.lock().unwrap().push(result.clone());

                        // Save immediately to disk
                        save_found_keys(&output_file, &vec![result]).ok();

                        if stop_on_match {
                            eprintln!("\n\n🎯 BALANCE FOUND! Stopping all threads...");
                            eprintln!("  Key: {}", key_hex);
                            eprintln!("  Balance: {} sats ({:.8} BTC)", total_sats, total_sats as f64 / 100_000_000.0);
                            for addr in &matched_addrs {
                                eprintln!("  Address: {}", addr);
                            }
                            eprintln!("  Saved to: {}", output_file);
                            stop_flag.store(true, Ordering::Relaxed);
                            break;
                        }
                    }

                    local_count += 1;
                }

                keys_tested.fetch_add(local_count, Ordering::Relaxed);

                // Progress report
                let total = keys_tested.load(Ordering::Relaxed);
                let elapsed = start_time.elapsed();
                let rate = if elapsed.as_secs_f64() > 0.0 {
                    total as f64 / elapsed.as_secs_f64()
                } else {
                    0.0
                };
                eprintln!("[Thread {}] {} keys tested | {:.0} keys/sec | {} matches",
                    thread_id, total, rate, results.lock().unwrap().len());
                local_count = 0; // Reset for next batch
            }
        });

        handles.push(handle);
    }

    // Wait for threads to finish
    for handle in handles {
        let _ = handle.join();
    }
}

// ─── Data structures ────────────────────────────────────────────────────

#[derive(Serialize, serde::Deserialize, Clone)]
struct BalanceResult {
    key_hex: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    wif: Option<String>,
    sats: u64,
    btc: f64,
    addresses: Vec<String>,
    #[serde(default)]
    timestamp: String,
}

/// Save found keys to a JSON file (appends to existing results)
fn save_found_keys(output_file: &str, new_results: &[BalanceResult]) -> Result<()> {
    // Load existing results if file exists
    let mut all_results: Vec<BalanceResult> = if std::path::Path::new(output_file).exists() {
        let content = std::fs::read_to_string(output_file)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Append new results
    all_results.extend_from_slice(new_results);

    // Write back
    let json = serde_json::to_string_pretty(&all_results)?;
    std::fs::write(output_file, &json)?;
    Ok(())
}

// ─── UTXO Index Loading ─────────────────────────────────────────────────

/// Load index from binary snapshot (lock-free) or fall back to redb.
/// Returns: (script_bytes -> Vec<(txid, vout, value)>, last_file)
fn load_index_with_retry(
    db_path: &str,
    max_retries: u32,
    retry_delay_secs: u64,
) -> Result<(HashMap<Vec<u8>, Vec<(Txid, u32, u64)>>, Option<u64>)> {
    // Wait for snapshot file (lock-free read - no DB lock conflicts!)
    let snapshot_path = db_path.replace(".redb", ".snapshot");
    let mut attempt = 0u32;
    let infinite = max_retries == 0;

    loop {
        if std::path::Path::new(&snapshot_path).exists() {
            match load_snapshot_to_ram(&snapshot_path) {
                Ok(result) => {
                    println!("  Loaded from snapshot (lock-free)!");
                    return Ok(result);
                }
                Err(e) => {
                    eprintln!("  Snapshot load failed: {}. Retrying...", e);
                }
            }
        }

        attempt += 1;
        let retry_label = if infinite { "∞" } else { &format!("{}", max_retries) };
        eprintln!("  Waiting for snapshot (indexer creates at checkpoint). Retry {}/{} in {}s...",
            attempt, retry_label, retry_delay_secs);
        std::thread::sleep(std::time::Duration::from_secs(retry_delay_secs));
        if !infinite && attempt >= max_retries {
            anyhow::bail!("No snapshot after {} retries. Run indexer to create a checkpoint.", max_retries);
        }
    }
}

/// Load binary snapshot into RAM (no locks needed)
fn load_snapshot_to_ram(snapshot_path: &str) -> Result<(HashMap<Vec<u8>, Vec<(Txid, u32, u64)>>, Option<u64>)> {
    use std::io::Read;
    let mut f = std::io::BufReader::new(std::fs::File::open(snapshot_path)?);
    let file_len = f.get_ref().metadata()?.len();

    let mut header = [0u8; 4];
    f.read_exact(&mut header)?;
    let num_scripts = u32::from_le_bytes(header);

    let mut index: HashMap<Vec<u8>, Vec<(Txid, u32, u64)>> = HashMap::new();
    let mut total_scripts = 0u64;

    for _ in 0..num_scripts {
        let mut slen_buf = [0u8; 2];
        f.read_exact(&mut slen_buf)?;
        let script_len = u16::from_le_bytes(slen_buf) as usize;

        let mut script = vec![0u8; script_len];
        f.read_exact(&mut script)?;

        let mut count_buf = [0u8; 4];
        f.read_exact(&mut count_buf)?;
        let count = u32::from_le_bytes(count_buf);

        let mut entries = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let mut txid = [0u8; 32];
            f.read_exact(&mut txid)?;

            let mut vout_buf = [0u8; 4];
            f.read_exact(&mut vout_buf)?;
            let vout = u32::from_le_bytes(vout_buf);

            let mut val_buf = [0u8; 8];
            f.read_exact(&mut val_buf)?;
            let value = u64::from_le_bytes(val_buf);

            entries.push((Txid::from_byte_array(txid), vout, value));
        }

        if !entries.is_empty() {
            index.insert(script, entries);
            total_scripts += 1;
        }
    }

    let snapshot_mb = file_len as f64 / 1_048_576.0;
    println!("  Loaded {} scripts from {} ({:.1} MB snapshot)", total_scripts, snapshot_path, snapshot_mb);
    Ok((index, None))
}

// ─── Key Utilities ──────────────────────────────────────────────────────

fn increment_key(key: &mut [u8; 32]) {
    for byte in key.iter_mut().rev() {
        if *byte == u8::MAX {
            *byte = 0;
        } else {
            *byte += 1;
            return;
        }
    }
}

fn is_valid_private_key(key: &[u8; 32]) -> bool {
    // Must be > 0 and < secp256k1 order
    // Order = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
    if key.iter().all(|&b| b == 0) {
        return false;
    }
    // Quick check: first byte of order is 0xFF, so keys starting with 0xFF
    // need more careful comparison. For simplicity, reject 0xFFxx... keys
    // that exceed the order. This is approximate but catches most invalid keys.
    const ORDER: [u8; 32] = [
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
        0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B,
        0xBF, 0xD2, 0x5E, 0x8C, 0xD0, 0x36, 0x41, 0x41,
    ];
    if key[0] < 0xFF {
        return true; // Definitely less than order
    }
    // key[0] == 0xFF, need full comparison
    for i in 0..32 {
        if key[i] < ORDER[i] {
            return true;
        }
        if key[i] > ORDER[i] {
            return false;
        }
    }
    false // key == order, invalid
}
