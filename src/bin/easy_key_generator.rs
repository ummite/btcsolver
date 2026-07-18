//! Easy Key Generator — generates 256-bit private keys that a human in 2009 might have created
//!
//! Categories:
//!   1. Mathematical constants (pi, e, sqrt(2), phi, gamma, etc.)
//!   2. Famous numbers and sequences (Fibonacci, primes, etc.)
//!   3. Date-based keys (birthdays, significant dates)
//!   4. Keyboard patterns (qwerty, diagonal, etc.)
//!   5. Simple passwords and passphrases (hashed)
//!   6. Famous quotes and phrases (hashed)
//!   7. Physical constants (speed of light, Planck, etc.)
//!   8. Chess positions and games (hashed)
//!   9. Pop culture references (2009 era)
//!  10. Hex patterns (repeating, sequential, palindrome)
//!
//! Usage:
//!   easy_key_generator --output easy-keys-corpus.txt
//!   easy_key_generator --categories math,dates,passwords --output subset.txt
//!
//! Output: one hex string (64 chars = 256 bits) per line, ready for brainwallet scan

use anyhow::Result;
use clap::Parser;
use digest::Digest;
use md5::Md5;
use sha2::Sha256;
use std::collections::BTreeSet;

#[derive(Parser)]
struct Cli {
    /// Output file
    #[arg(long, default_value = "easy-keys-corpus.txt")]
    output: String,

    /// Comma-separated categories to generate (default: all)
    #[arg(long)]
    categories: Option<String>,

    /// Include transforms (reverse, rotate, etc.) on generated keys
    #[arg(long, default_value = "true")]
    transforms: bool,

    /// Max keys to generate (0 = unlimited)
    #[arg(long, default_value = "0")]
    max_keys: usize,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let categories: Option<Vec<&str>> = cli.categories.as_ref().map(|s| {
        s.split(',').map(|c| c.trim()).collect()
    });

    let mut all_keys: BTreeSet<[u8; 32]> = BTreeSet::new();

    // === 1. Mathematical constants ===
    if should_include(&categories, "math") {
        println!("[1/10] Mathematical constants...");
        let math_keys = generate_math_constants();
        println!("  -> {} keys", math_keys.len());
        all_keys.extend(math_keys);
    }

    // === 2. Famous sequences ===
    if should_include(&categories, "sequences") {
        println!("[2/10] Famous sequences...");
        let seq_keys = generate_famous_sequences();
        println!("  -> {} keys", seq_keys.len());
        all_keys.extend(seq_keys);
    }

    // === 3. Date-based keys ===
    if should_include(&categories, "dates") {
        println!("[3/10] Date-based keys...");
        let date_keys = generate_date_keys();
        println!("  -> {} keys", date_keys.len());
        all_keys.extend(date_keys);
    }

    // === 4. Keyboard patterns ===
    if should_include(&categories, "keyboard") {
        println!("[4/10] Keyboard patterns...");
        let kb_keys = generate_keyboard_patterns();
        println!("  -> {} keys", kb_keys.len());
        all_keys.extend(kb_keys);
    }

    // === 5. Simple passwords ===
    if should_include(&categories, "passwords") {
        println!("[5/10] Simple passwords...");
        let pw_keys = generate_simple_passwords();
        println!("  -> {} keys", pw_keys.len());
        all_keys.extend(pw_keys);
    }

    // === 6. Famous phrases ===
    if should_include(&categories, "phrases") {
        println!("[6/10] Famous phrases...");
        let phrase_keys = generate_famous_phrases();
        println!("  -> {} keys", phrase_keys.len());
        all_keys.extend(phrase_keys);
    }

    // === 7. Physical constants ===
    if should_include(&categories, "physics") {
        println!("[7/10] Physical constants...");
        let phys_keys = generate_physical_constants();
        println!("  -> {} keys", phys_keys.len());
        all_keys.extend(phys_keys);
    }

    // === 8. Chess positions ===
    if should_include(&categories, "chess") {
        println!("[8/10] Chess positions...");
        let chess_keys = generate_chess_keys();
        println!("  -> {} keys", chess_keys.len());
        all_keys.extend(chess_keys);
    }

    // === 9. Pop culture ===
    if should_include(&categories, "popculture") {
        println!("[9/10] Pop culture...");
        let pop_keys = generate_pop_culture();
        println!("  -> {} keys", pop_keys.len());
        all_keys.extend(pop_keys);
    }

    // === 10. Hex patterns ===
    if should_include(&categories, "hexpatterns") {
        println!("[10/10] Hex patterns...");
        let hex_keys = generate_hex_patterns();
        println!("  -> {} keys", hex_keys.len());
        all_keys.extend(hex_keys);
    }

    // Apply transforms if requested
    if cli.transforms {
        let before = all_keys.len();
        let mut transformed = BTreeSet::new();
        for key in &all_keys {
            // Reverse bytes
            let mut rev = *key;
            rev.reverse();
            transformed.insert(rev);

            // Reverse bits
            let mut rev_bits = [0u8; 32];
            for i in 0..256 {
                let src_byte = i / 8;
                let src_bit = 7 - (i % 8);
                if (key[src_byte] >> src_bit) & 1 == 1 {
                    let dst = 255 - i;
                    let dst_byte = dst / 8;
                    let dst_bit = 7 - (dst % 8);
                    rev_bits[dst_byte] |= 1 << dst_bit;
                }
            }
            transformed.insert(rev_bits);

            // Rotate left 8 bits
            let mut rotl = [0u8; 32];
            rotl[..31].copy_from_slice(&key[1..]);
            rotl[31] = key[0];
            transformed.insert(rotl);

            // Rotate right 8 bits
            let mut rotr = [0u8; 32];
            rotr[0] = key[31];
            rotr[1..].copy_from_slice(&key[..31]);
            transformed.insert(rotr);
        }
        all_keys.extend(transformed);
        println!("\nTransforms: {} -> {} keys", before, all_keys.len());
    }

    // Filter out zero key and secp256k1 order boundary
    let zero_key = [0u8; 32];
    all_keys.remove(&zero_key);

    // Apply max limit and write output
    let mut lines: Vec<String> = all_keys.iter().map(|k| hex::encode(k)).collect();
    if cli.max_keys > 0 {
        lines.truncate(cli.max_keys);
    }
    let total = lines.len();

    std::fs::write(&cli.output, lines.join("\n") + "\n")?;
    println!("\nTotal: {} unique keys written to {}", lines.len(), cli.output);

    Ok(())
}

fn should_include(categories: &Option<Vec<&str>>, name: &str) -> bool {
    match categories {
        None => true,
        Some(cats) => cats.contains(&name),
    }
}

fn hash_to_key(input: &str) -> [u8; 32] {
    let h = Sha256::digest(input.as_bytes());
    let mut key = [0u8; 32];
    key.copy_from_slice(&h);
    key
}

fn bytes_to_key(bytes: &[u8]) -> Option<[u8; 32]> {
    if bytes.len() >= 32 {
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes[..32]);
        Some(key)
    } else {
        let mut key = [0u8; 32];
        key[..bytes.len()].copy_from_slice(bytes);
        Some(key)
    }
}

