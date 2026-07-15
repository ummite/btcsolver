//! Timestamp Brainwallet Scanner
//!
//! Tests SHA256(timestamp_string) as private keys for every millisecond
//! from Bitcoin genesis to now, checking all 4 address types.
//!
//! Timestamp formats tested (per millisecond):
//!   - Unix ms:       "1231006505000"
//!   - Unix seconds:  "1231006505"
//!   - ISO 8601:      "2009-01-03T18:15:05.000Z"
//!   - DateTime:      "2009-01-03 18:15:05.000"
//!   - Win FILETIME:  "128834823050000000" (100-ns since 1601)
//!   - MAC:           "1231006505.000"
//!
//! Usage:
//!   timestamp_scan --snapshot utxo-index.snapshot --threads 22
//!   timestamp_scan --start 1231006505 --end 1752633600 --threads 22
//!   timestamp_scan --start-ms 1231006505000 --end-ms 1752633600000 --threads 22

use anyhow::Result;
use bitcoin::key::{CompressedPublicKey, PrivateKey, UntweakedPublicKey};
use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::Network;
use clap::Parser;
use crossbeam_channel::unbounded;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

mod flat_index;

#[derive(Parser)]
struct Cli {
    /// Path to the UTXO index snapshot
    #[arg(short, long, default_value = "utxo-index.snapshot")]
    snapshot: String,

    /// Start Unix timestamp in seconds (default: Bitcoin genesis)
    #[arg(long, default_value = "1231006505")]
    start: u64,

    /// End Unix timestamp in seconds (default: now)
    #[arg(long, default_value = "0")]
    end: u64,

    /// Number of CPU threads
    #[arg(short, long, default_value = "0")]
    threads: usize,

    /// Minimum UTXO value in satoshis
    #[arg(long, default_value = "0")]
    min_value: u64,

    /// Output file for matches
    #[arg(long, default_value = "timestamp-matches.json")]
    output: String,

    /// Granularity: ms (milliseconds) or sec (seconds only)
    #[arg(long, default_value = "ms")]
    granularity: String,

    /// Enable specific formats (comma-separated): unix_ms, unix_sec, iso, datetime, filetime, mac
    #[arg(long, default_value = "unix_ms,unix_sec")]
    formats: String,

    /// Enable 512 bit rotations (256 + 256 reversed). Without this flag: direct SHA256 only (faster).
    #[arg(long)]
    rotations: bool,

