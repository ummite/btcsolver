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

/// Bitmask flags for address types — avoids string allocs in hot loop
#[derive(Clone, Copy)]
struct AddrFlags(u8);
impl AddrFlags {
    const LEGACY:  u8 = 1 << 0;
    const SEGWIT:  u8 = 1 << 1;
    const WRAPPED: u8 = 1 << 2;
    const TAPROOT: u8 = 1 << 3;
    const ALL: u8   = Self::LEGACY | Self::SEGWIT | Self::WRAPPED | Self::TAPROOT;

    fn has(&self, flag: u8) -> bool { self.0 & flag != 0 }

    fn from_str(s: &str) -> Self {
        let mut flags: u8 = 0;
        for part in s.split(',') {
            match part.trim().to_lowercase().as_str() {
                "legacy"  => flags |= Self::LEGACY,
                "segwit"  => flags |= Self::SEGWIT,
                "wrapped" => flags |= Self::WRAPPED,
                "taproot" => flags |= Self::TAPROOT,
                other => eprintln!("WARNING: unknown address type '{}'", other),
            }
        }
        AddrFlags(flags)
    }

    fn to_display(&self) -> String {
        let mut parts = Vec::new();
        if self.0 & Self::LEGACY  != 0 { parts.push("legacy"); }
        if self.0 & Self::SEGWIT  != 0 { parts.push("segwit"); }
        if self.0 & Self::WRAPPED != 0 { parts.push("wrapped"); }
        if self.0 & Self::TAPROOT != 0 { parts.push("taproot"); }
        parts.join(",")
    }
}

/// Per-thread progress entry for resume support
#[derive(Serialize, serde::Deserialize, Clone, Debug)]
struct ThreadProgress {
    thread_id: usize,
    keys_tested: u64,
    last_key_hex: String,
    mode: String, // "random" or "sequential"
}

#[derive(Serialize, serde::Deserialize, Clone, Debug)]
struct ProgressFile {
    version: u32,
    mode: String,
    total_keys_tested: u64,
    threads: Vec<ThreadProgress>,
    timestamp: String,
}

/// BTCSolver - Brute-force de cl^es priv^ees (CPU multi-core + FlatIndex v9)
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

    /// Progress file for resume (JSON with last keys per thread). Default: brute-force-progress.json
    #[arg(long, default_value = "brute-force-progress.json")]
    progress_file: String,

    /// Save progress file every N seconds (0 = disabled). Default: 60
    #[arg(long, default_value = "60")]
    progress_interval: u64,

    /// Maximum snapshot file age in seconds (0 = no check). Default: 86400 (24h)
    #[arg(long, default_value = "86400")]
    max_snapshot_age: u64,

    /// Check snapshot freshness against blockchain API (mempool.space). Exits if snapshot is too old.
    #[arg(long)]
    check_freshness: bool,
}