// ============================================================================
// 1. MATHEMATICAL CONSTANTS
// ============================================================================
fn generate_math_constants() -> BTreeSet<[u8; 32]> {
    let mut keys = BTreeSet::new();

    // Pi - first 100 digits
    let pi_digits = "314159265358979323846264338327950288419716939937510";
    // e - first 100 digits
    let e_digits = "271828182845904523536028747135266249775724709369995";
    // sqrt(2)
    let sqrt2 = "141421356237309504880168872420969807856967187537694";
    // sqrt(3)
    let sqrt3 = "173205080756887729352744634150587236694280525381038";
    // sqrt(5)
    let sqrt5 = "223606797749978969640917366873127623544061835961152";
    // Golden ratio (phi)
    let phi = "161803398874989484820458683436563811772030917980576";
    // Euler-Mascheroni constant (gamma)
    let gamma = "57721566490153286060651209008240243104215933593992";
    // Apéry's constant (zeta(3))
    let zeta3 = "1202056903159594285399738161511449990764986292";
    // Catalan's constant
    let catalan = "91596559417721901505460351493238411077414937428167";
    // Silver ratio
    let silver = "241421356237309504880168872420969807856967187537694";
    // Tribonacci constant
    let tribonacci = "18392867552141611995758140967374008433342527";
    // Plastic number
    let plastic = "1324717957244746025960908854478097346735388658";
    // Omega constant
    let omega = "5671432904097838729999686622";
    // Glaisher-Kinkelin constant
    let glaisher = "82281594152338996814200275452216445221330577";
    // Khinchin's constant
    let khinchin = "268545200106530644530971483548";
    // Landau's constants
    let landau_g = "549306144334054847692011382";
    // Feigenbaum constants
    let feigenbaum_r = "466920160910299067185320382";
    let feigenbaum_delta = "25029078750958928222839028";

    let constants: Vec<(&str, &str)> = vec![
        ("pi", pi_digits),
        ("e", e_digits),
        ("sqrt2", sqrt2),
        ("sqrt3", sqrt3),
        ("sqrt5", sqrt5),
        ("phi", phi),
        ("gamma", gamma),
        ("zeta3", zeta3),
        ("catalan", catalan),
        ("silver", silver),
        ("tribonacci", tribonacci),
        ("plastic", plastic),
        ("omega", omega),
        ("glaisher", glaisher),
        ("khinchin", khinchin),
        ("landau_g", landau_g),
        ("feigenbaum_r", feigenbaum_r),
        ("feigenbaum_delta", feigenbaum_delta),
    ];

    for (name, digits) in constants {
        // Direct bytes from digit string
        if let Some(k) = bytes_to_key(digits.as_bytes()) {
            keys.insert(k);
        }

        // Hex interpretation (pairs of digits as hex bytes where valid)
        let hex_bytes: Vec<u8> = digits.chars()
            .collect::<String>()
            .chars()
            .filter(|c| c.is_ascii_hexdigit())
            .collect::<String>()
            .chars()
            .collect::<Vec<_>>()
            .chunks(2)
            .filter_map(|chunk| {
                let s: String = chunk.iter().collect();
                u8::from_str_radix(&s, 16).ok()
            })
            .collect();
        if let Some(k) = bytes_to_key(&hex_bytes) {
            keys.insert(k);
        }

        // SHA256 of the digit string
        keys.insert(hash_to_key(digits));

        // SHA256 of "constant_name"
        keys.insert(hash_to_key(name));

        // SHA256 of "constant_name: digits"
        keys.insert(hash_to_key(&format!("{}: {}", name, digits)));

        // First 32 ASCII bytes of the digits (zero-padded)
        let mut padded = digits.as_bytes().to_vec();
        padded.resize(32, 0);
        if padded.len() == 32 {
            let arr: [u8; 32] = padded.clone().try_into().unwrap();
            keys.insert(arr);
            // Reverse of first 32 bytes
            let mut rev = padded;
            rev.reverse();
            if let Ok(rev_arr) = <[u8; 32]>::try_from(rev) {
                keys.insert(rev_arr);
            }
        }

        // Pi * 10^n for various n (as hex)
        if name == "pi" {
            for n in 0..10 {
                let shifted = format!("{:0>32}", &pi_digits[..(n+1).min(pi_digits.len())]);
                if let Some(k) = bytes_to_key(shifted.as_bytes()) {
                    keys.insert(k);
                }
            }
        }
    }

    // sqrt(n) for n = 2..100
    for n in 2..=100 {
        let sqrt_n = (n as f64).sqrt();
        let s = format!("{:.50}", sqrt_n);
        // Remove decimal point
        let digits_only: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
        if let Some(k) = bytes_to_key(digits_only.as_bytes()) {
            keys.insert(k);
        }
        keys.insert(hash_to_key(&format!("sqrt({})", n)));
        keys.insert(hash_to_key(&digits_only));
    }

    // nth root of 2 for n = 2..20
    for n in 2..=20 {
        let root = 2.0_f64.powf(1.0 / n as f64);
        let s = format!("{:.50}", root);
        let digits_only: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
        if let Some(k) = bytes_to_key(digits_only.as_bytes()) {
            keys.insert(k);
        }
        keys.insert(hash_to_key(&format!("{}th_root_of_2", n)));
    }

    // n! (factorial) for n = 1..20
    for n in 1..=20 {
        let mut fact = 1u128;
        for i in 2..=n {
            fact = fact.saturating_mul(i as u128);
        }
        let s = format!("{}", fact);
        if let Some(k) = bytes_to_key(s.as_bytes()) {
            keys.insert(k);
        }
        keys.insert(hash_to_key(&format!("{}!", n)));
        keys.insert(hash_to_key(&s));
    }

    // Power towers: 2^n for n = 1..100
    for n in 1..=100 {
        let power = 1u128 << n.min(127);
        let s = format!("{}", power);
        if let Some(k) = bytes_to_key(s.as_bytes()) {
            keys.insert(k);
        }
        keys.insert(hash_to_key(&format!("2^{}", n)));
    }

    // n^n for n = 1..20
    for n in 1..=20 {
        let mut power = 1u128;
        for _ in 0..n {
            power = power.saturating_mul(n as u128);
        }
        let s = format!("{}", power);
        if let Some(k) = bytes_to_key(s.as_bytes()) {
            keys.insert(k);
        }
        keys.insert(hash_to_key(&format!("{}^{}", n, n)));
    }

    println!("  Math constants: {} keys", keys.len());
    keys
}

