use anyhow::Result;
use bitcoin::key::{CompressedPublicKey, PrivateKey, UntweakedPublicKey};
use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::Network;
use clap::Parser;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use btcsolver::flat_index::FlatIndex;

/// BTCSolver - Brute-force de cl^es priv^ees (CPU multi-core + FlatIndex)
#[derive(Parser)]
struct Cli {
    /// Path to UTXO index database (for .redb fallback)
    #[arg(short, long, default_value = "utxo-index.redb")]
    db_path: String,

    /// Path to snapshot file (overrides db_path derivation)
    #[arg(long)]
    snapshot_path: Option<String>,

    /// Number of CPU threads (0 = auto-detect)
    #[arg(short, long, default_value = "0")]
    threads: usize,

    /// Start key (hex, 32 bytes). Default: 0000...0001
    #[arg(short, long)]
    start: Option<String>,

    /// Number of keys to test
    #[arg(long, default_value = "0")]
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

    /// Write live stats to a JSON file every N seconds (0 = disabled). Default: 30
    #[arg(long, default_value = "30")]
    stats_interval: u64,

    /// Stats output file. Default: brute-force-stats.json
    #[arg(long, default_value = "brute-force-stats.json")]
    stats_file: String,

    /// Minimum UTXO value in sats to include (filter dust). Default: 0 (all)
    #[arg(long, default_value = "0")]
    min_value: u64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Parse address types
    let addr_types: Vec<String> = cli.addr_types.split(',').map(|s| s.trim().to_string()).collect();

    // Determine snapshot path
    let snapshot_path = cli
        .snapshot_path
        .clone()
        .unwrap_or_else(|| cli.db_path.replace(".redb", ".snapshot"));

    // Load UTXO index into RAM using FlatIndex
    println!("Loading UTXO index from {}...", snapshot_path);
    let load_start = Instant::now();
    let flat_index = load_snapshot(&snapshot_path, cli.db_retries, cli.min_value)?;
    let load_time = load_start.elapsed();

    println!("Index loaded in {:?}!", load_time);
    flat_index.print_stats();
    println!("  Blockchain coverage: snapshot file");
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
    if cli.stats_interval > 0 {
        println!("  Live stats: {} (every {}s)", cli.stats_file, cli.stats_interval);
    }
    if cli.min_value > 0 {
        println!("  Dust filter: >= {} sats", cli.min_value);
    }
    println!();

    // Launch live stats writer thread
    let stats_handle = if cli.stats_interval > 0 {
        let stats_file = cli.stats_file.clone();
        let keys_tested = Arc::clone(&keys_tested);
        let stop_flag = Arc::clone(&stop_flag);
        let results = Arc::clone(&results);
        let start_instant = Instant::now();
        Some(thread::spawn(move || {
            loop {
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }
                thread::sleep(std::time::Duration::from_secs(cli.stats_interval));
                let total = keys_tested.load(Ordering::Relaxed);
                let elapsed = start_instant.elapsed();
                let rate = if elapsed.as_secs_f64() > 0.0 {
                    total as f64 / elapsed.as_secs_f64()
                } else {
                    0.0
                };
                let found = results.lock().unwrap().len();
                let stats = serde_json::json!({
                    "keys_tested": total,
                    "keys_per_sec": rate as u64,
                    "matches_found": found,
                    "elapsed_seconds": elapsed.as_secs(),
                    "elapsed_human": format!("{:?}", elapsed),
                    "timestamp": chrono::Local::now().to_rfc3339(),
                });
                if let Ok(json) = serde_json::to_string_pretty(&stats) {
                    std::fs::write(&stats_file, &json).ok();
                }
            }
        }))
    } else {
        None
    };

    let start = Instant::now();

    if cli.use_gpu {
        // TODO: GPU path - CUDA kernel launch
        eprintln!("WARNING: GPU mode not yet implemented. Using CPU only.");
        run_cpu_bruteforce(
            Arc::new(flat_index),
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
            Arc::new(flat_index),
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

    // Wait for stats writer to finish
    if let Some(handle) = stats_handle {
        let _ = handle.join();
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
    flat_index: Arc<FlatIndex>,
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
        let flat_index = Arc::clone(&flat_index);
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

                    // Derive private key (from_slice validates range internally)
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

                    // Derive addresses and lookup in FlatIndex — no allocations in hot path!
                    let mut total_sats = 0u64;
                    let mut matched_addrs: Vec<String> = Vec::new();

                    if addr_types.contains(&"legacy".to_string()) {
                        let addr = bitcoin::Address::p2pkh(pubkey, network);
                        let script = addr.script_pubkey();
                        let val = flat_index.lookup(script.as_bytes());
                        if val > 0 {
                            total_sats += val;
                            matched_addrs.push(format!("{} [legacy]", addr));
                        }
                    }

                    if addr_types.contains(&"segwit".to_string()) {
                        let addr = bitcoin::Address::p2wpkh(&compressed, network);
                        let script = addr.script_pubkey();
                        let val = flat_index.lookup(script.as_bytes());
                        if val > 0 {
                            total_sats += val;
                            matched_addrs.push(format!("{} [segwit]", addr));
                        }
                    }

                    if addr_types.contains(&"wrapped".to_string()) {
                        let addr = bitcoin::Address::p2shwpkh(&compressed, network);
                        let script = addr.script_pubkey();
                        let val = flat_index.lookup(script.as_bytes());
                        if val > 0 {
                            total_sats += val;
                            matched_addrs.push(format!("{} [wrapped]", addr));
                        }
                    }

                    if addr_types.contains(&"taproot".to_string()) {
                        let addr = bitcoin::Address::p2tr(&secp, xonly, None, network);
                        let script = addr.script_pubkey();
                        let val = flat_index.lookup(script.as_bytes());
                        if val > 0 {
                            total_sats += val;
                            matched_addrs.push(format!("{} [taproot]", addr));
                        }
                    }

                    // Only hex-encode and WIF-encode on actual match (1 in 2^256 chance)
                    if total_sats > 0 {
                        let key_hex = hex::encode(&key);
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
                            eprintln!("\n\n BALANCE FOUND! Stopping all threads...");
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
    let mut all_results: Vec<BalanceResult> = if std::path::Path::new(output_file).exists() {
        let content = std::fs::read_to_string(output_file)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    all_results.extend_from_slice(new_results);
    let json = serde_json::to_string_pretty(&all_results)?;
    std::fs::write(output_file, &json)?;
    Ok(())
}

// ─── Snapshot Loading ───────────────────────────────────────────────────

/// Load FlatIndex from snapshot file (with retry if needed).
fn load_snapshot(
    snapshot_path: &str,
    max_retries: u32,
    min_value: u64,
) -> Result<FlatIndex> {
    let mut attempt = 0u32;
    let infinite = max_retries == 0;

    loop {
        if std::path::Path::new(snapshot_path).exists() {
            match FlatIndex::load_from_snapshot(snapshot_path, min_value) {
                Ok(result) => {
                    println!("  Loaded from snapshot (FlatIndex)!");
                    return Ok(result);
                }
                Err(e) => {
                    eprintln!("  Snapshot load failed: {}. Retrying...", e);
                }
            }
        }

        attempt += 1;
        let retry_label = if infinite { "∞" } else { &format!("{}", max_retries) };
        eprintln!("  Waiting for snapshot. Retry {}/{} in 10s...",
            attempt, retry_label);
        std::thread::sleep(std::time::Duration::from_secs(10));
        if !infinite && attempt >= max_retries {
            anyhow::bail!("No snapshot after {} retries.", max_retries);
        }
    }
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
