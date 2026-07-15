//! Check specific private keys against the UTXO FlatIndex (instant lookup).
//!
//! Usage:
//!   check_keys --key "a1b2c3..."                          (hex 32 bytes)
//!   check_keys --key "5HueCGU..."                          (WIF)
//!   check_keys --key "word1 word2 ... word12"             (BIP39 mnemonic)
//!   check_keys --key "hello world"                        (brainwallet - SHA256)
//!   check_keys --keys-file keys.txt                       (one key per line)
//!   check_keys --interactive                              (interactive mode)

use anyhow::Result;
use bitcoin::key::{CompressedPublicKey, PrivateKey, UntweakedPublicKey};
use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::{Address, Network};
use bip39::Mnemonic;
use clap::Parser;
use sha2::{Digest, Sha256};
use std::io::{self, BufRead};
use std::str::FromStr;

mod flat_index;

#[derive(Parser)]
struct Cli {
    /// Private key to check (hex, WIF, or BIP39 phrase)
    #[arg(short, long)]
    key: Option<String>,

    /// File with one key per line
    #[arg(long)]
    keys_file: Option<String>,

    /// Interactive mode (type keys one by one)
    #[arg(long)]
    interactive: bool,

    /// Path to the UTXO index snapshot
    #[arg(short, long, default_value = "utxo-index.snapshot")]
    snapshot: String,

    /// Minimum UTXO value in satoshis (dust filter)
    #[arg(long, default_value = "0")]
    min_value: u64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load UTXO index
    println!("Loading UTXO index from {}...", cli.snapshot);
    let load_start = std::time::Instant::now();
    let index = flat_index::FlatIndex::load_from_snapshot(&cli.snapshot, cli.min_value)?;
    let load_time = load_start.elapsed();
    index.print_stats();
    println!("  Loaded in {:?}\n", load_time);

    let secp = Secp256k1::<All>::new();
    let network = Network::Bitcoin;

    if cli.interactive {
        run_interactive(&index, &secp, network)?;
    } else if let Some(ref keys_file) = cli.keys_file {
        run_keys_file(&index, &secp, network, keys_file)?;
    } else if let Some(ref key) = cli.key {
        run_single_key(&index, &secp, network, key)?;
    } else {
        println!("Usage:");
        println!("  --key \"hex|WIF|phrase\"     Check a single key");
        println!("  --keys-file keys.txt        Check keys from file");
        println!("  --interactive               Interactive mode");
        println!();
        println!("Key formats:");
        println!("  Hex:    a1b2c3d4... (64 hex chars = 32 bytes)");
        println!("  WIF:    5HueCGU...");
        println!("  BIP39:  word1 word2 ... word12 (12+ words)");
        println!("  Text:   any text (hashed with SHA256 as brainwallet)");
    }

    Ok(())
}

fn run_single_key(index: &flat_index::FlatIndex, secp: &Secp256k1<All>, network: Network, key_input: &str) -> Result<()> {
    let results = check_key(index, secp, network, key_input);
    print_key_results(key_input, &results);
    Ok(())
}

fn run_keys_file(index: &flat_index::FlatIndex, secp: &Secp256k1<All>, network: Network, path: &str) -> Result<()> {
    let file = std::fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut total = 0u64;
    let mut with_balance = 0u64;
    let start = std::time::Instant::now();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        total += 1;
        let results = check_key(index, secp, network, &trimmed);
        let has_balance = results.iter().any(|r| r.balance_sats > 0);
        if has_balance {
            with_balance += 1;
        }
        print_key_results(&trimmed, &results);

        if total % 100 == 0 {
            let elapsed = start.elapsed();
            eprintln!("\r  Progress: {} keys checked in {:?} ({:.0} keys/sec)",
                total, elapsed, total as f64 / elapsed.as_secs_f64());
        }
    }

    let elapsed = start.elapsed();
    println!("\n{}", "=".repeat(60));
    println!("  File scan complete: {} keys in {:?}", total, elapsed);
    println!("  Speed: {:.0} keys/sec", total as f64 / elapsed.as_secs_f64());
    println!("  Keys with balance: {}", with_balance);
    println!("{}", "=".repeat(60));

    Ok(())
}

fn run_interactive(index: &flat_index::FlatIndex, secp: &Secp256k1<All>, network: Network) -> Result<()> {
    println!("Mode interactif - tapez une clé par ligne");
    println!("  Formats: hex (32 octets), WIF, BIP39 (12+ mots), ou texte libre");
    println!("  Tapez 'quit' ou 'q' pour sortir");
    println!("  Tapez 'file NOM_FICHIER' pour charger un fichier de clés");
    println!();

    let stdin = io::stdin();
    let mut total_checked = 0u64;
    let mut found_count = 0u64;
    let start = std::time::Instant::now();

    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim().to_string();

        if trimmed.eq_ignore_ascii_case("quit") || trimmed.eq_ignore_ascii_case("q") {
            break;
        }

        if let Some(filename) = trimmed.strip_prefix("file ") {
            run_keys_file(index, secp, network, filename.trim())?;
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        total_checked += 1;
        let results = check_key(index, secp, network, &trimmed);
        let has_balance = results.iter().any(|r| r.balance_sats > 0);
        if has_balance {
            found_count += 1;
        }

        print_key_results(&trimmed, &results);

        let elapsed = start.elapsed();
        println!("  [Total: {} clés vérifiées | {} avec solde | {:?}]",
            total_checked, found_count, elapsed);
        println!();
    }

    println!("\nSession terminée: {} clés vérifiées, {} avec solde", total_checked, found_count);
    Ok(())
}

struct KeyResult {
    address_type: String,
    address: String,
    balance_sats: u64,
    balance_btc: f64,
}