// ============================================================================
// 2. FAMOUS SEQUENCES
// ============================================================================
fn generate_famous_sequences() -> BTreeSet<[u8; 32]> {
    let mut keys = BTreeSet::new();

    // Fibonacci numbers (first 50)
    let mut a: u128 = 0;
    let mut b: u128 = 1;
    for i in 0..50 {
        let s = format!("{}", a);
        if let Some(k) = bytes_to_key(s.as_bytes()) {
            keys.insert(k);
        }
        keys.insert(hash_to_key(&format!("fib({})", i)));
        keys.insert(hash_to_key(&s));
        let temp = a + b;
        a = b;
        b = temp;
    }

    // Prime numbers (first 200)
    let primes: Vec<u128> = (2..).filter(|&n| {
        (2..=(n as f64).sqrt() as u128).all(|i| n % i != 0)
    }).take(200).collect();

    for (i, p) in primes.iter().enumerate() {
        let s = format!("{}", p);
        if let Some(k) = bytes_to_key(s.as_bytes()) {
            keys.insert(k);
        }
        keys.insert(hash_to_key(&format!("prime({})", i)));
        keys.insert(hash_to_key(&s));
    }

    // Concatenated primes as bytes
    let prime_concat: String = primes.iter().map(|p| p.to_string()).collect();
    if let Some(k) = bytes_to_key(prime_concat.as_bytes()) {
        keys.insert(k);
    }
    keys.insert(hash_to_key(&prime_concat));

    // Mersenne primes (2^p - 1 for known Mersenne prime exponents)
    let mersenne_exponents = vec![2, 3, 5, 7, 13, 17, 19, 31, 61, 89, 107, 127];
    for p in &mersenne_exponents {
        let mersenne = (1u128 << (*p).min(127)) - 1;
        let s = format!("{}", mersenne);
        if let Some(k) = bytes_to_key(s.as_bytes()) {
            keys.insert(k);
        }
        keys.insert(hash_to_key(&format!("mersenne({})", p)));
        keys.insert(hash_to_key(&s));
    }

    // Perfect numbers
    let perfects = vec![6u128, 28, 496, 8128, 33550336, 8589869056, 137438691328];
    for p in &perfects {
        let s = format!("{}", p);
        if let Some(k) = bytes_to_key(s.as_bytes()) {
            keys.insert(k);
        }
        keys.insert(hash_to_key(&format!("perfect({})", p)));
    }

    // Lucas numbers
    let mut la: u128 = 2;
    let mut lb: u128 = 1;
    for i in 0..30 {
        let s = format!("{}", la);
        if let Some(k) = bytes_to_key(s.as_bytes()) {
            keys.insert(k);
        }
        keys.insert(hash_to_key(&format!("lucas({})", i)));
        let temp = la + lb;
        la = lb;
        lb = temp;
    }

    // Triangular numbers
    for n in 1..=100 {
        let tri = n * (n + 1) / 2;
        let s = format!("{}", tri);
        if let Some(k) = bytes_to_key(s.as_bytes()) {
            keys.insert(k);
        }
        keys.insert(hash_to_key(&format!("triangular({})", n)));
    }

    // Square numbers
    for n in 1..=1000 {
        let sq = n * n;
        let s = format!("{}", sq);
        if let Some(k) = bytes_to_key(s.as_bytes()) {
            keys.insert(k);
        }
    }

    // Cube numbers
    for n in 1..=100 {
        let cb = n * n * n;
        let s = format!("{}", cb);
        if let Some(k) = bytes_to_key(s.as_bytes()) {
            keys.insert(k);
        }
    }

    // Factorial numbers
    for n in 1..=20 {
        let mut fact = 1u128;
        for i in 2..=n {
            fact = fact.saturating_mul(i as u128);
        }
        keys.insert(hash_to_key(&format!("factorial({})", n)));
    }

    // Powers of 2, 3, 5, 10
    for base in [2u128, 3, 5, 10] {
        let mut p = 1u128;
        for _ in 0..40 {
            let s = format!("{}", p);
            if let Some(k) = bytes_to_key(s.as_bytes()) {
                keys.insert(k);
            }
            p = p.saturating_mul(base);
        }
    }

    // Collatz sequence starting values 1..1000
    for start in 1..=1000u128 {
        let mut n = start;
        let mut seq = vec![n];
        while n != 1 && seq.len() < 100 {
            n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
            seq.push(n);
        }
        let seq_str: String = seq.iter().map(|x| x.to_string()).collect();
        keys.insert(hash_to_key(&format!("collatz({})", start)));
        if let Some(k) = bytes_to_key(seq_str.as_bytes()) {
            keys.insert(k);
        }
    }

    println!("  Famous sequences: {} keys", keys.len());
    keys
}

// ============================================================================
// 3. DATE-BASED KEYS
// ============================================================================
fn generate_date_keys() -> BTreeSet<[u8; 32]> {
    let mut keys = BTreeSet::new();

    // Bitcoin-related dates
    let bitcoin_dates = vec![
        "20081031",  // Whitepaper
        "20090103",  // Genesis block
        "20090109",  // First block reward
        "20090212",  // First transaction
        "20090809",  // First purchase (pizza)
        "20090522",  // Pizza day (alternative)
        "20100522",  // Bitcoin pizza day
        "20100717",  // Bitcoin Talk launch
        "20100718",  // First BTC USD rate
        "20110209",  // BTC = $1
        "20130429",  // BTC = $100
        "20131128",  // BTC = $1000
        "20140101",  // MtGox collapse
        "20171217",  // BTC = $20000
    ];

    for date in &bitcoin_dates {
        // Direct bytes
        if let Some(k) = bytes_to_key(date.as_bytes()) {
            keys.insert(k);
        }
        // Hex interpretation
        if let Ok(hex_bytes) = hex::decode(date) {
            if let Some(k) = bytes_to_key(&hex_bytes) {
                keys.insert(k);
            }
        }
        // SHA256
        keys.insert(hash_to_key(date));
        // Various formats
        keys.insert(hash_to_key(&format!("{}-{}-{}", &date[..4], &date[4..6], &date[6..])));
        keys.insert(hash_to_key(&format!("/{}/{}", &date[4..6], &date[..4])));
    }

    // Years 1900..2030 as keys
    for year in 1900..=2030 {
        let s = format!("{}", year);
        if let Some(k) = bytes_to_key(s.as_bytes()) {
            keys.insert(k);
        }
        keys.insert(hash_to_key(&s));

        // Year as hex
        let hex_year = format!("{:08x}", year);
        if let Ok(hex_bytes) = hex::decode(&hex_year) {
            if let Some(k) = bytes_to_key(&hex_bytes) {
                keys.insert(k);
            }
        }
    }

    // Birthdays (MMDD format, all combinations)
    for month in 1..=12 {
        for day in 1..=31 {
            let birthday = format!("{:02}{:02}", month, day);
            if let Some(k) = bytes_to_key(birthday.as_bytes()) {
                keys.insert(k);
            }
            keys.insert(hash_to_key(&birthday));

            // With common years
            for year in [1980, 1985, 1990, 1995, 2000, 2005, 2008, 2009] {
                let full = format!("{:02}{:02}{}", month, day, year);
                if let Some(k) = bytes_to_key(full.as_bytes()) {
                    keys.insert(k);
                }
                keys.insert(hash_to_key(&full));
            }
        }
    }

    // Unix timestamps for significant dates
    let timestamps = vec![
        1231006505u64,  // Genesis block
        1230764428u64,  // Whitepaper
        1274524800u64,  // 2010-01-01
        1274784000u64,  // 2010-05-22 (pizza)
        1356998400u64,  // 2013-01-01
        1511280000u64,  // 2017-11-21
        1609459200u64,  // 2021-01-01
    ];

    for ts in &timestamps {
        let bytes = ts.to_le_bytes();
        if let Some(k) = bytes_to_key(&bytes) {
            keys.insert(k);
        }
        let bytes_be = ts.to_be_bytes();
        if let Some(k) = bytes_to_key(&bytes_be) {
            keys.insert(k);
        }
        let s = format!("{}", ts);
        if let Some(k) = bytes_to_key(s.as_bytes()) {
            keys.insert(k);
        }
        keys.insert(hash_to_key(&s));
    }

    println!("  Date-based: {} keys", keys.len());
    keys
}

