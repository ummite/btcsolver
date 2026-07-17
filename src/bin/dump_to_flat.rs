//! Convert Bitcoin Core dumptxoutset (.dat) → FlatIndex snapshot (BTCS v2)
//! for high-speed brainwallet / brute_force scanning.
//!
//! Usage:
//!   dump_to_flat --snapshot W:\Temp\utxo-935000.dat --output W:\Temp\utxo-day.snapshot

use anyhow::{bail, Context, Result};
use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::time::Instant;

use btcsolver::flat_index::export_flat_snapshot;

#[derive(Parser, Debug)]
#[command(name = "dump_to_flat", about = "dumptxoutset → FlatIndex snapshot")]
struct Cli {
    /// Path to dumptxoutset file (utxo-XXXXX.dat)
    #[arg(long)]
    snapshot: String,

    /// Output FlatIndex snapshot path (.snapshot)
    #[arg(long)]
    output: String,

    /// Minimum value in sats to keep (dust filter). Default: 1 (drop zero)
    #[arg(long, default_value = "1")]
    min_value: u64,
}

fn read_varint<R: Read>(reader: &mut R) -> Result<u64> {
    let mut n: u64 = 0;
    loop {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        let dat = buf[0];
        n = (n << 7) | ((dat & 0x7f) as u64);
        if (dat & 0x80) > 0 {
            n += 1;
        } else {
            return Ok(n);
        }
    }
}

fn read_compact_size<R: Read>(reader: &mut R) -> Result<u64> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    let n = buf[0] as u64;
    match n {
        253 => {
            let mut b = [0u8; 2];
            reader.read_exact(&mut b)?;
            Ok(u16::from_le_bytes(b) as u64)
        }
        254 => {
            let mut b = [0u8; 4];
            reader.read_exact(&mut b)?;
            Ok(u32::from_le_bytes(b) as u64)
        }
        255 => {
            let mut b = [0u8; 8];
            reader.read_exact(&mut b)?;
            Ok(u64::from_le_bytes(b))
        }
        _ => Ok(n),
    }
}

fn decompress_amount(x: u64) -> u64 {
    if x == 0 {
        return 0;
    }
    let mut x = x - 1;
    let e = x % 10;
    x /= 10;
    let mut n;
    if e < 9 {
        let d = (x % 9) + 1;
        x /= 9;
        n = x * 10 + d;
    } else {
        n = x + 1;
    }
    let mut e = e;
    while e > 0 {
        n *= 10;
        e -= 1;
    }
    n
}

