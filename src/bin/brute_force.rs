use anyhow::Result;
use bitcoin::key::{CompressedPublicKey, PrivateKey, UntweakedPublicKey};
use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::Network;
use clap::Parser;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

/// Compteurs live pour stats (vitesses par GPU + CPU) — partagés workers ↔ writer
struct LiveRateCounters {
    /// IDs CUDA utilisés (ordre = index dans gpu_tested)
    gpu_ids: Mutex<Vec<i32>>,
    gpu_tested: [AtomicU64; 16],
    cpu_tested: AtomicU64,
    cpu_threads: AtomicUsize,
}

impl LiveRateCounters {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            gpu_ids: Mutex::new(Vec::new()),
            gpu_tested: std::array::from_fn(|_| AtomicU64::new(0)),
            cpu_tested: AtomicU64::new(0),
            cpu_threads: AtomicUsize::new(0),
        })
    }

    fn set_gpus(&self, ids: &[i32]) {
        if let Ok(mut g) = self.gpu_ids.lock() {
            *g = ids.to_vec();
        }
        for a in &self.gpu_tested {
            a.store(0, Ordering::Relaxed);
        }
        self.cpu_tested.store(0, Ordering::Relaxed);
    }

    fn add_gpu(&self, slot: usize, n: u64) {
        if slot < self.gpu_tested.len() {
            self.gpu_tested[slot].fetch_add(n, Ordering::Relaxed);
        }
    }

    fn add_cpu(&self, n: u64) {
        self.cpu_tested.fetch_add(n, Ordering::Relaxed);
    }
}

use btcsolver::flat_index::FlatIndex;
use btcsolver::key_archive::{ArchivedKey, KeyArchive};

/// Key-space transforms applied to each base key before UTXO lookup
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyTransform {
    Identity,
    ReverseBytes,
    ReverseBits,
    Rotl8,
    Rotr8,
    Sha256,
    DoubleSha256,
}

impl KeyTransform {
    fn name(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::ReverseBytes => "reverse_bytes",
            Self::ReverseBits => "reverse_bits",
            Self::Rotl8 => "rotl8",
            Self::Rotr8 => "rotr8",
            Self::Sha256 => "sha256",
            Self::DoubleSha256 => "double_sha256",
        }
    }

    fn apply(self, key: &[u8; 32]) -> [u8; 32] {
        match self {
            Self::Identity => *key,
            Self::ReverseBytes => {
                let mut o = *key;
                o.reverse();
                o
            }
            Self::ReverseBits => {
                let mut o = [0u8; 32];
                for i in 0..256 {
                    let src_byte = i / 8;
                    let src_bit = 7 - (i % 8);
                    if (key[src_byte] >> src_bit) & 1 == 1 {
                        let dst = 255 - i;
                        let dst_byte = dst / 8;
                        let dst_bit = 7 - (dst % 8);
                        o[dst_byte] |= 1 << dst_bit;
                    }
                }
                o
            }
            Self::Rotl8 => {
                let mut o = [0u8; 32];
                o[..31].copy_from_slice(&key[1..]);
                o[31] = key[0];
                o
            }
            Self::Rotr8 => {
                let mut o = [0u8; 32];
                o[0] = key[31];
                o[1..].copy_from_slice(&key[..31]);
                o
            }
            Self::Sha256 => {
                let h = Sha256::digest(key);
                let mut o = [0u8; 32];
                o.copy_from_slice(&h);
                o
            }
            Self::DoubleSha256 => {
                let h1 = Sha256::digest(key);
                let h2 = Sha256::digest(&h1);
                let mut o = [0u8; 32];
                o.copy_from_slice(&h2);
                o
            }
        }
    }
}