// ============================================================================
// 4. KEYBOARD PATTERNS
// ============================================================================
fn generate_keyboard_patterns() -> BTreeSet<[u8; 32]> {
    let mut keys = BTreeSet::new();

    // QWERTY rows
    let rows = vec!["qwertyuiop", "asdfghjkl", "zxcvbnm"];
    for row in &rows {
        keys.insert(hash_to_key(row));
        if let Some(k) = bytes_to_key(row.as_bytes()) {
            keys.insert(k);
        }
        // Reversed
        let rev: String = row.chars().rev().collect();
        keys.insert(hash_to_key(&rev));
    }

    // Full keyboard patterns
    let patterns = vec![
        "qwertyuiopasdfghjklzxcvbnm",
        "mqnbvcxzlkjhgfdsapoiuytrewq",
        "1234567890",
        "0987654321",
        "qwerty",
        "asdfgh",
        "zxcvbn",
        "qazwsx",
        "edcrfv",
        "tgbnhy",
        "yhnnum",
        "poijuy",
        "lkjhgf",
        "mnbvcx",
        "qweasdzxc",
        "rfvtgbyhn",
        "yhnujmik",
        "olp;",
        "!@#$%^&*()",
        "1q2w3e4r5t6y7u8i9o0p",
        "1qaz2wsx3edc4rfv5tgb6yhn7ujm8ik9ol0p",
        "zaqwsxedcrtfvbgynhujmikolp",
        "abcdefghijklmnopqrstuvwxyz",
        "zyxwvutsrqponmlkjihgfedcba",
        "ABCDEF",
        "abcdef",
        "0123456789abcdef",
        "fedcba9876543210",
    ];

    for p in &patterns {
        keys.insert(hash_to_key(p));
        if let Some(k) = bytes_to_key(p.as_bytes()) {
            keys.insert(k);
        }
        // Repeated to fill 32 bytes
        let repeated = p.repeat(4);
        if let Some(k) = bytes_to_key(repeated.as_bytes()) {
            keys.insert(k);
        }
    }

    // Numpad patterns
    let numpad = vec![
        "7894561230",
        "1234567890",
        "7418529630",
        "1357924680",
        "9630741852",
    ];
    for p in &numpad {
        keys.insert(hash_to_key(p));
        if let Some(k) = bytes_to_key(p.as_bytes()) {
            keys.insert(k);
        }
    }

    // Diagonal patterns
    let diagonals = vec![
        "1tsadgfhjk;l'",
        "2yzxcvbnm,./",
        "puiojyhgtgfds",
        "8ik9ol0p;-[",
    ];
    for p in &diagonals {
        keys.insert(hash_to_key(p));
    }

    // Z-patterns and other common patterns
    let z_patterns = vec![
        "zszszszs",
        "mzmzmzmz",
        "qkqkqkqk",
        "jljljljl",
        "zxzxzxzx",
        "mqqmmqqm",
        "zqzmzqzm",
    ];
    for p in &z_patterns {
        keys.insert(hash_to_key(p));
        let repeated = p.repeat(4);
        if let Some(k) = bytes_to_key(repeated.as_bytes()) {
            keys.insert(k);
        }
    }

    println!("  Keyboard patterns: {} keys", keys.len());
    keys
}

// ============================================================================
// 5. SIMPLE PASSWORDS
// ============================================================================
fn generate_simple_passwords() -> BTreeSet<[u8; 32]> {
    let mut keys = BTreeSet::new();

    // Most common passwords
    let common_passwords: Vec<&str> = vec![
        "password", "123456", "12345678", "qwerty", "abc123", "monkey", "1234567",
        "letmein", "trustno1", "dragon", "baseball", "iloveyou", "master", "sunshine",
        "ashley", "bailey", "shadow", "123123", "654321", "superman", "qazwsx",
        "michael", "football", "password1", "password123", "1234", "12345", "123456789",
        "1234567890", "0987654321", "admin", "login", "welcome", "hello", "charlie",
        "donald", "batman", "access", "thunder", "matrix", "mustang", "password1",
        "test", "guest", "pass", "love", "sex", "god", "god1", "god2",
        "bitcoin", "btc", "crypto", "blockchain", "satoshi", "nakamoto",
        "satoshi1", "bitcoin1", "btc1", "crypto1", "btc2009", "bitcoin2009",
        "genesis", "block1", "block0", "coinbase", "mining", "mine",
        "wallet", "money", "cash", "rich", "free", "free1",
        "p2p", "p2p1", "p2p2", "peer", "network",
        "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
        "first", "last", "begin", "start", "end", "finish",
        "aaa", "aaaa", "aaaaaa", "aaaaaaaa", "aaaaaaaaaaaaaaaa",
        "zzz", "zzzz", "zzzzzz",
        "xxx", "xxxx", "xxxxxx",
        "yyy", "yyyy", "yyyyyy",
        "qqq", "qqqq", "qqqqqq",
        "www", "wwww", "wwwwww",
        "eee", "eeee", "eeeeee",
        "rrr", "rrrr", "rrrrrr",
        "ttt", "tttt", "tttttt",
        "ggg", "gggg", "gggggg",
        "fff", "ffff", "ffffff",
        "ddd", "dddd", "dddddd",
        "ccc", "cccc", "cccccc",
        "vvv", "vvvv", "vvvvvv",
        "bbb", "bbbb", "bbbbbb",
        "nnn", "nnnn", "nnnnnn",
        "mmm", "mmmm", "mmmmmm",
        "kkk", "kkkk", "kkkkkk",
        "jjj", "jjjj", "jjjjjj",
        "hhh", "hhhh", "hhhhhh",
        "uuu", "uuuu", "uuuuuu",
        "iii", "iiii", "iiiiii",
        "ooo", "oooo", "oooooo",
        "ppp", "pppp", "pppppp",
        "lll", "llll", "llllll",
    ];

    for pw in &common_passwords {
        // SHA256 hash
        keys.insert(hash_to_key(pw));

        // Direct bytes (zero-padded)
        if let Some(k) = bytes_to_key(pw.as_bytes()) {
            keys.insert(k);
        }

        // Double SHA256
        let h1 = Sha256::digest(pw.as_bytes());
        let h2 = Sha256::digest(&h1);
        let mut k = [0u8; 32];
        k.copy_from_slice(&h2);
        keys.insert(k);

        // MD5 (padded to 32 bytes)
        let md5 = Md5::digest(pw.as_bytes());
        let mut k = [0u8; 32];
        k[..16].copy_from_slice(&md5);
        keys.insert(k);

        // With common suffixes
        for suffix in ["", "1", "12", "123", "!", "@", "#", "2009", "2010", "btc", "bit"] {
            let combined = format!("{}{}", pw, suffix);
            keys.insert(hash_to_key(&combined));
        }

        // Uppercase
        keys.insert(hash_to_key(&pw.to_uppercase()));

        // Title case
        keys.insert(hash_to_key(&pw.chars().map(|c| if c.is_ascii_lowercase() { c.to_ascii_uppercase() } else { c }).collect::<String>()));
    }

    // Number sequences
    for i in 0..=99999 {
        let s = format!("{}", i);
        keys.insert(hash_to_key(&s));
        if let Some(k) = bytes_to_key(s.as_bytes()) {
            keys.insert(k);
        }
        // Zero-padded versions
        for pad_len in [4, 6, 8, 10, 12, 16, 20, 32] {
            let padded = format!("{:0>width$}", s, width = pad_len);
            keys.insert(hash_to_key(&padded));
            if let Some(k) = bytes_to_key(padded.as_bytes()) {
                keys.insert(k);
            }
        }
    }

    println!("  Simple passwords: {} keys", keys.len());
    keys
}

