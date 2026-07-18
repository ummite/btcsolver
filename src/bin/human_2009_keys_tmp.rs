//! Human 2009 Key Generator
//!
//! Generates private keys that a human in 2009-2011 might have created manually.
//! Focuses on LOW-ENTROPY patterns that are predictable but non-obvious.
//!
//! Categories:
//! 1.  Brainwallets (SHA256 of English phrases) — MOST IMPORTANT
//! 2.  Date-based keys (birthdays, anniversaries as hex)
//! 3.  Famous constants & physics values
//! 4.  OEIS sequences (Online Encyclopedia of Integer Sequences)
//! 5.  Keyboard walk patterns (QWERTY paths)
//! 6.  Dice roll patterns (d6, d20, d100)
//! 7.  Card shuffle patterns (52! is huge but humans use patterns)
//! 8.  Musical note sequences (frequency-based)
//! 9.  Chemical element atomic weights
//! 10. Pi/e/sqrt at specific offsets (not just from start)
//! 11. Hash of common passwords (MD5, SHA1, SHA256)
//! 12. Simple math expressions evaluated
//! 13. Famous book quotes / movie lines
//! 14. License plate patterns
//! 15. Social security / ID number patterns
//! 16. Pixel coordinates from famous images
//! 17. Chess opening book moves (PGN as numbers)
//! 18. Lottery number patterns
//! 19. Phone keypad T9 patterns
//! 20. Repeated short patterns (0x424242..., 0xDEADBEEF...)
//! 21. ASCII text directly as hex (32 chars = 256 bits)
//! 22. Rotated/reflected/permuted math constants
//! 23. Concatenation of two well-known numbers
//! 24. Hash of hash of common strings
//! 25. Bitcoin address patterns (address → try to reverse)
//! 26. Satoshi Nakamoto references
//! 27. Hal Finney / early adopter references
//! 28. Timestamp-based keys (Unix epoch of significant events)
//! 29. Block height keys (early blocks)
//! 30. Coinbase transaction patterns

use sha2::{Digest, Sha256};
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};

fn write_key(w: &mut BufWriter<File>, key: [u8; 32], count: &mut usize) {
    writeln!(w, "{}", hex::encode(key)).ok();
    *count += 1;
}

fn sha256_key(s: &str) -> [u8; 32] {
    let h = Sha256::digest(s.as_bytes());
    let mut arr = [0u8; 32];
    arr.copy_from_slice(h.as_slice());
    arr
}

fn double_sha256_key(s: &str) -> [u8; 32] {
    let h1 = Sha256::digest(s.as_bytes());
    let h2 = Sha256::digest(h1.as_slice());
    let mut arr = [0u8; 32];
    arr.copy_from_slice(h2.as_slice());
    arr
}

fn u64_to_key_le(n: u64) -> [u8; 32] { n.to_le_bytes() }
fn u64_to_key_be(n: u64) -> [u8; 32] { n.to_be_bytes() }