fn parse_transforms(s: &str) -> Vec<KeyTransform> {
    let mut out = Vec::new();
    for part in s.split(',') {
        let p = part.trim().to_lowercase();
        if p.is_empty() {
            continue;
        }
        let t = match p.as_str() {
            "identity" | "none" | "raw" => KeyTransform::Identity,
            "reverse_bytes" | "rev_bytes" | "byte_reverse" | "reverse" => KeyTransform::ReverseBytes,
            "reverse_bits" | "rev_bits" | "bit_reverse" => KeyTransform::ReverseBits,
            "rotl8" | "rotl" | "rotate_left" | "rol8" => KeyTransform::Rotl8,
            "rotr8" | "rotr" | "rotate_right" | "ror8" => KeyTransform::Rotr8,
            "sha256" | "sha" => KeyTransform::Sha256,
            "double_sha256" | "sha256d" | "double" => KeyTransform::DoubleSha256,
            other => {
                eprintln!("WARNING: unknown transform '{}', ignored", other);
                continue;
            }
        };
        if !out.contains(&t) {
            out.push(t);
        }
    }
    if out.is_empty() {
        out.push(KeyTransform::Identity);
    }
    out
}

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

    /// Number of CPU threads (0 = auto via --cpu-pct)
    #[arg(short, long, default_value = "0")]
    threads: usize,

    /// % des cœurs logiques pour workers CPU quand --threads 0 (défaut 50). 0 = pas de CPU.
    #[arg(long, default_value = "50")]
    cpu_pct: u32,

    /// Start key (hex, 32 bytes). Default: 0000...0001
    #[arg(short, long)]
    start: Option<String>,

    /// End key exclusive (hex 64) — stop séquentiel quand la clé de base >= end
    #[arg(long)]
    end: Option<String>,

    /// Number of keys to test
    #[arg(long, default_value = "0")]
    count: u64, // 0 = unlimited

    /// Ne pas reprendre depuis brute-force-progress (.position)
    #[arg(long)]
    no_resume: bool,

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

    /// Key transforms applied to each base key (comma-separated).
    /// identity,reverse_bytes,reverse_bits,rotl8,rotr8,sha256,double_sha256
    #[arg(long, default_value = "identity")]
    transforms: String,

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
    // Compteur de session repart à 0 (évite les stats absurdes type 10^12 si fichier corrompu).
    let mut keys_already_tested: u64 = 0;
    if !cli.random && !cli.no_resume {
        let (resumed_key, already_tested) = try_resume_position(&cli.progress_file, &start_key);
        start_key = resumed_key;
        // plafonner un compteur de reprise aberrant
        keys_already_tested = already_tested.min(1_000_000_000_000);
    } else if cli.no_resume {
        eprintln!("[RANGE] --no-resume : départ forcé {}", hex::encode(&start_key));
    }

    // Fin de fenêtre exclusive (optionnel)
    let end_key: Option<[u8; 32]> = if let Some(ref e) = cli.end {
        let bytes = hex::decode(e.trim()).map_err(|err| anyhow::anyhow!("Invalid --end hex: {}", err))?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("--end must be 32 bytes (64 hex)"))?;
        eprintln!("[RANGE] end exclusive: {}", hex::encode(&arr));
        Some(arr)
    } else {
        None
    };

    // Shared state — compteur = clés de CETTE session (reprise = position only)
    let results = Arc::new(Mutex::new(Vec::new()));
    let keys_tested = Arc::new(AtomicU64::new(0));
    let stop_flag = Arc::new(AtomicBool::new(false));
    let found_keys_file = cli.output_file.clone();

    // Workers CPU : --threads N prioritaire, sinon --cpu-pct % des cœurs (défaut 50).
    let logical = num_cpus::get().max(1);
    let cpu_pct = cli.cpu_pct.min(100);
    let mut num_threads = if cli.threads > 0 {
        cli.threads
    } else if cpu_pct == 0 {
        0
    } else {
        (logical * cpu_pct as usize / 100).max(1)
    };
    // Mode CPU pur : au moins 1 worker
    if !cli.use_gpu && num_threads == 0 {
        num_threads = 1;
    }
    if cli.threads > 0 {
        println!(
            "  CPU workers: {} (forcé --threads, {} cœurs logiques)",
            num_threads, logical
        );
    } else {
        println!(
            "  CPU workers: {} ({}% de {} cœurs logiques)",
            num_threads, cpu_pct, logical
        );
    }
    if cli.use_gpu {
        println!("  GPU acceleration: ENABLED (CUDA)");
    }
    println!();
    let batch_size = cli.batch_size;
    let live_rates = LiveRateCounters::new();

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
    let transforms = parse_transforms(&cli.transforms);
    let transform_names: Vec<String> = transforms.iter().map(|t| t.name().to_string()).collect();
    println!("  Batch size: {}", batch_size);
    println!("  Address types: {}", addr_flags.to_display());
    println!("  Transforms: {}", transform_names.join(", "));
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

    let mode_label = if cli.random { "random" } else { "sequential" }.to_string();
    let start_key_hex = hex::encode(&start_key);
    let progress_file_for_stats = cli.progress_file.clone();

    // Launch live stats writer thread
    let stats_handle = if cli.stats_interval > 0 {
        let stats_file = cli.stats_file.clone();
        let keys_tested = Arc::clone(&keys_tested);
        let stop_flag = Arc::clone(&stop_flag);
        let results = Arc::clone(&results);
        let live_rates = Arc::clone(&live_rates);
        let start_instant = Instant::now();
        let transform_names_stats = transform_names.clone();
        let mode_stats = mode_label.clone();
        let start_hex_stats = start_key_hex.clone();
        let progress_path = progress_file_for_stats.clone();
        let interval = cli.stats_interval;
        Some(thread::spawn(move || {
            let mut prev_total = 0u64;
            let mut prev_gpu = [0u64; 16];
            let mut prev_cpu = 0u64;
            let mut prev_tick = Instant::now();
            loop {
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }
                thread::sleep(std::time::Duration::from_secs(interval));
                let total = keys_tested.load(Ordering::Relaxed);
                let elapsed = start_instant.elapsed();
                let elapsed_s = elapsed.as_secs_f64().max(0.001);
                let avg_rate = total as f64 / elapsed_s;
                // Vitesse instantanée sur la dernière fenêtre (prouve que ce n'est pas figé)
                let dt = prev_tick.elapsed().as_secs_f64().max(0.001);
                let live_rate = (total.saturating_sub(prev_total)) as f64 / dt;
                prev_total = total;
                prev_tick = Instant::now();
                let found = results.lock().unwrap().len();
                let now = chrono::Local::now();
                let stats_updated_at = now.format("%H:%M:%S").to_string();
                let timestamp = now.to_rfc3339();

                let gpu_ids = live_rates
                    .gpu_ids
                    .lock()
                    .map(|g| g.clone())
                    .unwrap_or_default();
                let mut gpu_rates = Vec::new();
                for (i, &id) in gpu_ids.iter().enumerate() {
                    if i >= 16 {
                        break;
                    }
                    let k = live_rates.gpu_tested[i].load(Ordering::Relaxed);
                    let inst = (k.saturating_sub(prev_gpu[i])) as f64 / dt;
                    prev_gpu[i] = k;
                    gpu_rates.push(serde_json::json!({
                        "id": id,
                        "keys_tested": k,
                        "keys_per_sec": inst.round() as u64,
                        "keys_per_sec_avg": (k as f64 / elapsed_s).round() as u64,
                    }));
                }
                let cpu_k = live_rates.cpu_tested.load(Ordering::Relaxed);
                let cpu_inst = (cpu_k.saturating_sub(prev_cpu)) as f64 / dt;
                prev_cpu = cpu_k;
                let cpu_threads = live_rates.cpu_threads.load(Ordering::Relaxed);

                // Derive live range from progress file thread keys
                let mut range_start = if mode_stats == "sequential" {
                    start_hex_stats.clone()
                } else {
                    String::new()
                };
                let mut range_end = String::new();
                let mut current_pos = String::new();
                let mut thread_keys: Vec<String> = Vec::new();
                if let Ok(content) = std::fs::read_to_string(&progress_path) {
                    if let Ok(p) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(threads) = p.get("threads").and_then(|v| v.as_array()) {
                            let mut keys: Vec<String> = threads
                                .iter()
                                .filter_map(|t| {
                                    t.get("last_key_hex")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_lowercase())
                                        .filter(|s| s.len() == 64)
                                })
                                .collect();
                            keys.sort();
                            if let Some(first) = keys.first() {
                                if mode_stats == "random" || range_start.is_empty() {
                                    range_start = first.clone();
                                }
                            }
                            if let Some(last) = keys.last() {
                                range_end = last.clone();
                                current_pos = last.clone();
                            }
                            thread_keys = keys.into_iter().take(12).collect();
                        }
                    }
                }
                if range_start.is_empty() {
                    range_start = start_hex_stats.clone();
                }
                if range_end.is_empty() {
                    range_end = current_pos.clone();
                }
                if current_pos.is_empty() {
                    current_pos = range_end.clone();
                }

                // Prefer live (window) rate for UI; keep avg as secondary
                let display_rate = if live_rate > 0.0 {
                    live_rate
                } else {
                    avg_rate
                };

                let stats = serde_json::json!({
                    "keys_tested": total,
                    "keys_per_sec": display_rate.round() as u64,
                    "keys_per_sec_avg": avg_rate.round() as u64,
                    "keys_per_sec_live": live_rate.round() as u64,
                    "matches_found": found,
                    "elapsed_seconds": elapsed.as_secs(),
                    "elapsed_human": format!("{:?}", elapsed),
                    "timestamp": timestamp,
                    "stats_updated_at": stats_updated_at,
                    "mode": mode_stats,
                    "range_start": range_start,
                    "range_end": range_end,
                    "current_position": current_pos,
                    "start_key": start_hex_stats,
                    "transforms": transform_names_stats,
                    "thread_keys": thread_keys,
                    "gpu_rates": gpu_rates,
                    "cpu_keys_tested": cpu_k,
                    "cpu_keys_per_sec": cpu_inst.round() as u64,
                    "cpu_threads": cpu_threads,
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

    // Parse --gpus 0,1,2 (default = all detected)
    let gpu_ids: Vec<i32> = if let Some(ref s) = cli.gpus {
        s.split(',')
            .filter_map(|x| x.trim().parse::<i32>().ok())
            .collect()
    } else {
        let n = btcsolver::gpu::gpu_device_count().max(0);
        (0..n).collect()
    };
    if use_gpu && !gpu_ids.is_empty() {
        eprintln!("[GPU] Using device IDs: {:?}", gpu_ids);
        live_rates.set_gpus(&gpu_ids);
        live_rates
            .cpu_threads
            .store(num_threads, Ordering::Relaxed);
    }

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
            transforms.clone(),
            gpu_ids,
            Arc::clone(&live_rates),
            end_key,
        );
        btcsolver::gpu::gpu_cleanup();
    } else {
        live_rates
            .cpu_threads
            .store(num_threads, Ordering::Relaxed);
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
            transforms.clone(),
            Some(Arc::clone(&live_rates)),
            end_key,
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
    transforms: Vec<KeyTransform>,
    live_rates: Option<Arc<LiveRateCounters>>,
    end_key: Option<[u8; 32]>,
) {
    // Stride partition (évite chevauchement multi-thread séquentiel)
    let stride = num_threads.max(1) as u64;
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let flat_index = Arc::clone(&flat_index);
        let results = Arc::clone(results);
        let keys_tested = Arc::clone(keys_tested);
        let stop_flag = Arc::clone(stop_flag);
        let output_file = output_file.to_string();
        let start_time = *start;
        let progress = thread_progress.as_ref().map(|p| Arc::clone(p));
        let transforms = transforms.clone();
        let live_rates = live_rates.clone();
        let progress_slot = thread_id;
        let end_key = end_key;

        let seed = {
            let mut s = [0u8; 32];
            getrandom::getrandom(&mut s).expect("Failed to get random bytes for seeding");
            let tid_bytes = (thread_id as u64).to_le_bytes();
            for i in 0..8 {
                s[i] ^= tid_bytes[i];
            }
            s
        };

        let thread_start_key = {
            let mut k = start_key;
            if !random {
                add_offset_to_key(&mut k, thread_id as u64);
            }
            k
        };

        let handle = thread::spawn(move || {
            run_one_cpu_worker(
                flat_index,
                addr_flags,
                max_keys,
                batch_size,
                results,
                keys_tested,
                stop_flag,
                output_file,
                start_time,
                random,
                stop_on_match,
                progress,
                progress_slot,
                thread_start_key,
                transforms,
                stride,
                live_rates,
                true, // count toward cpu_tested
                seed,
                thread_id,
                end_key,
            );
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }
}

/// Worker CPU unique (chemin CPU pur ou hybride GPU+CPU)
fn run_one_cpu_worker(
    flat_index: Arc<FlatIndex>,
    addr_flags: AddrFlags,
    max_keys: u64,
    batch_size: usize,
    results: Arc<Mutex<Vec<BalanceResult>>>,
    keys_tested: Arc<AtomicU64>,
    stop_flag: Arc<AtomicBool>,
    output_file: String,
    start_time: Instant,
    random: bool,
    stop_on_match: bool,
    progress: Option<Arc<Mutex<Vec<ThreadProgress>>>>,
    progress_slot: usize,
    thread_start_key: [u8; 32],
    transforms: Vec<KeyTransform>,
    stride: u64,
    live_rates: Option<Arc<LiveRateCounters>>,
    count_as_cpu: bool,
    seed: [u8; 32],
    log_id: usize,
    end_key: Option<[u8; 32]>,
) {
    let secp = Secp256k1::<All>::new();
    let network = Network::Bitcoin;
    let mut rng = if random {
        Some(StdRng::from_seed(seed))
    } else {
        None
    };
    let mut key = thread_start_key;
    let mut local_count = 0u64;

    loop {
        if stop_flag.load(Ordering::Relaxed) {
            break;
        }

        for _ in 0..batch_size {
            if max_keys > 0 && keys_tested.load(Ordering::Relaxed) >= max_keys {
                return;
            }
            if let Some(ref ek) = end_key {
                if !random && key.as_slice() >= ek.as_slice() {
                    stop_flag.store(true, Ordering::Relaxed);
                    return;
                }
            }

            if random {
                let r = rng.as_mut().unwrap();
                let mut candidate = [0u8; 32];
                r.fill(&mut candidate);
                key = candidate;
            } else {
                // key already points to this worker's slot; advance by stride after use
            }

            let base = key;
            if let Some(ref ek) = end_key {
                if !random && base.as_slice() >= ek.as_slice() {
                    stop_flag.store(true, Ordering::Relaxed);
                    return;
                }
            }
            if !random {
                add_offset_to_key(&mut key, stride.max(1));
            }

            for transform in &transforms {
                let try_key = transform.apply(&base);
                if try_key.iter().all(|&b| b == 0) {
                    continue;
                }

                let secp_key = match bitcoin::secp256k1::SecretKey::from_slice(&try_key) {
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

                let mut total_sats = 0u64;
                let mut matched_addrs: Vec<String> = Vec::new();

                if addr_flags.has(AddrFlags::LEGACY) {
                    let addr = bitcoin::Address::p2pkh(pubkey, network);
                    let val = flat_index.lookup(addr.script_pubkey().as_bytes());
                    if val > 0 {
                        total_sats += val;
                        matched_addrs.push(format!("{} [legacy]", addr));
                    }
                }
                if addr_flags.has(AddrFlags::SEGWIT) {
                    let addr = bitcoin::Address::p2wpkh(&compressed, network);
                    let val = flat_index.lookup(addr.script_pubkey().as_bytes());
                    if val > 0 {
                        total_sats += val;
                        matched_addrs.push(format!("{} [segwit]", addr));
                    }
                }
                if addr_flags.has(AddrFlags::WRAPPED) {
                    let addr = bitcoin::Address::p2shwpkh(&compressed, network);
                    let val = flat_index.lookup(addr.script_pubkey().as_bytes());
                    if val > 0 {
                        total_sats += val;
                        matched_addrs.push(format!("{} [wrapped]", addr));
                    }
                }
                if addr_flags.has(AddrFlags::TAPROOT) {
                    let addr = bitcoin::Address::p2tr(&secp, xonly, None, network);
                    let val = flat_index.lookup(addr.script_pubkey().as_bytes());
                    if val > 0 {
                        total_sats += val;
                        matched_addrs.push(format!("{} [taproot]", addr));
                    }
                }

                if total_sats > 0 {
                    let key_hex = hex::encode(&try_key);
                    let wif = Some(
                        PrivateKey {
                            inner: secp_key,
                            network: network.into(),
                            compressed: true,
                        }
                        .to_wif(),
                    );
                    let result = BalanceResult {
                        key_hex: key_hex.clone(),
                        wif,
                        sats: total_sats,
                        btc: total_sats as f64 / 100_000_000.0,
                        addresses: matched_addrs.clone(),
                        timestamp: chrono::Local::now()
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string(),
                    };
                    results.lock().unwrap().push(result.clone());
                    save_found_keys(&output_file, &vec![result]).ok();
                    if stop_on_match {
                        eprintln!("\n\n BALANCE FOUND! Stopping all threads...");
                        eprintln!("  Key: {} | {} sats", key_hex, total_sats);
                        stop_flag.store(true, Ordering::Relaxed);
                        break;
                    }
                }
                local_count += 1;
            }
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }
        }

        keys_tested.fetch_add(local_count, Ordering::Relaxed);
        if count_as_cpu {
            if let Some(ref lr) = live_rates {
                lr.add_cpu(local_count);
            }
        }

        if let Some(ref prog) = progress {
            if let Ok(mut guard) = prog.lock() {
                while guard.len() <= progress_slot {
                    let idx = guard.len();
                    guard.push(ThreadProgress {
                        thread_id: idx,
                        keys_tested: 0,
                        last_key_hex: String::new(),
                        mode: if random {
                            "random".into()
                        } else {
                            "sequential".into()
                        },
                    });
                }
                guard[progress_slot].keys_tested += local_count;
                guard[progress_slot].last_key_hex = hex::encode(&key);
            }
        }

        let total = keys_tested.load(Ordering::Relaxed);
        let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
        if log_id == 0 {
            eprintln!(
                "[CPU{}] {} keys | {:.0} keys/sec | {} matches",
                log_id,
                total,
                total as f64 / elapsed,
                results.lock().unwrap().len()
            );
        }
        local_count = 0;
    }
}

fn serialize_script_entries_for_gpu(fi: &FlatIndex) -> Vec<u8> {
    fi.serialize_script_entries_for_gpu()
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
    cpu_workers: usize,
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
    transforms: Vec<KeyTransform>,
    gpu_ids: Vec<i32>,
    live_rates: Arc<LiveRateCounters>,
    end_key: Option<[u8; 32]>,
) {
    let addr_types = addr_flags_to_gpu_bits(addr_flags);

    // Prefer FULL GPU (index in VRAM + derive+lookup on device) — eliminates CPU lookup pauses.
    // Set BTC_GPU_DERIVE_ONLY=1 to force old path. BTC_GPU_FULL=0 also forces derive-only.
    let force_derive = std::env::var("BTC_GPU_DERIVE_ONLY").map(|v| v == "1").unwrap_or(false)
        || std::env::var("BTC_GPU_FULL").map(|v| v == "0").unwrap_or(false);
    let try_full = !force_derive;
    if try_full {
        let gpu_entries = serialize_script_entries_for_gpu(&flat_index);
        let num_entries = flat_index.script_entries.len() as u32;
        eprintln!(
            "[GPU] FULL auto — loading FlatIndex (~{:.0} MB/GPU: entries {:.0} + data {:.0} + utxo {:.0})...",
            flat_index.gpu_index_bytes() as f64 / 1_048_576.0,
            gpu_entries.len() as f64 / 1_048_576.0,
            flat_index.all_data.len() as f64 / 1_048_576.0,
            flat_index.utxo_data.len() as f64 / 1_048_576.0
        );
        let load_rc = btcsolver::gpu::gpu_load_index(
            &gpu_entries,
            &flat_index.all_data,
            &flat_index.utxo_data,
            num_entries,
        );
        if load_rc != 0 {
            eprintln!("[GPU] Full index load failed (rc={}) — DERIVE-ONLY fallback", load_rc);
            run_gpu_bruteforce_fallback(
                flat_index,
                addr_flags,
                max_keys,
                cpu_workers,
                batch_size,
                results,
                keys_tested,
                stop_flag,
                output_file,
                start,
                random,
                stop_on_match,
                thread_progress,
                start_key,
                transforms,
                gpu_ids,
                live_rates,
                end_key,
            );
            return;
        }
        eprintln!("[GPU] Full index loaded — derive+lookup ON DEVICE + CPU auxiliaire");
    } else {
        eprintln!("[GPU] DERIVE-ONLY (BTC_GPU_DERIVE_ONLY=1 or BTC_GPU_FULL=0)");
        run_gpu_bruteforce_fallback(
            flat_index,
            addr_flags,
            max_keys,
            cpu_workers,
            batch_size,
            results,
            keys_tested,
            stop_flag,
            output_file,
            start,
            random,
            stop_on_match,
            thread_progress,
            start_key,
            transforms,
            gpu_ids,
            live_rates,
            end_key,
        );
        return;
    }

    // Multi-GPU = 1 host / carte + workers CPU (~50 % cœurs) en stride partagé.
    // BTC_GPU_LAUNCH : taille lot (défaut max(batch, 2M), max 8M).
    // BTC_CPU_WORKERS=0 pour désactiver le CPU hybride.
    let n_gpu = gpu_ids.len().max(1);
    let cpu_n = std::env::var("BTC_CPU_WORKERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(cpu_workers);
    let total_stride = (n_gpu + cpu_n).max(1) as u64;
    let gpu_batch = std::env::var("BTC_GPU_LAUNCH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(batch_size.max(2_097_152))
        .clamp(524_288, 8_388_608);
    live_rates.cpu_threads.store(cpu_n, Ordering::Relaxed);
    eprintln!(
        "[GPU] multi-GPU + CPU: hosts_gpu={} cpu_workers={} stride={} batch/GPU={} devices={:?}",
        n_gpu, cpu_n, total_stride, gpu_batch, gpu_ids
    );

    let mut handles = Vec::new();

    for thread_id in 0..n_gpu {
        let results = Arc::clone(results);
        let keys_tested = Arc::clone(keys_tested);
        let stop_flag = Arc::clone(stop_flag);
        let output_file = output_file.to_string();
        let start_time = *start;
        let progress = thread_progress.as_ref().map(|p| Arc::clone(p));
        let fi = Arc::clone(&flat_index);
        let transforms = transforms.clone();
        let my_gpu = gpu_ids[thread_id % n_gpu];
        let live_rates = Arc::clone(&live_rates);
        let gpu_slot = thread_id;
        let end_key = end_key;

        let seed = {
            let mut s = [0u8; 32];
            getrandom::getrandom(&mut s).expect("Failed to get random bytes");
            let tid_bytes = (thread_id as u64).to_le_bytes();
            for i in 0..8 {
                s[i] ^= tid_bytes[i];
            }
            s
        };

        // Séquentiel : GPU i → start+i, pas = n_gpu+cpu_n
        let thread_start_key = {
            let mut k = start_key;
            if !random {
                add_offset_to_key(&mut k, thread_id as u64);
            }
            k
        };

        let handle = thread::spawn(move || {
            let secp = Secp256k1::<All>::new();
            let network = Network::Bitcoin;
            let mut rng = if random {
                Some(StdRng::from_seed(seed))
            } else {
                None
            };
            let mut privkeys_buf = vec![0u8; gpu_batch * 32];
            let mut total_values = vec![0u64; gpu_batch];
            let mut key = thread_start_key;
            let dev_ids = [my_gpu];
            let mut batches = 0u64;

            loop {
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }
                if max_keys > 0 && keys_tested.load(Ordering::Relaxed) >= max_keys {
                    break;
                }
                if let Some(ref ek) = end_key {
                    if !random && key.as_slice() >= ek.as_slice() {
                        stop_flag.store(true, Ordering::Relaxed);
                        break;
                    }
                }

                let mut count = 0usize;
                while count < gpu_batch {
                    if stop_flag.load(Ordering::Relaxed) {
                        break;
                    }
                    let mut base = [0u8; 32];
                    if random {
                        rng.as_mut().unwrap().fill(&mut base);
                        key = base;
                    } else {
                        if let Some(ref ek) = end_key {
                            if key.as_slice() >= ek.as_slice() {
                                stop_flag.store(true, Ordering::Relaxed);
                                break;
                            }
                        }
                        base = key;
                        add_offset_to_key(&mut key, total_stride);
                    }
                    if base.iter().all(|&b| b == 0) {
                        continue;
                    }
                    for transform in &transforms {
                        if count >= gpu_batch {
                            break;
                        }
                        let key_buf = transform.apply(&base);
                        if key_buf.iter().all(|&b| b == 0) {
                            continue;
                        }
                        let off = count * 32;
                        privkeys_buf[off..off + 32].copy_from_slice(&key_buf);
                        count += 1;
                    }
                }
                if count == 0 {
                    break;
                }

                // FULL: derive+lookup sur CETTE carte (parallèle avec les autres hosts)
                let rc = btcsolver::gpu::gpu_derive_lookup(
                    &privkeys_buf[..count * 32],
                    &mut total_values[..count],
                    count,
                    addr_types,
                    &dev_ids,
                );

                if rc == 0 {
                    keys_tested.fetch_add(count as u64, Ordering::Relaxed);
                    live_rates.add_gpu(gpu_slot, count as u64);
                    for i in 0..count {
                        let val = total_values[i];
                        if val == 0 {
                            continue;
                        }
                        let key_bytes = &privkeys_buf[i * 32..(i + 1) * 32];
                        if let Ok(secp_key) =
                            bitcoin::secp256k1::SecretKey::from_slice(key_bytes)
                        {
                            let pk = PrivateKey {
                                inner: secp_key,
                                network: network.into(),
                                compressed: true,
                            };
                            let pubkey = pk.public_key(&secp);
                            let compressed =
                                CompressedPublicKey::from_private_key(&secp, &pk).ok();
                            let xonly: Option<UntweakedPublicKey> = compressed
                                .as_ref()
                                .map(|c| UntweakedPublicKey::from(c.clone()));
                            let mut matched_addrs: Vec<String> = Vec::new();
                            if addr_types & 0x01 != 0 {
                                let addr = bitcoin::Address::p2pkh(pubkey, network);
                                if fi.lookup(addr.script_pubkey().as_bytes()) > 0 {
                                    matched_addrs.push(format!("{} [legacy]", addr));
                                }
                            }
                            if addr_types & 0x02 != 0 {
                                if let Some(comp) = &compressed {
                                    let addr = bitcoin::Address::p2wpkh(comp, network);
                                    if fi.lookup(addr.script_pubkey().as_bytes()) > 0 {
                                        matched_addrs.push(format!("{} [segwit]", addr));
                                    }
                                }
                            }
                            if addr_types & 0x04 != 0 {
                                if let Some(comp) = &compressed {
                                    let addr = bitcoin::Address::p2shwpkh(comp, network);
                                    if fi.lookup(addr.script_pubkey().as_bytes()) > 0 {
                                        matched_addrs.push(format!("{} [wrapped]", addr));
                                    }
                                }
                            }
                            if addr_types & 0x08 != 0 {
                                if let Some(xo) = &xonly {
                                    let addr = bitcoin::Address::p2tr(&secp, *xo, None, network);
                                    if fi.lookup(addr.script_pubkey().as_bytes()) > 0 {
                                        matched_addrs.push(format!("{} [taproot]", addr));
                                    }
                                }
                            }
                            if matched_addrs.is_empty() {
                                continue;
                            }
                            let key_hex = hex::encode(key_bytes);
                            let result = BalanceResult {
                                key_hex: key_hex.clone(),
                                wif: None,
                                sats: val,
                                btc: val as f64 / 100_000_000.0,
                                addresses: matched_addrs,
                                timestamp: chrono::Local::now()
                                    .format("%Y-%m-%d %H:%M:%S")
                                    .to_string(),
                            };
                            results.lock().unwrap().push(result.clone());
                            save_found_keys(&output_file, &vec![result]).ok();
                            eprintln!(
                                "\n\n  BALANCE FOUND! Key: {} Balance: {} sats",
                                key_hex, val
                            );
                            if stop_on_match {
                                stop_flag.store(true, Ordering::Relaxed);
                                break;
                            }
                        }
                    }
                } else {
                    eprintln!("[GPU{}] derive_lookup rc={} — retry", my_gpu, rc);
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }

                batches += 1;
                if thread_id == 0 && batches % 2 == 0 {
                    let total = keys_tested.load(Ordering::Relaxed);
                    let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                    eprintln!(
                        "[GPU-FULL×{}+CPU{}] {} keys | {:.0} keys/sec | {} matches | batch={} gpu{}",
                        n_gpu,
                        cpu_n,
                        total,
                        total as f64 / elapsed,
                        results.lock().unwrap().len(),
                        count,
                        my_gpu
                    );
                }

                // lock bloquant : le curseur De→À doit avancer (try_lock sautait trop sous charge)
                if let Some(ref prog) = progress {
                    if let Ok(mut guard) = prog.lock() {
                        while guard.len() <= thread_id {
                            let idx = guard.len();
                            guard.push(ThreadProgress {
                                thread_id: idx,
                                keys_tested: 0,
                                last_key_hex: String::new(),
                                mode: if random {
                                    "random".into()
                                } else {
                                    "sequential".into()
                                },
                            });
                        }
                        guard[thread_id].last_key_hex = hex::encode(&key);
                        guard[thread_id].keys_tested =
                            live_rates.gpu_tested[gpu_slot].load(Ordering::Relaxed);
                    }
                }
            }
        });
        handles.push(handle);
    }

    // Workers CPU (~50 %) : slots n_gpu .. n_gpu+cpu_n-1, même stride
    let cpu_batch = batch_size.min(4096).max(256);
    for j in 0..cpu_n {
        let flat_index = Arc::clone(&flat_index);
        let results = Arc::clone(results);
        let keys_tested = Arc::clone(keys_tested);
        let stop_flag = Arc::clone(stop_flag);
        let output_file = output_file.to_string();
        let start_time = *start;
        let progress = thread_progress.as_ref().map(|p| Arc::clone(p));
        let transforms = transforms.clone();
        let live_rates = Arc::clone(&live_rates);
        let progress_slot = n_gpu + j;
        let worker_offset = (n_gpu + j) as u64;

        let seed = {
            let mut s = [0u8; 32];
            getrandom::getrandom(&mut s).expect("rng");
            let tid_bytes = (progress_slot as u64).to_le_bytes();
            for i in 0..8 {
                s[i] ^= tid_bytes[i];
            }
            s
        };
        let thread_start_key = {
            let mut k = start_key;
            if !random {
                add_offset_to_key(&mut k, worker_offset);
            }
            k
        };

        let end_key_cpu = end_key;
        let handle = thread::spawn(move || {
            run_one_cpu_worker(
                flat_index,
                addr_flags,
                max_keys,
                cpu_batch,
                results,
                keys_tested,
                stop_flag,
                output_file,
                start_time,
                random,
                stop_on_match,
                progress,
                progress_slot,
                thread_start_key,
                transforms,
                total_stride,
                Some(live_rates),
                true,
                seed,
                progress_slot,
                end_key_cpu,
            );
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }

    btcsolver::gpu::gpu_unload_index();
}

/// Fallback: GPU secp256k1 derive (all devices) + CPU hash/address + FlatIndex lookup
fn run_gpu_bruteforce_fallback(
    flat_index: Arc<FlatIndex>,
    addr_flags: AddrFlags,
    max_keys: u64,
    _cpu_workers: usize,
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
    transforms: Vec<KeyTransform>,
    gpu_ids: Vec<i32>,
    live_rates: Arc<LiveRateCounters>,
    end_key: Option<[u8; 32]>,
) {
    // Use selected CUDA devices via derive_multi (split batch across cards)
    let available = btcsolver::gpu::gpu_device_count().max(0);
    let device_ids: Vec<i32> = if gpu_ids.is_empty() {
        (0..available).collect()
    } else {
        gpu_ids
            .into_iter()
            .filter(|&id| id >= 0 && id < available)
            .collect()
    };
    let device_ids = if device_ids.is_empty() {
        vec![0]
    } else {
        device_ids
    };
    let n_gpu = device_ids.len();
    eprintln!(
        "[GPU] DERIVE-ONLY multi-GPU: {} device(s) {:?} — secp on GPU + FlatIndex CPU",
        n_gpu, device_ids
    );
    // Bigger batch when multi-GPU so each card gets enough work
    let gpu_batch = (batch_size.clamp(262_144, 786_432) * n_gpu.max(1)).min(2_097_152);
    let out_stride = 85usize;
    let mut privkeys_buf = vec![0u8; gpu_batch * 32];
    let mut pub_out = vec![0u8; gpu_batch * out_stride];
    let mut key = start_key;
    let mut rng = if random {
        let mut s = [0u8; 32];
        getrandom::getrandom(&mut s).ok();
        Some(StdRng::from_seed(s))
    } else {
        None
    };
    let secp = Secp256k1::<All>::new();
    let network = Network::Bitcoin;
    let start_time = *start;

    loop {
        if stop_flag.load(Ordering::Relaxed) {
            break;
        }
        if max_keys > 0 && keys_tested.load(Ordering::Relaxed) >= max_keys {
            break;
        }
        if let Some(ref ek) = end_key {
            if !random && key.as_slice() >= ek.as_slice() {
                break;
            }
        }

        let mut count = 0usize;
        while count < gpu_batch {
            if max_keys > 0 && keys_tested.load(Ordering::Relaxed) + count as u64 >= max_keys {
                break;
            }
            if let Some(ref ek) = end_key {
                if !random && key.as_slice() >= ek.as_slice() {
                    break;
                }
            }
            let mut base = [0u8; 32];
            if random {
                rng.as_mut().unwrap().fill(&mut base);
                key = base;
            } else {
                increment_key(&mut key);
                base = key;
            }
            if base.iter().all(|&b| b == 0) {
                continue;
            }
            for transform in &transforms {
                if count >= gpu_batch {
                    break;
                }
                let kb = transform.apply(&base);
                if kb.iter().all(|&b| b == 0) {
                    continue;
                }
                privkeys_buf[count * 32..count * 32 + 32].copy_from_slice(&kb);
                count += 1;
            }
        }
        if count == 0 {
            break;
        }

        // Prefer multi-GPU split; fall back to single-device derive
        let rc = if n_gpu > 1 {
            btcsolver::gpu::gpu_derive_multi(
                &privkeys_buf[..count * 32],
                &mut pub_out[..count * out_stride],
                count,
                &device_ids,
            )
        } else {
            btcsolver::gpu::gpu_derive(
                &privkeys_buf[..count * 32],
                &mut pub_out[..count * out_stride],
                count,
            )
        };
        if rc != 0 {
            eprintln!("[GPU] derive failed rc={} — CPU batch fallback", rc);
            // minimal CPU fallback for this batch
            for i in 0..count {
                let kb: [u8; 32] = privkeys_buf[i * 32..i * 32 + 32].try_into().unwrap();
                if let Ok(sk) = bitcoin::secp256k1::SecretKey::from_slice(&kb) {
                    let pk = PrivateKey {
                        inner: sk,
                        network: network.into(),
                        compressed: true,
                    };
                    let _ = check_key_cpu(
                        &flat_index,
                        &pk,
                        &secp,
                        network,
                        addr_flags,
                        &kb,
                        results,
                        output_file,
                        stop_on_match,
                        stop_flag,
                    );
                }
            }
            keys_tested.fetch_add(count as u64, Ordering::Relaxed);
            live_rates.add_gpu(0, count as u64);
            continue;
        }

        keys_tested.fetch_add(count as u64, Ordering::Relaxed);
        live_rates.add_gpu(0, count as u64);
        // GPU output layout per key: pubkey[33] + hash160[20] + sha256[32] = 85 bytes
        for i in 0..count {
            let base = i * out_stride;
            let hash160 = &pub_out[base + 33..base + 53];
            let mut total_sats = 0u64;
            let mut matched = Vec::new();

            // P2PKH: OP_DUP OP_HASH160 OP_PUSH20 <h160> OP_EQUALVERIFY OP_CHECKSIG
            if addr_flags.has(AddrFlags::LEGACY) {
                let mut script = [0u8; 25];
                script[0] = 0x76;
                script[1] = 0xa9;
                script[2] = 0x14;
                script[3..23].copy_from_slice(hash160);
                script[23] = 0x88;
                script[24] = 0xac;
                let v = flat_index.lookup(&script);
                if v > 0 {
                    total_sats += v;
                    matched.push(format!("P2PKH h160 hit {v} sats"));
                }
            }
            // P2WPKH: OP_0 OP_PUSH20 <h160>
            if addr_flags.has(AddrFlags::SEGWIT) {
                let mut script = [0u8; 22];
                script[0] = 0x00;
                script[1] = 0x14;
                script[2..22].copy_from_slice(hash160);
                let v = flat_index.lookup(&script);
                if v > 0 {
                    total_sats += v;
                    matched.push(format!("P2WPKH h160 hit {v} sats"));
                }
            }
            // P2SH-P2WPKH: hash160(0x0014||h160) — needs extra hash, do via CPU only on rare path
            // Taproot needs x-only pubkey — use CPU for match verification if any hit or if those types only
            if total_sats == 0
                && (addr_flags.has(AddrFlags::WRAPPED) || addr_flags.has(AddrFlags::TAPROOT))
            {
                let kb: [u8; 32] = privkeys_buf[i * 32..i * 32 + 32].try_into().unwrap();
                if let Ok(sk) = bitcoin::secp256k1::SecretKey::from_slice(&kb) {
                    let pk = PrivateKey {
                        inner: sk,
                        network: network.into(),
                        compressed: true,
                    };
                    check_key_cpu(
                        &flat_index,
                        &pk,
                        &secp,
                        network,
                        addr_flags,
                        &kb,
                        results,
                        output_file,
                        stop_on_match,
                        stop_flag,
                    );
                }
                continue;
            }
            if total_sats == 0 {
                continue;
            }
            let kb = &privkeys_buf[i * 32..i * 32 + 32];
            let key_hex = hex::encode(kb);
            let result = BalanceResult {
                key_hex: key_hex.clone(),
                wif: None,
                sats: total_sats,
                btc: total_sats as f64 / 100_000_000.0,
                addresses: matched,
                timestamp: chrono::Local::now()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
            };
            results.lock().unwrap().push(result.clone());
            save_found_keys(output_file, &vec![result]).ok();
            eprintln!(
                "\n\n  BALANCE FOUND! Key: {} Balance: {} sats",
                key_hex, total_sats
            );
            if stop_on_match {
                stop_flag.store(true, Ordering::Relaxed);
                break;
            }
        }

        let total = keys_tested.load(Ordering::Relaxed);
        let elapsed = start_time.elapsed();
        let rate = if elapsed.as_secs_f64() > 0.0 {
            total as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        eprintln!(
            "[GPU-DERIVE] {} keys | {:.0} keys/sec | {} matches | batch={}",
            total,
            rate,
            results.lock().unwrap().len(),
            count
        );

        if let Some(ref prog) = thread_progress {
            if let Ok(mut guard) = prog.try_lock() {
                if guard.is_empty() {
                    guard.push(ThreadProgress {
                        thread_id: 0,
                        keys_tested: total,
                        last_key_hex: hex::encode(&key),
                        mode: if random {
                            "random".into()
                        } else {
                            "sequential".into()
                        },
                    });
                } else {
                    guard[0].keys_tested = total;
                    guard[0].last_key_hex = hex::encode(&key);
                }
            }
        }
        if stop_flag.load(Ordering::Relaxed) {
            break;
        }
    }
}

/// CPU address derive + FlatIndex lookup for one key (used by GPU-derive fallback)
fn check_key_cpu(
    flat_index: &FlatIndex,
    pk: &PrivateKey,
    secp: &Secp256k1<All>,
    network: Network,
    addr_flags: AddrFlags,
    key_bytes: &[u8; 32],
    results: &Arc<Mutex<Vec<BalanceResult>>>,
    output_file: &str,
    stop_on_match: bool,
    stop_flag: &Arc<AtomicBool>,
) {
    let pubkey = pk.public_key(secp);
    let compressed = match CompressedPublicKey::from_private_key(secp, pk) {
        Ok(c) => c,
        Err(_) => return,
    };
    let xonly: UntweakedPublicKey = compressed.into();
    let mut total_sats = 0u64;
    let mut matched_addrs = Vec::new();

    if addr_flags.has(AddrFlags::LEGACY) {
        let addr = bitcoin::Address::p2pkh(pubkey, network);
        let v = flat_index.lookup(addr.script_pubkey().as_bytes());
        if v > 0 {
            total_sats += v;
            matched_addrs.push(format!("{} [legacy]", addr));
        }
    }
    if addr_flags.has(AddrFlags::SEGWIT) {
        let addr = bitcoin::Address::p2wpkh(&compressed, network);
        let v = flat_index.lookup(addr.script_pubkey().as_bytes());
        if v > 0 {
            total_sats += v;
            matched_addrs.push(format!("{} [segwit]", addr));
        }
    }
    if addr_flags.has(AddrFlags::WRAPPED) {
        let addr = bitcoin::Address::p2shwpkh(&compressed, network);
        let v = flat_index.lookup(addr.script_pubkey().as_bytes());
        if v > 0 {
            total_sats += v;
            matched_addrs.push(format!("{} [wrapped]", addr));
        }
    }
    if addr_flags.has(AddrFlags::TAPROOT) {
        let addr = bitcoin::Address::p2tr(secp, xonly, None, network);
        let v = flat_index.lookup(addr.script_pubkey().as_bytes());
        if v > 0 {
            total_sats += v;
            matched_addrs.push(format!("{} [taproot]", addr));
        }
    }
    if total_sats == 0 {
        return;
    }
    let key_hex = hex::encode(key_bytes);
    let result = BalanceResult {
        key_hex: key_hex.clone(),
        wif: None,
        sats: total_sats,
        btc: total_sats as f64 / 100_000_000.0,
        addresses: matched_addrs,
        timestamp: chrono::Local::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    };
    results.lock().unwrap().push(result.clone());
    save_found_keys(output_file, &vec![result]).ok();
    eprintln!(
        "\n\n  BALANCE FOUND! Key: {} Balance: {} sats",
        key_hex, total_sats
    );
    if stop_on_match {
        stop_flag.store(true, Ordering::Relaxed);
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

/// Save found keys to a JSON file (appends to existing results) + permanent activity archive.
fn save_found_keys(output_file: &str, new_results: &[BalanceResult]) -> Result<()> {
    // Alerte sonore pour tout hit avec solde
    if new_results.iter().any(|r| r.sats > 0) {
        btcsolver::alert_beep::alert_balance_found();
        eprintln!("\n*** BIP: CLE AVEC SOLDE TROUVEE ***\n");
    }

    let mut all_results: Vec<BalanceResult> = if std::path::Path::new(output_file).exists() {
        let content = std::fs::read_to_string(output_file)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Dedupe by key_hex (keep max sats)
    for r in new_results {
        if let Some(existing) = all_results
            .iter_mut()
            .find(|e| e.key_hex.eq_ignore_ascii_case(&r.key_hex))
        {
            if r.sats > existing.sats {
                *existing = r.clone();
            }
        } else {
            all_results.push(r.clone());
        }
    }
    let json = serde_json::to_string_pretty(&all_results)?;
    std::fs::write(output_file, &json)?;

    // Permanent archive: every UTXO hit = on-chain activity (kept even if later spent)
    let project = std::path::Path::new(output_file)
        .parent()
        .and_then(|p| {
            // target/release -> project root
            if p.ends_with("release") {
                p.parent().and_then(|x| x.parent())
            } else {
                Some(p)
            }
        })
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from(r"Y:\btcsolver"));
    // Prefer Y:\btcsolver if archive lives there
    let project = if project.join("data").exists() {
        project
    } else {
        std::path::PathBuf::from(r"Y:\btcsolver")
    };
    let archive = KeyArchive::new(&project);
    for r in new_results {
        if r.sats == 0 {
            continue;
        }
        let _ = archive.record(ArchivedKey::from_utxo_hit(
            &r.key_hex,
            r.wif.clone(),
            r.addresses.clone(),
            r.sats,
            "brute_force",
            None,
            None,
        ));
    }
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

/// Add a u64 offset to a 32-byte big-endian key (for per-thread sequential strides)
fn add_offset_to_key(key: &mut [u8; 32], offset: u64) {
    // offset.to_be_bytes() fait 8 octets — il faut l’ajouter sur les 32 octets BE
    // (l’ancienne version faisait skip(24) sur 8 octets → n’ajoutait JAMAIS l’offset)
    let mut carry = offset as u128;
    for i in (0..32).rev() {
        if carry == 0 {
            break;
        }
        let sum = key[i] as u128 + carry;
        key[i] = (sum & 0xff) as u8;
        carry = sum >> 8;
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