// ============================================================================
// 6. FAMOUS PHRASES
// ============================================================================
fn generate_famous_phrases() -> BTreeSet<[u8; 32]> {
    let mut keys = BTreeSet::new();

    let phrases: Vec<&str> = vec![
        // Classic brainwallet phrases
        "hello world",
        "hello",
        "world",
        "let it be",
        "to be or not to be",
        "i think therefore i am",
        "god is great",
        "god is love",
        "jesus christ",
        "jesus saves",
        "hallelujah",
        "amen",
        "peace love joy",
        "love is all you need",
        "think different",
        "just do it",
        "may the force be with you",
        "hello there",
        "general kenobi",
        "i am your father",
        "life is beautiful",
        "the meaning of life",
        "42",
        "answer is 42",
        "the answer is 42",
        "all your base are belong to us",
        "i can has cheezburger",
        "lolcat",
        "omg",
        "wtф",
        "brb",
        "gtg",
        "afk",
        "ttyl",
        "imo",
        "imho",
        "fyi",
        "asap",
        "idk",
        "idc",
        "smh",
        "ftw",
        "gg",
        "gg wp",
        "np",
        "gl hf",
        "pwned",
        "noob",
        "hacker",
        "elite",
        "l33t",
        "1337",
        "leetspeak",

        // Bitcoin whitepaper quotes
        "a purely peer-to-peer version of electronic cash",
        "double spending",
        "timestamp server",
        "proof of work",
        "network publishes a list of transactions",
        "incentive",
        "retrieving spent outputs",
        "satoshi nakamoto",
        "satoshi",
        "nakamoto",
        "bitcoin whitepaper",
        "bitcoin paper",
        "trust but verify",
        "be your own bank",
        "not your keys not your coins",
        "hodl",
        "hODL",
        "diamond hands",
        "paper hands",
        "to the moon",
        "moon",
        "lambo",
        "buy the dip",
        "fud",
        "ngmi",
        "wagmi",
        "ape in",
        "revenge trading",

        // Famous movie quotes
        "here's looking at you kid",
        "i'll be back",
        "may the force be with you",
        "i am your father",
        "you talkin to me",
        "here's johnny",
        "eto shoga nakya",
        "road goes ever on",
        "one ring to rule them all",
        "after all this time always",
        "i am groot",
        "why so serious",
        "why so serious",
        "elementary my dear watson",
        "game over man game over",
        "ride the lightning",
        "enter the dragon",
        "kill bill",
        "pulp fiction",
        "reservoir dogs",
        "the matrix",
        "follow the white rabbit",
        "red pill blue pill",

        // Song lyrics (famous)
        "is this the real life",
        "is this just fantasy",
        "hello darkness my old friend",
        "cause we are all we need",
        "we are the champions",
        "bohemian rhapsody",
        "stairway to heaven",
        "yellow submarine",
        "imagine",
        "imagine there is no heaven",
        "let it be",
        "hey jude",
        "nothing can stop us now",
        "we are family",
        "i will survive",
        "dont stop believing",
        "dream on",
        "sweet child o mine",
        "paradise city",
        "free bird",
        "born to run",
        "thriller",
        "billie jean",
        "beat it",
        "smooth criminal",
        "like a virgin",
        "material girl",
        "vogue",
        "nothing compares 2 u",
        "torn",
        "irreplaceable",
        "umbrella",
        "single ladies",
        "tiK tok",
        "lemon tree",
        "hey ya",
        "crazy in love",
        "yesterday",
        "here comes the sun",
        "let it be",
        "come together",
        "revolution",
        "all you need is love",
        "hey jude",
        "the long and winding road",
        "help",
        "strawberry fields forever",
        "penny lane",
        "a hard days night",
        "she loves you",
        "can you feel the love tonight",
        "canon in d",
        "fur elise",
        "moonlight sonata",
        "four seasons",
        "the blue danube",
        "adele",
        "rolling stones",
        "beatles",
        "queen",
        "led zeppelin",
        "pink floyd",
        "nirvana",
        "smashing pumpkins",
        "radiohead",
        "coldplay",
        "linkin park",
        "metallica",
        "gun n roses",
        "acdc",
        "guns n roses",
        "bon jovi",
        "aerosmith",
        "van halen",
        "deep purple",
        "black sabbath",
        "iron maiden",
        "megadeth",
        "slayer",
        "anthrax",
        "motley crue",
        "poison",
        "bon jovi",
        "def leppard",
        "judas priest",
        "scorpions",
        "europa city",
        "wind of change",

        // Programming
        "hello world",
        "hello, world",
        "hello world!",
        "fn main",
        "def main",
        "main function",
        "void main",
        "int main",
        "public static void main",
        "console.log",
        "print hello",
        "printf hello",
        "echo hello",
        "system halt",
        "kernel panic",
        "segmentation fault",
        "null pointer",
        "buffer overflow",
        "stack overflow",
        "blue screen of death",
        "it works on my machine",
        "it depends",
        "undefined behavior",
        "race condition",
        "deadlock",
        "memory leak",
        "garbage collection",
        "recursion",
        "lambda",
        "closure",
        "monad",
        "polymorphism",
        "inheritance",
        "encapsulation",
        "abstraction",
        "design pattern",
        "singleton",
        "factory",
        "observer",
        "strategy",
        "adapter",
        "decorator",
        "proxy",
        "facade",
        "bridge",
        "composite",
        "flyweight",
        "memento",
        "visitor",
        "command",
        "iterator",
        "builder",
        "prototype",
        "chain of responsibility",
        "mediator",
        "interpreter",
        "specification",
        "template method",

        // Math phrases
        "e equals mc squared",
        "e=mc2",
        "emc2",
        "pythagorean theorem",
        "a squared plus b squared",
        "fermats last theorem",
        "goldbachs conjecture",
        "riemann hypothesis",
        "p equals np",
        "p vs np",
        "turing machine",
        "halting problem",
        "godel incompleteness",
        "chinese remainder theorem",
        "euler formula",
        "stokes theorem",
        "greens theorem",
        "gauss lemma",
        "bayes theorem",
        "central limit theorem",
        "law of large numbers",
        "monte carlo",
        "markov chain",
        "four color theorem",
        "birthday paradox",
        "monty hall problem",
        "prisoners dilemma",
        "game theory",
        "nash equilibrium",
        "pareto optimal",
        "efficient market hypothesis",
        "random walk",
        "brownian motion",
        "black scholes",
        "monte hall",
        "pascal triangle",
        "fibonacci sequence",
        "golden ratio",
        "decimal expansion",
        "irrational number",
        "transcendental number",
        "prime number",
        "twin prime",
        "mersenne prime",
        "perfect number",
        "amicable numbers",
        "happy number",
        "armstrong number",
        "palindrome",
        "repdigit",
        "smiley number",
    ];

    for phrase in &phrases {
        // SHA256
        keys.insert(hash_to_key(phrase));

        // Direct bytes
        if let Some(k) = bytes_to_key(phrase.as_bytes()) {
            keys.insert(k);
        }

        // Lowercase
        keys.insert(hash_to_key(&phrase.to_lowercase()));

        // Uppercase
        keys.insert(hash_to_key(&phrase.to_uppercase()));

        // No spaces
        let no_spaces: String = phrase.chars().filter(|c| !c.is_whitespace()).collect();
        keys.insert(hash_to_key(&no_spaces));

        // Reversed
        let rev: String = phrase.chars().rev().collect();
        keys.insert(hash_to_key(&rev));

        // With exclamation
        keys.insert(hash_to_key(&format!("{}!", phrase)));

        // With question mark
        keys.insert(hash_to_key(&format!("{}?", phrase)));

        // Double SHA256
        let h1 = Sha256::digest(phrase.as_bytes());
        let h2 = Sha256::digest(&h1);
        let mut k = [0u8; 32];
        k.copy_from_slice(&h2);
        keys.insert(k);

        // MD5 padded
        let md5 = Md5::digest(phrase.as_bytes());
        let mut k = [0u8; 32];
        k[..16].copy_from_slice(&md5);
        keys.insert(k);
    }

    println!("  Famous phrases: {} keys", keys.len());
    keys
}

