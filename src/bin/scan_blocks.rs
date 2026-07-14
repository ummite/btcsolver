use anyhow::Result;
use bitcoin::blockdata::block::Block;
use bitcoin::key::{CompressedPublicKey, UntweakedPublicKey};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::{Address, Network, ScriptBuf, Txid};
use bip39::Mnemonic;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::str::FromStr;
use std::time::Instant;

type ScriptKey = Vec<u8>;
type UtxoEntry = (Txid, u32, u64); // txid, vout, value_in_sats

fn main() -> Result<()> {
    let blocks_dir = Path::new("Y:\\Bitcoin\\blocks");
    let obf_key = [0xb3, 0xa2, 0xcd, 0x52, 0x2d, 0xf3, 0xa0, 0x49]; // BE bytes

    eprintln!("🔍 BTCSolver - Scanner de blocs direct (sans bitcoind)");
    eprintln!("   Blocks dir: {}", blocks_dir.display());
    eprintln!("   Obfuscation key: b3a2cd522df3a049\n");

    // Collect target scripts from the 128 valid phrases
    let target_scripts = get_target_scripts()?;
    let target_set: HashMap<ScriptKey, String> = target_scripts
        .iter()
        .map(|(s, l)| (s.as_bytes().to_vec(), l.clone()))
        .collect();

    eprintln!("📋 {} scripts cibles ({} phrases × 4 types)\n", target_scripts.len(), target_scripts.len() / 4);

    // UTXO set: script_bytes → list of (txid, vout, value)
    let mut utxo_map: HashMap<ScriptKey, Vec<UtxoEntry>> = HashMap::new();
    // Track target outpoints for spend detection: (txid, vout) → script_bytes
    let mut target_outpoints: HashMap<(Txid, u32), ScriptKey> = HashMap::new();

    let start = Instant::now();
    let mut total_blocks = 0u64;
    let mut total_txs = 0u64;

    // Find and sort block files
    let mut block_files: Vec<_> = std::fs::read_dir(blocks_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let binding = e.file_name();
            let name = binding.to_string_lossy();
            name.starts_with("blk") && name.ends_with(".dat")
        })
        .map(|e| e.path())
        .collect();
    block_files.sort();

    eprintln!("📁 {} fichiers de blocs (~472 Go)", block_files.len());
    eprintln!("⏱️  Démarrage du scan...\n");

    for (file_idx, block_file) in block_files.iter().enumerate() {
        if file_idx % 50 == 0 {
            let _elapsed = start.elapsed();
            let target_utxos: usize = utxo_map.values().map(|v| v.len()).sum();
            eprintln!("[{}/{}] {} blocs | {} txs | {} UTXOs cibles | {:?} restant",
                file_idx, block_files.len(), total_blocks, total_txs, target_utxos,
                start.elapsed());
        }

        scan_block_file(block_file, obf_key, &target_set, &target_scripts,
            &mut utxo_map, &mut target_outpoints,
            &mut total_blocks, &mut total_txs)?;
    }

    let elapsed = start.elapsed();
    eprintln!("\n\n✅ Scan terminé en {}!", format_duration(elapsed));
    eprintln!("   Blocs: {} | Transactions: {}\n", total_blocks, total_txs);

    // Display results
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!("💰  RESULTATS");
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut any_balance = false;
    for (script, label) in &target_scripts {
        if let Some(utxos) = utxo_map.get(script.as_bytes()) {
            if !utxos.is_empty() {
                let total_sats: u64 = utxos.iter().map(|(_, _, v)| v).sum();
                let btc = total_sats as f64 / 100_000_000.0;
                eprintln!("  ✅ {} ", label);
                eprintln!("     UTXOs: {} | {} sats ({:.8} BTC)\n", utxos.len(), total_sats, btc);
                any_balance = true;
            }
        }
    }

    if !any_balance {
        eprintln!("  ❌ Aucun solde trouvé pour les 128 phrases.\n");
    }

    // Save UTXO cache for future incremental updates
    let cache_path = "utxo-cache.bin";
    eprintln!("💾 Sauvegarde de la cache UTXO dans {}...", cache_path);
    save_cache(cache_path, &utxo_map, &target_scripts, total_blocks)?;
    eprintln!("   ✅ Cache sauvegardée ({:?})", start.elapsed());
    eprintln!("\n   📌 Prochaine update: scan_blocks.exe --update");
    eprintln!("   📌 Vérifier solde: btcsolver.exe balance --key \"...\" --index utxo-cache.bin\n");

    Ok(())
}