/// Check that the snapshot file is fresh enough before starting the scan.
fn check_snapshot_freshness(
    snapshot_path: &str,
    max_age_seconds: u64,
    _check_api: bool,
) -> Result<()> {
    let path = std::path::Path::new(snapshot_path);
    if !path.exists() {
        eprintln!("[FRESHNESS] Snapshot file not found: {}", snapshot_path);
        return Err(anyhow::anyhow!("Snapshot file not found: {}", snapshot_path));
    }

    // Check file modification time
    let metadata = std::fs::metadata(path)?;
    let modified = metadata.modified()?;
    let age = modified.elapsed()?;
    let age_seconds = age.as_secs();

    println!("[FRESHNESS] Snapshot: {}", snapshot_path);
    println!("[FRESHNESS] File age: {}s ({:.1}h, {:.1} days)", age_seconds, age_seconds as f64 / 3600.0, age_seconds as f64 / 86400.0);

    if max_age_seconds > 0 && age_seconds > max_age_seconds {
        eprintln!("[FRESHNESS] ERROR: Snapshot is {}s old, exceeds max age of {}s ({:.1}h)", age_seconds, max_age_seconds, max_age_seconds as f64 / 3600.0);
        eprintln!("[FRESHNESS] The UTXO snapshot is too old. Regenerate it with a fresh blockchain sync.");
        eprintln!("[FRESHNESS] Use --max-snapshot-age 0 to disable this check.");
        return Err(anyhow::anyhow!("Snapshot too old ({}s > {}s)", age_seconds, max_age_seconds));
    }

    // Optional: query mempool.space API for current block height
    if _check_api {
        let height_result = (|| -> Result<u64> {
            let resp = ureq::get("https://mempool.space/api/blocks/tip/height")
                .timeout(std::time::Duration::from_secs(10))
                .call()?;
            let s = resp.into_string()?;
            Ok(s.trim().parse()?)
        })();
        match height_result {
            Ok(current_height) => {
                println!("[FRESHNESS] Current blockchain height: {}", current_height);
                println!("[FRESHNESS] Note: Snapshot does not embed block height metadata. Only file age is checked.");
            }
            Err(e) => {
                eprintln!("[FRESHNESS] Warning: Could not query blockchain API: {}", e);
            }
        }
    }

    if max_age_seconds > 0 {
        println!("[FRESHNESS] OK — snapshot within {}s ({:.1}h) max age", max_age_seconds, max_age_seconds as f64 / 3600.0);
    } else {
        println!("[FRESHNESS] OK — age check disabled");
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Parse address types into bitmask (no string allocs in hot loop)
    let addr_flags = AddrFlags::from_str(&cli.addr_types);

    // Determine snapshot path
    let snapshot_path = cli
        .snapshot_path
        .clone()
        .unwrap_or_else(|| cli.db_path.replace(".redb", ".snapshot"));

    // Check snapshot freshness
    check_snapshot_freshness(&snapshot_path, cli.max_snapshot_age, cli.check_freshness)?;

    // Load UTXO index into RAM using FlatIndex
    println!("Loading UTXO index from {}...", snapshot_path);
    let load_start = Instant::now();
    let flat_index = load_snapshot(&snapshot_path, cli.db_retries, cli.min_value)?;
    let load_time = load_start.elapsed();

    println!("Index loaded in {:?}!", load_time);
    flat_index.print_stats();
    println!("  Blockchain coverage: snapshot file");
    println!("  CPU threads: {}", if cli.threads == 0 {
        num_cpus::get().saturating_sub(1).max(1)
    } else {
        cli.threads
    });
    if cli.use_gpu {
        println!("  GPU acceleration: ENABLED (CUDA)");
    }
    println!();

    // Determine start key
    let mut start_key: [u8; 32] = if let Some(hex_str) = &cli.start {
        let bytes = hex::decode(hex_str).map_err(|e| anyhow::anyhow!("Invalid hex: {}", e))?;
        bytes.try_into().map_err(|_| anyhow::anyhow!("Key must be 32 bytes (64 hex chars)"))?
    } else {
        let mut k = [0u8; 32];
        k[31] = 1;
        k
    };

    // Try to resume from previous position (sequential mode only)
    let mut keys_already_tested: u64 = 0;
    if !cli.random {
        let (resumed_key, already_tested) = try_resume_position(&cli.progress_file, &start_key);
        start_key = resumed_key;
        keys_already_tested = already_tested;
    }

    // Shared state
    let results = Arc::new(Mutex::new(Vec::new()));
    let keys_tested = Arc::new(AtomicU64::new(keys_already_tested));
    let stop_flag = Arc::new(AtomicBool::new(false));
    let found_keys_file = cli.output_file.clone();

    let num_threads = if cli.threads == 0 {
        // Leave 1 core free for the OS / other tasks
        num_cpus::get().saturating_sub(1).max(1)
    } else {
        cli.threads
    };
    let batch_size = cli.batch_size;

    println!("Starting brute-force...");
    if cli.random {
        println!("  Mode: RANDOM keys (cryptographic RNG via getrandom)");
    } else {
        println!("  Mode: SEQUENTIAL keys from {}", hex::encode(&start_key));
        if keys_already_tested > 0 {
            println!("  Resumed: {} keys already tested before this run", keys_already_tested);
        }
    }
    if cli.count > 0 {
        println!("  Keys to test: {}", cli.count);
    } else if cli.stop_on_match {
        println!("  Keys to test: unlimited (stops on first balance)");
    } else {
        println!("  Keys to test: unlimited (Ctrl+C to stop)");
    }
    println!("  Batch size: {}", batch_size);
    println!("  Address types: {}", addr_flags.to_display());
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
    if cli.progress_interval > 0 {
        println!("  Progress file: {} (every {}s)", cli.progress_file, cli.progress_interval);
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

    // Launch progress saver thread (for resume after crash)
    let progress_handle = if cli.progress_interval > 0 {
        let progress_file = cli.progress_file.clone();
        let keys_tested = Arc::clone(&keys_tested);
        let stop_flag = Arc::clone(&stop_flag);
        let thread_progress = Arc::new(Mutex::new(Vec::<ThreadProgress>::new()));
        let mode_str = if cli.random { "random" } else { "sequential" }.to_string();
        let interval = cli.progress_interval;
        let tp_for_closure = Arc::clone(&thread_progress);
        let is_sequential = !cli.random;
        let handle = thread::spawn(move || {
            loop {
                if stop_flag.load(Ordering::Relaxed) {
                    let threads = tp_for_closure.lock().unwrap();
                    if !threads.is_empty() {
                        let total = keys_tested.load(Ordering::Relaxed);
                        let pf = ProgressFile {
                            version: 1,
                            mode: mode_str.clone(),
                            total_keys_tested: total,
                            threads: threads.clone(),
                            timestamp: chrono::Local::now().to_rfc3339(),
                        };
                        if let Ok(json) = serde_json::to_string_pretty(&pf) {
                            std::fs::write(&progress_file, &json).ok();
                        }
                        // Write simple position file for sequential mode
                        if is_sequential {
                            let mut max_key = [0u8; 32];
                            for tp in threads.iter() {
                                if let Ok(kb) = hex::decode(&tp.last_key_hex) {
                                    if let Ok(ka) = <[u8; 32]>::try_from(kb) {
                                        if ka > max_key { max_key = ka; }
                                    }
                                }
                            }
                            if max_key != [0u8; 32] {
                                write_position_file(&progress_file, "sequential", &max_key, total);
                            }
                        }
                    }
                    break;
                }
                thread::sleep(std::time::Duration::from_secs(interval));
                let threads = tp_for_closure.lock().unwrap();
                if !threads.is_empty() {
                    let total = keys_tested.load(Ordering::Relaxed);
                    let pf = ProgressFile {
                        version: 1,
                        mode: mode_str.clone(),
                        total_keys_tested: total,
                        threads: threads.clone(),
                        timestamp: chrono::Local::now().to_rfc3339(),
                    };
                    if let Ok(json) = serde_json::to_string_pretty(&pf) {
                        std::fs::write(&progress_file, &json).ok();
                    }
                    // Write simple position file for sequential mode
                    if is_sequential {
                        let mut max_key = [0u8; 32];
                        for tp in threads.iter() {
                            if let Ok(kb) = hex::decode(&tp.last_key_hex) {
                                if let Ok(ka) = <[u8; 32]>::try_from(kb) {
                                    if ka > max_key { max_key = ka; }
                                }
                            }
                        }
                        if max_key != [0u8; 32] {
                            write_position_file(&progress_file, "sequential", &max_key, total);
                        }
                    }
                }
            }
        });
        Some((handle, thread_progress))
    } else {
        None
    };

    let start = Instant::now();

    let use_gpu = if cli.use_gpu {
        let gpu_count = btcsolver::gpu::gpu_device_count();
        if gpu_count <= 0 {
            eprintln!("[GPU] No CUDA device found. Falling back to CPU.");
            false
        } else {
            let n_init = btcsolver::gpu::gpu_init();
            if n_init <= 0 {
                eprintln!("[GPU] Init failed. Falling back to CPU.");
                false
            } else {
                for info in btcsolver::gpu::detect_gpus() {
                    println!("    GPU {}: {}", info.id, info.name);
                }
                true
            }
        }
    } else {
        false
    };

    if use_gpu {
        run_gpu_bruteforce(
            Arc::new(flat_index),
            addr_flags,
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
            &progress_handle.as_ref().map(|(_, p)| Arc::clone(p)),
            start_key,
        );
        btcsolver::gpu::gpu_cleanup();
    } else {
        run_cpu_bruteforce(
            Arc::new(flat_index),
            addr_flags,
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
            &progress_handle.as_ref().map(|(_, p)| Arc::clone(p)),
            start_key,
        );
    }

    // Wait for stats writer to finish
    if let Some(handle) = stats_handle {
        let _ = handle.join();
    }
    // Wait for progress saver to finish
    if let Some((handle, _)) = progress_handle {
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
    addr_flags: AddrFlags,
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
    thread_progress: &Option<Arc<Mutex<Vec<ThreadProgress>>>>,
    start_key: [u8; 32],
) {
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let flat_index = Arc::clone(&flat_index);
        let results = Arc::clone(results);
        let keys_tested = Arc::clone(keys_tested);
        let stop_flag = Arc::clone(stop_flag);
        let output_file = output_file.to_string();
        let start_time = *start;
        let progress = thread_progress.as_ref().map(|p| Arc::clone(p));

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

        // Calculate per-thread starting key for sequential mode
        let thread_start_key = {
            let mut k = start_key;
            if !random {
                let offset = (thread_id as u64) * (batch_size as u64);
                add_offset_to_key(&mut k, offset);
            }
            k
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

            let mut key = thread_start_key; // Sequential start with per-thread offset
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

                    if addr_flags.has(AddrFlags::LEGACY) {
                        let addr = bitcoin::Address::p2pkh(pubkey, network);
                        let script = addr.script_pubkey();
                        let val = flat_index.lookup(script.as_bytes());
                        if val > 0 {
                            total_sats += val;
                            matched_addrs.push(format!("{} [legacy]", addr));
                        }
                    }

                    if addr_flags.has(AddrFlags::SEGWIT) {
                        let addr = bitcoin::Address::p2wpkh(&compressed, network);
                        let script = addr.script_pubkey();
                        let val = flat_index.lookup(script.as_bytes());
                        if val > 0 {
                            total_sats += val;
                            matched_addrs.push(format!("{} [segwit]", addr));
                        }
                    }

                    if addr_flags.has(AddrFlags::WRAPPED) {
                        let addr = bitcoin::Address::p2shwpkh(&compressed, network);
                        let script = addr.script_pubkey();
                        let val = flat_index.lookup(script.as_bytes());
                        if val > 0 {
                            total_sats += val;
                            matched_addrs.push(format!("{} [wrapped]", addr));
                        }
                    }

                    if addr_flags.has(AddrFlags::TAPROOT) {
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

                // Update progress file data (for crash recovery)
                if let Some(ref prog) = progress {
                    let mut last_key = [0u8; 32];
                    last_key.copy_from_slice(&key);
                    if let Ok(mut guard) = prog.try_lock() {
                        let mut idx = guard.len();
                        while idx <= thread_id {
                            guard.push(ThreadProgress {
                                thread_id: idx,
                                keys_tested: 0,
                                last_key_hex: "".to_string(),
                                mode: if random { "random".to_string() } else { "sequential".to_string() },
                            });
                            idx += 1;
                        }
                        guard[thread_id].keys_tested += local_count;
                        guard[thread_id].last_key_hex = hex::encode(&last_key);
                    }
                }

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

/// Serialize FlatIndex script_entries to packed format for GPU upload.
/// GPU struct: u32(offset) + u16(len) + u32(utxo_offset) + u32(utxo_count) = 12 bytes packed
/// Rust struct has padding, so we need to pack manually.
fn serialize_script_entries_for_gpu(fi: &FlatIndex) -> Vec<u8> {
    let mut buf = Vec::with_capacity(fi.script_entries.len() * 12);
    for entry in &fi.script_entries {
        buf.extend_from_slice(&entry.script_offset.to_le_bytes());  // 4 bytes
        buf.extend_from_slice(&entry.script_len.to_le_bytes());     // 2 bytes
        buf.extend_from_slice(&entry.utxo_offset.to_le_bytes());    // 4 bytes
        buf.extend_from_slice(&entry.utxo_count.to_le_bytes());     // 4 bytes
    }
    buf
}

/// Convert AddrFlags bitmask to GPU addr_types u32
fn addr_flags_to_gpu_bits(flags: AddrFlags) -> u32 {
    let mut bits: u32 = 0;
    if flags.has(AddrFlags::LEGACY)  { bits |= 0x01; }
    if flags.has(AddrFlags::SEGWIT)  { bits |= 0x02; }
    if flags.has(AddrFlags::WRAPPED) { bits |= 0x04; }
    if flags.has(AddrFlags::TAPROOT) { bits |= 0x08; }
    bits
}

/// GPU-accelerated brute-force with FlatIndex on GPU:
/// Each thread: generate keys → GPU derive+lookup (all on GPU) → check matches → CPU verify
/// The GPU does: pubkey derivation, SHA256, RIPEMD160, script building, binary search, UTXO summing
/// The CPU only: generates keys, checks GPU results, verifies matches
fn run_gpu_bruteforce(
    flat_index: Arc<FlatIndex>,
    addr_flags: AddrFlags,
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
    thread_progress: &Option<Arc<Mutex<Vec<ThreadProgress>>>>,
    start_key: [u8; 32],
) {
    /* Upload FlatIndex to GPU */
    let gpu_entries = serialize_script_entries_for_gpu(&flat_index);
    let num_entries = flat_index.script_entries.len() as u32;
    let addr_types = addr_flags_to_gpu_bits(addr_flags);

    eprintln!("[GPU] Loading FlatIndex to GPU ({:.0} MB entries, {:.0} MB data, {:.0} MB utxo)...",
        gpu_entries.len() as f64 / 1_048_576.0,
        flat_index.all_data.len() as f64 / 1_048_576.0,
        flat_index.utxo_data.len() as f64 / 1_048_576.0);

    let load_start = Instant::now();
    let load_rc = btcsolver::gpu::gpu_load_index(
        &gpu_entries,
        &flat_index.all_data,
        &flat_index.utxo_data,
        num_entries,
    );
    let load_time = load_start.elapsed();
    if load_rc != 0 {
        eprintln!("[GPU] Failed to load index (rc={}). Falling back to derive-only mode.", load_rc);
        run_gpu_bruteforce_fallback(
            flat_index, addr_flags, max_keys, num_threads, batch_size,
            results, keys_tested, stop_flag, output_file, start, random, stop_on_match, thread_progress,
            start_key,
        );
        return;
    }
    eprintln!("[GPU] Index loaded in {:?}. Using derive+lookup mode.", load_time);

    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let results = Arc::clone(results);
        let keys_tested = Arc::clone(keys_tested);
        let stop_flag = Arc::clone(stop_flag);
        let output_file = output_file.to_string();
        let start_time = *start;
        let progress = thread_progress.as_ref().map(|p| Arc::clone(p));
        let fi = Arc::clone(&flat_index);

        // Seed each thread's RNG with true entropy
        let seed = {
            let mut s = [0u8; 32];
            getrandom::getrandom(&mut s).expect("Failed to get random bytes");
            let tid_bytes = (thread_id as u64).to_le_bytes();
            for i in 0..8 { s[i] ^= tid_bytes[i]; }
            s
        };

        // Calculate per-thread starting key for sequential mode
        let thread_start_key = {
            let mut k = start_key;
            if !random {
                let offset = (thread_id as u64) * (batch_size as u64);
                add_offset_to_key(&mut k, offset);
            }
            k
        };

        let handle = thread::spawn(move || {
            let secp = Secp256k1::<All>::new();
            let network = Network::Bitcoin;

            let mut rng = if random { Some(StdRng::from_seed(seed)) } else { None };

            // Buffers: privkeys in, total_values out (8 bytes per key)
            let mut privkeys_buf = vec![0u8; batch_size * 32];
            let mut total_values = vec![0u64; batch_size];
            let mut local_count = 0u64;
            let mut key = thread_start_key; // Sequential start with per-thread offset

            loop {
                if stop_flag.load(Ordering::Relaxed) { break; }

                for _ in 0..batch_size {
                    if max_keys > 0 && keys_tested.load(Ordering::Relaxed) >= max_keys {
                        break;
                    }

                    // Generate key: random or sequential
                    let mut key_buf = [0u8; 32];
                    if random {
                        rng.as_mut().unwrap().fill(&mut key_buf);
                    } else {
                        increment_key(&mut key);
                        key_buf = key;
                    }

                    if key_buf.iter().all(|&b| b == 0) { continue; }

                    // Count keys as generated
                    keys_tested.fetch_add(1, Ordering::Relaxed);

                    // Store in batch buffer
                    let off = (local_count as usize) * 32;
                    privkeys_buf[off..off+32].copy_from_slice(&key_buf);
                    local_count += 1;

                    // Process batch when full
                    if local_count >= batch_size as u64 {
                        let count = local_count as usize;

                        // GPU: derive pubkey + hash + binary search + UTXO sum (ALL on GPU)
                        if btcsolver::gpu::gpu_derive_lookup_single(
                            &privkeys_buf[..count * 32],
                            &mut total_values[..count],
                            count,
                            addr_types,
                        ) == 0 {
                            // Check for matches (total_values[i] > 0 means UTXO found)
                            for i in 0..count {
                                let val = total_values[i];
                                if val > 0 {
                                    // Re-derive on CPU to get address details
                                    let key_bytes = &privkeys_buf[i*32..(i+1)*32];
                                    if let Ok(secp_key) = bitcoin::secp256k1::SecretKey::from_slice(key_bytes) {
                                        let pk = PrivateKey {
                                            inner: secp_key, network: network.into(), compressed: true,
                                        };
                                        let pubkey = pk.public_key(&secp);
                                        let compressed = CompressedPublicKey::from_private_key(&secp, &pk).ok();
                                        let xonly: Option<UntweakedPublicKey> = compressed.as_ref().map(|c| UntweakedPublicKey::from(c.clone()));

                                        let mut matched_addrs: Vec<String> = Vec::new();

                                        if addr_types & 0x01 != 0 {
                                            let addr = bitcoin::Address::p2pkh(pubkey, network);
                                            let s = addr.script_pubkey();
                                            let v = fi.lookup(s.as_bytes());
                                            if v > 0 { matched_addrs.push(format!("{} [legacy]", addr)); }
                                        }
                                        if addr_types & 0x02 != 0 {
                                            if let Some(comp) = &compressed {
                                                let addr = bitcoin::Address::p2wpkh(comp, network);
                                                let s = addr.script_pubkey();
                                                let v = fi.lookup(s.as_bytes());
                                                if v > 0 { matched_addrs.push(format!("{} [segwit]", addr)); }
                                            }
                                        }
                                        if addr_types & 0x04 != 0 {
                                            if let Some(comp) = &compressed {
                                                let addr = bitcoin::Address::p2shwpkh(comp, network);
                                                let s = addr.script_pubkey();
                                                let v = fi.lookup(s.as_bytes());
                                                if v > 0 { matched_addrs.push(format!("{} [wrapped]", addr)); }
                                            }
                                        }
                                        if addr_types & 0x08 != 0 {
                                            if let Some(xo) = &xonly {
                                                let addr = bitcoin::Address::p2tr(&secp, *xo, None, network);
                                                let s = addr.script_pubkey();
                                                let v = fi.lookup(s.as_bytes());
                                                if v > 0 { matched_addrs.push(format!("{} [taproot]", addr)); }
                                            }
                                        }

                                        let key_hex = hex::encode(key_bytes);
                                        let result = BalanceResult {
                                            key_hex: key_hex.clone(), wif: None, sats: val,
                                            btc: val as f64 / 100_000_000.0,
                                            addresses: matched_addrs.clone(),
                                            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                        };
                                        results.lock().unwrap().push(result.clone());
                                        save_found_keys(&output_file, &vec![result]).ok();
                                        if stop_on_match {
                                            eprintln!("\n\n  BALANCE FOUND! Key: {} Balance: {} sats", key_hex, val);
                                            stop_flag.store(true, Ordering::Relaxed);
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        // Progress report
                        let total = keys_tested.load(Ordering::Relaxed);
                        let elapsed = start_time.elapsed();
                        let rate = if elapsed.as_secs_f64() > 0.0 { total as f64 / elapsed.as_secs_f64() } else { 0.0 };
                        eprintln!("[GPU-Lookup-Thread {}] {} keys | {:.0} keys/sec | {} matches",
                            thread_id, total, rate, results.lock().unwrap().len());
                        local_count = 0;
                    }
                }

                // Flush remaining keys
                if local_count > 0 {
                    let count = local_count as usize;
                    if btcsolver::gpu::gpu_derive_lookup_single(
                        &privkeys_buf[..count * 32],
                        &mut total_values[..count],
                        count,
                        addr_types,
                    ) == 0 {
                        for i in 0..count {
                            let val = total_values[i];
                            if val > 0 {
                                let key_bytes = &privkeys_buf[i*32..(i+1)*32];
                                let key_hex = hex::encode(key_bytes);
                                eprintln!("\n\n  BALANCE FOUND! Key: {} Balance: {} sats", key_hex, val);
                                let result = BalanceResult {
                                    key_hex: key_hex.clone(), wif: None, sats: val,
                                    btc: val as f64 / 100_000_000.0,
                                    addresses: vec!["GPU match - verify on CPU".to_string()],
                                    timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                };
                                results.lock().unwrap().push(result.clone());
                                save_found_keys(&output_file, &vec![result]).ok();
                                if stop_on_match { stop_flag.store(true, Ordering::Relaxed); break; }
                            }
                        }
                    }
                    local_count = 0;
                }

                // Update progress file data
                if let Some(ref prog) = progress {
                    if let Ok(mut guard) = prog.try_lock() {
                        let mut idx = guard.len();
                        while idx <= thread_id {
                            guard.push(ThreadProgress {
                                thread_id: idx, keys_tested: 0,
                                last_key_hex: "".to_string(),
                                mode: if random { "random".to_string() } else { "sequential".to_string() },
                            });
                            idx += 1;
                        }
                        // Update last_key_hex for this thread
                        let entry = &mut guard[thread_id];
                        entry.last_key_hex = hex::encode(&key);
                        entry.keys_tested = keys_tested.load(Ordering::Relaxed);
                    }
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }

    // Unload index from GPU
    btcsolver::gpu::gpu_unload_index();
}

/// Fallback: derive-only GPU mode (no index on GPU, CPU does lookups)
fn run_gpu_bruteforce_fallback(
    flat_index: Arc<FlatIndex>,
    addr_flags: AddrFlags,
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
    thread_progress: &Option<Arc<Mutex<Vec<ThreadProgress>>>>,
    _start_key: [u8; 32],
) {
    // Uses the old derive-only mode (gpu_derive + CPU lookup)
    // This is the same as the original run_gpu_bruteforce implementation
    run_cpu_bruteforce(
        flat_index, addr_flags, max_keys, num_threads, batch_size,
        results, keys_tested, stop_flag, output_file, start, random, stop_on_match, thread_progress,
        [0u8; 32], // fallback doesn't have start_key context
    );
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

/// Add a u64 offset to a 32-byte big-endian key (for per-thread sequential offsets)
fn add_offset_to_key(key: &mut [u8; 32], offset: u64) {
    let bytes = offset.to_be_bytes();
    let mut carry: u16 = if bytes[0] != 0 { 1 } else { 0 };
    // Only need to add to the last 8 bytes (big-endian)
    for (i, &b) in bytes.iter().enumerate().skip(24) {
        let idx = i; // 24..31
        let sum = key[idx] as u16 + b as u16 + carry;
        key[idx] = sum as u8;
        carry = if sum > 255 { 1 } else { 0 };
    }
    // Propagate carry through remaining upper bytes if needed
    if carry != 0 {
        for byte in key[..24].iter_mut().rev() {
            if *byte == u8::MAX {
                *byte = 0;
            } else {
                *byte += 1;
                break;
            }
        }
    }
}

/// Try to resume from a position file. Returns (start_key, keys_already_tested).
fn try_resume_position(progress_file: &str, start_key: &[u8; 32]) -> ([u8; 32], u64) {
    // Try the simple position file first
    let pos_file = progress_file.replace(".json", ".position");
    if let Ok(content) = std::fs::read_to_string(&pos_file) {
        let content = content.trim();
        // Format: "sequential <hex_key> <keys_tested>"
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 3 && parts[0] == "sequential" {
            if let Ok(last_key) = hex::decode(parts[1]) {
                if let Ok(key_arr) = <[u8; 32]>::try_from(last_key) {
                    let keys_tested: u64 = parts[2].parse().unwrap_or(0);
                    let mut resume_key = key_arr;
                    increment_key(&mut resume_key);
                    eprintln!("[RESUME] Position file found: resuming from {} ({} keys already tested)",
                        hex::encode(&resume_key), keys_tested);
                    return (resume_key, keys_tested);
                }
            }
        }
    }

    // Fall back to the JSON progress file
    if let Ok(content) = std::fs::read_to_string(progress_file) {
        if let Ok(pf) = serde_json::from_str::<ProgressFile>(&content) {
            if pf.mode == "sequential" {
                // Find the max last_key_hex among all threads
                let mut max_key = [0u8; 32];
                for tp in &pf.threads {
                    if let Ok(key_bytes) = hex::decode(&tp.last_key_hex) {
                        if let Ok(key_arr) = <[u8; 32]>::try_from(key_bytes) {
                            if key_arr > max_key {
                                max_key = key_arr;
                            }
                        }
                    }
                }
                if max_key != [0u8; 32] {
                    let mut resume_key = max_key;
                    increment_key(&mut resume_key);
                    eprintln!("[RESUME] Progress file found: resuming from {} ({} keys already tested)",
                        hex::encode(&resume_key), pf.total_keys_tested);
                    return (resume_key, pf.total_keys_tested);
                }
            }
        }
    }

    // No resume data found
    (*start_key, 0)
}

/// Write a simple position file for easy resume
fn write_position_file(progress_file: &str, mode: &str, current_key: &[u8; 32], keys_tested: u64) {
    let pos_file = progress_file.replace(".json", ".position");
    let content = format!("sequential {} {}\n", hex::encode(current_key), keys_tested);
    std::fs::write(&pos_file, &content).ok();
}