// ============================================================================
// 7. PHYSICAL CONSTANTS
// ============================================================================
fn generate_physical_constants() -> BTreeSet<[u8; 32]> {
    let mut keys = BTreeSet::new();

    // Physical constants (high precision)
    let constants: Vec<(&str, &str)> = vec![
        ("speed_of_light", "299792458"),
        ("planck_constant", "662607015"),
        ("boltzmann_constant", "1380649"),
        ("avogadro_constant", "602214076"),
        ("elementary_charge", "1602176634"),
        ("gravitational_constant", "667430"),
        ("electron_mass", "91093837015"),
        ("proton_mass", "167262192369"),
        ("neutron_mass", "167492749804"),
        ("fine_structure", "137035999"),
        ("riemann_zeta_3", "1202056903"),
        ("stefan_boltzmann", "56703744"),
        ("wien_displacement", "28977719"),
        ("faraday_constant", "9648533212"),
        ("gas_constant", "8314462618"),
        ("magnetic_constant", "1256637062"),
        ("electric_constant", "8854187817"),
        ("impedance_of_vacuum", "376730313"),
        ("bohr_radius", "5291772109"),
        ("rydberg_constant", "10973731568"),
        ("thomson_cross_section", "6652458732"),
        ("classical_electron_radius", "2817940326"),
        ("compton_wavelength", "242631023867"),
        ("hartree_energy", "43597447222071"),
        ("molar_volume", "2271095434"),
        ("magnetron_frequency", "285962148"),
        ("proton_gyromagnetic", "2675122181"),
        ("neutron_gyromagnetic", "1832471857"),
        ("electron_gyromagnetic", "1760859644"),
        ("planck_length", "1616255"),
        ("planck_mass", "2176434"),
        ("planck_time", "539124"),
        ("planck_temperature", "1416784"),
    ];

    for (name, value) in &constants {
        keys.insert(hash_to_key(name));
        keys.insert(hash_to_key(value));
        keys.insert(hash_to_key(&format!("{}:{}", name, value)));

        if let Some(k) = bytes_to_key(value.as_bytes()) {
            keys.insert(k);
        }

        // As integer bytes
        if let Ok(n) = value.parse::<u128>() {
            if let Some(k) = bytes_to_key(&n.to_le_bytes()) {
                keys.insert(k);
            }
            if let Some(k) = bytes_to_key(&n.to_be_bytes()) {
                keys.insert(k);
            }
        }
    }

    // Atomic numbers 1..118
    for z in 1..=118u128 {
        let bytes_le = z.to_le_bytes();
        if let Some(k) = bytes_to_key(&bytes_le) {
            keys.insert(k);
        }
    }

    println!("  Physical constants: {} keys", keys.len());
    keys
}

// ============================================================================
// 8. CHESS POSITIONS
// ============================================================================
fn generate_chess_keys() -> BTreeSet<[u8; 32]> {
    let mut keys = BTreeSet::new();

    // Famous chess games (algebraic notation)
    let games: Vec<&str> = vec![
        "Opera Game",
        "Immortal Game",
        "Evergreen Game",
        "Game of the Century",
        "Anderssen vs Kieseritzky",
        "Morphy vs Duke",
        "Kasparov vs Deep Blue",
        "Fischer vs Sparovsky",
        "Boden's Opera Game",
        "Legal's Immortal Game",
        "Paulsen vs Wayte",
        "Schmidt vs Paulsen",
        "Lasker vs Schlechter",
        "Capablanca vs Marshall",
        "Alekhine vs Capablanca",
        "Fischer vs Spassky",
        "Kasparov vs Karpov",
        "Carlsen vs Caruana",
        "Ruy Lopez",
        "Sicilian Defense",
        "French Defense",
        "Caro-Kann",
        "King's Gambit",
        "Queen's Gambit",
        "English Opening",
        "Reti Opening",
        "Nimzo-Indian",
        "Kings Indian",
        "Grunfeld Defense",
        "Pirc Defense",
        "Modern Defense",
        "Scandinavian Defense",
        "Scotch Game",
        "Vienna Game",
        "Italian Game",
        "Giuoco Piano",
        "Evans Gambit",
        "Philidor Defense",
        "Petrov Defense",
        "Elephant Gambit",
        "Damiano Defense",
        "Bongcloud",
        "Nimzowitsch-Larsen",
        "Old Indian Defense",
        "Benoni Defense",
        "Benko Gambit",
        "Old Benoni",
        "King's Indian Attack",
        "Four Knights",
        "Scotch Counter-Gambit",
        "Center Game",
        "Danish Gambit",
        "Elephant Trap",
        "Latvian Gambit",
        "Polish Defense",
        "Bird Opening",
        "Grob Opening",
        "Sokolov Opening",
        "Jaenisch Opening",
        "Ware Opening",
        "Van Geet Opening",
        "Dutch Defense",
        "Lasker Defense",
        "Staunton Gambit",
        "Hollander Variation",
        "Stonewall Dutch",
        "Austrian Attack",
        "Main Line",
        "Classical Variation",
        "Fianchetto Variation",
        "Leningrad Dutch",
        "Ivanchuk System",
        "8 Nbd7",
        "8 Nd7",
        "7 e5",
        "7 d5",
        "6 e4",
        "6 d5",
        "5 Nc3",
        "5 e4",
        "4 Nf3",
        "4 e4",
        "3 Nc3",
        "3 e4",
        "2 Nf3",
        "2 e4",
        "1 e4",
        "1 d4",
        "1 c4",
        "1 Nf3",
        "1 b3",
        "1 f4",
        "1 g4",
        "1 a3",
        "1 h3",
        "1 b4",
        "1 Nc3",
        "1 e3",
        "1 d3",
        "1 c3",
        "1 f3",
    ];

    for game in &games {
        keys.insert(hash_to_key(game));
        keys.insert(hash_to_key(&game.to_lowercase()));

        // FEN starting position
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        keys.insert(hash_to_key(&format!("{}:{}", game, fen)));
    }

    // Starting position variations
    let openings: Vec<(&str, &str)> = vec![
        ("e4", "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"),
        ("d4", "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq - 0 1"),
        ("c4", "rnbqkbnr/pppppppp/8/8/2P5/8/PP1PPPPP/RNBQKBNR b KQkq - 0 1"),
        ("Nf3", "rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 1 1"),
        ("f4", "rnbqkbnr/pppppppp/8/8/5P2/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"),
    ];

    for (name, fen) in &openings {
        keys.insert(hash_to_key(fen));
        keys.insert(hash_to_key(&format!("chess:{}", name)));
    }

    // Chess piece values as keys
    let piece_values_str = "133590"; // P=1, N=3, B=3, R=5, Q=9, K=0
    if let Some(k) = bytes_to_key(piece_values_str.as_bytes()) {
        keys.insert(k);
    }
    keys.insert(hash_to_key("133590"));

    println!("  Chess: {} keys", keys.len());
    keys
}