    /// Progress report interval in seconds
    #[arg(long, default_value = "5")]
    progress_interval: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct MatchResult {
    timestamp_ms: u64,
    format: String,
    timestamp_string: String,
    hash_hex: String,
    key_type: String,
    address: String,
    address_type: String,
    value_sats: u64,
    value_btc: f64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let end_time = if cli.end == 0 {
        duration_since_epoch()?.as_secs()
    } else {
        cli.end
    };

    println!("============================================================");
    println!("  Timestamp Brainwallet Scanner");
    println!("============================================================");
    println!("  Range: {} to {} ({:.1} years)",
        cli.start, end_time, (end_time - cli.start) as f64 / (365.25 * 24.0 * 3600.0));
    println!("  Granularity: {}", cli.granularity);
    println!("  Bit rotations: {}", if cli.rotations { "512 (256 + 256 reversed)" } else { "OFF (direct SHA256 only)" });

    // Parse enabled formats
    let enabled_formats: Vec<String> = cli.formats.split(',').map(|s| s.trim().to_string()).collect();
    println!("  Formats: {}", cli.formats);

    // Calculate total timestamps
    let granularity_ms = if cli.granularity == "ms" { 1 } else { 1000 };
    let total_ms = end_time * 1000 - cli.start * 1000;
    let total_timestamps = total_ms / granularity_ms as u64 + 1;
    let formats_per_ts = enabled_formats.len();
    let rotations_per_ts = if cli.rotations { 512 } else { 1 };
    let total_checks = total_timestamps * formats_per_ts as u64 * rotations_per_ts;

    println!("  Timestamps to test: {}", format_number(total_timestamps));
    println!("  Total key checks: {}", format_number(total_checks));
    let est_speed = if cli.rotations { 60_000.0 } else { 400_000.0 };
    println!("  Est. speed: {:.0} checks/sec", est_speed);
    println!("  Est. time: {:.1} hours ({:.1} days)",
        total_checks as f64 / est_speed / 3600.0,
        total_checks as f64 / est_speed / 86400.0);

    // Load UTXO index
    println!("\nLoading UTXO index from {}...", &cli.snapshot);
    let index = flat_index::FlatIndex::load_from_snapshot(&cli.snapshot, cli.min_value)?;
    index.print_stats();

    let num_threads = if cli.threads == 0 {
        num_cpus::get().saturating_sub(1).max(1)
    } else {
        cli.threads
    };

    // Split timestamp range among threads
    let chunk_size_ms = (total_ms + num_threads as u64 - 1) / num_threads as u64;
    // Round up to granularity
    let chunk_size_ms = (chunk_size_ms + granularity_ms as u64 - 1) / granularity_ms as u64 * granularity_ms as u64;

    let secp = Secp256k1::<All>::new();
    let network = Network::Bitcoin;
    let total_tested = Arc::new(AtomicU64::new(0));
    let total_matches = Arc::new(AtomicU64::new(0));
    let (tx, rx) = unbounded::<MatchResult>();
    let start = std::time::Instant::now();
    let progress_interval = cli.progress_interval;

    let index = Arc::new(index);
    let enabled_formats = Arc::new(enabled_formats);
    let do_rotations = cli.rotations;

    // Spawn worker threads
    let mut handles = Vec::new();
    for thread_id in 0..num_threads {
        let ts_start_ms = cli.start * 1000 + thread_id as u64 * chunk_size_ms;
        let ts_end_ms = std::cmp::min(ts_start_ms + chunk_size_ms, end_time * 1000 + 1);

        if ts_start_ms >= ts_end_ms {
            continue;
        }

        let secp = secp.clone();
        let fi = Arc::clone(&index);
        let total_tested = Arc::clone(&total_tested);
        let total_matches = Arc::clone(&total_matches);
        let tx = tx.clone();
        let formats = Arc::clone(&enabled_formats);
        let do_rotations = do_rotations;

        let handle = thread::Builder::new()
            .name(format!("TS-{}", thread_id))
            .spawn(move || {
                let mut count = 0u64;
                let chunk_total = (ts_end_ms - ts_start_ms) / granularity_ms as u64 + 1;

                for ts_ms in (ts_start_ms..ts_end_ms).step_by(granularity_ms as usize) {
                    let ts_sec = ts_ms / 1000;
                    let ts_frac = ts_ms % 1000;

                    for fmt in formats.iter() {
                        let ts_string = match fmt.as_str() {
                            "unix_ms" => format!("{}", ts_ms),
                            "unix_sec" => format!("{}", ts_sec),
                            "iso" => {
                                let dt = unix_to_datetime(ts_sec, ts_frac);
                                format!("{}T{}.000Z", dt, pad3(ts_frac))
                            }
                            "datetime" => {
                                let dt = unix_to_datetime(ts_sec, ts_frac);
                                format!("{} {}.000", dt, pad3(ts_frac))
                            }
                            "filetime" => {
                                // Windows FILETIME: 100-ns intervals since 1601-01-01
                                // Unix epoch is 11644473600 seconds after FILETIME epoch
                                let filetime_100ns = ts_sec as u128 * 10_000_000 + ts_frac as u128 * 10_000 + 11_644_473_600_000_000;
                                format!("{}", filetime_100ns)
                            }
                            "mac" => format!("{}.000", ts_sec),
                            _ => continue,
                        };

                        // SHA256 hash -> base key material
                        let hash = Sha256::digest(ts_string.as_bytes());
                        let key_bytes: [u8; 32] = hash.into();

                        if do_rotations {
                            let reversed = reverse_bytes(&key_bytes);

                            // Try 512 bit variations: 256 rotations + 256 rotations of reversed
                            for rotation in 0..256u32 {
                                // Original hash rotated right by `rotation` bits
                                let rotated = rotate_right_256(&key_bytes, rotation);
                                let rot_label = if rotation == 0 { "R0" } else { &format!("R{}", rotation) };

                                if let Ok(secp_key) = bitcoin::secp256k1::SecretKey::from_slice(&rotated) {
                                    check_all_addresses(
                                        &secp, network, &fi, &tx, &total_matches,
                                        ts_ms, fmt.as_str(), &ts_string,
                                        &hex::encode(&rotated),
                                        &format!("C-{}", rot_label),
                                        &secp_key, true,
                                    );
                                    check_all_addresses(
                                        &secp, network, &fi, &tx, &total_matches,
                                        ts_ms, fmt.as_str(), &ts_string,
                                        &hex::encode(&rotated),
                                        &format!("U-{}", rot_label),
                                        &secp_key, false,
                                    );
                                }

                                // Reversed hash rotated right by `rotation` bits
                                let rev_rotated = rotate_right_256(&reversed, rotation);
                                let rev_label = if rotation == 0 { "RV0" } else { &format!("RV{}", rotation) };

                                if let Ok(secp_key) = bitcoin::secp256k1::SecretKey::from_slice(&rev_rotated) {
                                    check_all_addresses(
                                        &secp, network, &fi, &tx, &total_matches,
                                        ts_ms, fmt.as_str(), &ts_string,
                                        &hex::encode(&rev_rotated),
                                        &format!("C-{}", rev_label),
                                        &secp_key, false,
                                    );
                                }

                                count += 1;
                            }
                        } else {
                            // Direct SHA256 only (no rotations) - much faster
                            if let Ok(secp_key) = bitcoin::secp256k1::SecretKey::from_slice(&key_bytes) {
                                check_all_addresses(
                                    &secp, network, &fi, &tx, &total_matches,
                                    ts_ms, fmt.as_str(), &ts_string,
                                    &hex::encode(&key_bytes),
                                    "Direct",
                                    &secp_key, true,
                                );
                            }
                            count += 1;
                        }
                    }

                    // Periodically update global counter for progress reporting
                    if count % 1_000_000 == 0 {
                        total_tested.fetch_add(1_000_000, Ordering::Relaxed);
                    }

                    if count % 5_000_000 == 0 {
                        eprintln!(
                            "[Thread {}] {} / {} ({:.1}%) - {:.0} total/sec",
                            thread_id,
                            count,
                            chunk_total,
                            count as f64 / chunk_total as f64 * 100.0,
                            count as f64 / std::time::Instant::now().duration_since(start).as_secs_f64()
                        );
                    }
                }
                total_tested.fetch_add(count, Ordering::Relaxed);
            })
            .expect("failed to spawn thread");

        handles.push(handle);
    }

    drop(tx);

    // Progress reporting
    let progress_tested = Arc::clone(&total_tested);
    let progress_start = start;
    let progress_interval = progress_interval;
    let progress_handle = thread::spawn(move || {
        let interval = std::time::Duration::from_secs(progress_interval);
        loop {
            thread::sleep(interval);
            let tested = progress_tested.load(Ordering::Relaxed);
            let elapsed = progress_start.elapsed().as_secs_f64();
            let rate = if elapsed > 0.0 { tested as f64 / elapsed } else { 0.0 };
            eprintln!("[Progress] {} checks in {:.0}s ({:.0}/sec)",
                format_number(tested), elapsed, rate);
        }
    });
    let progress_handle = Arc::new(std::sync::Mutex::new(Some(progress_handle)));

    let mut matches: Vec<MatchResult> = Vec::new();
    for result in rx {
        matches.push(result);
    }

    for h in handles {
        h.join().expect("thread panicked");
    }

    // Stop progress thread
    if let Some(p) = progress_handle.lock().unwrap().take() {
        // Thread will exit when it's dropped
        let _ = p;
    }

    let elapsed = start.elapsed();
    let rate = if elapsed.as_secs_f64() > 0.0 {
        total_tested.load(Ordering::Relaxed) as f64 / elapsed.as_secs_f64()
    } else { 0.0 };

    println!("\n{}", "=".repeat(60));
    println!("  Timestamp Brainwallet Scan Complete");
    println!("{}", "=".repeat(60));
    println!("  Checks performed: {}", format_number(total_tested.load(Ordering::Relaxed)));
    println!("  Speed: {:.0} checks/sec", rate);
    println!("  Time: {:.1}s ({:.1} hours)", elapsed.as_secs_f64(), elapsed.as_secs_f64() / 3600.0);
    println!("  Matches: {}", total_matches.load(Ordering::Relaxed));

    if !matches.is_empty() {
        serde_json::to_writer_pretty(
            &std::fs::File::create(&cli.output)?,
            &matches,
        )?;
        println!("\n  *** MATCHES FOUND! Written to {} ***", &cli.output);
        for m in &matches {
            println!(
                "    [{}] [{}] [{}] {} BTC ({} sats)",
                m.format, m.key_type, m.address_type, m.value_btc, m.value_sats,
            );
            println!("      Timestamp: {} ({})", m.timestamp_string, m.format);
            println!("      Hash: {}", m.hash_hex);
            println!("      Address: {}", m.address);
        }
    } else {
        println!("  No matches found.");
    }

    Ok(())
}

/// Check all address types for a given private key.
fn check_all_addresses(
    secp: &Secp256k1<All>,
    network: Network,
    fi: &flat_index::FlatIndex,
    tx: &crossbeam_channel::Sender<MatchResult>,
    total_matches: &Arc<AtomicU64>,
    ts_ms: u64,
    fmt: &str,
    ts_string: &str,
    hash_hex: &str,
    key_label: &str,
    secp_key: &bitcoin::secp256k1::SecretKey,
    _compressed: bool,
) {
    let pk = PrivateKey {
        inner: *secp_key,
        network: network.into(),
        compressed: true,
    };

    if let Ok(compressed) = CompressedPublicKey::from_private_key(secp, &pk) {
        let pubkey = pk.public_key(secp);

        // P2PKH compressed
        let addr = bitcoin::Address::p2pkh(&pubkey, network);
        let v = fi.lookup(addr.script_pubkey().as_bytes());
        if v > 0 {
            send_match(tx, total_matches, ts_ms, fmt, ts_string, hash_hex, key_label, &addr, "P2PKH", v);
        }

        // P2WPKH
        let addr = bitcoin::Address::p2wpkh(&compressed, network);
        let v = fi.lookup(addr.script_pubkey().as_bytes());
        if v > 0 {
            send_match(tx, total_matches, ts_ms, fmt, ts_string, hash_hex, key_label, &addr, "P2WPKH", v);
        }

        // P2SH-P2WPKH
        let addr = bitcoin::Address::p2shwpkh(&compressed, network);
        let v = fi.lookup(addr.script_pubkey().as_bytes());
        if v > 0 {
            send_match(tx, total_matches, ts_ms, fmt, ts_string, hash_hex, key_label, &addr, "P2SH-P2WPKH", v);
        }

        // P2TR
        let xonly: UntweakedPublicKey = compressed.into();
        let addr = bitcoin::Address::p2tr(secp, xonly, None, network);
        let v = fi.lookup(addr.script_pubkey().as_bytes());
        if v > 0 {
            send_match(tx, total_matches, ts_ms, fmt, ts_string, hash_hex, key_label, &addr, "P2TR", v);
        }
    }

    // P2PKH uncompressed
    let pk_uncomp = PrivateKey {
        inner: *secp_key,
        network: network.into(),
        compressed: false,
    };
    let pubkey = pk_uncomp.public_key(secp);
    let addr = bitcoin::Address::p2pkh(&pubkey, network);
    let v = fi.lookup(addr.script_pubkey().as_bytes());
    if v > 0 {
        send_match(tx, total_matches, ts_ms, fmt, ts_string, hash_hex, &format!("{}-UC", key_label), &addr, "P2PKH", v);
    }
}

fn send_match(
    tx: &crossbeam_channel::Sender<MatchResult>,
    total_matches: &Arc<AtomicU64>,
    ts_ms: u64,
    fmt: &str,
    ts_string: &str,
    hash_hex: &str,
    key_type: &str,
    addr: &bitcoin::Address,
    addr_type: &str,
    value_sats: u64,
) {
    total_matches.fetch_add(1, Ordering::Relaxed);
    eprintln!("\n  🔥 MATCH! [{}] [{}] [{}] {} BTC <- \"{}\"",
        fmt, key_type, addr_type, value_sats as f64 / 1e8, ts_string);
    let _ = tx.send(MatchResult {
        timestamp_ms: ts_ms,
        format: fmt.to_string(),
        timestamp_string: ts_string.to_string(),
        hash_hex: hash_hex.to_string(),
        key_type: key_type.to_string(),
        address: addr.to_string(),
        address_type: addr_type.to_string(),
        value_sats,
        value_btc: value_sats as f64 / 1e8,
    });
}

fn unix_to_datetime(sec: u64, _frac: u64) -> String {
    // Convert Unix timestamp to "YYYY-MM-DD HH:MM:SS"
    let duration = std::time::Duration::from_secs(sec);
    let sys_time = UNIX_EPOCH + duration;
    let dt: chrono::DateTime<chrono::Utc> = chrono::DateTime::from(sys_time);
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Rotate a 256-bit value (32 bytes) right by `shift` bits.
/// Uses a 64-byte buffer for efficient wrap-around.
#[inline]
fn rotate_right_256(input: &[u8; 32], shift: u32) -> [u8; 32] {
    let mut result = [0u8; 32];
    if shift == 0 {
        return *input;
    }
    let shift = shift % 256; // 256-bit rotation wraps at 256

    // Efficient rotation using bit-level operations on u64 chunks
    let mut src = [0u64; 4];
    let mut dst = [0u64; 4];
    for i in 0..4 {
        src[i] = u64::from_le_bytes(input[i*8..(i+1)*8].try_into().unwrap());
    }

    let shift_mod64 = (shift % 64) as u32;
    let chunk_shift = (shift / 64) as usize;

    for i in 0..4 {
        let src_idx = (i + chunk_shift) % 4;
        let a = src[src_idx];
        let next_idx = (src_idx + 1) % 4;
        let b = src[next_idx];
        if shift_mod64 == 0 {
            dst[i] = a;
        } else {
            dst[i] = a >> shift_mod64 | b << (64 - shift_mod64);
        }
    }

    for i in 0..4 {
        result[i*8..(i+1)*8].copy_from_slice(&dst[i].to_le_bytes());
    }
    result
}

/// Reverse the byte order of a 32-byte array.
#[inline]
fn reverse_bytes(input: &[u8; 32]) -> [u8; 32] {
    let mut result = *input;
    result.reverse();
    result
}

fn pad3(n: u64) -> String {
    format!("{:03}", n)
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000_000_000 {
        format!("{:.1}T", n as f64 / 1_000_000_000_000.0)
    } else if n >= 1_000_000_000 {
        format!("{:.1}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn duration_since_epoch() -> Result<std::time::Duration> {
    SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.into())
}
