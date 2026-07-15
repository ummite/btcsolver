//! Brainwallet Scanner — tests deterministic keys derived from text (Bible, songs, common phrases)
//!
//! Usage:
//!   brainwallet_scan --texts bible-and-songs.txt --snapshot utxo-index.snapshot [--threads 23]
//!
//! Each line in the text file is a candidate phrase. The tool generates variations
//! (lowercase, uppercase, titlecase, trimmed, etc.), hashes each with SHA256,
//! derives the Bitcoin address, and checks against the UTXO index.

use anyhow::Result;
use bitcoin::key::{CompressedPublicKey, PrivateKey};
use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::Network;
use clap::Parser;
use crossbeam_channel::unbounded;
use sha2::{Digest, Sha256};
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
    #[arg(long, default_value = "brainwallet-matches.json")]
    output: String,
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

    // Deduplicate
    all_variants.sort();
    all_variants.dedup();
    println!("Total unique variations to test: {}", all_variants.len());

    // Load UTXO index
    println!("Loading UTXO index from {}...", &cli.snapshot);
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

    // Wrap index in Arc for thread-safe sharing (read-only after load)
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
            .name(format!("BW-Worker-{}", thread_id))
            .spawn(move || {
                let mut count = 0u64;
                for variant in &chunk {
                    // SHA256 hash -> private key
                    let hash = Sha256::digest(variant.as_bytes());
                    let key_bytes: [u8; 32] = hash.into();

                    // Validate key (must be < secp256k1 order)
                    if let Ok(secp_key) = bitcoin::secp256k1::SecretKey::from_slice(&key_bytes) {
                        let pk = PrivateKey {
                            inner: secp_key,
                            network: network.into(),
                            compressed: true,
                        };

                        if let Ok(compressed) = CompressedPublicKey::from_private_key(&secp, &pk) {
                            // Check P2PKH (legacy)
                            let addr = bitcoin::Address::p2pkh(&compressed, network);
                            let s = addr.script_pubkey();
                            let v = fi.lookup(s.as_bytes());
                            if v > 0 {
                                let sat_btc = v as f64 / 1e8;
                                let _ = tx.send(MatchResult {
                                    phrase: variant.clone(),
                                    address: addr.to_string(),
                                    address_type: "P2PKH (legacy)".to_string(),
                                    value_sats: v,
                                    value_btc: sat_btc,
                                });
                                total_matches.fetch_add(1, Ordering::Relaxed);
                            }

                            // Check P2WPKH (native segwit)
                            let addr_segwit = bitcoin::Address::p2wpkh(&compressed, network);
                            let s_segwit = addr_segwit.script_pubkey();
                            let v_segwit = fi.lookup(s_segwit.as_bytes());
                            if v_segwit > 0 {
                                let sat_btc = v_segwit as f64 / 1e8;
                                let _ = tx.send(MatchResult {
                                    phrase: variant.clone(),
                                    address: addr_segwit.to_string(),
                                    address_type: "P2WPKH (segwit)".to_string(),
                                    value_sats: v_segwit,
                                    value_btc: sat_btc,
                                });
                                total_matches.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }

                    count += 1;
                    if count % 100_000 == 0 {
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

    // Drop the sender in main thread so rx iterator ends
    drop(tx);

    // Collect results while threads run
    let mut matches: Vec<MatchResult> = Vec::new();
    for result in rx {
        matches.push(result);
    }

    // Wait for all threads
    for h in handles {
        h.join().expect("thread panicked");
    }

    let elapsed = start.elapsed();
    let rate = total_tested.load(Ordering::Relaxed) as f64 / elapsed.as_secs_f64();

    println!("\n============================================================");
    println!("  Brainwallet scan complete");
    println!("============================================================");
    println!("  Phrases tested: {}", total_tested.load(Ordering::Relaxed));
    println!("  Speed: {:.0} phrases/sec", rate);
    println!("  Time: {:.1}s", elapsed.as_secs_f64());
    println!("  Matches: {}", total_matches.load(Ordering::Relaxed));

    // Write matches to output file
    if !matches.is_empty() {
        serde_json::to_writer_pretty(
            &std::fs::File::create(&cli.output)?,
            &matches,
        )?;
        println!("\n  *** MATCHES FOUND! Written to {} ***", &cli.output);
        for m in &matches {
            println!(
                "    {} [{}]: {} BTC ({} sats) <- \"{}\"",
                m.address, m.address_type, m.value_btc, m.value_sats, m.phrase
            );
        }
    } else {
        println!("  No matches found.");
    }

    Ok(())
}

/// Generate variations of a phrase for brainwallet testing
fn generate_variations(phrase: &str) -> Vec<String> {
    let mut variants = Vec::new();

    // Original
    variants.push(phrase.to_string());

    // Trimmed
    let trimmed = phrase.trim();
    if trimmed != phrase {
        variants.push(trimmed.to_string());
    }

    // Lowercase
    let lower = trimmed.to_lowercase();
    if lower != trimmed {
        variants.push(lower.clone());
    }

    // Uppercase
    let upper = trimmed.to_uppercase();
    if upper != trimmed {
        variants.push(upper.clone());
    }

    // Title case (first letter of each word capitalized)
    let title = lower
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    if title != lower && title != upper && title != trimmed {
        variants.push(title);
    }

    // Remove all punctuation
    let no_punct: String = trimmed.chars().filter(|c| !c.is_ascii_punctuation()).collect();
    if no_punct != trimmed {
        variants.push(no_punct.clone());
        variants.push(no_punct.to_lowercase());
        variants.push(no_punct.to_uppercase());
    }

    // Remove extra spaces (collapse multiple spaces)
    let collapsed: String = trimmed
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if collapsed != trimmed {
        variants.push(collapsed.to_lowercase());
    }

    // With common suffixes (people often add "private key" or "bitcoin")
    for suffix in &[" private key", " bitcoin", " wallet", " btc", " key"] {
        variants.push(format!("{}{}", lower, suffix));
        variants.push(format!("{}{}", upper, suffix));
    }

    // With common prefixes
    for prefix in &["my ", "the ", "bitcoin ", "my bitcoin "] {
        variants.push(format!("{}{}", prefix, lower));
    }

    // Number substitutions (leet speak variants)
    if lower.contains('a') || lower.contains('e') || lower.contains('o') || lower.contains('t') {
        let leet: String = lower
            .chars()
            .map(|c| match c {
                'a' => '4',
                'e' => '3',
                'i' => '1',
                'o' => '0',
                't' => '7',
                _ => c,
            })
            .collect();
        if leet != lower {
            variants.push(leet);
        }
    }

    // Add year suffixes (common for brain wallets)
    for year in &["2009", "2010", "2011", "2012", "2013", "2014", "2015", "2016", "2017", "2018", "2019", "2020"] {
        variants.push(format!("{} {}", lower, year));
        variants.push(format!("{}{}", lower, year));
    }

    // With "!" and "?" suffixes
    variants.push(format!("{}!", lower));
    variants.push(format!("{}?", lower));
    variants.push(format!("{}!!!", lower));

    // Remove spaces entirely
    let no_spaces: String = lower.chars().filter(|c| !c.is_whitespace()).collect();
    if no_spaces != lower {
        variants.push(no_spaces);
    }

    // Underscore instead of spaces
    let underscores = lower.replace(' ', "_");
    if underscores != lower {
        variants.push(underscores);
    }

    // Hyphen instead of spaces
    let hyphens = lower.replace(' ', "-");
    if hyphens != lower {
        variants.push(hyphens);
    }

    variants
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct MatchResult {
    phrase: String,
    address: String,
    address_type: String,
    value_sats: u64,
    value_btc: f64,
}
