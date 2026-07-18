//! Math Constants Key Generator v3
//!
//! Generates private keys from mathematical constants, sequences, and patterns
//! that a human in 2009 might have used. These are "low entropy" keys that could
//! be brute-forced.
//!
//! Categories:
//! 1. Pi digits (3.1415926535...) in various encodings
//! 2. e (Euler's number) digits
//! 3. Square roots of primes
//! 4. Golden ratio (phi)
//! 5. Other constants (gamma, ln2, sqrt2, sqrt3, etc.)
//! 6. Famous sequences (Fibonacci, primes, powers of 2)
//! 7. Date-based keys (birthdays, significant dates)
//! 8. Simple passwords hashed (SHA256)
//! 9. Keyboard patterns
//! 10. Repetitive patterns
//! 11. Phone number patterns
//! 12. Lucky numbers
//! 13. Pi/e/sqrt combinations
//! 14. Mathematical expressions
//! 15. Famous numbers (666, 42, 137, etc.)
//!
//! Output: hex-encoded 256-bit private keys, one per line

use sha2::{Digest, Sha256};
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    let output = args.get(1).map(|s| s.as_str()).unwrap_or("data/math-constants-keys.txt");

    let mut file = File::create(output).expect("Failed to create output file");
    let mut w = BufWriter::new(&mut file);

    let mut count = 0usize;

    // ── 1. Pi digits ─────────────────────────────────────────────────────
    eprintln!("[1/15] Pi digits...");
    // First 1000 digits of pi (after decimal point)
    let pi_digits = "1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679821480865132823066470938446095505822317253594081284811174502841027019385211055596446229489549303819644288109756659334461284756482337867831652712019091456485669234603486104543266482133936072602491412737245870066063155881748815209209628292540917153643678925903600113305305488204665213841469519415116094330572703657595919530921861173819326117931051185480744623799627495673518857527248912279381830119491298336733624406566430860213949463952247371907021798609437027705392171762931767523846748184676694051320005681271452635608277857713427577896091736371787214684409012249534301465495853710507922796892589235420199561121290219608640344181598136297747713099605187072113499999983729780499510597317328160963185950244594553469083026425223082533446850352619311881710100031378387528865875332083814206171776691473035982534904287554687311595628638823537875937519577818577805321712268066130019278766111959092164201989";

    // Sliding windows of pi digits → 256-bit keys
    for window in [32usize, 64, 128, 256, 512, 1000] {
        if window > pi_digits.len() { continue; }
        for start in 0..=pi_digits.len() - window {
            let chunk = &pi_digits[start..start + window];
            let key = digits_to_key(chunk, window == 32);
            writeln!(w, "{}", hex::encode(key)).ok();
            count += 1;
        }
    }

    // ── 2. Euler's number (e) ───────────────────────────────────────────
    eprintln!("[2/15] Euler's number (e)...");
    let e_digits = "7182818284590452353602874713526624977572470936999595749669676277240766303535475945713821785251664274274663919320030599218174135966290435729003342952605956307381323286279434907632338298807531952510190115738341879307021540891499348841675092447614606680822648001684774118537423454424371075390777449920695517027618386062613313845830007520449338265602976067371132007093287091274437470472306969772093101416928368190255151086574637721112523897844250569536967707854499699679468644549059879316368892300987931277361782154249992295763514822082698951936680331825288693984964651058209392398294887933203625094431173012381970684161403970198376793206832823764648042953118023287825098194558153017567173613320698112509961818815930416903515988885193458072738667385894228792284998920868058257492796104841984443634632449684875602336248270419786232090021609902353043699418491463140934317381436405462531520961836908887070167683964243781409935632815228613187249472252773033897316114103267437024814155657063068157752880898777352706666439937611232953082117167";
    for window in [32usize, 64, 128, 256, 512, 1000] {
        if window > e_digits.len() { continue; }
        for start in 0..=e_digits.len() - window {
            let chunk = &e_digits[start..start + window];
            let key = digits_to_key(chunk, window == 32);
            writeln!(w, "{}", hex::encode(key)).ok();
            count += 1;
        }
    }

    // ── 3. Square roots of first 100 primes ──────────────────────────────
    eprintln!("[3/15] Square roots of primes...");
    let primes = [2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71,73,79,83,89,97,
        101,103,107,109,113,127,131,137,139,149,151,157,163,167,173,179,181,191,193,197,199,
        211,223,227,229,233,239,241,251,257,263,269,271,277,281,283,293,307,311,313,317,331,
        337,347,349,353,359,367,373,379,383,389,397,401,409,419,421,431,433,439,443,449,457,461,463,467,479,487,491,499];
    for &p in &primes {
        let sqrt_val = (p as f64).sqrt();
        // Use the decimal representation (after decimal point)
        let sqrt_str = format!("{:.100}", sqrt_val);
        let digits = sqrt_str.replace(".", "").replace("1", ""); // remove "1." prefix for sqrt(1)=1 case
        // Actually, let's use the full decimal after the point
        let after_dot = sqrt_str.split('.').nth(1).unwrap_or("");
        // f64 sqrt only gives ~15 significant digits, so limit window sizes
        for window in [32usize, 64, 128, 256] {
            if window > after_dot.len() { continue; }
            for start in 0..=after_dot.len() - window {
                let chunk = &after_dot[start..start + window];
                let key = digits_to_key(chunk, window == 32);
                writeln!(w, "{}", hex::encode(key)).ok();
                count += 1;
            }
        }
        // Also use the prime number itself as a key
        for &prefix in &[0u64, 1, 42, 666, 1337, 2026, 1985, 1970, 2000, 2009] {
            let mut key = [0u8; 32];
            let combined = ((prefix as u128) << 64) | (p as u128);
            key[0..16].copy_from_slice(&combined.to_le_bytes());
            writeln!(w, "{}", hex::encode(key)).ok();
            count += 1;
        }
    }

    // ── 4. Golden ratio (phi) ────────────────────────────────────────────
    eprintln!("[4/15] Golden ratio (phi)...");
    let phi_digits = "61803398874989484820458683436563811772030917980576286213544862270526046281890244970720720418939113748475408807538689175212663386222353693179318006076672635443338908659593958290563832266131993808202376";
    for window in [32usize, 64, 128, 256, 512] {
        if window > phi_digits.len() { continue; }
        for start in 0..=phi_digits.len() - window {
            let chunk = &phi_digits[start..start + window];
            let key = digits_to_key(chunk, window == 32);
            writeln!(w, "{}", hex::encode(key)).ok();
            count += 1;
        }
    }

    // ── 5. Other mathematical constants ──────────────────────────────────
    eprintln!("[5/15] Other constants...");
    let constants: Vec<(&str, &str)> = vec![
        ("sqrt2", "41421356237309504880168872420969807856967187537694807317667973799073247846210703885038753432764157273501384623091229702492483605585073721264412149709993583141322266592750559275579995050115278206057147"),
        ("sqrt3", "732050807568877293527446341505872366942805253810380628055806979451933016908800037081146186757248575601235000"),
        ("ln2", "301029995663981195213738894724493026768189881462108541310427461202025318169032332482359033663742117834321379079"),
        ("gamma", "57721566490153286060651209008240243104215933593992"), // Euler-Mascheroni
        ("apery", "61245401160128572432783606139922675933935689741547"), // Apery's constant
        ("twinprime", "68955562410935241897522937102255403554326768734506"), // Twin prime constant
    ];
    for (name, digits) in &constants {
        eprintln!("  {name}...");
        for window in [32usize, 64, 128, 256] {
            if window > digits.len() { continue; }
            for start in 0..=digits.len() - window {
                let chunk = &digits[start..start + window];
                let key = digits_to_key(chunk, window == 32);
                writeln!(w, "{}", hex::encode(key)).ok();
                count += 1;
            }
        }
        // Also hash the constant name
        let key = sha256_key(name);
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;
        let key = sha256_key(&format!("{}{}", name, digits));
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;
    }

    // ── 6. Famous sequences ──────────────────────────────────────────────
    eprintln!("[6/15] Famous sequences...");
    // Fibonacci numbers (first 100)
    let mut fib: Vec<u128> = vec![0, 1];
    for i in 2..100 {
        let next = fib[i - 1] + fib[i - 2];
        if next < u128::MAX {
            fib.push(next);
        } else {
            break;
        }
    }
    for &f in &fib {
        let mut key = [0u8; 32];
        key[16..32].copy_from_slice(&f.to_le_bytes());
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;
    }
    // Powers of 2
    for exp in 0..128 {
        let val = 1u128 << exp;
        let mut key = [0u8; 32];
        key[16..32].copy_from_slice(&val.to_le_bytes());
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;
    }
    // Factorials
    let mut fact: u128 = 1;
    for i in 1..=20 {
        fact *= i as u128;
        let mut key = [0u8; 32];
        key[16..32].copy_from_slice(&fact.to_le_bytes());
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;
    }

    // ── 7. Date-based keys ───────────────────────────────────────────────
    eprintln!("[7/15] Date-based keys...");
    // Years 1900-2030, months 1-12, days 1-31
    for year in 1900..=2030 {
        for month in 1..=12 {
            for day in 1..=31 {
                // YYYYMMDD format
                let date_str = format!("{:04}{:02}{:02}", year, month, day);
                let key = sha256_key(&date_str);
                writeln!(w, "{}", hex::encode(key)).ok();
                count += 1;

                // Also as raw bytes
                let mut key = [0u8; 32];
                key[24..28].copy_from_slice(&date_str.parse::<u32>().unwrap_or(0).to_le_bytes());
                writeln!(w, "{}", hex::encode(key)).ok();
                count += 1;
            }
        }
    }

    // ── 8. Simple passwords hashed ───────────────────────────────────────
    eprintln!("[8/15] Simple passwords...");
    let passwords = [
        "password", "password1", "password123", "123456", "12345678", "123456789", "1234567890",
        "qwerty", "abc123", "monkey", "master", "letmein", "login", "princess", "admin",
        "welcome", "shadow", "sunshine", "trustno1", "iloveyou", "batman", "access",
        "hello", "charlie", "donald", "password1234", "test", "love", "god",
        "bitcoin", "btc", "satoshi", "nakamoto", "blockchain", "crypto", "wallet",
        "money", "rich", "fortune", "lucky", "winner", "jackpot", "diamond",
        "freedom", "peace", "hope", "dream", "star", "moon", "sun",
        "dragon", "penguin", "football", "baseball", "soccer", "hockey",
        "mustang", "tiger", "bear", "wolf", "eagle", "hawk", "falcon",
        "thunder", "lightning", "storm", "fire", "ice", "water", "earth",
        "a1b2c3", "aa11bb22", "zzzzzz", "xxxxxx", "qqqqqq", "wwwwww",
        "aaa111", "bbb222", "ccc333", "111111", "222222", "333333", "444444", "555555",
        "abcdef", "abcdefg", "abcdef123", "abc123456", "qwerty123", "asdfgh", "zxcvbn",
        "pass", "pass1", "pass123", "passwd", "secret", "secret123",
        "changeme", "default", "guest", "user", "root", "toor",
        "1q2w3e4r", "1q2w3e", "qwe123", "q1w2e3", "1qaz2wsx",
        "zaq1xsw2", "qazwsx", "asdf1234", "zxcv1234",
        "letmein123", "welcome1", "admin123", "root123",
        "bitcoin2009", "bitcoin2024", "bitcoin2025", "bitcoin2026",
        "satoshi2009", "nakamoto2009", "genesis2009",
        "1jan2009", "3jan2009", "9jan2009", // Bitcoin genesis date
        "magic", "cookies", "horse", "stapler", "correct", "battery",
    ];
    for pw in &passwords {
        // SHA256 of password
        let key = sha256_key(pw);
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;

        // SHA256 of password with common suffixes
        for suffix in &["", "1", "!", "1!", "123", "!", "@", "#"] {
            let combined = format!("{}{}", pw, suffix);
            let key = sha256_key(&combined);
            writeln!(w, "{}", hex::encode(key)).ok();
            count += 1;
        }

        // Double SHA256
        let key = sha256_key(&hex::encode(sha256_key(pw)));
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;

        // MD5-like (truncated SHA256 to 16 bytes, zero-padded)
        let full = sha256_key(pw);
        let mut key = [0u8; 32];
        key[..16].copy_from_slice(&full[..16]);
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;
    }

    // ── 9. Keyboard patterns ─────────────────────────────────────────────
    eprintln!("[9/15] Keyboard patterns...");
    let kb_patterns = [
        "qwertyuiop", "asdfghjkl", "zxcvbnm",
        "qwertyuiopasdfghjklzxcvbnm",
        "1234567890", "0987654321",
        "qazwsx", "edcrfv", "tgbnhy", "ynhnbg", "ujmik,",
        "poiuytrewq", "lkjhgfdsa", "mnbvcxz",
        "1qaz2wsx3edc", "4rfv5tgb6yhn", "7ujm8ik9ol0p",
        "qweasdzxc", "rfvtgbyhn", "ujmik,ol.",
        "zaq1xsw2cde3", "rfv4tgb5yhn6", "ujm7ik8ol9p0",
        "aaaaa", "bbbbb", "ccccc", "dddd", "eeee",
        "qwer", "asdf", "zxcv", "rtyu", "fghj", "vbnm",
        "1111", "2222", "3333", "4444", "5555", "6666", "7777", "8888", "9999", "0000",
        "1212", "2323", "3434", "4545", "5656", "6767", "7878", "8989", "9090",
        "abcd", "bcde", "cdef", "defg", "efgh", "fghi", "ghij",
        "dcba", "edcb", "fedc", "gfed", "hgfe", "ihgf", "jihg",
    ];
    for pat in &kb_patterns {
        let key = sha256_key(pat);
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;
        // Also raw
        let mut key = [0u8; 32];
        for (i, b) in pat.as_bytes().iter().enumerate() {
            if i < 32 { key[i] = *b; }
        }
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;
    }

    // ── 10. Repetitive patterns ──────────────────────────────────────────
    eprintln!("[10/15] Repetitive patterns...");
    // Single byte repeated
    for byte in 1..=255u8 {
        let key = [byte; 32];
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;
    }
    // Two-byte patterns
    for a in 0..=255u8 {
        for b in 0..=255u8 {
            let mut key = [0u8; 32];
            for i in (0..32).step_by(2) {
                key[i] = a;
                key[i + 1] = b;
            }
            writeln!(w, "{}", hex::encode(key)).ok();
            count += 1;
        }
    }
    // Incrementing patterns
    for start in 0..=254u8 {
        let mut key = [0u8; 32];
        for i in 0..32usize {
            key[i] = (start as u16 + i as u16) as u8;
        }
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;
    }
    // Decreasing patterns
    for start in 1..=255u8 {
        let mut key = [0u8; 32];
        for i in 0..32usize {
            key[i] = (start as i16 - i as i16) as u8;
        }
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;
    }

    // ── 11. Phone number patterns ────────────────────────────────────────
    eprintln!("[11/15] Phone number patterns...");
    // US phone numbers: NPA-NXX-XXXX (where N is 2-9, X is 0-9)
    // Common area codes
    let area_codes = [201,202,203,205,206,207,208,209,210,212,213,214,215,216,217,218,219,
        22,224,225,227,228,229,23,234,239,240,248,251,252,253,254,256,
        301,302,303,304,305,307,308,309,310,312,313,314,315,316,317,318,319,
        32,320,321,323,325,327,330,331,334,336,337,339,341,347,351,352,
        401,402,404,405,406,407,408,409,410,412,413,414,415,417,419,
        423,424,425,430,432,434,435,440,442,443,445,447,458,463,469,470,475,478,479,480,484,
        501,502,503,504,505,507,508,509,510,512,513,515,516,517,518,520,530,531,534,539,
        540,541,551,557,559,561,562,563,564,567,570,571,573,574,575,580,582,585,586,
        601,602,603,605,606,607,608,609,610,612,614,615,616,617,618,619,
        62,623,626,628,629,630,631,636,641,646,650,651,657,660,661,662,667,669,
        701,702,703,704,706,707,708,712,713,714,715,716,717,718,719,
        72,720,724,725,726,727,731,732,734,737,740,743,747,754,757,760,762,763,765,769,
        770,772,773,774,775,779,781,785,786,801,802,803,804,805,806,808,810,812,813,814,815,816,817,818,
        828,830,831,832,843,845,847,848,850,856,857,858,859,860,862,863,864,865,
        901,903,904,906,907,908,909,910,912,913,914,915,916,917,918,919,920,925,928,929,930,931,936,937,940,941,947,949,951,952,954,956,959,970,971,972,973,975,978,979,980,984,985,989];
    for area in &area_codes {
        // Common exchanges (001-999, excluding 000, 111, 911, 999)
        for exchange in 1..=999 {
            if exchange == 111 || exchange == 911 || exchange == 999 { continue; }
            // Last 4 digits: 0000-9999 (just test 0000 for each combo)
            for last4 in [0, 1, 10, 100, 1000, 1111, 1234, 2222, 3333, 4444, 5555, 6666, 7777, 8888, 9999] {
                let phone_str = format!("{:03}{:03}{:04}", area, exchange, last4);
                let key = sha256_key(&phone_str);
                writeln!(w, "{}", hex::encode(key)).ok();
                count += 1;
            }
        }
    }

    // ── 12. Lucky numbers ────────────────────────────────────────────────
    eprintln!("[12/15] Lucky numbers...");
    let lucky = [3, 7, 8, 9, 13, 17, 21, 33, 37, 42, 44, 47, 48, 66, 69, 77, 78, 79, 88, 89, 99,
        100, 108, 123, 137, 144, 169, 193, 200, 216, 256, 333, 360, 365, 400, 404, 500, 512,
        555, 600, 617, 666, 667, 693, 700, 707, 711, 729, 733, 777, 778, 800, 808, 888, 889,
        900, 909, 911, 999, 1000, 1024, 1089, 1234, 1337, 1440, 1492, 1500, 1600, 1701,
        1776, 1812, 1911, 1914, 1918, 1929, 1945, 1969, 1984, 1989, 1991, 1999, 2000, 2001,
        2020, 2024, 2025, 2026, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10000, 20000, 100000];
    for &n in &lucky {
        let mut key = [0u8; 32];
        key[24..28].copy_from_slice(&(n as u32).to_le_bytes());
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;
        // SHA256 of the number as string
        let key = sha256_key(&n.to_string());
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;
        // Repeated pattern
        let mut key = [0u8; 32];
        let bytes = (n as u32).to_le_bytes();
        for chunk in key.chunks_mut(4) {
            chunk.copy_from_slice(&bytes);
        }
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;
    }

    // ── 13. Pi/e/sqrt combinations ───────────────────────────────────────
    eprintln!("[13/15] Pi/e/sqrt combinations...");
    // Concatenate pi + e digits
    let pi_e = format!("{}{}", &pi_digits[..500.min(pi_digits.len())], &e_digits[..500.min(e_digits.len())]);
    for window in [32usize, 64, 128, 256] {
        if window > pi_e.len() { continue; }
        for start in 0..=pi_e.len() - window {
            let chunk = &pi_e[start..start + window];
            let key = digits_to_key(chunk, false);
            writeln!(w, "{}", hex::encode(key)).ok();
            count += 1;
        }
    }
    // Pi * e approximation
    let pi_e_product = std::f64::consts::PI * std::f64::consts::E;
    let pi_e_str = format!("{:.100}", pi_e_product);
    let after_dot = pi_e_str.split('.').nth(1).unwrap_or("");
    for window in [32usize, 64, 128, 256] {
        if window > after_dot.len() { continue; }
        for start in 0..=after_dot.len() - window {
            let chunk = &after_dot[start..start + window];
            let key = digits_to_key(chunk, false);
            writeln!(w, "{}", hex::encode(key)).ok();
            count += 1;
        }
    }

    // ── 14. Mathematical expressions ─────────────────────────────────────
    eprintln!("[14/15] Mathematical expressions...");
    let expressions = [
        "pi*e", "pi+e", "pi-e", "pi/e", "e-pi", "e/pi",
        "2*pi", "2*e", "pi^2", "e^2", "pi^e", "e^pi",
        "sqrt(2)", "sqrt(3)", "sqrt(5)", "sqrt(6)", "sqrt(7)", "sqrt(10)",
        "ln(2)", "ln(3)", "ln(10)", "log(2)", "log(10)",
        "2^10", "2^16", "2^32", "2^64", "2^128", "2^256",
        "10^100", "googol", "googolplex",
        "1/3", "1/7", "1/13", "1/17", "1/19", "1/23",
        "phi^2", "phi^3", "1/phi", "phi-1",
        "e^(i*pi)", "bernoulli", "catalan", "glacier",
        "1+1", "2+2", "1+2+3", "1*2*3", "0+0",
        "42", "the answer", "meaning of life",
        "0x0", "0x1", "0xFF", "0xFFFF", "0xFFFFFFFF",
        "NULL", "nullptr", "nil", "none", "undefined", "nan", "inf", "-inf",
    ];
    for expr in &expressions {
        let key = sha256_key(expr);
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;
    }

    // ── 15. Famous numbers ───────────────────────────────────────────────
    eprintln!("[15/15] Famous numbers...");
    let famous = [
        (42u64, "Answer to everything"),
        (666, "Number of the beast"),
        (137, "Fine structure constant (approx)"),
        (1729, "Hardy-Ramanujan number"),
        (163, "Heegner number"),
        (163, "Largest Heegner number"),
        (271828, "e * 100000"),
        (314159, "pi * 100000"),
        (141421, "sqrt(2) * 100000"),
        (231406, "sqrt(3) * 100000 (approx)"),
        (618033, "phi * 100000"),
        (12345, "Sequential"),
        (54321, "Reverse sequential"),
        (11111, "All ones"),
        (22222, "All twos"),
        (8675309, "Jenny"),
        (7777777, "Lucky sevens"),
        (123456789, "Sequential 9"),
        (987654321, "Reverse 9"),
        (314159265, "pi extended"),
        (271828182, "e extended"),
        (141421356, "sqrt(2) extended"),
        (161803398, "phi extended"),
        (1000000000, "Billion"),
        (1000000000000u64, "Trillion"),
    ];
    for &(n, _desc) in &famous {
        let mut key = [0u8; 32];
        key[24..32].copy_from_slice(&n.to_le_bytes());
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;
        let key = sha256_key(&n.to_string());
        writeln!(w, "{}", hex::encode(key)).ok();
        count += 1;
    }

    // Apply transforms to all generated keys
    eprintln!("Applying transforms...");
    // (transforms are applied by the scanner, not here — keep it simple)

    w.flush().ok();
    eprintln!("Done! Generated {} keys → {}", count, output);
}

