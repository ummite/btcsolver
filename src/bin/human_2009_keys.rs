//! Human 2009 Key Generator — keys a human in 2009-2011 might have created.
//!
//! Categories:
//! 1. Brainwallets (SHA256 of phrases) — MOST IMPORTANT
//! 2. Date-based keys
//! 3. Physics/math constants
//! 4. Common password hashes
//! 5. Repeated patterns
//! 6. ASCII text as hex
//! 7. Satoshi references
//! 8. Block height keys
//! 9. Timestamp keys
//! 10. Dice/card patterns

use sha2::{Digest, Sha256};
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};

struct W { w: BufWriter<File>, count: usize }

impl W {
    fn key(&mut self, k: [u8; 32]) {
        writeln!(self.w, "{}", hex::encode(k)).ok();
        self.count += 1;
    }
    fn sha256(&mut self, s: &str) {
        let h = Sha256::digest(s.as_bytes());
        let mut k = [0u8; 32];
        k.copy_from_slice(h.as_slice());
        self.key(k);
    }
    fn dsha256(&mut self, s: &str) {
        let h1 = Sha256::digest(s.as_bytes());
        let h2 = Sha256::digest(h1.as_slice());
        let mut k = [0u8; 32];
        k.copy_from_slice(h2.as_slice());
        self.key(k);
    }
    fn u64le(&mut self, n: u64) {
        let mut k = [0u8; 32];
        k[24..32].copy_from_slice(&n.to_le_bytes());
        self.key(k);
    }
    fn u64be(&mut self, n: u64) {
        let mut k = [0u8; 32];
        k[24..32].copy_from_slice(&n.to_be_bytes());
        self.key(k);
    }
    fn bytes32(&mut self, b: &[u8]) {
        if b.len() >= 32 {
            let mut k = [0u8; 32];
            k.copy_from_slice(&b[..32]);
            self.key(k);
        }
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let output = args.get(1).map(|s| s.as_str()).unwrap_or("data/human-2009-keys.txt");
    let file = File::create(output).expect("Failed to create output");
    let mut w = W { w: BufWriter::new(file), count: 0 };

    // ═══ 1. BRAINWALLETS ═══
    eprintln!("[1/10] Brainwallets...");
    let before = w.count;

    let phrases: &[&str] = &[
        "bitcoin", "money", "satoshi", "nakamoto", "crypto", "wallet",
        "password", "private", "key", "blockchain", "digital", "cash",
        "free", "anonymous", "peer", "p2p", "decentralized",
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
        "lightning", "thunder", "storm", "fire", "water", "wind",
        "i love bitcoin", "bitcoin is the future", "satoshi nakamoto",
        "to be or not to be", "hello world", "the quick brown fox",
        "one two three four five", "abc def ghi jkl mno",
        "let it be", "think different", "just do it",
        "may the force be with you", "i am the walrus",
        "all your base are belong to us",
        "e equals mc squared", "pi is 3.14159",
        "the speed of light", "planck constant",
        "january 1 2009", "january 3 2009", "jan 3 2009",
        "bitcoin genesis", "genesis block", "the timestamp",
        "1234567890", "0987654321", "qwertyuiop",
        "abcdefghijklmnopqrstuvwxyz",
        "aaaaa", "aaaaaa", "aaaaaaaa", "aaaaaaaaa", "aaaaaaaaaa",
        "11111", "111111", "1111111", "11111111",
        "bitcoin france", "argent numerique", "cle privee",
        "bonjour", "mot de passe", "secret", "tresor",
        "bitcoin deutschland", "digitales geld", "passwort",
        "bitcoin espana", "dinero digital", "contrasena",
    ];

    for p in phrases { w.sha256(p); }

    // word + number combos
    let words: &[&str] = &["bitcoin", "satoshi", "money", "crypto", "wallet", "password", "god", "love", "free", "coin"];
    for word in words {
        for num in 0..=99999u32 {
            w.sha256(&format!("{}{}", word, num));
        }
        for num in 1..=9999u32 {
            w.sha256(&format!("{}-{}", word, num));
        }
    }
    // word + year
    for year in 1950..=2020u32 {
        for word in words {
            w.sha256(&format!("{}{}", word, year));
        }
    }

    eprintln!("  {} keys (total {})", w.count - before, w.count);

    // ═══ 2. DATE KEYS ═══
    eprintln!("[2/10] Date keys...");
    let before = w.count;

    for year in 1900..=2020u64 {
        for month in 1..=12u64 {
            for day in 1..=31u64 {
                let s = format!("{:04}{:02}{:02}", year, month, day);
                w.bytes32(s.as_bytes());
            }
        }
    }

    let timestamps: &[u64] = &[
        1231006505, 1231484578, 1231469665, 1182967200,
        1293792000, 1325376000, 1356998400, 0,
    ];
    for &ts in timestamps {
        w.u64le(ts); w.u64be(ts);
        w.sha256(&ts.to_string());
    }

    eprintln!("  {} keys (total {})", w.count - before, w.count);

    // ═══ 3. PHYSICS CONSTANTS ═══
    eprintln!("[3/10] Physics constants...");
    let before = w.count;

    let constants: &[(&str, &str)] = &[
        ("speed_of_light", "299792458"),
        ("planck", "662607015"),
        ("boltzmann", "138064852"),
        ("avogadro", "602214076"),
        ("gravitational", "667408"),
        ("elementary_charge", "1602176634"),
        ("electron_mass", "910938356"),
        ("fine_structure", "7297352569"),
        ("euler_mascheroni", "5772156649"),
        ("apery", "6010009975"),
    ];
    for (name, digits) in constants {
        w.bytes32(digits.as_bytes());
        w.sha256(name);
        w.sha256(&format!("{} = {}", name, digits));
    }

    // Fibonacci
    let mut fib: Vec<u64> = vec![0, 1];
    for i in 2..90u32 {
        let next = fib[i as usize-1].saturating_add(fib[i as usize-2]);
        if next < u64::MAX { fib.push(next); } else { break; }
    }
    for &f in &fib { w.u64le(f); }

    // Primes
    let primes: Vec<u64> = (2..).filter(|n| (*n..*n).all(|d| n % d != 0)).take(500).collect();
    for &p in &primes { w.u64le(p); }

    eprintln!("  {} keys (total {})", w.count - before, w.count);

    // ═══ 4. PASSWORD HASHES ═══
    eprintln!("[4/10] Password hashes...");
    let before = w.count;

    let passwords: &[&str] = &[
        "password", "123456", "12345678", "qwerty", "abc123",
        "monkey", "1234567", "letmein", "trustno1", "dragon",
        "baseball", "iloveyou", "master", "sunshine", "ashley",
        "shadow", "123123", "654321", "superman", "qazwsx",
        "football", "password1", "password123", "batman",
        "login", "admin", "welcome", "hello", "charlie",
        "password2", "qwerty123", "1q2w3e4r", "zaq1@wsx",
        "1qaz2wsx", "starwars", "solo", "vader",
        "skywalker", "obiwan", "yoda", "luke", "leia",
    ];

    for pw in passwords {
        w.sha256(pw);
        w.dsha256(pw);
        for salt in ["", "!", "@", "#", "$", "1", "12", "123", "!@#", "btc", "bit", "coin"] {
            w.sha256(&format!("{}{}", pw, salt));
            if !salt.is_empty() { w.sha256(&format!("{}{}", salt, pw)); }
        }
    }

    eprintln!("  {} keys (total {})", w.count - before, w.count);

    // ═══ 5. REPEATED PATTERNS ═══
    eprintln!("[5/10] Repeated patterns...");
    let before = w.count;

    for &b in &[0x42u8, 0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA,
                0x13, 0x37, 0x69, 0x7B, 0x00, 0xFF, 0x55, 0xAA] {
        w.key([b; 32]);
    }
    for i in 0..=255u8 { w.key([i; 32]); }

    let patterns32: &[u32] = &[
        0xDEADBEEF, 0xCAFEBABE, 0x1337BEEF, 0x42424242,
        0x69696969, 0x7B7B7B7B, 0x01020304, 0x0A0B0C0D,
        0xFEEDFACE, 0xD1110CCC, 0xFFFFFFFF, 0x00000001,
    ];
    for &p in patterns32 {
        let bytes = p.to_le_bytes();
        let mut k = [0u8; 32];
        for i in 0..8 { k[i*4..(i+1)*4].copy_from_slice(&bytes); }
        w.key(k);
    }

    // Incrementing
    let mut k = [0u8; 32];
    for i in 0..32usize { k[i] = i as u8; }
    w.key(k);
    for i in 0..32usize { k[i] = (31 - i) as u8; }
    w.key(k);

    eprintln!("  {} keys (total {})", w.count - before, w.count);

    // ═══ 6. ASCII TEXT ═══
    eprintln!("[6/10] ASCII text keys...");
    let before = w.count;

    let texts: &[&str] = &[
        "abcdefghijklmnopqrstuvwxyz012345",
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123",
        "01234567890123456789012345678901",
        "98765432109876543210987654321098",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "the quick brown fox jumps over a",
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
    for t in texts { w.bytes32(t.as_bytes()); }

    eprintln!("  {} keys (total {})", w.count - before, w.count);

    // ═══ 7. SATOSHI REFERENCES ═══
    eprintln!("[7/10] Satoshi references...");
    let before = w.count;

    let refs: &[&str] = &[
        "satoshi nakamoto", "nakamoto satoshi",
        "david claire", "hal finney", "nick szabo", "adam back", "wei dai",
        "b-money", "bit gold", "hashcash",
        "the timestamp server", "proof-of-work",
        "bitcoin.org", "bitcointalk.org",
        "genesis block", "block 0", "block height 0",
        "the times 03/jan/2009 chancellor on brink of second bailout for banks",
        "bitcoin whitepaper", "bitcoin pdf",
        "h4w9vJaqvmwBtX8Gr1Q77q5aXqRDYSFA66",
        "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
    ];
    for r in refs { w.sha256(r); w.dsha256(r); }

    eprintln!("  {} keys (total {})", w.count - before, w.count);

    // ═══ 8. BLOCK HEIGHT KEYS ═══
    eprintln!("[8/10] Block height keys...");
    let before = w.count;

    for h in 0..100000u64 {
        w.u64le(h); w.u64be(h);
        w.sha256(&h.to_string());
    }

    eprintln!("  {} keys (total {})", w.count - before, w.count);

    // ═══ 9. TIMESTAMP KEYS ═══
    eprintln!("[9/10] Timestamp keys...");
    let before = w.count;

    let start_ts = 1230768000u64; // 2009-01-01
    let end_ts = 1356998400u64;   // 2013-01-01
    for ts in (start_ts..=end_ts).step_by(86400) {
        w.u64le(ts); w.u64be(ts);
    }
    // Hourly for 2009
    for ts in (start_ts..1262304000u64).step_by(3600) {
        w.sha256(&ts.to_string());
    }

    eprintln!("  {} keys (total {})", w.count - before, w.count);

    // ═══ 10. DICE/CARD PATTERNS ═══
    eprintln!("[10/10] Dice/card patterns...");
    let before = w.count;

    // All-same dice rolls (d6, d20)
    for num_dice in [10, 20, 30, 40, 50, 60, 66] {
        for face in 1..=6u8 {
            let rolls: Vec<u8> = std::iter::repeat(face).take(num_dice).collect();
            let k = base_n_to_key(&rolls, 6);
            w.key(k);
        }
        // Sequential
        for start in 1..=6u8 {
            let mut rolls = Vec::new();
            let mut val = start;
            for _ in 0..num_dice {
                rolls.push(val);
                val = if val >= 6 { 1 } else { val + 1 };
            }
            w.key(base_n_to_key(&rolls, 6));
        }
    }

    // Card deck natural order
    let suits: [u8; 4] = [0, 1, 2, 3];
    let ranks: [u8; 13] = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
    let mut k = [0u8; 32];
    let mut idx = 0;
    for &s in &suits {
        for &r in &ranks {
            if idx < 32 {
                k[idx] = (s << 4) | (r & 0xF);
                idx += 1;
            }
        }
    }
    w.key(k);

    eprintln!("  {} keys (total {})", w.count - before, w.count);

    drop(w.w);
    eprintln!();
    eprintln!("Total keys generated: {} ({:.1} MB)", w.count, w.count as f64 * 64.0 / 1024.0 / 1024.0);
    eprintln!("Output: {}", output);
}

fn base_n_to_key(rolls: &[u8], base: u64) -> [u8; 32] {
    let mut val: [u64; 4] = [0; 4];
    for &roll in rolls {
        let digit = roll as u64;
        let mut carry = digit;
        for limb in val.iter_mut().rev() {
            let product = *limb as u128 * base as u128 + carry as u128;
            *limb = product as u64;
            carry = (product >> 64) as u64;
        }
    }
    let mut out = [0u8; 32];
    for (i, limb) in val.iter().enumerate() {
        out[i*8..(i+1)*8].copy_from_slice(&limb.to_le_bytes());
    }
    out
}
