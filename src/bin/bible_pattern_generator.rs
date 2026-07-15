//! Bible Pattern Generator — generates Bible-derived patterns for brainwallet testing
//!
//! Patterns generated:
//!   1. Each phrase: original, lowercase, uppercase, titlecase
//!   2. With/without punctuation, with/without spaces, underscores, hyphens
//!   3. First letter of each phrase (English + French) — build words
//!   4. First word and last word of each phrase
//!   5. First word + last word combined
//!   6. Multi-word combinations (1 to N consecutive words)
//!   7. Cross-language combinations (English phrase + French translation)
//!   8. ASCII-based patterns (hex, byte sum, XOR, ROT13, reversed)
//!   9. Brainwallet suffixes/prefixes on core text patterns only
//!  10. Leet-speak on core text patterns only
//!  11. Number patterns extracted from phrases
//!  12. Reverse word order
//!
//! Usage:
//!   bible_pattern_generator --english bible-english.txt --french bible-french.txt --output bible-patterns-all.txt

use anyhow::Result;
use clap::Parser;
use std::collections::BTreeSet;

#[derive(Parser)]
struct Cli {
    /// Path to English Bible phrases file
    #[arg(short, long)]
    english: String,

    /// Path to French Bible phrases file
    #[arg(short, long)]
    french: String,

    /// Optional: additional custom phrases file
    #[arg(short, long)]
    custom: Option<String>,

    /// Output file for all generated patterns
    #[arg(long, default_value = "bible-patterns-all.txt")]
    output: String,

    /// Generate first-letter acronyms of up to this many chars (default 12)
    #[arg(long, default_value = "12")]
    max_acronym_len: usize,

    /// Generate word combinations up to N words (default 3)
    #[arg(long, default_value = "3")]
    max_word_combo: usize,

    /// Skip brainwallet suffixes (reduces pattern count significantly)
    #[arg(long)]
    no_suffixes: bool,

    /// Skip leet-speak variants
    #[arg(long)]
    no_leet: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load source texts
    let en_phrases = load_phrases(&cli.english)?;
    let fr_phrases = load_phrases(&cli.french)?;
    let mut custom_phrases = Vec::new();
    if let Some(path) = &cli.custom {
        custom_phrases = load_phrases(path)?;
    }

    println!("Loaded {} English phrases", en_phrases.len());
    println!("Loaded {} French phrases", fr_phrases.len());
    println!("Loaded {} custom phrases", custom_phrases.len());

    let mut all_patterns = BTreeSet::new();

    // === PHRASE-LEVEL patterns: basic text transformations of original phrases ===
    // These are the ONLY patterns that get suffixes/leet applied
    let mut phrase_level = BTreeSet::new();

    println!("\n[1/12] Generating basic text transformations...");
    for phrase in en_phrases.iter().chain(fr_phrases.iter()).chain(custom_phrases.iter()) {
        let variants = generate_basic_variations(phrase);
        phrase_level.extend(variants);
    }
    println!("  -> {} phrase-level patterns", phrase_level.len());

    // === DERIVED patterns: acronyms, word combos, etc. (NO suffixes/leet) ===
    let mut derived_patterns = BTreeSet::new();

    // === 2. First letter of each phrase — build words (English) ===
    println!("[2/12] Generating first-letter acronyms (English)...");
    derived_patterns.extend(generate_first_letter_words(&en_phrases, cli.max_acronym_len));

    // === 3. First letter of each phrase — build words (French) ===
    println!("[3/12] Generating first-letter acronyms (French)...");
    derived_patterns.extend(generate_first_letter_words(&fr_phrases, cli.max_acronym_len));

    // === 4. First word of each phrase ===
    println!("[4/12] Extracting first words...");
    for phrase in en_phrases.iter().chain(fr_phrases.iter()) {
        if let Some(first) = extract_first_word(phrase) {
            derived_patterns.extend(generate_basic_variations(&first));
        }
    }

    // === 5. Last word of each phrase ===
    println!("[5/12] Extracting last words...");
    for phrase in en_phrases.iter().chain(fr_phrases.iter()) {
        if let Some(last) = extract_last_word(phrase) {
            derived_patterns.extend(generate_basic_variations(&last));
        }
    }

