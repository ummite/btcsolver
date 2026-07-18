//! Easy Key Generator v2 — Extended patterns for human-created private keys
//!
//! Focuses on patterns a human in 2009-2013 might have used:
//!   11. Bitcoin genesis block patterns
//!   12. Satoshi/Nakamoto references
//!   13. C/C++ programmer defaults (time(NULL), 0xDEADBEEF, etc.)
//!   14. Early mining software seeds
//!   15. Common brainwallet mistakes
//!   16. Simple wallet patterns (sequential, counter-based)
//!   17. Dice roll patterns (dice-based key generation was popular)
//!   18. Coin flip patterns (binary from coin flips)
//!   19. Card shuffle patterns
//!   20. ASCII art / emoji patterns (as hex)
//!
//! Usage: easy_key_generator_v2 --output data/easy-keys-v2.txt

use anyhow::Result;
use clap::Parser;
use digest::Digest;
use sha2::{Sha256, Sha512};
use std::collections::BTreeSet;
use std::io::Write;

#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = "data/easy-keys-v2.txt")]
    output: String,
    #[arg(long, default_value = "true")]
    transforms: bool,
}

fn hash_to_key(input: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hasher.finalize().into()
}

fn hex_to_key(hex: &str) -> Option<[u8; 32]> {
    let bytes = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i.min(i+2)], 16).ok())
        .collect::<Option<Vec<u8>>>()?;
    if bytes.len() == 32 {
        Some(bytes.try_into().unwrap())
    } else if bytes.len() < 32 {
        let mut padded = [0u8; 32];
        padded[32 - bytes.len()..].copy_from_slice(&bytes);
        Some(padded)
    } else {
        Some(hash_to_key(hex))
    }
}

