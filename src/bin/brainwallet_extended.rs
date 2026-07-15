//! Extended Brainwallet Scanner — SHA256 + MD5, compressed + uncompressed, all 4 address types
//!
//! Usage:
//!   brainwallet_extended --texts bible-full-corpus.txt --snapshot utxo-index.snapshot [--threads 22]

use anyhow::Result;
use bitcoin::key::{CompressedPublicKey, PrivateKey, UntweakedPublicKey};
use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::Network;
use clap::Parser;
use crossbeam_channel::unbounded;
use digest::Digest;
use md5::Md5;
use sha2::Sha256;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

mod flat_index;

#[derive(Parser)]
struct Cli {
    /// Path to the text file with one candidate phrase per line
    #[arg(short, long)]
    texts: String,

    /// Path to the UTXO index snapshot
    #[arg(short, long)]
    snapshot: String,

    /// Number of CPU threads
    #[arg(short, long, default_value = "0")]
    threads: usize,

    /// Minimum UTXO value in satoshis (dust filter)
    #[arg(long, default_value = "0")]
    min_value: u64,

    /// Output file for matches
    #[arg(long, default_value = "brainwallet-extended-matches.json")]
    output: String,

    /// Hash method: sha256, md5, or both
    #[arg(long, default_value = "both")]
    hash: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct MatchResult {
    phrase: String,
    hash_method: String,
    key_type: String,
    address: String,
    address_type: String,
    value_sats: u64,
    value_btc: f64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load texts
    let raw_lines = std::fs::read_to_string(&cli.texts)?;
    let phrases: Vec<String> = raw_lines
        .lines()
        .map(|l| l.to_string())
        .filter(|l| !l.trim().is_empty())
        .collect();

    println!("Loaded {} candidate phrases from {}", phrases.len(), &cli.texts);

    // Generate all variations
    let mut all_variants: Vec<String> = Vec::new();
    for phrase in &phrases {
        let variants = generate_variations(phrase);
        all_variants.extend(variants);
    }

    all_variants.sort();
    all_variants.dedup();
    println!("Total unique variations to test: {}", all_variants.len());

    // Determine hash methods
    let use_sha256 = cli.hash == "both" || cli.hash == "sha256";
    let use_md5 = cli.hash == "both" || cli.hash == "md5";

    println!("Hash methods: SHA256={}, MD5={}", use_sha256, use_md5);
    println!("Key types: compressed + uncompressed");
    println!("Address types: P2PKH, P2WPKH, P2SH-P2WPKH, P2TR");

    let total_checks = all_variants.len() as u64
        * (if use_sha256 { 1 } else { 0 } + if use_md5 { 1 } else { 0 }) as u64
        * 2; // compressed + uncompressed
    println!("Total hash+key combos to check: ~{}", total_checks);

    // Load UTXO index
    println!("\nLoading UTXO index from {}...", &cli.snapshot);
    let index = flat_index::FlatIndex::load_from_snapshot(&cli.snapshot, cli.min_value)?;
    index.print_stats();

    // Thread count
    let num_threads = if cli.threads == 0 {
        num_cpus::get().saturating_sub(1).max(1)
    } else {
        cli.threads
    };

    // Split work among threads
    let chunk_size = (all_variants.len() + num_threads - 1) / num_threads;
    let chunks: Vec<Vec<String>> = all_variants
        .chunks(chunk_size)
        .map(|c| c.to_vec())
        .collect();

    let secp = Secp256k1::<All>::new();
    let network = Network::Bitcoin;
    let total_tested = Arc::new(AtomicU64::new(0));
    let total_matches = Arc::new(AtomicU64::new(0));
    let (tx, rx) = unbounded::<MatchResult>();
    let start = std::time::Instant::now();

    let index = Arc::new(index);

    // Spawn worker threads
    let mut handles = Vec::new();
    for (thread_id, chunk) in chunks.into_iter().enumerate() {
        let chunk_len = chunk.len();
        let secp = secp.clone();
        let fi = Arc::clone(&index);
        let total_tested = Arc::clone(&total_tested);
        let total_matches = Arc::clone(&total_matches);
        let tx = tx.clone();

        let handle = thread::Builder::new()
            .name(format!("BW-Ext-{}", thread_id))
            .spawn(move || {
                let mut count = 0u64;
                for variant in &chunk {
                    // SHA256 hash
                    if use_sha256 {
                        let hash = Sha256::digest(variant.as_bytes());
                        let key_bytes: [u8; 32] = hash.into();
                        check_key_bytes(
                            &secp, network, &fi, &total_matches,
                            &tx, variant, "SHA256", &key_bytes,
                        );
                        count += 1;
                    }

                    // MD5 hash (padded to 32 bytes)
                    if use_md5 {
                        let md5_hash = Md5::digest(variant.as_bytes());
                        let mut key_bytes = [0u8; 32];
                        key_bytes[..16].copy_from_slice(&md5_hash);
                        check_key_bytes(
                            &secp, network, &fi, &total_matches,
                            &tx, variant, "MD5", &key_bytes,
                        );
                        count += 1;
                    }

                    if count % 200_000 == 0 {
                        eprintln!(
                            "[Thread {}] {} / {} phrases tested ({:.1}%)",
                            thread_id,
                            count,
                            chunk_len,
                            count as f64 / chunk_len as f64 * 100.0
                        );
                    }
                }
                total_tested.fetch_add(count, Ordering::Relaxed);
            })
            .expect("failed to spawn thread");

        handles.push(handle);
    }

    drop(tx);

    let mut matches: Vec<MatchResult> = Vec::new();
    for result in rx {
        matches.push(result);
    }

    for h in handles {
        h.join().expect("thread panicked");
    }

    let elapsed = start.elapsed();
    let rate = total_tested.load(Ordering::Relaxed) as f64 / elapsed.as_secs_f64();

    println!("\n{}", "=".repeat(60));
    println!("  Extended Brainwallet scan complete");
    println!("{}","=".repeat(60));
    println!("  Phrases tested: {}", total_tested.load(Ordering::Relaxed));
    println!("  Speed: {:.0} phrases/sec", rate);
    println!("  Time: {:.1}s", elapsed.as_secs_f64());
    println!("  Matches: {}", total_matches.load(Ordering::Relaxed));

    if !matches.is_empty() {
        serde_json::to_writer_pretty(
            &std::fs::File::create(&cli.output)?,
            &matches,
        )?;
        println!("\n  *** MATCHES FOUND! Written to {} ***", &cli.output);
        for m in &matches {
            println!(
                "    [{}] [{}] [{}] {} BTC ({} sats) <- \"{}\"",
                m.hash_method, m.key_type, m.address_type,
                m.value_btc, m.value_sats, m.phrase
            );
        }
    } else {
        println!("  No matches found.");
    }

    Ok(())
}

fn check_key_bytes(
    secp: &Secp256k1<All>,
    network: Network,
    index: &flat_index::FlatIndex,
    total_matches: &Arc<AtomicU64>,
    tx: &crossbeam_channel::Sender<MatchResult>,
    phrase: &str,
    hash_method: &str,
    key_bytes: &[u8; 32],
) {
    if let Ok(secp_key) = bitcoin::secp256k1::SecretKey::from_slice(key_bytes) {
        let pk = PrivateKey {
            inner: secp_key,
            network: network.into(),
            compressed: true,
        };

        // === Compressed public key ===
        if let Ok(compressed) = CompressedPublicKey::from_private_key(secp, &pk) {
            let pubkey = pk.public_key(secp);

            // P2PKH
            let addr = bitcoin::Address::p2pkh(&pubkey, network);
            let v = index.lookup(addr.script_pubkey().as_bytes());
            if v > 0 {
                send_match(tx, total_matches, phrase, hash_method,
                    "Compressed", &addr, "P2PKH", v);
            }

            // P2WPKH
            let addr = bitcoin::Address::p2wpkh(&compressed, network);
            let v = index.lookup(addr.script_pubkey().as_bytes());
            if v > 0 {
                send_match(tx, total_matches, phrase, hash_method,
                    "Compressed", &addr, "P2WPKH", v);
            }

            // P2SH-P2WPKH (Wrapped)
            let addr = bitcoin::Address::p2shwpkh(&compressed, network);
            let v = index.lookup(addr.script_pubkey().as_bytes());
            if v > 0 {
                send_match(tx, total_matches, phrase, hash_method,
                    "Compressed", &addr, "P2SH-P2WPKH", v);
            }

            // P2TR (Taproot)
            let xonly: UntweakedPublicKey = compressed.into();
            let addr = bitcoin::Address::p2tr(secp, xonly, None, network);
            let v = index.lookup(addr.script_pubkey().as_bytes());
            if v > 0 {
                send_match(tx, total_matches, phrase, hash_method,
                    "Compressed", &addr, "P2TR", v);
            }
        }

        // === Uncompressed public key ===
        {
            let uncompressed_pk = bitcoin::PrivateKey {
                inner: secp_key,
                network: network.into(),
                compressed: false,
            };
            let pubkey = uncompressed_pk.public_key(secp);

            // P2PKH uncompressed
            let addr = bitcoin::Address::p2pkh(&pubkey, network);
            let v = index.lookup(addr.script_pubkey().as_bytes());
            if v > 0 {
                send_match(tx, total_matches, phrase, hash_method,
                    "Uncompressed", &addr, "P2PKH", v);
            }
        }
    }
}

fn send_match(
    tx: &crossbeam_channel::Sender<MatchResult>,
    total_matches: &Arc<AtomicU64>,
    phrase: &str,
    hash_method: &str,
    key_type: &str,
    addr: &bitcoin::Address,
    addr_type: &str,
    value_sats: u64,
) {
    total_matches.fetch_add(1, Ordering::Relaxed);
    let _ = tx.send(MatchResult {
        phrase: phrase.to_string(),
        hash_method: hash_method.to_string(),
        key_type: key_type.to_string(),
        address: addr.to_string(),
        address_type: addr_type.to_string(),
        value_sats,
        value_btc: value_sats as f64 / 1e8,
    });
}

fn generate_variations(phrase: &str) -> Vec<String> {
    let mut variants = Vec::new();
    let trimmed = phrase.trim();

    variants.push(trimmed.to_string());

    let lower = trimmed.to_lowercase();
    if lower != trimmed { variants.push(lower.clone()); }

    let upper = trimmed.to_uppercase();
    if upper != trimmed { variants.push(upper.clone()); }

    // Title case
    let title = lower.split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    if title != lower && title != upper && title != trimmed { variants.push(title); }

    // No punctuation
    let no_punct: String = trimmed.chars().filter(|c| !c.is_ascii_punctuation()).collect();
    if no_punct != trimmed {
        variants.push(no_punct.clone());
        variants.push(no_punct.to_lowercase());
        variants.push(no_punct.to_uppercase());
    }

    // Collapsed spaces
    let collapsed: String = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed != trimmed { variants.push(collapsed.to_lowercase()); }

    // Suffixes
    for suffix in &[" private key", " bitcoin", " wallet", " btc", " key"] {
        variants.push(format!("{}{}", lower, suffix));
        variants.push(format!("{}{}", upper, suffix));
    }

    // Prefixes
    for prefix in &["my ", "the ", "bitcoin ", "my bitcoin "] {
        variants.push(format!("{}{}", prefix, lower));
    }

    // Leet speak
    if lower.contains('a') || lower.contains('e') || lower.contains('o') || lower.contains('t') {
        let leet: String = lower.chars().map(|c| match c {
            'a' => '4', 'e' => '3', 'i' => '1', 'o' => '0', 't' => '7', _ => c,
        }).collect();
        if leet != lower { variants.push(leet); }
    }

    // Year suffixes
    for year in &["2009","2010","2011","2012","2013","2014","2015","2016","2017","2018","2019","2020"] {
        variants.push(format!("{} {}", lower, year));
        variants.push(format!("{}{}", lower, year));
    }

    // Punctuation suffixes
    variants.push(format!("{}!", lower));
    variants.push(format!("{}?", lower));
    variants.push(format!("{}!!!", lower));

    // No spaces
    let no_spaces: String = lower.chars().filter(|c| !c.is_whitespace()).collect();
    if no_spaces != lower { variants.push(no_spaces); }

    // Underscores
    let underscores = lower.replace(' ', "_");
    if underscores != lower { variants.push(underscores); }

    // Hyphens
    let hyphens = lower.replace(' ', "-");
    if hyphens != lower { variants.push(hyphens); }

    variants
}
