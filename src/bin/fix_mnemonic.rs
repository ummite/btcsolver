use anyhow::{Context, Result};
use bip39::Mnemonic;
use std::process::Command;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 11 {
        eprintln!("Usage: fix_mnemonic <mot1> <mot2> ... <mot11>");
        eprintln!("Calcule le 12e mot valide et teste le solde avec BTCSolver");
        std::process::exit(1);
    }

    let partial = args.join(" ");
    eprintln!("📝 {0} mots fournis: {1}", args.len(), partial);
    eprintln!("\n🔍 Recherche du mot valide parmi 2048...\n");

    // Load BIP39 English wordlist
    let wordlist: Vec<String> = std::fs::read_to_string("bip39-words.txt")
        .context("Fichier bip39-words.txt introuvable")?
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    eprintln!("   {} mots chargés", wordlist.len());

    // Try each word as the 12th word
    let mut found = Vec::new();
    for (i, word) in wordlist.iter().enumerate() {
        let phrase = format!("{} {}", partial, word);

        if let Ok(mnemonic) = Mnemonic::parse(&phrase) {
            // Verify it's valid
            let words_count: usize = mnemonic.words().count();
            if words_count == 12 {
                found.push((i, word.clone()));
            }
        }
    }

    eprintln!("\n✅ {} phrase(s) valide(s) trouvée(s):\n", found.len());

    for (idx, (word_idx, word)) in found.iter().enumerate() {
        let phrase = format!("{} {}", partial, word);
        eprintln!("   [{}] {} → \"{}\"", idx + 1, phrase, word);
    }

    // For each valid phrase, derive addresses and try balance
    for (idx, (_word_idx, word)) in found.iter().enumerate() {
        let phrase = format!("{} {}", partial, word);

        eprintln!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("🔑 Phrase #{}: \"{}\"", idx + 1, phrase);
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // Run BTCSolver to derive addresses
        let output = Command::new("./target/release/btcsolver.exe")
            .args(["balance", "--key", &phrase, "--derive-only", "--show-all"])
            .output()
            .context("Impossible de lancer BTCSolver")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        print!("{}", stdout);
        if !stderr.is_empty() {
            print!("{}", stderr);
        }

        eprintln!("\n💰 Pour scanner le solde réel, il faut un nœud Bitcoin ou un index UTXO:");
        eprintln!("   ./btcsolver.exe balance --key \"{}\" --cookie-file Y:\\Bitcoin\\.cookie --sats", phrase);
    }

    Ok(())
}
