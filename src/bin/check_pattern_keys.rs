//! Check specific "memorable" private key patterns against the UTXO index.
//!
//! Usage:
//!   check_pattern_keys --snapshot C:\btcsolver-cache\utxo-index.snapshot

use anyhow::Result;
use bitcoin::key::{CompressedPublicKey, PrivateKey};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::Network;
use clap::Parser;

mod flat_index;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    snapshot: String,

    /// Minimum UTXO value in satoshis
    #[arg(long, default_value = "0")]
    min_value: u64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("Loading UTXO index from {}...", &cli.snapshot);
    let index = flat_index::FlatIndex::load_from_snapshot(&cli.snapshot, cli.min_value)?;
    index.print_stats();

    let secp = Secp256k1::new();
    let network = Network::Bitcoin;

    // Generate all pattern keys
    let patterns = generate_patterns();
    println!("\nTesting {} pattern keys...\n", patterns.len());

    let mut matches_found = 0u64;

    for (i, (desc, key_bytes)) in patterns.iter().enumerate() {
        // Validate key
        if let Ok(secp_key) = bitcoin::secp256k1::SecretKey::from_slice(key_bytes) {
            let pk = PrivateKey {
                inner: secp_key,
                network: network.into(),
                compressed: true,
            };

            if let Ok(compressed) = CompressedPublicKey::from_private_key(&secp, &pk) {
                let mut found_any = false;

                // P2PKH (legacy)
                let addr = bitcoin::Address::p2pkh(&compressed, network);
                let s = addr.script_pubkey();
                let v = index.lookup(s.as_bytes());
                if v > 0 {
                    matches_found += 1;
                    println!(
                        "  *** MATCH #{} [{}] P2PKH: {} | {} BTC ({} sats) ***",
                        i + 1, desc, addr, v as f64 / 1e8, v
                    );
                    found_any = true;
                }

                // P2WPKH (native segwit)
                let addr_segwit = bitcoin::Address::p2wpkh(&compressed, network);
                let s_segwit = addr_segwit.script_pubkey();
                let v_segwit = index.lookup(s_segwit.as_bytes());
                if v_segwit > 0 {
                    matches_found += 1;
                    println!(
                        "  *** MATCH [{}] P2WPKH: {} | {} BTC ({} sats) ***",
                        desc, addr_segwit, v_segwit as f64 / 1e8, v_segwit
                    );
                    found_any = true;
                }

                if !found_any && (i + 1) % 10 == 0 {
                    println!("  ... {} patterns tested, 0 match jusqu'ici", i + 1);
                }
            }
        } else {
            // Invalid key (>= order or zero)
            if i < 10 {
                println!("  [{}] key invalide (>= secp256k1 order)", desc);
            }
        }
    }

    println!("\n============================================================");
    println!("  Patterns testes: {}", patterns.len());
    println!("  Correspondances: {}", matches_found);
    println!("============================================================");

    Ok(())
}