    // === 6. First word + Last word combinations ===
    println!("[6/12] Generating first+last word combos...");
    for phrase in en_phrases.iter().chain(fr_phrases.iter()) {
        if let Some(first) = extract_first_word(phrase) {
            if let Some(last) = extract_last_word(phrase) {
                derived_patterns.extend(generate_basic_variations(&format!("{}{}", first, last)));
                derived_patterns.extend(generate_basic_variations(&format!("{} {}", first, last)));
            }
        }
    }

    // === 7. Multi-word combinations (first N words) ===
    println!("[7/12] Generating multi-word combinations...");
    for phrase in en_phrases.iter().chain(fr_phrases.iter()) {
        let words: Vec<String> = phrase.split_whitespace()
            .map(|w| clean_word(w))
            .filter(|w| !w.is_empty())
            .collect();
        for n in 1..=cli.max_word_combo.min(words.len()) {
            let combo: String = words[..n].join(" ");
            derived_patterns.extend(generate_basic_variations(&combo));
        }
    }

    // === 8. Cross-language combinations ===
    println!("[8/12] Generating cross-language combinations...");
    let min_len = en_phrases.len().min(fr_phrases.len());
    for i in 0..min_len {
        let en = &en_phrases[i];
        let fr = &fr_phrases[i];
        derived_patterns.extend(generate_basic_variations(&format!("{}{}", en, fr)));
        derived_patterns.extend(generate_basic_variations(&format!("{} {}", en, fr)));
        if let Some(en_first) = extract_first_word(en) {
            if let Some(fr_first) = extract_first_word(fr) {
                derived_patterns.extend(generate_basic_variations(&format!("{}{}", en_first, fr_first)));
            }
        }
    }

    // === 9. Reverse word order ===
    println!("[9/12] Generating reverse word order patterns...");
    for phrase in en_phrases.iter().chain(fr_phrases.iter()) {
        let words: Vec<&str> = phrase.split_whitespace().collect();
        if words.len() > 1 {
            let rev_words: String = words.iter().rev().copied().collect::<Vec<_>>().join(" ");
            derived_patterns.extend(generate_basic_variations(&rev_words));
        }
    }
    println!("  -> {} derived patterns (acronyms, words, combos, etc.)", derived_patterns.len());

    // === 10. Brainwallet suffixes/prefixes — ONLY on phrase-level patterns ===
    if !cli.no_suffixes {
        println!("[10/12] Generating brainwallet suffix/prefix variants (phrase-level only)...");
        let phrase_count = phrase_level.len();
        let suffix_expansions: Vec<String> = phrase_level.iter()
            .flat_map(|p| generate_brainwallet_suffixes(p))
            .collect();
        phrase_level.extend(suffix_expansions);
        println!("  -> {} phrase-level patterns after suffixes ({:.1}x)",
            phrase_level.len(), phrase_level.len() as f64 / phrase_count as f64);
    } else {
        println!("[10/12] Skipping brainwallet suffixes (--no-suffixes)");
    }

    // === 11. Leet-speak — ONLY on phrase-level patterns ===
    if !cli.no_leet {
        println!("[11/12] Generating leet-speak variants (phrase-level only)...");
        let pre_leet = phrase_level.len();
        let phrase_clone: Vec<String> = phrase_level.iter().cloned().collect();
        for phrase in &phrase_clone {
            if let Some(leet) = to_leet_speak(phrase) {
                phrase_level.insert(leet);
            }
        }
        println!("  -> {} phrase-level patterns after leet (+{})",
            phrase_level.len(), phrase_level.len() - pre_leet);
    } else {
        println!("[11/12] Skipping leet-speak (--no-leet)");
    }

    // Merge phrase-level and derived into final set
    all_patterns.extend(phrase_level);
    all_patterns.extend(derived_patterns);
    println!("\n  After merge: {} patterns", all_patterns.len());

    // === 12. ASCII-based patterns (NO suffixes/leet) ===
    println!("[12/12] Generating ASCII-based patterns...");
    let mut ascii_count = 0u64;
    for phrase in en_phrases.iter().chain(fr_phrases.iter()).chain(custom_phrases.iter()) {
        let ascii_patterns = generate_ascii_patterns(phrase);
        ascii_count += ascii_patterns.len() as u64;
        for p in ascii_patterns {
            all_patterns.insert(p);
        }
    }
    println!("  -> {} ASCII patterns added", ascii_count);