// ============================================================================
// 9. POP CULTURE
// ============================================================================
fn generate_pop_culture() -> BTreeSet<[u8; 32]> {
    let mut keys = BTreeSet::new();

    // 2009 era pop culture
    let pop_2009: Vec<&str> = vec![
        "ice ice baby",
        "rick astley",
        "never gonna give you up",
        "rickroll",
        "lolcat",
        "i can has cheezburger",
        "omg",
        "ftw",
        "lol",
        "rofl",
        "lmao",
        "brb",
        "gtg",
        "afk",
        "ttyl",
        "idk",
        "smh",
        "nvm",
        "tbh",
        "imo",
        "idc",
        "stfu",
        "aslf",
        "l33t",
        "1337",
        "h4x0r",
        "pwn",
        "pwned",
        "noob",
        "nooblet",
        "nerd",
        "geek",
        "hacker",
        "script kiddie",
        "phreaker",
        "cracker",
        "rootkit",
        "trojan",
        "worm",
        "virus",
        "malware",
        "spyware",
        "adware",
        "ransomware",
        "keylogger",
        "root",
        "sudo",
        "chmod",
        "chown",
        "rm -rf",
        "format c",
        "deltree",
        "debug",
        "comspec",
        "command com",
        "cmd exe",
        "autoexec bat",
        "config sys",
        "msdos sys",
        "io sys",
        "ntldr",
        "bootmgr",
        "bcdedit",
        "msconfig",
        "taskmgr",
        "regedit",
        "gpedit",
        "services",
        "eventvwr",
        "perfmon",
        "resmon",
        "devmgmt",
        "diskmgmt",
        "compmgmt",
        "sysdm",
        "appwiz",
        "control",
        "explorer",
        "notepad",
        "mspaint",
        "calc",
        "cmd",
        "powershell",
        "bash",
        "zsh",
        "fish",
        "tcsh",
        "csh",
        "ksh",
        "sh",
        "dash",
        "ash",
        "busybox",
        "alpine",
        "debian",
        "ubuntu",
        "fedora",
        "arch",
        "gentoo",
        "slackware",
        "redhat",
        "centos",
        "suse",
        "mint",
        "elementary",
        "pop os",
        "zorin",
        "linux",
        "windows",
        "macos",
        "ios",
        "android",
        "chromeos",
        "freebsd",
        "openbsd",
        "netbsd",
        "dragonfly",
        "solaris",
        "aix",
        "hpux",
        "irix",
        "tru64",
        "osf1",
        "unix",
        "plan9",
        "inferno",
        "minix",
        "reactos",
        "haiku",
        "amigaos",
        "morphos",
        "aros",
        "beos",
        "palm os",
        "symbian",
        "blackberry",
        "webos",
        "qnx",
        "vxworks",
        "lynxos",
        "integri",
        "wind river",
        "vxworks",
        "threadx",
        "free rtos",
        "embos",
        "micrium",
        "salvo",
        "rtx",
        "ucos",
        "contiki",
        "riot",
        "zephyr",
        "nuttx",
        "arm mbed",
        "tinyos",
        "contiki",
        "cocoos",
        "nesos",
        "blip",
        "jinux",
        "djava",
        "kjava",
        "me java",
        "feature java",
        "java se",
        "java ee",
        "java me",
        "javafx",
        "android java",
        "robovm",
        "graalvm",
        "substance vm",
        "dalvik",
        "art",
        "openjdk",
        "oracle jdk",
        "amazon corretto",
        "adoptopenjdk",
        "zulu",
        "dragonwell",
        "liberica",
        "sap machine",
        "ibm j9",
        "wasmer",
        "wasmtime",
        "wasmedge",
        "wamr",
        "wavelan",
        "wasi",
        "wasm",
        "webassembly",
        "javascript",
        "typescript",
        "coffeescript",
        "dart",
        "flutter",
        "react",
        "angular",
        "vue",
        "svelte",
        "nextjs",
        "nuxt",
        "gatsby",
        "remix",
        "astro",
        "solid",
        "qwik",
        "htmx",
        "alpinejs",
        "stimulus",
        "ember",
        "backbone",
        "knockout",
        "marionette",
        "rivets",
        "aurelia",
        "mithril",
        "preact",
        "inferno",
        "hyperapp",
        "riot",
        "decentral",
        "canjs",
        "polymer",
        "aurelia",
        "native script",
        "ionic",
        "react native",
        "flutter",
        "xamarin",
        "maui",
        "unity",
        "unreal",
        "godot",
        "defold",
        "love2d",
        "monogame",
        "fna",
        "stride",
        "berserk",
        "bevy",
        "flecs",
        "enTT",
        "sol2",
        "lua",
        "rust",
        "cpp",
        "csharp",
        "python",
        "java",
        "go",
        "swift",
        "kotlin",
        "scala",
        "clojure",
        "haskell",
        "erlang",
        "elixir",
        "fsharp",
        "ocaml",
        "reason",
        "rescript",
        "purescript",
        "elm",
        "idris",
        "agda",
        "coq",
        "lean",
        "isabelle",
        "hol",
        "mizar",
        "matita",
        "cayenne",
        "epigram",
        "alf",
        "clash",
        "clean",
        "joy",
        "factor",
        "io",
        "rebol",
        "red",
        "forth",
        "postscript",
        "pdf",
        "ghostscript",
        "xpdf",
        "mupdf",
        "poppler",
        "cups",
        "systemd",
        "upstart",
        "sysvinit",
        "runit",
        "s6",
        "openrc",
        "dinit",
        "rcos",
        "busybox",
        "alpine",
        "debian",
        "ubuntu",
    ];

    for item in &pop_2009 {
        keys.insert(hash_to_key(item));
        keys.insert(hash_to_key(&item.to_lowercase()));

        // With year suffixes
        for year in ["", "2009", "2010", "2011", "2012", "2013", "2008", "2007"] {
            keys.insert(hash_to_key(&format!("{}{}", item, year)));
        }
    }

    // Famous usernames/handles from early internet
    let usernames: Vec<&str> = vec![
        "admin", "root", "user", "test", "guest", "demo",
        "bitcoin", "btc", "crypto", "satoshi", "nakamoto",
        "hal finney", "halfinney", "nicolas dot", "artforz",
        "gavin andresen", "amase", "michael clear",
        "marc stever", "steven hash", "rogery",
        "luke jr", "lukej", "peter todd", "peterdavidtodd",
        "wladimir j van der laan", "laanwj",
        "jonas schnelli", "sipa", "pieter winkel",
        "adrian matthews", "adrianv", "marijn",
        "ephin", "jtimon", "cdecker",
        "the stack", "blockstream", "lightning",
        "core lightning", "lnd", "eclair", "btcpay",
        "blockstream", "unchained", "mempool",
        "mempool space", "mempool", "liquid",
        "federated sidechain", "fedimint", "cashu",
        "darwin", "apple", "steve jobs", "steve wozniak",
        "bill gates", "paul allen", "microsoft",
        "mark zuckerberg", "facebook", "meta",
        "elon musk", "tesla", "spacex", "twitter", "x",
        "jack dorsey", "biz stone", "noah glass",
        "evan william", "twitter",
        "digg", "reddit", "alexis ohanian", "steve huffman",
        "4chan", "anon", "anonymous",
        "lulzsec", "anon", "topiary", "sabu",
        "kali linux", "backtrack", "parrot",
        "metasploit", "burp suite", "wireshark",
        "nmap", "nessus", "openvas",
        "sqlmap", "nikto", "dirb", "gobuster",
        "hashcat", "john", "john the ripper",
        "aircrack", "wifite", "reaver", "bully",
        "hashcat", "cuda", "opencl", "oclhashcat",
        "phoenix", "cudahashcat", "hcstat",
        "rules", "best64", "one3r", "rockyou",
        "weakpass", "crackstation", "have i been pwned",
        "troy hunt", "hibp", "pwned",
        "password", "passphrase", "secret", "key",
        "private", "public", "encrypted", "decrypted",
        "hash", "salt", "nonce", "iv", "cipher",
        "aes", "des", "3des", "blowfish", "twofish",
        "rc4", "rc5", "rc6", "skipjack",
        "idea", "cast", "serpent", "camellia",
        "sha1", "sha256", "sha512", "md5", "ripemd",
        "hmac", "pbkdf2", "bcrypt", "scrypt", "argon2",
        "ed25519", "curve25519", "nacl", "libsodium",
        "secp256k1", "nist p256", "brainpool",
        "ed448", "x448", "ed25519", "x25519",
        "sponge", "keccak", "sha3", "blake2", "blake3",
        "chacha20", "poly1305", "aes gcm", "aes cbc",
        "rsa", "dsa", "ecdsa", "eddsa", "diffie hellman",
        "ellgamal", "mcEliece", "ntru", "lattice",
        "post quantum", "kyber", "dilithium", "spike",
        "falcon", "saber", "frodo", "picnic",
        "mqs", "rainbow", "ges al", "luov",
        "pqcrystals", "liboqs", "open quantum safe",
    ];

    for user in &usernames {
        keys.insert(hash_to_key(user));
        keys.insert(hash_to_key(&user.to_lowercase()));

        if let Some(k) = bytes_to_key(user.as_bytes()) {
            keys.insert(k);
        }
    }

    println!("  Pop culture: {} keys", keys.len());
    keys
}