fn should_include(categories: &Option<Vec<&str>>, name: &str) -> bool {
    match categories {
        None => true,
        Some(cats) => cats.contains(&name),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut all_keys: BTreeSet<[u8; 32]> = BTreeSet::new();

    // === 11. Bitcoin Genesis Patterns ===
    println!("[11/20] Bitcoin genesis patterns...");
    {
        let genesis_msg = "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";
        all_keys.insert(hash_to_key(genesis_msg));

        // Variations of the genesis message
        let variations = vec![
            "The Times 03/Jan/2009",
            "03/Jan/2009",
            "2009-01-03",
            "january 3 2009",
            "bitcoin genesis",
            "genesis block",
            "bit coin 2009",
            "bitcoin2009",
            "bitcoin genesis block",
            "satoshi nakamoto 2009",
            "nakamoto 2009",
            "bitcoin whitepaper",
            "bitcoin whitepaper 2008",
            "2008-10-31", // whitepaper date
            "october 31 2008",
        ];
        for v in variations {
            all_keys.insert(hash_to_key(v));
            // Also try without hashing (direct hex if short enough)
            if let Some(k) = hex_to_key(v) { all_keys.insert(k); }
        }

        // Genesis block hash components
        let genesis_hash = "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f";
        if let Some(k) = hex_to_key(genesis_hash) { all_keys.insert(k); }

        // First 32 chars of genesis hash
        if let Some(k) = hex_to_key(&genesis_hash[..64]) { all_keys.insert(k); }

        // Halving patterns
        for h in 0..100 {
            let halving_block = 210000 * h;
            all_keys.insert(hash_to_key(&halving_block.to_string()));
        }

        println!("  -> {} genesis keys", all_keys.len());
    }

    // === 12. Satoshi/Nakamoto References ===
    println!("[12/20] Satoshi references...");
    {
        let satoshi_refs = vec![
            "satoshi", "satoshi nakamoto", "nakamoto", "satoshinakamoto",
            "bitcoin founder", "bitcoin creator", "original bitcoin",
            "satoshi@bitcoin.org", "satoshi.nakamoto@gmail.com",
            "nakamotoinstitute", "bitcoin.org",
            "hal finney", "halfinney", "nicolas dobbins",
            "adam back", "adamback", "b hash", "bhash",
            "wei dai", "weidai", "nick szabo", "szabonick",
            "benjamin buggenhagen", "dorian nakamoto",
            "crypto cat", "metacat",
            // Satoshi's known PGP key ID
            "479C21C7",
            // Early bitcoin talk usernames
            "satoshi bitcoin talk",
            "bitcoin talk 2009",
            "bitcointalk",
            "bitcointalk.org",
        ];
        for ref_str in satoshi_refs {
            all_keys.insert(hash_to_key(ref_str));
            // Try common hash functions
            all_keys.insert(hash_to_key(&ref_str.to_uppercase()));
        }
        println!("  -> {} total keys", all_keys.len());
    }

    // === 13. C/C++ Programmer Defaults ===
    println!("[13/20] C/C++ programmer defaults...");
    {
        let before = all_keys.len();

        // Common magic numbers used as seeds
        let magic_numbers = vec![
            "DEADBEEF", "CAFEBABE", "BADC0DE", "FEEDFACE",
            "1337", "42", "0", "1", "42424242",
            "FFFF", "0000", "1234", "ABCD",
            "12345678", "87654321",
            // time(NULL) values for significant dates
            "1231006505", // 2009-01-03 18:15:05 (genesis block time)
            "1225478400", // 2008-11-01 (whitepaper ~)
            "1230768000", // 2009-01-01
            "1199145600", // 2008-01-01
            "1262304000", // 2010-01-01
            "1293840000", // 2011-01-01
            "1325376000", // 2012-01-01
            "1356998400", // 2013-01-01
        ];

        for mag in &magic_numbers {
            if let Some(k) = hex_to_key(mag) { all_keys.insert(k); }
            all_keys.insert(hash_to_key(mag));
            // As decimal string
            if let Ok(val) = mag.parse::<u64>() {
                all_keys.insert(hash_to_key(&val.to_string()));
                // Common srand() patterns
                for i in 0..1000 {
                    all_keys.insert(hash_to_key(&format!("{}:{}", mag, i)));
                }
            }
        }

        // getpid() values (1-4096 typical for early processes)
        for pid in 1..4097 {
            all_keys.insert(hash_to_key(&pid.to_string()));
        }

        // LCG (Linear Congruential Generator) with common seeds
        // glibc LCG: state = state * 1103515245 + 12345
        for seed in 0u32..10000 {
            let mut state = seed;
            for _ in 0..10 {
                state = state.wrapping_mul(1103515245).wrapping_add(12345);
                let key_bytes = state.to_le_bytes();
                let mut key = [0u8; 32];
                key[28..32].copy_from_slice(&key_bytes);
                all_keys.insert(key);
            }
        }

        println!("  -> {} new keys ({} total)", all_keys.len() - before, all_keys.len());
    }

    // === 14. Early Mining Software Seeds ===
    println!("[14/20] Mining software patterns...");
    {
        let before = all_keys.len();

        let mining_refs = vec![
            "hashcpu", "bmminer", "diablominer", "poclbm",
            "cgminer", "bfgminer", "cpuminer", "cpuguru",
            "bitcoin miner", "bitcoin mining", "mining 2009",
            "gpu mining", "cpu mining",
            "pool mining", "solo mining",
            "nicehash", "slush pool", "deepbit",
            // Common mining config values
            "stratum+tcp://", "getwork", "getblocktemplate",
        ];

        for ref_str in &mining_refs {
            all_keys.insert(hash_to_key(ref_str));
            // With common suffixes
            for i in 0..100 {
                all_keys.insert(hash_to_key(&format!("{}{}", ref_str, i)));
            }
        }

        // Common nonce patterns (miners use sequential nonces)
        for nonce in (0..65536u32).step_by(1) {
            let mut key = [0u8; 32];
            key[28..32].copy_from_slice(&nonce.to_le_bytes());
            all_keys.insert(key);
        }

        println!("  -> {} new keys ({} total)", all_keys.len() - before, all_keys.len());
    }

    // === 15. Common Brainwallet Mistakes ===
    println!("[15/20] Brainwallet mistakes...");
    {
        let before = all_keys.len();

        // Common phrases people actually used (from brainwallet.com era)
        let common_phrases = vec![
            "bitcoin", "bitcoin bitcoin", "my bitcoin", "i love bitcoin",
            "money", "free money", "easy money", "rich",
            "password", "password1", "password123", "123456",
            "letmein", "admin", "root", "master",
            "qwerty", "abc123", "trustno1",
            "sunshine", "princess", "football", "charlie",
            "shadow", "superman", "michael", "password123",
            "batman", "access", "hello", "charlie",
            "donald", "loveme", "fuckme", "mustang",
            // Common "creative" brainwallet phrases
            "i will be rich", "i am rich", "make me rich",
            "bitcoin is the future", "digital gold",
            "decentralized currency", "peer to peer",
            "trust but verify", "end money slavery",
            "math is beautiful", "entropy is life",
            // Wrong hash function usage (MD5 instead of SHA256)
            // These would be different from the correct SHA256 hash
        ];

        for phrase in &common_phrases {
            // SHA256 (correct)
            all_keys.insert(hash_to_key(phrase));
            // MD5 (common mistake — pad to 32 bytes)
            let md5_hash = md5::Md5::digest(phrase.as_bytes());
            let mut key = [0u8; 32];
            key[..16].copy_from_slice(&md5_hash[..16]);
            all_keys.insert(key);
            // Double SHA256
            let double = hash_to_key(&hex::encode(hash_to_key(phrase)));
            all_keys.insert(double);
            // SHA512 truncated
            let sha512 = sha2::Sha512::digest(phrase.as_bytes());
            let mut key512 = [0u8; 32];
            key512.copy_from_slice(&sha512[..32]);
            all_keys.insert(key512);
            // With common prefixes/suffixes
            for prefix in &["my key is ", "private key: ", "secret: ", ""] {
                all_keys.insert(hash_to_key(&format!("{}{}", prefix, phrase)));
            }
            for suffix in &[" key", " private", " secret", " bitcoin", " btc"] {
                all_keys.insert(hash_to_key(&format!("{}{}", phrase, suffix)));
            }
        }

        println!("  -> {} new keys ({} total)", all_keys.len() - before, all_keys.len());
    }

    // === 16. Simple Wallet Patterns ===
    println!("[16/20] Simple wallet patterns...");
    {
        let before = all_keys.len();

        // Sequential key generation (early wallets used simple counters)
        for i in 0u64..100000 {
            let mut key = [0u8; 32];
            key[24..32].copy_from_slice(&i.to_le_bytes());
            all_keys.insert(key);

            // Big-endian variant
            let mut key_be = [0u8; 32];
            key_be[0..8].copy_from_slice(&i.to_be_bytes());
            all_keys.insert(key_be);
        }

        // BIP32-style derivation from simple seeds
        for seed in 0u32..10000 {
            let seed_bytes = seed.to_be_bytes();
            let mut key = [0u8; 32];
            key[28..32].copy_from_slice(&seed_bytes);
            // m/0'/0'/i pattern (simplified)
            for i in 0..100u32 {
                let derived = hash_to_key(&format!("{}:{}:{}", seed, 0, i));
                all_keys.insert(derived);
            }
        }

        println!("  -> {} new keys ({} total)", all_keys.len() - before, all_keys.len());
    }

    // === 17. Dice Roll Patterns ===
    println!("[17/20] Dice roll patterns...");
    {
        let before = all_keys.len();

        // d6 dice (1-6) — 26 dice = 26*2.585 = 67.2 bits (not enough for 256)
        // d100 dice (1-100) — 9 dice = 9*6.644 = 59.8 bits
        // d10 dice (0-9) — 27 dice = 27*3.322 = 89.7 bits
        // People often used many dice or repeated patterns

        // All same dice: 111111..., 222222..., etc.
        for d in 1u8..=6 {
            let mut key = [d; 32];
            all_keys.insert(key);
        }

        // Sequential dice: 123456123456...
        let seq = [1u8, 2, 3, 4, 5, 6];
        let mut key = [0u8; 32];
        for i in 0..32 { key[i] = seq[i % 6]; }
        all_keys.insert(key);

        // Reverse sequential: 654321654321...
        let rev = [6u8, 5, 4, 3, 2, 1];
        for i in 0..32 { key[i] = rev[i % 6]; }
        all_keys.insert(key);

        // All combinations of N dice rolls (for small N)
        // 6 dice = 6^6 = 46656 combinations
        for i in 0u32..46656 {
            let mut key = [0u8; 32];
            let mut val = i;
            for j in (0..6).rev() {
                key[j] = (val % 6 + 1) as u8;
                val /= 6;
            }
            all_keys.insert(key);
        }

        // Decimal dice (d10): 0-9 patterns
        for i in 0u32..100000 {
            let s = format!("{:05}", i);
            let mut key = [0u8; 32];
            for (j, c) in s.bytes().enumerate() {
                key[j] = c - b'0';
            }
            all_keys.insert(key);
        }

        println!("  -> {} new keys ({} total)", all_keys.len() - before, all_keys.len());
    }

    // === 18. Coin Flip Patterns ===
    println!("[18/20] Coin flip patterns...");
    {
        let before = all_keys.len();

        // All heads / all tails
        all_keys.insert([0u8; 32]);
        all_keys.insert([0xFF; 32]);

        // Alternating: HTHTHT...
        let mut key = [0u8; 32];
        for i in 0..32 { if i % 2 == 0 { key[i] = 0xFF; } }
        all_keys.insert(key);

        // Common patterns: HHHHHTTTTT, HTHT, etc.
        let patterns = vec![
            [0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00], // HHHHHTTT
            [0xF0, 0xF0, 0xF0, 0xF0, 0x0F, 0x0F, 0x0F, 0x0F], // HHLL pattern
            [0xAA, 0xAA, 0xAA, 0xAA, 0x55, 0x55, 0x55, 0x55], // alternating nibbles
            [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80], // powers of 2
            [0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01], // reverse powers of 2
        ];
        for pat in &patterns {
            let mut key = [0u8; 32];
            for i in 0..32 { key[i] = pat[i % 8]; }
            all_keys.insert(key);
        }

        // Single bit set at each position (256 keys)
        for bit in 0..256 {
            let mut key = [0u8; 32];
            key[bit / 8] = 1 << (7 - (bit % 8));
            all_keys.insert(key);
        }

        // Single bit clear at each position
        for bit in 0..256 {
            let mut key = [0xFFu8; 32];
            key[bit / 8] &= !(1 << (7 - (bit % 8)));
            all_keys.insert(key);
        }

        println!("  -> {} new keys ({} total)", all_keys.len() - before, all_keys.len());
    }

    // === 19. Card Shuffle Patterns ===
    println!("[19/20] Card shuffle patterns...");
    {
        let before = all_keys.len();

        // Standard deck order (52 cards = 52 * log2(52) ≈ 285 bits)
        // People sometimes used card shuffles for entropy
        // Common patterns: sorted deck, reverse sorted, bridge sort

        // Sorted deck: 2C, 3C, ..., AC, 2D, ..., AD, 2H, ..., AH, 2S, ..., AS
        let suits = ['C', 'D', 'H', 'S'];
        let ranks = ['2', '3', '4', '5', '6', '7', '8', '9', 'T', 'J', 'Q', 'K', 'A'];
        let mut deck_str = String::new();
        for s in &suits {
            for r in &ranks {
                deck_str.push(*r);
                deck_str.push(*s);
            }
        }
        all_keys.insert(hash_to_key(&deck_str));

        // Reverse sorted
        let rev_deck: String = deck_str.chars().rev().collect();
        all_keys.insert(hash_to_key(&rev_deck));

        // Bridge sort (group by rank)
        let mut bridge = String::new();
        for r in &ranks {
            for s in &suits {
                bridge.push(*r);
                bridge.push(*s);
            }
        }
        all_keys.insert(hash_to_key(&bridge));

        // Common poker hands as seeds
        let poker_hands = vec![
            "AAAAK", "AAAAQ", "AAAAJ", "AAAAT", // quads
            "AKQJT", "AKJT9", "KQJT9", // straight flush patterns
            "23457", "75432", // wheel
            "10JQKA", "AJKT9", // common starting hands
        ];
        for hand in &poker_hands {
            all_keys.insert(hash_to_key(hand));
        }

        println!("  -> {} new keys ({} total)", all_keys.len() - before, all_keys.len());
    }

    // === 20. ASCII Art / Emoji / Special Patterns ===
    println!("[20/20] Special patterns...");
    {
        let before = all_keys.len();

        // Bitcoin symbol patterns
        let btc_patterns = vec![
            "BTC", "btc", "XBT", "xbt", "bitcoin",
            "bitcoin bitcoin bitcoin", // repetition
            "btc btc btc btc btc btc btc btc",
            "BBBBB", // B = bitcoin
            "$$$", "money", "satoshi", "sats",
            "1BTC", "1btc", "0.1BTC", "100BTC", "21000000",
            "21 million", "21000000 bitcoin",
            // Keyboard rows
            "qwertyuiop", "asdfghjkl", "zxcvbnm",
            "QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM",
            "1234567890", "0987654321",
            "!@#$%^&*()", "~`1234567890-=",
            // Diagonal keyboard patterns
            "1qaz", "2wsx", "3edc", "4rfv", "5tgb", "6yhn", "7ujm",
            // Common passphrase patterns
            "correct horse battery staple", // xkcd reference
            "satoshi vision decentralized",
            "trust no one verify everything",
        ];

        for pat in &btc_patterns {
            all_keys.insert(hash_to_key(pat));
            // Common variations
            all_keys.insert(hash_to_key(&pat.replace(" ", "")));
            all_keys.insert(hash_to_key(&pat.to_uppercase()));
            all_keys.insert(hash_to_key(&pat.to_lowercase()));
        }

        // Repeating byte patterns
        for byte in (0..=255u8) {
            let key = [byte; 32];
            all_keys.insert(key);
        }

        // Incrementing byte patterns: 00 01 02 ... 1F
        let mut key = [0u8; 32];
        for i in 0..32 { key[i] = i as u8; }
        all_keys.insert(key);

        // Decrementing: 1F 1E ... 00
        for i in 0..32 { key[i] = (31 - i) as u8; }
        all_keys.insert(key);

        // Fibonacci mod 256
        let mut a: u16 = 0;
        let mut b: u16 = 1;
        for i in 0..32 {
            key[i] = (a % 256) as u8;
            let c = a + b;
            a = b;
            b = c;
        }
        all_keys.insert(key);

        println!("  -> {} new keys ({} total)", all_keys.len() - before, all_keys.len());
    }

    // Apply transforms
    let before = all_keys.len();
    if cli.transforms {
        let mut transformed = BTreeSet::new();
        for key in &all_keys {
            let mut rev = *key;
            rev.reverse();
            transformed.insert(rev);

            let mut rotl = [0u8; 32];
            rotl[..31].copy_from_slice(&key[1..]);
            rotl[31] = key[0];
            transformed.insert(rotl);

            let mut rotr = [0u8; 32];
            rotr[0] = key[31];
            rotr[1..].copy_from_slice(&key[..31]);
            transformed.insert(rotr);
        }
        all_keys.extend(transformed);
        println!("\nTransforms: {} -> {} keys", before, all_keys.len());
    }

    // Filter invalid keys
    let zero_key = [0u8; 32];
    all_keys.remove(&zero_key);

    // secp256k1 order (keys >= n are invalid)
    let n: [u8; 32] = [
        0xFC, 0x62, 0xFE, 0xE6, 0xAF, 0x95, 0x33, 0x14,
        0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
        0xBC, 0x6E, 0xFE, 0xE3, 0x64, 0x65, 0xE9, 0x66,
        0xCF, 0x7B, 0x09, 0x56, 0xFE, 0x9B, 0xAC, 0xDC,
    ];
    all_keys.retain(|k| *k < n);

    // Write output
    let output_path = std::path::Path::new(&cli.output);
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = std::fs::File::create(&cli.output)?;
    let mut count = 0;
    for key in &all_keys {
        let hex: String = key.iter().map(|b| format!("{:02x}", b)).collect();
        writeln!(file, "{}", hex)?;
        count += 1;
    }

    let file_size = std::fs::metadata(&cli.output)?.len();

    println!("\n=== Summary ===");
    println!("Total unique keys: {}", count);
    println!("Output: {}", cli.output);
    println!("File size: {:.2} MB", file_size as f64 / (1024.0 * 1024.0));

    Ok(())
}
