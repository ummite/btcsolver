use anyhow::Result;
use bitcoin::key::{CompressedPublicKey, PrivateKey, UntweakedPublicKey};
use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::{Network, Txid};
use bitcoin_hashes::Hash;
use clap::Parser;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

const SCRIPT_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("by_script");
const META_TABLE: TableDefinition<&str, u64> = TableDefinition::new("meta");

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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Parse address types
    let addr_types: Vec<String> = cli.addr_types.split(',').map(|s| s.trim().to_string()).collect();

    // Load UTXO index into RAM
    println!("Loading UTXO index from {}...", cli.db_path);
    let load_start = Instant::now();
    let utxo_index = load_index_to_ram(&cli.db_path)?;
    let load_time = load_start.elapsed();

    let last_file = {
        let db = Database::open(&cli.db_path)?;
        let rx = db.begin_read()?;
        let m = rx.open_table(META_TABLE)?;
        m.get("last_file")?.map(|v| v.value())
    };

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

    // Shared results
    let results = Arc::new(Mutex::new(Vec::new()));
    let keys_tested = Arc::new(Mutex::new(0u64));

    let num_threads = if cli.threads == 0 { num_cpus::get() } else { cli.threads };
    let batch_size = cli.batch_size;

    println!("Starting brute-force...");
    println!("  Start key: {}", hex::encode(&start_key));
    if cli.count > 0 {
        println!("  Keys to test: {}", cli.count);
    }
    println!("  Batch size: {}", batch_size);
    println!("  Address types: {}", cli.addr_types);
    println!();

    let start = Instant::now();

    if cli.use_gpu {
        // TODO: GPU path - CUDA kernel launch
        // For now, fall back to CPU with a warning
        eprintln!("WARNING: GPU mode not yet implemented. Using CPU only.");
        run_cpu_bruteforce(
            utxo_index.clone(),
            addr_types.clone(),
            start_key,
            cli.count,
            num_threads,
            batch_size,
            &results,
            &keys_tested,
            &start,
        );
    } else {
        run_cpu_bruteforce(
            utxo_index.clone(),
            addr_types.clone(),
            start_key,
            cli.count,
            num_threads,
            batch_size,
            &results,
            &keys_tested,
            &start,
        );
    }

    let elapsed = start.elapsed();
    let total = *keys_tested.lock().unwrap();
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
    start_key: [u8; 32],
    max_keys: u64,
    num_threads: usize,
    batch_size: usize,
    results: &Arc<Mutex<Vec<BalanceResult>>>,
    keys_tested: &Arc<Mutex<u64>>,
    start: &Instant,
) {
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let utxo_index = utxo_index.clone();
        let addr_types = addr_types.to_vec();
        let results = Arc::clone(results);
        let keys_tested = Arc::clone(keys_tested);
        let start_time = *start;

        let start_for_thread = {
            let mut k = start_key;
            // Offset by thread_id * some_stride
            let offset = (thread_id as u64).to_be_bytes();
            for (i, b) in offset.iter().enumerate() {
                if 32 - 8 + i < 32 {
                    k[32 - 8 + i] = k[32 - 8 + i].wrapping_add(*b);
                }
            }
            // Add thread_id to the key
            let tid = thread_id as u32;
            for i in 0..4 {
                k[31 - i] = k[31 - i].wrapping_add((tid >> (i * 8)) as u8);
            }
            k
        };

        let handle = thread::spawn(move || {
            let secp = Secp256k1::<All>::new();
            let network = Network::Bitcoin;
            let mut key = start_for_thread;
            let mut local_count = 0u64;

            loop {
                for _ in 0..batch_size {
                    if max_keys > 0 && *keys_tested.lock().unwrap() >= max_keys {
                        return;
                    }

                    // Increment key (simple sequential)
                    increment_key(&mut key);

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
                        results.lock().unwrap().push(BalanceResult {
                            key_hex,
                            sats: total_sats,
                            btc: total_sats as f64 / 100_000_000.0,
                            addresses: matched_addrs,
                        });
                    }

                    local_count += 1;
                }

                *keys_tested.lock().unwrap() += local_count;

                // Progress report
                let total = *keys_tested.lock().unwrap();
                let elapsed = start_time.elapsed();
                let rate = total as f64 / elapsed.as_secs_f64();
                eprintln!("[Thread {}] {} keys tested | {:.0} keys/sec | {} matches",
                    thread_id, total, rate, results.lock().unwrap().len());
            }
        });

        handles.push(handle);
    }

    // If max_keys specified, we'd join threads here.
    // For unlimited mode, threads run until interrupted.
    if max_keys > 0 {
        // Wait for completion (threads check max_keys and exit)
        for handle in handles {
            let _ = handle.join();
        }
    }
    // For unlimited: threads keep running. User presses Ctrl+C to stop.
}

// ─── Data structures ────────────────────────────────────────────────────

struct BalanceResult {
    key_hex: String,
    sats: u64,
    btc: f64,
    addresses: Vec<String>,
}

// ─── UTXO Index Loading ─────────────────────────────────────────────────

/// Load the by_script table from redb into a HashMap in RAM.
/// Returns: script_bytes -> Vec<(txid, vout, value)>
fn load_index_to_ram(db_path: &str) -> Result<HashMap<Vec<u8>, Vec<(Txid, u32, u64)>>> {
    let db = Database::open(db_path)?;
    let rx = db.begin_read()?;
    let table = rx.open_table(SCRIPT_TABLE)?;

    let mut index: HashMap<Vec<u8>, Vec<(Txid, u32, u64)>> = HashMap::new();

    for entry in table.iter()? {
        let (key, val) = entry?;
        let script = key.value().to_vec();
        let vbuf = val.value();

        if vbuf.len() < 4 {
            continue;
        }

        let count = u32::from_le_bytes(vbuf[..4].try_into().unwrap());
        let mut pos = 4usize;
        let mut entries = Vec::with_capacity(count as usize);

        for _ in 0..count {
            if pos + 44 > vbuf.len() {
                break;
            }
            let txid_bytes: [u8; 32] = vbuf[pos..pos + 32].try_into().unwrap();
            let vout = u32::from_le_bytes(vbuf[pos + 32..pos + 36].try_into().unwrap());
            let value = u64::from_le_bytes(vbuf[pos + 36..pos + 44].try_into().unwrap());
            pos += 44;
            entries.push((Txid::from_byte_array(txid_bytes), vout, value));
        }

        index.insert(script, entries);
    }

    println!("  Loaded {} unique scripts with UTXOs", index.len());
    Ok(index)
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