fn scan_block_file(
    path: &Path,
    obf_key: [u8; 8],
    _target_set: &HashMap<ScriptKey, String>,
    target_scripts: &[(ScriptBuf, String)],
    utxo_map: &mut HashMap<ScriptKey, Vec<UtxoEntry>>,
    target_outpoints: &mut HashMap<(Txid, u32), ScriptKey>,
    total_blocks: &mut u64,
    total_txs: &mut u64,
) -> Result<()> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    // Deobfuscate in-place (BE key bytes)
    for (i, b) in buf.iter_mut().enumerate() {
        *b ^= obf_key[i % 8];
    }

    let data = buf;
    let mut offset = 0usize;

    while offset + 8 <= data.len() {
        // Magic number (4 bytes) + block size (4 bytes)
        let magic = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]);

        if magic != 0xd9b4bef9 {
            offset += 1;
            continue;
        }

        let block_size = u32::from_le_bytes([
            data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
        ]) as usize;

        if offset + 8 + block_size > data.len() {
            break;
        }

        let block_data = &data[offset + 8..offset + 8 + block_size];

        if let Ok(block) = bitcoin::consensus::encode::deserialize::<Block>(block_data) {
            *total_blocks += 1;

            for tx in block.txdata.iter() {
                *total_txs += 1;
                let txid = tx.compute_txid();

                // 1. Check if any input spends a target UTXO
                for input in tx.input.iter() {
                    let prev = &input.previous_output;
                    if let Some(script_key) = target_outpoints.remove(&(prev.txid, prev.vout)) {
                        // Remove from UTXO map
                        if let Some(utxos) = utxo_map.get_mut(&script_key) {
                            utxos.retain(|(tid, vout, _)| (*tid, *vout) != (prev.txid, prev.vout));
                            if utxos.is_empty() {
                                utxo_map.remove(&script_key);
                            }
                        }
                    }
                }

                // 2. Add new outputs that match target scripts
                for (vout_idx, txout) in tx.output.iter().enumerate() {
                    let script_bytes = txout.script_pubkey.as_bytes().to_vec();

                    for (target_script, _label) in target_scripts.iter() {
                        if script_bytes == target_script.as_bytes() {
                            utxo_map
                                .entry(script_bytes.clone())
                                .or_default()
                                .push((txid, vout_idx as u32, txout.value.to_sat()));

                            target_outpoints.insert((txid, vout_idx as u32), script_bytes);
                            break;
                        }
                    }
                }
            }
        }

        offset += 8 + block_size;
    }

    Ok(())
}

fn get_target_scripts() -> Result<Vec<(ScriptBuf, String)>> {
    let network = Network::Bitcoin;
    let secp = Secp256k1::new();
    let mut scripts = Vec::new();

    let partial = "zoo zone zoo zone zoo zone zoo zone zoo zone zoo";
    let wordlist: Vec<String> = std::fs::read_to_string("bip39-words.txt")?
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    for word in &wordlist {
        let phrase = format!("{} {}", partial, word);
        if let Ok(mnemonic) = Mnemonic::parse(&phrase) {
            if mnemonic.words().count() == 12 {
                let seed: [u8; 64] = {
                    let salt = "mnemonic";
                    let mut out = [0u8; 64];
                    pbkdf2::pbkdf2_hmac::<sha2::Sha512>(
                        phrase.as_bytes(), salt.as_bytes(), 2048, &mut out,
                    );
                    out
                };

                let xpriv = Xpriv::new_master(network, &seed)?;
                let path = DerivationPath::from_str("m/44'/0'/0'/0/0")?;
                let derived = xpriv.derive_priv(&secp, &path)?;

                let pk = bitcoin::PrivateKey {
                    inner: derived.private_key,
                    network: network.into(),
                    compressed: true,
                };

                let pubkey = pk.public_key(&secp);
                let compressed = CompressedPublicKey::from_private_key(&secp, &pk)?;
                let xonly: UntweakedPublicKey = compressed.into();

                let p2pkh = Address::p2pkh(pubkey, network);
                let p2wpkh = Address::p2wpkh(&compressed, network);
                let p2sh_wpkh = Address::p2shwpkh(&compressed, network);
                let p2tr = Address::p2tr(&secp, xonly, None, network);

                let lb = format!("zoo zone...{}", word);
                scripts.push((p2pkh.script_pubkey(), format!("{} [legacy]", lb)));
                scripts.push((p2wpkh.script_pubkey(), format!("{} [segwit]", lb)));
                scripts.push((p2sh_wpkh.script_pubkey(), format!("{} [wrapped]", lb)));
                scripts.push((p2tr.script_pubkey(), format!("{} [taproot]", lb)));
            }
        }
    }

    Ok(scripts)
}

fn format_duration(d: std::time::Duration) -> String {
    let h = d.as_secs() / 3600;
    let m = (d.as_secs() % 3600) / 60;
    let s = d.as_secs() % 60;
    format!("{}h {}m {}s", h, m, s)
}

/// Save UTXO cache to a binary file for future incremental updates
fn save_cache(
    path: &str,
    utxo_map: &HashMap<ScriptKey, Vec<UtxoEntry>>,
    _target_scripts: &[(ScriptBuf, String)],
    total_blocks: u64,
) -> Result<()> {
    use std::io::Write;

    let mut file = File::create(path)?;

    // Header: magic + version + total_blocks + num_scripts + num_utxo_entries
    file.write_all(&0x55545842u32.to_le_bytes())?; // "UTXB" magic
    file.write_all(&1u32.to_le_bytes())?; // version
    file.write_all(&total_blocks.to_le_bytes())?;

    // Save scripts
    let num_scripts = _target_scripts.len() as u32;
    file.write_all(&num_scripts.to_le_bytes())?;
    for (script, label) in _target_scripts {
        let sb = script.as_bytes();
        file.write_all(&(sb.len() as u16).to_le_bytes())?;
        file.write_all(sb)?;
        let lb = label.as_bytes();
        file.write_all(&(lb.len() as u32).to_le_bytes())?;
        file.write_all(lb)?;
    }

    // Save UTXO entries
    let total_entries: u32 = utxo_map.values().map(|v| v.len() as u32).sum();
    file.write_all(&total_entries.to_le_bytes())?;
    for (script, entries) in utxo_map {
        file.write_all(&(script.len() as u16).to_le_bytes())?;
        file.write_all(script)?;
        file.write_all(&(entries.len() as u32).to_le_bytes())?;
        for (txid, vout, value) in entries {
            file.write_all(txid.as_ref())?;
            file.write_all(&vout.to_le_bytes())?;
            file.write_all(&value.to_le_bytes())?;
        }
    }

    eprintln!("   {} scripts, {} UTXO entries sauvegardés", num_scripts, total_entries);
    Ok(())
}