fn decompress_script<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    let size = read_varint(reader)? as usize;
    if size == 0 {
        let mut h160 = [0u8; 20];
        reader.read_exact(&mut h160)?;
        let mut script = vec![0x76, 0xa9, 0x14];
        script.extend_from_slice(&h160);
        script.extend_from_slice(&[0x88, 0xac]);
        Ok(script)
    } else if size == 1 {
        let mut h160 = [0u8; 20];
        reader.read_exact(&mut h160)?;
        let mut script = vec![0xa9, 0x14];
        script.extend_from_slice(&h160);
        script.push(0x87);
        Ok(script)
    } else if size == 2 || size == 3 {
        let mut key = [0u8; 32];
        reader.read_exact(&mut key)?;
        let mut script = vec![33, size as u8];
        script.extend_from_slice(&key);
        script.push(0xac);
        Ok(script)
    } else if size == 4 || size == 5 {
        let mut compressed = [0u8; 33];
        compressed[0] = (size - 2) as u8;
        reader.read_exact(&mut compressed[1..])?;
        // Approximate uncompressed P2PK (rare)
        let mut script = vec![0x41, 0x04];
        script.extend_from_slice(&compressed[1..]);
        script.extend_from_slice(&[0u8; 32]);
        script.push(0xac);
        Ok(script)
    } else {
        let real_size = size - 6;
        if real_size > 10000 {
            bail!("script trop long dans snapshot");
        }
        let mut script = vec![0u8; real_size];
        reader.read_exact(&mut script)?;
        Ok(script)
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    println!("=== dump_to_flat ===");
    println!("  Input : {}", cli.snapshot);
    println!("  Output: {}", cli.output);
    println!("  min_value: {} sats", cli.min_value);

    let file = File::open(&cli.snapshot)
        .with_context(|| format!("open {}", cli.snapshot))?;
    let mut reader = BufReader::with_capacity(16 * 1024 * 1024, file);

    let mut magic = [0u8; 5];
    reader.read_exact(&mut magic)?;
    if &magic != b"utxo\xff" {
        bail!("Magic invalide (pas un dumptxoutset v2)");
    }
    let mut ver = [0u8; 2];
    reader.read_exact(&mut ver)?;
    if u16::from_le_bytes(ver) != 2 {
        bail!("Version non supportée (besoin v2)");
    }
    let mut net_magic = [0u8; 4];
    reader.read_exact(&mut net_magic)?;
    let mut block_hash = [0u8; 32];
    reader.read_exact(&mut block_hash)?;
    let mut num_buf = [0u8; 8];
    reader.read_exact(&mut num_buf)?;
    let num_utxos = u64::from_le_bytes(num_buf);

    let mut display_hash = block_hash;
    display_hash.reverse();
    let hash_hex = hex::encode(display_hash);
    println!("  Base block: {}", hash_hex);
    println!("  UTXOs: {}", num_utxos);
    println!("  https://mempool.space/block/{}", hash_hex);

    // Aggregate: script -> (total_sats, first outpoint as representative)
    let mut aggregates: HashMap<Vec<u8>, (u64, [u8; 32], u32)> = HashMap::with_capacity(60_000_000);

    let start = Instant::now();
    let mut processed: u64 = 0;
    let mut coins_per_hash_left: u64 = 0;
    let mut prevout_hash = [0u8; 32];

    while processed < num_utxos {
        if coins_per_hash_left == 0 {
            reader.read_exact(&mut prevout_hash)?;
            coins_per_hash_left = read_compact_size(&mut reader)?;
        }
        let vout = read_compact_size(&mut reader)? as u32;
        let code = read_varint(&mut reader)?;
        let _height = code >> 1;
        let compressed_amt = read_varint(&mut reader)?;
        let amount_sats = decompress_amount(compressed_amt);
        let script = decompress_script(&mut reader)?;

        if amount_sats >= cli.min_value {
            aggregates
                .entry(script)
                .and_modify(|(total, _, _)| *total = total.saturating_add(amount_sats))
                .or_insert((amount_sats, prevout_hash, vout));
        }

        coins_per_hash_left -= 1;
        processed += 1;
        if processed % 5_000_000 == 0 {
            let pct = processed as f64 / num_utxos as f64 * 100.0;
            let rate = processed as f64 / start.elapsed().as_secs_f64().max(0.001);
            println!(
                "  {processed} / {num_utxos} ({pct:.1}%) scripts={} ({:.0} UTXO/s)",
                aggregates.len(),
                rate
            );
        }
    }

    println!(
        "  Parse done in {:.1}s — {} scripts with balance >= {} sats",
        start.elapsed().as_secs_f32(),
        aggregates.len(),
        cli.min_value
    );

    // Convert to export format: script -> Vec<(txid, vout, value)>
    // One synthetic entry per script (total balance) — enough for balance lookups.
    let mut script_index: HashMap<Vec<u8>, Vec<([u8; 32], u32, u64)>> =
        HashMap::with_capacity(aggregates.len());
    let mut total_sats: u128 = 0;
    for (script, (value, txid, vout)) in aggregates {
        total_sats += value as u128;
        script_index.insert(script, vec![(txid, vout, value)]);
    }
    println!(
        "  Total value indexed: {:.8} BTC",
        total_sats as f64 / 100_000_000.0
    );

    println!("  Writing FlatIndex snapshot...");
    let exp_start = Instant::now();
    export_flat_snapshot(&script_index, &cli.output)?;
    println!(
        "  Snapshot written in {:.1}s → {}",
        exp_start.elapsed().as_secs_f32(),
        cli.output
    );

    // Sidecar meta
    let meta_path = format!("{}.meta.json", cli.output);
    let meta = serde_json::json!({
        "base_block_hash": hash_hex,
        "num_utxos_source": num_utxos,
        "num_scripts": script_index.len(),
        "min_value_sats": cli.min_value,
        "built_at": chrono::Utc::now().to_rfc3339(),
        "source": cli.snapshot,
        "mempool_space": format!("https://mempool.space/block/{}", hash_hex),
    });
    std::fs::write(&meta_path, serde_json::to_string_pretty(&meta)?)?;
    println!("  Meta: {}", meta_path);
    println!("✅ Done. Use with:");
    println!(
        "   brainwallet_scan --texts corpus.txt --snapshot {} --threads 23",
        cli.output
    );
    println!(
        "   brute_force --snapshot-path {} --use-gpu --threads 24 --random --batch-size 1024000",
        cli.output
    );
    Ok(())
}