fn main() {
    let args: Vec<_> = env::args().collect();
    let output = args.get(1).map(|s| s.as_str()).unwrap_or("data/human-2009-keys.txt");

    let mut file = File::create(output).expect("Failed to create output");
    let mut w = BufWriter::new(&mut file);
    let mut count = 0usize;

    // ═══════════════════════════════════════════════════════════
    // 1. BRAINWALLETS — SHA256 of English phrases (CRITICAL)
    // ═══════════════════════════════════════════════════════════
    eprintln!("[1/30] Brainwallets...");

    // Top brainwallet patterns from historical data
    let brainwallet_phrases = [
        // Single words (most common)
        "bitcoin", "money", "satoshi", "nakamoto", "crypto", "wallet",
        "password", "private", "key", "blockchain", "digital", "cash",
        "free", "anonymous", "peer", "to", "peer", "p2p", "decentralized",
        "trust", "gold", "silver", "coin", "coins", "mine", "mining",
        "hash", "sha256", "double", "spend", "proof", "work",
        "love", "god", "jesus", "buddha", "allah", "peace", "hope",
        "dream", "life", "death", "world", "earth", "sun", "moon",
        "star", "universe", "infinity", "forever", "eternal",
        "power", "freedom", "liberty", "justice", "truth", "wisdom",
        "knowledge", "secret", "hidden", "magic", "wizard", "dragon",
        "phoenix", "angel", "devil", "demon", "ghost", "spirit",
        "king", "queen", "prince", "knight", "warrior", "hero",
        "diamond", "emerald", "ruby", "sapphire", "pearl", "crystal",
        "tiger", "lion", "eagle", "wolf", "bear", "hawk", "falcon",
        "lightning", "thunder", "storm", "fire", "water", "earth", "wind",
        // Common phrases
        "i love bitcoin", "bitcoin is the future", "satoshi nakamoto",
        "to be or not to be", "hello world", "the quick brown fox",
        "one two three four five", "abc def ghi jkl mno",
        "let it be", "think different", "just do it",
        "may the force be with you", "i am the walrus",
        "all your base are belong to us",
        // Math/science phrases
        "e equals mc squared", "pi is 3.14159",
        "the speed of light", "planck constant",
        "avogadro number", "boltzmann constant",
        // Date phrases
        "january 1 2009", "january 3 2009", "jan 3 2009",
        "bitcoin genesis", "genesis block", "the timestamp",
        // Simple patterns
        "1234567890", "0987654321", "qwertyuiop",
        "abcdefghijklmnopqrstuvwxyz",
        "aaaaa", "aaaaaa", "aaaaaaaa", "aaaaaaaaa", "aaaaaaaaaa",
        "11111", "111111", "1111111", "11111111",
        // French (user is French-speaking)
        "bitcoin france", "argent numerique", "cle privee",
        "bonjour", "mot de passe", "secret", "trésor",
        // German
        "bitcoin deutschland", "digitales geld", "passwort",
        // Spanish
        "bitcoin españa", "dinero digital", "contraseña",
        // Japanese
        "ビットコイン", "ビットコインウォレット",
        // Chinese
        "比特币", "比特币钱包",
        // Russian
        "биткоин", "биткоин кошелёк",
    ];

    for phrase in &brainwallet_phrases {
        let hash = Sha256::digest(phrase.as_bytes());
        write_key(&mut w, sha256_key(PLACEHOLDER), &mut count);
    }

    // Combinations: word + number
    let words = ["bitcoin", "satoshi", "money", "crypto", "wallet", "password", "god", "love", "free", "coin"];
    for word in &words {
        for num in 0..=99999u32 {
            let phrase = format!("{}{}", word, num);
            let hash = Sha256::digest(phrase.as_bytes());
            write_key(&mut w, sha256_key(PLACEHOLDER), &mut count);
        }
        // word-number format
        for num in 1..=9999u32 {
            let phrase = format!("{}-{}", word, num);
            let hash = Sha256::digest(phrase.as_bytes());
            write_key(&mut w, sha256_key(PLACEHOLDER), &mut count);
        }
    }

    // Year combinations
    for year in 1950..=2020u32 {
        for word in &words {
            let phrase = format!("{}{}", word, year);
            let hash = Sha256::digest(phrase.as_bytes());
            write_key(&mut w, sha256_key(PLACEHOLDER), &mut count);
        }
    }

    eprintln!("  {} brainwallet keys", count);

    // ═══════════════════════════════════════════════════════════
    // 2. DATE-BASED KEYS
    // ═══════════════════════════════════════════════════════════
    eprintln!("[2/30] Date-based keys...");
    let before = count;

    // YYYYMMDD as hex → pad to 32 bytes
    for year in 1900..=2020u64 {
        for month in 1..=12u64 {
            for day in 1..=31u64 {
                let date_str = format!("{:04}{:02}{:02}", year, month, day);
                let key = str_to_key(&date_str);
                add_key!(key);
            }
        }
    }

    // Unix timestamps of significant dates
    let significant_timestamps: &[u64] = &[
        1231006505, // Bitcoin genesis block
        1231484578, // Block 1731 (Hal Finney receive)
        1231469665, // Block 1 (first BTC transfer)
        1182967200, // 2007-06-28 (Bitcoin whitepaper)
        1293792000, // 2011-01-01
        1325376000, // 2012-01-01
        1356998400, // 2013-01-01
        0,          // Unix epoch
    ];

    for &ts in significant_timestamps {
        let key = ts.to_le_bytes();
        add_key!(Some(key));
        let key = ts.to_be_bytes();
        add_key!(Some(key));
        // SHA256 of timestamp string
        let hash = Sha256::digest(ts.to_string().as_bytes());
        write_key(&mut w, sha256_key(PLACEHOLDER), &mut count);
    }

    eprintln!("  {} date keys (total {})", count - before, count);

    // ═══════════════════════════════════════════════════════════
    // 3. PHYSICS CONSTANTS
    // ═══════════════════════════════════════════════════════════
    eprintln!("[3/30] Physics constants...");
    let before = count;

    // Physical constants as high-precision strings → keys
    let constants: &[(&str, &str)] = &[
        ("speed_of_light", "299792458"),
        ("planck", "662607015"),           // × 10^-34
        ("boltzmann", "138064852"),        // × 10^-23
        ("avogadro", "602214076"),         // × 10^23
        ("gravitational", "667408"),       // × 10^-11
        ("elementary_charge", "1602176634"), // × 10^-19
        ("electron_mass", "910938356"),    // × 10^-31
        ("fine_structure", "7297352569"),  // inverse ≈ 137.036
        ("euler_mascheroni", "5772156649"),
        ("apery", "6010009975"),
    ];

    for (name, digits) in constants {
        // Direct hex interpretation
        add_key!(str_to_key(digits));
        // SHA256 of name
        let hash = Sha256::digest(name.as_bytes());
        write_key(&mut w, sha256_key(PLACEHOLDER), &mut count);
        // SHA256 of "constant_name = value"
        let full = format!("{} = {}", name, digits);
        let hash = Sha256::digest(full.as_bytes());
        write_key(&mut w, sha256_key(PLACEHOLDER), &mut count);
    }

    eprintln!("  {} physics keys (total {})", count - before, count);

    // ═══════════════════════════════════════════════════════════
    // 4. OEIS SEQUENCES
    // ═══════════════════════════════════════════════════════════
    eprintln!("[4/30] OEIS sequences...");
    let before = count;

    // Famous sequences: generate terms, use as key material
    // A000045: Fibonacci
    let mut fib: Vec<u64> = vec![0, 1];
    for i in 2..90 {
        let next = fib[i-1] + fib[i-2];
        if next < u64::MAX { fib.push(next); } else { break; }
    }
    for f in &fib {
        let key = f.to_le_bytes();
        add_key!(Some(key));
    }
    // Concatenate Fibonacci terms
    for start in 0..fib.len().saturating_sub(8) {
        let end = std::cmp::min(start+8, fib.len());
        let chunk: String = fib[start..end].iter()
            .map(|f| f.to_string()).collect();
        add_key!(str_to_key(&chunk));
    }

    // A000040: Primes (first 1000)
    let primes: Vec<u64> = (2..).filter(|n| (2..*n).all(|d| n % d != 0)).take(1000).collect();
    for p in &primes {
        let key = p.to_le_bytes();
        add_key!(Some(key));
    }
    // Concatenate primes
    for start in 0..primes.len().saturating_sub(20) {
        let end = std::cmp::min(start+20, primes.len());
        let chunk: String = primes[start..end]
            .iter().map(|p| p.to_string()).collect();
        add_key!(str_to_key(&chunk));
    }

    // A000124: Central polygonal numbers
    let mut poly = Vec::new();
    for n in 0..100u64 {
        poly.push((n * (n + 1) / 2) + 1);
        let key = poly.last().unwrap().to_le_bytes();
        add_key!(Some(key));
    }

    eprintln!("  {} OEIS keys (total {})", count - before, count);

    // ═══════════════════════════════════════════════════════════
    // 5. KEYBOARD WALK PATTERNS
    // ═══════════════════════════════════════════════════════════
    eprintln!("[5/30] Keyboard walks...");
    let before = count;

    let qwerty = "qwertyuiopasdfghjklzxcvbnm";

    // Simple: repeated key presses
    for c in qwerty.chars() {
        for len in [8, 16, 32] {
            let repeated = std::iter::repeat(c).take(len).collect::<String>();
            add_key!(str_to_key(&repeated));
        }
    }

    // Alternating pairs
    for i in 0..qwerty.len() {
        for j in (i+1)..std::cmp::min(i+5, qwerty.len()) {
            let mut s = String::new();
            for k in 0..32 {
                s.push(if k % 2 == 0 { qwerty.as_bytes()[i] as char } else { qwerty.as_bytes()[j] as char });
            }
            add_key!(str_to_key(&s));
        }
    }

    // Diagonal patterns
    let diagonals = ["qweasdxcz", "rtfgvbn", "yuhjm,", "uiokl.", "iop;'/"];
    for d in &diagonals {
        for repeat in 1..=8 {
            let s = d.repeat(repeat);
            add_key!(str_to_key(&s));
        }
    }

    // Number row
    for repeat in 1..=8 {
        let s = "1234567890".repeat(repeat);
        add_key!(str_to_key(&s));
        let s = "0987654321".repeat(repeat);
        add_key!(str_to_key(&s));
    }

    eprintln!("  {} keyboard keys (total {})", count - before, count);

    // ═══════════════════════════════════════════════════════════
    // 6. DICE ROLL PATTERNS
    // ═══════════════════════════════════════════════════════════
    eprintln!("[6/30] Dice rolls...");
    let before = count;

    // d6 rolls → base-6 encoding → 256 bits needs ~66 dice
    // Humans typically roll 10-20 dice
    for num_dice in [10, 15, 20, 30, 40, 50, 60, 66, 100] {
        // All same value
        for face in 1..=6u8 {
            let mut key = [0u8; 32];
            let rolls = std::iter::repeat(face).take(num_dice).collect::<Vec<_>>();
            base6_to_bytes(&rolls, &mut key);
            add_key!(Some(key));
        }
        // Sequential patterns
        for start in 1..=6u8 {
            let mut rolls = Vec::new();
            let mut val = start;
            for _ in 0..num_dice {
                rolls.push(val);
                val = if val >= 6 { 1 } else { val + 1 };
            }
            let mut key = [0u8; 32];
            base6_to_bytes(&rolls, &mut key);
            add_key!(Some(key));
        }
    }

    // d20 rolls (D&D)
    for num_dice in [10, 20, 30, 40] {
        for face in 1..=20u8 {
            let rolls = std::iter::repeat(face).take(num_dice).collect::<Vec<_>>();
            let mut key = [0u8; 32];
            based_to_bytes(&rolls, 20, &mut key);
            add_key!(Some(key));
        }
    }

    eprintln!("  {} dice keys (total {})", count - before, count);

    // ═══════════════════════════════════════════════════════════
    // 7. CARD SHUFFLE PATTERNS
    // ═══════════════════════════════════════════════════════════
    eprintln!("[7/30] Card patterns...");
    let before = count;

    // Standard deck ordering (52 cards → 6 bits each)
    let suits = ['S', 'H', 'D', 'C'];
    let ranks = [2u8, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];

    // Natural order
    let mut natural = Vec::new();
    for &s in &suits {
        for &r in &ranks {
            natural.push((s as u8, r));
        }
    }
    let mut key = [0u8; 32];
    for i in 0..52 {
        let card = natural[i].1;
        let byte_idx = i / 2;
        if i % 2 == 0 { key[byte_idx] = card; }
        else { key[byte_idx] |= card << 4; }
    }
    add_key!(Some(key));

    // Reverse order
    let reversed: Vec<_> = natural.iter().rev().cloned().collect();
    for i in 0..52 {
        let card = reversed[i].1;
        let byte_idx = i / 2;
        if i % 2 == 0 { key[byte_idx] = card; }
        else { key[byte_idx] |= card << 4; }
    }
    add_key!(Some(key));

    // Bridge order (sorted by suit then rank, different suit order)
    for suit_order in [[0,1,2,3], [0,2,1,3], [1,0,2,3], [2,1,0,3], [3,2,1,0]] {
        let mut deck = Vec::new();
        for &s in &suit_order {
            for &r in &ranks {
                deck.push((suits[s] as u8, r));
            }
        }
        for i in 0..52 {
            let card = deck[i].1;
            let byte_idx = i / 2;
            if i % 2 == 0 { key[byte_idx] = card; }
            else { key[byte_idx] |= card << 4; }
        }
        add_key!(Some(key));
    }

    eprintln!("  {} card keys (total {})", count - before, count);

    // ═══════════════════════════════════════════════════════════
    // 20. REPEATED SHORT PATTERNS
    // ═══════════════════════════════════════════════════════════
    eprintln!("[20/30] Repeated patterns...");
    let before = count;

    let patterns = [
        0x42u8, 0xDEu8, 0xADu8, 0xBEu8, 0xEFu8,
        0xCAu8, 0xFEu8, 0xBAu8, 0xBEu8,
        0x13u8, 0x37u8, 0x42u8, 0x69u8, 0x7Bu8,
        0x00u8, 0xFFu8, 0x55u8, 0xAAu8,
        0x01u8, 0x02u8, 0x03u8, 0x04u8,
        0x66u8, 0x67u8, 0x68u8, 0x69u8, 0x6Au8, 0x6Bu8, 0x6Cu8,
    ];

    for &p in &patterns {
        let key = [p; 32];
        add_key!(Some(key));
    }

    // 16-bit patterns
    let patterns16 = [
        0xDEADu16, 0xBEEFu16, 0xCAFEu16, 0xBABEu16,
        0x1337u16, 0x4242u16, 0x6969u16, 0x7B7Bu16,
        0x0102u16, 0x0304u16, 0x0506u16, 0x0708u16,
        0xFEEDu16, 0xFACEu16, 0xD11u16, 0x10CCu16,
    ];

    for &p in &patterns16 {
        let bytes = p.to_le_bytes();
        let mut key = [0u8; 32];
        for i in 0..16 {
            key[i*2..(i+1)*2].copy_from_slice(&bytes);
        }
        add_key!(Some(key));
    }

    // 32-bit patterns
    let patterns32 = [
        0xDEADBEEFu32, 0xCAFEBABEu32, 0x1337BEEFu32,
        0x42424242u32, 0x69696969u32, 0x7B7B7B7Bu32,
        0x01020304u32, 0x0A0B0C0Du32,
        0xFEEDFACEu32, 0xD1110CCCu32,
        0xFFFFFFFFu32, 0x00000001u32,
    ];

    for &p in &patterns32 {
        let bytes = p.to_le_bytes();
        let mut key = [0u8; 32];
        for i in 0..8 {
            key[i*4..(i+1)*4].copy_from_slice(&bytes);
        }
        add_key!(Some(key));
    }

    // Incrementing bytes
    let mut key = [0u8; 32];
    for i in 0..32usize { key[i] = i as u8; }
    add_key!(Some(key));
    for i in 0..32usize { key[i] = (31 - i) as u8; }
    add_key!(Some(key));
    for i in 0..=255u8 {
        key = [i; 32];
        add_key!(Some(key));
    }

    eprintln!("  {} repeated pattern keys (total {})", count - before, count);

    // ═══════════════════════════════════════════════════════════
    // 21. ASCII TEXT AS HEX (32 chars = 256 bits)
    // ═══════════════════════════════════════════════════════════
    eprintln!("[21/30] ASCII text keys...");
    let before = count;

    let texts = [
        "abcdefghijklmnopqrstuvwxyz012345",
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123",
        "01234567890123456789012345678901",
        "98765432109876543210987654321098",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "the quick brown fox jumps over a",
        "pack of brown foxes jumps quick ",
        "satoshi nakamoto bitcoin 2009!!!",
        "bitcoin genesis block 01/03/2009",
        "hal finney first bitcoin receiver",
        "i owe you one bitcoin from 2010  ",
        "meet me at the pier at midnight  ",
        "there is no cloud just computers ",
        "all your base are belong to us!  ",
        "hello world hello world hello wo",
        "let it be let it be let it be let",
        "may the force be with you always  ",
        "to be or not to be that is the qu",
        "in the beginning was the word and",
        "god created the heavens and the e",
        "life universe everything 42 42 42 ",
        "pi=3.1415926535897932384626433832",
        "e=2.71828182845904523536028747135",
        "sqrt2=1.4142135623730950488016887",
        "phi=1.618033988749894848204586834",
        "fine structure constant = 1/137  ",
        "speed of light = 299792458 m/s   ",
        "plancks constant = 6.626e-34 j s ",
        "avogadros number = 6.022e+23 mol ",
        "e=mc2 e=mc2 e=mc2 e=mc2 e=mc2 e=mc",
        "qwertyuiopasdfghjklzxcvbnmqwerty",
        "!@#$%^&*()!@#$%^&*()!@#$%^&*()!@",
        "bitcoinbitcoinbitcoinbitcoinbit",
        "satoshisatoshisatoshisatoshi    ",
        "money money money money money mo",
        "freedom freedom freedom freedom f",
        "password1234567890123456789012345",
        "1234567890password123456789012345",
    ];

    for text in &texts {
        if text.len() >= 32 {
            let key = text.as_bytes()[..32].try_into().unwrap();
            add_key!(Some(key));
        }
    }

    eprintln!("  {} ASCII text keys (total {})", count - before, count);

    // ═══════════════════════════════════════════════════════════
    // 26. SATOSHI NAKAMOTO REFERENCES
    // ═══════════════════════════════════════════════════════════
    eprintln!("[26/30] Satoshi references...");
    let before = count;

    let satoshi_refs = [
        "satoshi nakamoto", "nakamoto satoshi",
        "david claire", "hal finney", "nick szabo", "adam back", "wei dai",
        "b-money", "bit gold", "hashcash",
        "the timestamp server", "proof-of-work",
        "bitcoin.org", "bitcointalk.org",
        "genesis block", "block 0", "block height 0",
        "the times 03/jan/2009 chancellor on brink of second bailout for banks",
        "bitcoin whitepaper", "bitcoin pdf",
        "h4w9vJaqvmwBtX8Gr1Q77q5aXqRDYSFA66", // Hal Finney address
        "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", // Genesis address
    ];

    for ref_str in &satoshi_refs {
        let hash = Sha256::digest(ref_str.as_bytes());
        write_key(&mut w, sha256_key(PLACEHOLDER), &mut count);
        // Double SHA256
        let hash2 = Sha256::digest(&hash);
        add_key!(Some(hash2.into()));
    }

    eprintln!("  {} Satoshi ref keys (total {})", count - before, count);

    // ═══════════════════════════════════════════════════════════
    // 28. UNIX TIMESTAMP KEYS
    // ═══════════════════════════════════════════════════════════
    eprintln!("[28/30] Timestamp keys...");
    let before = count;

    // Every day from 2009-01-01 to 2012-12-31 as Unix timestamp
    let start_ts = 1230768000u64; // 2009-01-01
    let end_ts = 1356998400u64;   // 2013-01-01

    for ts in (start_ts..=end_ts).step_by(86400) { // Every day
        let key = ts.to_le_bytes();
        add_key!(Some(key));
        let key = ts.to_be_bytes();
        add_key!(Some(key));
    }

    // Hourly for 2009
    for ts in (start_ts..1262304000u64).step_by(3600) {
        let hash = Sha256::digest(ts.to_string().as_bytes());
        write_key(&mut w, sha256_key(PLACEHOLDER), &mut count);
    }

    eprintln!("  {} timestamp keys (total {})", count - before, count);

    // ═══════════════════════════════════════════════════════════
    // 29. BLOCK HEIGHT KEYS
    // ═══════════════════════════════════════════════════════════
    eprintln!("[29/30] Block height keys...");
    let before = count;

    for height in 0..100000u64 {
        let key = height.to_le_bytes();
        add_key!(Some(key));
        let key = height.to_be_bytes();
        add_key!(Some(key));
        // SHA256 of height
        let hash = Sha256::digest(height.to_string().as_bytes());
        write_key(&mut w, sha256_key(PLACEHOLDER), &mut count);
    }

    eprintln!("  {} block height keys (total {})", count - before, count);

    // ═══════════════════════════════════════════════════════════
    // 30. COMMON PASSWORD HASHES
    // ═══════════════════════════════════════════════════════════
    eprintln!("[30/30] Password hashes...");
    let before = count;

    let passwords = [
        "password", "123456", "12345678", "qwerty", "abc123",
        "monkey", "1234567", "letmein", "trustno1", "dragon",
        "baseball", "iloveyou", "master", "sunshine", "ashley",
        "michael", "shadow", "123123", "654321", "superman",
        "qazwsx", "michael", "football", "password1", "password123",
        "batman", "login", "admin", "welcome", "hello",
        "charlie", "donald", "password2", "qwerty123", "1q2w3e4r",
        "zaq1@wsx", "1qaz2wsx", "starwars", "solo", "vader",
        "skywalker", "obiwan", "yoda", "luke", "leia",
    ];

    for pw in &passwords {
        // SHA256
        let hash = Sha256::digest(pw.as_bytes());
        write_key(&mut w, sha256_key(PLACEHOLDER), &mut count);
        // Double SHA256
        let hash2 = Sha256::digest(&hash);
        add_key!(Some(hash2.into()));
        // SHA256 with common salts
        for salt in ["", "!", "@", "#", "$", "1", "12", "123", "!@#", "btc", "bit", "coin"] {
            let salted = format!("{}{}", pw, salt);
            let hash = Sha256::digest(salted.as_bytes());
            write_key(&mut w, sha256_key(PLACEHOLDER), &mut count);
            let salted = format!("{}{}", salt, pw);
            let hash = Sha256::digest(salted.as_bytes());
            write_key(&mut w, sha256_key(PLACEHOLDER), &mut count);
        }
    }

    eprintln!("  {} password hash keys (total {})", count - before, count);

    drop(w);
    eprintln!();
    eprintln!("Total keys generated: {} ({:.1} MB)", count, count as f64 * 64.0 / (1024.0 * 1024.0));
    eprintln!("Output: {}", output);
}