    // Number-only patterns
    for phrase in en_phrases.iter().chain(fr_phrases.iter()) {
        let nums: String = phrase.chars().filter(|c| c.is_numeric()).collect();
        if !nums.is_empty() {
            all_patterns.extend(generate_basic_variations(&nums));
        }
    }

    // Write output
    let patterns_vec: Vec<String> = all_patterns.into_iter().collect();
    let count = patterns_vec.len();
    std::fs::write(&cli.output, patterns_vec.join("\n") + "\n")?;

    println!("\n{} total unique patterns written to {}", count, cli.output);

    let en_count = en_phrases.len();
    let fr_count = fr_phrases.len();
    let expansion = if (en_count + fr_count) > 0 {
        count as f64 / (en_count + fr_count) as f64
    } else {
        0.0
    };
    println!("Expansion ratio: {:.1}x ({} source -> {} patterns)",
        expansion, en_count + fr_count, count);

    Ok(())
}

fn load_phrases(path: &str) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(path)?;
    let phrases: Vec<String> = content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    Ok(phrases)
}

/// Generate basic text variations: original, lower, upper, title, no-punct, no-space, etc.
fn generate_basic_variations(phrase: &str) -> Vec<String> {
    let mut v = Vec::new();
    let trimmed = phrase.trim().to_string();

    // Skip empty
    if trimmed.is_empty() {
        return v;
    }

    // Original
    v.push(trimmed.clone());

    // Lowercase
    let lower = trimmed.to_lowercase();
    if lower != trimmed { v.push(lower.clone()); }

    // Uppercase
    let upper = trimmed.to_uppercase();
    if upper != trimmed { v.push(upper.clone()); }

    // Title case
    let title = to_title_case(&lower);
    if title != lower && title != upper && title != trimmed { v.push(title); }

    // No punctuation
    let no_punct: String = trimmed.chars().filter(|c| !c.is_ascii_punctuation()).collect();
    if no_punct != trimmed {
        v.push(no_punct.clone());
        let no_punct_lower = no_punct.to_lowercase();
        if no_punct_lower != no_punct { v.push(no_punct_lower); }
        let no_punct_upper = no_punct.to_uppercase();
        if no_punct_upper != no_punct { v.push(no_punct_upper); }
    }

    // No spaces
    let no_spaces: String = lower.chars().filter(|c| !c.is_whitespace()).collect();
    if no_spaces != lower { v.push(no_spaces); }

    // Underscores
    let underscores = lower.replace(' ', "_");
    if underscores != lower { v.push(underscores); }

    // Hyphens
    let hyphens = lower.replace(' ', "-");
    if hyphens != lower { v.push(hyphens); }

    // Collapsed spaces
    let collapsed: String = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed != trimmed { v.push(collapsed.to_lowercase()); }

    v
}

/// Generate first-letter words from consecutive phrases
/// e.g., if phrases start with "In", "For", "The" -> "IFT", "InForThe", etc.
fn generate_first_letter_words(phrases: &[String], max_len: usize) -> Vec<String> {
    let mut result = Vec::new();

    // Get first letter of each phrase
    let first_letters: Vec<char> = phrases.iter()
        .filter_map(|p| p.trim().chars().find(|c| !c.is_whitespace()))
        .collect();

    if first_letters.is_empty() {
        return result;
    }

    // Get first word of each phrase
    let first_words: Vec<String> = phrases.iter()
        .filter_map(|p| extract_first_word(p))
        .collect();

    // Build consecutive acronyms of increasing length
    for len in 1..=max_len.min(first_letters.len()) {
        for start in 0..=first_letters.len() - len {
            // Just first letters
            let word: String = first_letters[start..start + len].iter().collect();
            result.extend(generate_basic_variations(&word));

            // First full words concatenated
            if start + len <= first_words.len() {
                let word_combo: String = first_words[start..start + len].join("");
                result.extend(generate_basic_variations(&word_combo));

                let word_combo_spaced: String = first_words[start..start + len].join(" ");
                result.extend(generate_basic_variations(&word_combo_spaced));
            }
        }
    }

    // Full acronym (all first letters, capped)
    let full_acronym: String = first_letters.iter().take(max_len).collect();
    result.extend(generate_basic_variations(&full_acronym));

    result
}