// ============================================================================
// 10. HEX PATTERNS
// ============================================================================
fn generate_hex_patterns() -> BTreeSet<[u8; 32]> {
    let mut keys = BTreeSet::new();

    // Repeating byte patterns
    for byte in 0..=255u8 {
        let key = [byte; 32];
        keys.insert(key);
    }

    // Sequential patterns
    for start in 0..=255u8 {
        let key: [u8; 32] = std::array::from_fn(|i| (start.wrapping_add(i as u8)));
        keys.insert(key);
    }

    // Reverse sequential
    for start in 0..=255u8 {
        let key: [u8; 32] = std::array::from_fn(|i| (start.wrapping_sub(i as u8)));
        keys.insert(key);
    }

    // Alternating patterns
    for a in 0..=255u8 {
        for b in 0..=255u8 {
            let key: [u8; 32] = std::array::from_fn(|i| if i % 2 == 0 { a } else { b });
            keys.insert(key);
        }
    }

    // Incrementing by step
    for start in 0..=127u8 {
        for step in [1u8, 2, 4, 7, 8, 16, 32, 64, 128] {
            let key: [u8; 32] = std::array::from_fn(|i| start.wrapping_add((i as u8).wrapping_mul(step)));
            keys.insert(key);
        }
    }

    // Palindrome patterns (16 bytes mirrored)
    for first_byte in 0..=255u8 {
        for last_byte in 0..=255u8 {
            let mut key = [0u8; 32];
            key[0] = first_byte;
            key[31] = first_byte;
            key[1] = last_byte;
            key[30] = last_byte;
            // Fill middle with sequential
            for i in 2..16 {
                key[i] = (first_byte.wrapping_add(i as u8));
                key[31 - i] = key[i];
            }
            keys.insert(key);
        }
    }

    // Power of 2 patterns
    for i in 0..=255u8 {
        if i.count_ones() == 1 {
            let key = [i; 32];
            keys.insert(key);
        }
    }

    // All bits set patterns
    for mask in [0xFFu8, 0xFE, 0xFD, 0xFB, 0xF7, 0xEF, 0xDF, 0xBF, 0x7F] {
        let key = [mask; 32];
        keys.insert(key);
    }

    // Fibonacci as bytes
    let mut a: u8 = 0;
    let mut b: u8 = 1;
    let fib_bytes: Vec<u8> = (0..32).map(|_| {
        let temp = a.wrapping_add(b);
        a = b;
        b = temp;
        a
    }).collect();
    if fib_bytes.len() == 32 {
        keys.insert(fib_bytes.try_into().unwrap());
    }

    // PRNG patterns (LCG)
    for seed in 0..=255u8 {
        let mut s = seed as u32;
        let key: [u8; 32] = std::array::from_fn(|_| {
            s = s.wrapping_mul(1103515245).wrapping_add(12345);
            (s >> 16) as u8
        });
        keys.insert(key);
    }

    // Simple XOR patterns
    for a in 0..=255u8 {
        for b in (a+1)..=255u8 {
            let key: [u8; 32] = std::array::from_fn(|i| {
                if i % 3 == 0 { a } else if i % 3 == 1 { b } else { a ^ b }
            });
            keys.insert(key);
        }
    }

    // Counter patterns (big endian and little endian)
    for i in 0..=1000u128 {
        if let Some(k) = bytes_to_key(&i.to_le_bytes()) {
            keys.insert(k);
        }
        if let Some(k) = bytes_to_key(&i.to_be_bytes()) {
            keys.insert(k);
        }
    }

    // Specific well-known hex values
    let known_hex: Vec<&str> = vec![
        "0000000000000000000000000000000000000000000000000000000000000001",
        "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140",  // secp256k1 order
        "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",  // secp256k1 generator x
        "483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8",  // SHA256("hello")
        "6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d",  // SHA256("hello world")
        "a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e",  // SHA256("bitcoin")
        "f75a804094715ea5c4a1d4443bbaa79bee2ba3870963757152adf6a0d9f1c2a5",  // SHA256("satoshi")
        "b07c4e440187b43cbe1e6bb2a3c15e50c26571489e6c6463e0cfac5fcc42d0e7",  // SHA256("password")
        "5eb63bbbe01eeed093cb22bb8f5acdc3",  // MD5("hello")
        "25d55ad283aa400af464c76d713c07ad",  // MD5("hello world")
        "5f4dcc3b5aa765d61d8327deb882cf99",  // MD5("password")
    ];

    for hex_str in &known_hex {
        if let Ok(bytes) = hex::decode(hex_str) {
            if let Some(k) = bytes_to_key(&bytes) {
                keys.insert(k);
            }
        }
    }

    // secp256k1 order - 1, order - 2, etc.
    let order = [
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
        0xba, 0xae, 0xdc, 0x6a, 0xf4, 0x8a, 0x03, 0xbb,
        0xfd, 0x25, 0xe8, 0xcd, 0x03, 0x64, 0x14, 0x00,
    ];
    keys.insert(order);
    // order - 1
    let mut order_minus_1 = order;
    order_minus_1[31] -= 1;
    keys.insert(order_minus_1);

    // Small valid keys near order boundary
    for i in 0..100u8 {
        let mut key = order;
        key[31] = i;
        keys.insert(key);
    }

    // First 100 valid keys
    for i in 1..=100u128 {
        if let Some(k) = bytes_to_key(&i.to_le_bytes()) {
            keys.insert(k);
        }
        if let Some(k) = bytes_to_key(&i.to_be_bytes()) {
            keys.insert(k);
        }
    }

    // Large valid keys
    for i in 1..=100u128 {
        let mut key = order;
        // Subtract i from the last 8 bytes
        let last8_bytes: [u8; 8] = key[24..32].try_into().unwrap();
        let last8: u64 = u64::from_le_bytes(last8_bytes);
        let new_last8 = last8.saturating_sub(i as u64);
        key[24..32].copy_from_slice(&new_last8.to_le_bytes());
        keys.insert(key);
    }

    println!("  Hex patterns: {} keys", keys.len());
    keys
}