/// Convert decimal digit string to a 256-bit key
/// If exact_32 is true and string is exactly 32 chars, use directly as hex-ish bytes
/// Otherwise, SHA256 hash the string
fn digits_to_key(digits: &str, exact_32: bool) -> [u8; 32] {
    if exact_32 && digits.len() == 32 {
        // Convert each pair of digits to a byte (00-99 → 0x00-0x63)
        let mut key = [0u8; 16]; // 32 digits = 16 bytes (2 digits each)
        for (i, chunk) in digits.as_bytes().chunks(2).enumerate() {
            if i < 16 {
                let high = (chunk[0] - b'0') as u8;
                let low = (chunk[1] - b'0') as u8;
                key[i] = high * 10 + low;
            }
        }
        // Pad to 32 bytes with SHA256 of the 16 bytes
        let hash = Sha256::digest(&key);
        let mut full = [0u8; 32];
        full[..16].copy_from_slice(&key);
        full[16..].copy_from_slice(&hash[..16]);
        full
    } else if digits.len() >= 32 {
        // Take first 64 hex chars (32 bytes) from the digit string interpreted as decimal
        // Convert decimal string to bytes using SHA256
        sha256_key(digits)
    } else {
        // Short string: pad and hash
        sha256_key(digits)
    }
}

/// SHA256 hash of a string, returned as 32 bytes
fn sha256_key(input: &str) -> [u8; 32] {
    let h = Sha256::digest(input);
    let mut key = [0u8; 32];
    key.copy_from_slice(&h);
    key
}