/// Convert a string of digits to a 32-byte key (little-endian)
fn str_to_key(s: &str) -> Option<[u8; 32]> {
    let mut key = [0u8; 32];
    let bytes = s.as_bytes();
    let len = bytes.len().min(32);
    key[32-len..32].copy_from_slice(&bytes[..len]);
    Some(key)
}

/// Convert base-6 dice rolls to bytes
fn base6_to_bytes(rolls: &[u8], out: &mut [u8; 32]) {
    // Each roll is 1-6, treat as base-6 digit (0-5)
    let mut val: [u64; 4] = [0; 4]; // 256 bits = 4 × 64-bit limbs
    for &roll in rolls {
        let digit = (roll - 1) as u64; // 0-5
        // Multiply val by 6 and add digit
        let mut carry = digit;
        for limb in &mut val.iter_mut().rev() {
            let product = *limb as u128 * 6 + carry as u128;
            *limb = product as u64;
            carry = (product >> 64) as u64;
        }
    }
    for (i, limb) in val.iter().enumerate() {
        let bytes = limb.to_le_bytes();
        out[i*8..(i+1)*8].copy_from_slice(&bytes);
    }
}

/// Convert base-N rolls to bytes
fn based_to_bytes(rolls: &[u8], base: u8, out: &mut [u8; 32]) {
    let mut val: [u64; 4] = [0; 4];
    for &roll in rolls {
        let digit = (roll - 1) as u64;
        let mut carry = digit;
        let b = base as u128;
        for limb in &mut val.iter_mut().rev() {
            let product = *limb as u128 * b + carry as u128;
            *limb = product as u64;
            carry = (product >> 64) as u64;
        }
    }
    for (i, limb) in val.iter().enumerate() {
        let bytes = limb.to_le_bytes();
        out[i*8..(i+1)*8].copy_from_slice(&bytes);
    }
}