fn generate_patterns() -> Vec<(String, [u8; 32])> {
    let mut patterns = Vec::new();

    // --- Sequential bytes ---
    // 0,1,2,...,31
    let mut key = [0u8; 32];
    for (i, b) in key.iter_mut().enumerate() {
        *b = i as u8;
    }
    patterns.push(("0..31 sequential".to_string(), key));

    // 1,2,3,...,32
    for (i, b) in key.iter_mut().enumerate() {
        *b = (i + 1) as u8;
    }
    patterns.push(("1..32 sequential".to_string(), key));

    // --- All same byte (0x00 invalid, skip) ---
    for val in 1..=255u8 {
        key.fill(val);
        patterns.push((format!("all 0x{:02x} ({})", val, val), key));
    }

    // --- ASCII strings (zero-padded to 32 bytes) ---
    let ascii_strings = [
        "abcdefghi",
        "abcdefgh",
        "abcdef",
        "abc",
        "a",
        "abcdefghijklmnopqrstuvwxyz01234",
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ012345",
        "abcdefghijklmnopqrstuvwxyzabcdef",
        "bitcoin",
        "bitcoin private key",
        "bitcoin wallet",
        "password",
        "letmein",
        "12345678",
        "123456789",
        "1234567890",
        "12345678901234567890123456789012",
        "qwertyuiopasdfghjklzxcvbnm1234",
        "qwerty",
        "trustno1",
        "iloveyou",
        "sunshine",
        "princess",
        "football",
        "shadow",
        "monkey",
        "master",
        "michael",
        "hello",
        "welcome",
        "login",
        "admin",
        "test",
        "money",
        "rich",
        "wealth",
        "diamond",
        "satoshi",
        "satoshi nakamoto",
        "crypto",
        "blockchain",
        "wallet",
        "private",
        "key",
    ];
    for s in &ascii_strings {
        key.fill(0);
        let bytes = s.as_bytes();
        let len = bytes.len().min(32);
        key[..len].copy_from_slice(&bytes[..len]);
        patterns.push((format!("ASCII \"{}\" (zero-padded)", s), key));

        // Also try filling remaining bytes with 0xFF instead of 0x00
        if len < 32 {
            let mut key2 = [0u8; 32];
            key2[..len].copy_from_slice(&bytes[..len]);
            key2[len..].fill(0xFF);
            patterns.push((format!("ASCII \"{}\" (0xFF padded)", s), key2));
        }
    }

    // --- Repeating patterns ---
    let repeating: Vec<&[u8]> = vec![
        b"ab",
        b"abc",
        b"abcd",
        b"12",
        b"123",
        b"1234",
        b"aa",
        b"aaa",
        b"aaaa",
        b"00",
        b"01",
        b"0101",
        b"ffff",
        b"abcd1234",
        b"deadbeef",
        b"cafebabe",
        b"feedface",
    ];
    for pat in &repeating {
        key.fill(0);
        let plen = pat.len();
        for (i, b) in key.iter_mut().enumerate() {
            *b = pat[i % plen];
        }
        patterns.push((format!("repeat {:?}", std::str::from_utf8(pat).unwrap_or("?")), key));
    }

    // --- Hex patterns (common in examples/tutorials) ---
    let hex_patterns = [
        "0000000000000000000000000000000000000000000000000000000000000001", // key #1
        "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140", // secp256k1 order - 1 (max valid)
        "0000000000000000000000000000000000000000000000000000000000000000", // zero (invalid but check)
        "7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a0", // (order-1)/2
        "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f", // hex sequential
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", // all 0xFF (invalid but check)
        "0101010101010101010101010101010101010101010101010101010101010101", // all 0x01
        "0202020202020202020202020202020202020202020202020202020202020202", // all 0x02
        "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
        "cafebabecafebabecafebabecafebabe",
    ];
    for hex in &hex_patterns {
        if let Ok(bytes) = hex::decode(hex) {
            if bytes.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes);
                patterns.push((format!("hex {}", &hex[..16]), key));
            }
        }
    }

    // --- Byte increments from various starting points ---
    for start in [0x80u8, 0xC0u8, 0xE0u8, 0xF0u8, 0xFEu8] {
        for (i, b) in key.iter_mut().enumerate() {
            *b = start.wrapping_add(i as u8);
        }
        patterns.push((format!("sequential from 0x{:02x}", start), key));
    }

    // --- Decreasing sequences ---
    for (i, b) in key.iter_mut().enumerate() {
        *b = 31 - i as u8;
    }
    patterns.push(("31..0 descending".to_string(), key));

    for (i, b) in key.iter_mut().enumerate() {
        *b = 255 - i as u8;
    }
    patterns.push(("255..224 descending".to_string(), key));

    // --- Checkerboard patterns ---
    key.fill(0);
    for (i, b) in key.iter_mut().enumerate() {
        *b = if i % 2 == 0 { 0xFF } else { 0x00 };
    }
    patterns.push(("checkerboard FF/00".to_string(), key));

    for (i, b) in key.iter_mut().enumerate() {
        *b = if i % 2 == 0 { 0x00 } else { 0xFF };
    }
    patterns.push(("checkerboard 00/FF".to_string(), key));

    // --- Specific memorable byte arrays ---
    // [1, 2, 3, 4, 5, 6, 7, 8, ...] as user requested
    key.fill(0);
    for i in 0..8 {
        key[i] = i as u8 + 1;
    }
    patterns.push(("1,2,3,4,5,6,7,8 + zeros".to_string(), key));

    // [0, 1, 2, 3, 4, 5, 6, 7, ...] as user requested
    key.fill(0);
    for i in 0..8 {
        key[i] = i as u8;
    }
    patterns.push(("0,1,2,3,4,5,6,7 + zeros".to_string(), key));

    // All bytes = their index repeated: [0,0,1,1,2,2,...]
    for (i, b) in key.iter_mut().enumerate() {
        *b = (i / 2) as u8;
    }
    patterns.push(("00,11,22,...,ff pairs".to_string(), key));

    patterns
}