/// Extract first word from a phrase
fn extract_first_word(phrase: &str) -> Option<String> {
    phrase.split_whitespace().next().map(clean_word)
}

/// Extract last word from a phrase
fn extract_last_word(phrase: &str) -> Option<String> {
    phrase.split_whitespace().next_back().map(clean_word)
}

/// Clean a word of punctuation
fn clean_word(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_alphanumeric() || *c == '\'')
        .collect()
}

/// Generate ASCII-based patterns from a phrase
/// - Hex representation of bytes
/// - Byte sum as decimal string
/// - XOR of all bytes
/// - ASCII values as comma-separated
/// - ROT13
/// - Reversed
fn generate_ascii_patterns(phrase: &str) -> Vec<String> {
    let mut v = Vec::new();
    let bytes = phrase.as_bytes();

    if bytes.is_empty() {
        return v;
    }

    // Hex representation
    let hex_str: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    v.push(hex_str);

    // Byte sum as string
    let sum: u64 = bytes.iter().map(|&b| b as u64).sum();
    v.push(sum.to_string());

    // XOR of all bytes
    let xor = bytes.iter().fold(0u8, |acc, &b| acc ^ b);
    v.push(xor.to_string());

    // ASCII values as comma-separated
    let ascii_vals: String = bytes.iter().map(|&b| b.to_string()).collect::<Vec<_>>().join(",");
    v.push(ascii_vals.clone());

    // ASCII values concatenated
    let ascii_concat: String = bytes.iter().map(|&b| b.to_string()).collect();
    if ascii_concat != ascii_vals {
        v.push(ascii_concat);
    }

    // "ASCII:" prefix
    v.push(format!("ASCII:{}", phrase.trim()));

    // ROT13
    let rot13: String = phrase.chars().map(|c| {
        match c {
            'a'..='z' => ((c as u8 - b'a' + 13) % 26 + b'a') as char,
            'A'..='Z' => ((c as u8 - b'A' + 13) % 26 + b'A') as char,
            _ => c,
        }
    }).collect();
    if rot13 != phrase {
        v.push(rot13);
    }

    // Reversed
    let reversed: String = phrase.chars().rev().collect();
    if reversed != phrase {
        v.push(reversed);
    }

    v
}

/// Generate common brainwallet suffixes and prefixes
fn generate_brainwallet_suffixes(phrase: &str) -> Vec<String> {
    let mut v = Vec::new();
    let lower = phrase.to_lowercase();
    let trimmed = phrase.trim().to_string();

    // Suffixes
    for suffix in &[
        " private key", " bitcoin", " wallet", " btc", " key",
        " address", " seed", " passphrase", " brainwallet",
        " bitcoin private key", " btc wallet",
    ] {
        v.push(format!("{}{}", lower, suffix));
        v.push(format!("{}{}", trimmed, suffix));
    }

    // Prefixes
    for prefix in &[
        "my ", "the ", "bitcoin ", "my bitcoin ",
        "btc ", "wallet ", "private ",
    ] {
        v.push(format!("{}{}", prefix, lower));
    }

    // Year suffixes
    for year in 2009..=2025 {
        v.push(format!("{} {}", lower, year));
        v.push(format!("{}{}", lower, year));
    }

    // Punctuation suffixes
    v.push(format!("{}!", lower));
    v.push(format!("{}?", lower));
    v.push(format!("{}!!!", lower));

    v
}

/// Convert to leet-speak
fn to_leet_speak(s: &str) -> Option<String> {
    let leet: String = s.chars().map(|c| match c {
        'a' | 'A' => '4',
        'e' | 'E' => '3',
        'i' | 'I' => '1',
        'o' | 'O' => '0',
        't' | 'T' => '7',
        's' | 'S' => '5',
        'b' | 'B' => '8',
        _ => c,
    }).collect();
    if leet != s {
        Some(leet)
    } else {
        None
    }
}

/// Convert to title case
fn to_title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