fn check_key(
    index: &flat_index::FlatIndex,
    secp: &Secp256k1<All>,
    network: Network,
    key_input: &str,
) -> Vec<KeyResult> {
    let mut results = Vec::new();

    let parsed = parse_key(key_input, network, secp);
    let Some((pk, pubkey)) = parsed else {
        // Try brainwallet: SHA256 hash of the text
        let hash = Sha256::digest(key_input.as_bytes());
        let key_bytes: [u8; 32] = hash.into();
        if let Ok(secp_key) = bitcoin::secp256k1::SecretKey::from_slice(&key_bytes) {
            let pk = PrivateKey {
                inner: secp_key,
                network: network.into(),
                compressed: true,
            };
            let pubkey = pk.public_key(secp);
            check_addresses(index, secp, network, &pk, &pubkey, &mut results);
        }
        return results;
    };

    check_addresses(index, secp, network, &pk, &pubkey, &mut results);
    results
}

fn check_addresses(
    index: &flat_index::FlatIndex,
    secp: &Secp256k1<All>,
    network: Network,
    pk: &PrivateKey,
    pubkey: &bitcoin::key::PublicKey,
    results: &mut Vec<KeyResult>,
) {
    let compressed = match CompressedPublicKey::from_private_key(secp, pk) {
        Ok(c) => c,
        Err(_) => return,
    };
    let xonly: UntweakedPublicKey = compressed.into();

    // Legacy P2PKH
    let addr = Address::p2pkh(pubkey, network);
    let s = addr.script_pubkey();
    let val = index.lookup(s.as_bytes());
    results.push(KeyResult {
        address_type: "Legacy (P2PKH)".to_string(),
        address: addr.to_string(),
        balance_sats: val,
        balance_btc: val as f64 / 100_000_000.0,
    });

    // Native Segwit P2WPKH
    let addr = Address::p2wpkh(&compressed, network);
    let s = addr.script_pubkey();
    let val = index.lookup(s.as_bytes());
    results.push(KeyResult {
        address_type: "Segwit (P2WPKH)".to_string(),
        address: addr.to_string(),
        balance_sats: val,
        balance_btc: val as f64 / 100_000_000.0,
    });

    // Wrapped Segwit P2SH-P2WPKH
    let addr = Address::p2shwpkh(&compressed, network);
    let s = addr.script_pubkey();
    let val = index.lookup(s.as_bytes());
    results.push(KeyResult {
        address_type: "Wrapped (P2SH-P2WPKH)".to_string(),
        address: addr.to_string(),
        balance_sats: val,
        balance_btc: val as f64 / 100_000_000.0,
    });

    // Taproot P2TR
    let addr = Address::p2tr(secp, xonly, None, network);
    let s = addr.script_pubkey();
    let val = index.lookup(s.as_bytes());
    results.push(KeyResult {
        address_type: "Taproot (P2TR)".to_string(),
        address: addr.to_string(),
        balance_sats: val,
        balance_btc: val as f64 / 100_000_000.0,
    });
}

fn parse_key(
    input: &str,
    network: Network,
    secp: &Secp256k1<All>,
) -> Option<(PrivateKey, bitcoin::key::PublicKey)> {
    // Try WIF
    if let Ok(pk) = bitcoin::PrivateKey::from_str(input) {
        let pk = PrivateKey { inner: pk.inner, network: network.into(), compressed: true };
        let pubkey = pk.public_key(secp);
        return Some((pk, pubkey));
    }

    // Try raw hex (32 bytes = 64 hex chars)
    if let Ok(bytes) = hex::decode(input) {
        if bytes.len() == 32 {
            if let Ok(inner) = bitcoin::secp256k1::SecretKey::from_slice(&bytes) {
                let pk = PrivateKey { inner, network: network.into(), compressed: true };
                let pubkey = pk.public_key(secp);
                return Some((pk, pubkey));
            }
        }
    }

    // Try BIP39 mnemonic (12+ words)
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.len() >= 12 {
        if let Ok(mnemonic) = Mnemonic::parse(input) {
            if mnemonic.words().count() >= 12 {
                let seed_bytes: [u8; 64] = mnemonic.to_seed("");

                if let Ok(xpriv) = bitcoin::bip32::Xpriv::new_master(network, &seed_bytes) {
                    if let Ok(path) = bitcoin::bip32::DerivationPath::from_str("m/44'/0'/0'/0/0") {
                        if let Ok(derived) = xpriv.derive_priv(secp, &path) {
                            let pk = PrivateKey {
                                inner: derived.private_key,
                                network: network.into(),
                                compressed: true,
                            };
                            let pubkey = pk.public_key(secp);
                            return Some((pk, pubkey));
                        }
                    }
                }
            }
        }
    }

    None // Fall through to brainwallet mode
}

fn print_key_results(key_input: &str, results: &[KeyResult]) {
    let display_key = if key_input.len() > 50 {
        format!("{}... ({} chars)", &key_input[..30], key_input.len())
    } else {
        key_input.to_string()
    };

    let total_balance: u64 = results.iter().map(|r| r.balance_sats).sum();

    println!("\n{}", "─".repeat(60));

    if total_balance > 0 {
        println!("  🔑 {} ", display_key, );
        println!("  💰 SOLDE TOTAL: {} sats ({:.8} BTC)",
            total_balance, total_balance as f64 / 100_000_000.0);
    } else {
        println!("  🔑 {}", display_key);
        println!("  Solde: 0 BTC");
    }

    for r in results {
        let status = if r.balance_sats > 0 {
            format!("✅ {} sats ({:.8} BTC)", r.balance_sats, r.balance_btc)
        } else {
            "vide".to_string()
        };
        println!("    [{:<20}] {:<45} {}", r.address_type, r.address, status);
    }
}
