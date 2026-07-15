//! Debug: examine the UTXO read bug in lookup()
//!
//! The UTXO layout is: [txid:32][vout:4][value:8] = 44 bytes
//!
//! lookup() reads: chunk[32..40]  →  vout(4) + value_low(4)  ← BUG when vout != 0 !
//! Correct read:  chunk[36..44]  →  value(8)

use anyhow::Result;
use bitcoin::Address;
use clap::Parser;
use std::str::FromStr;

mod flat_index;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "utxo-index.snapshot")]
    snapshot: String,

    #[arg(long)]
    address: String,

    #[arg(long, default_value = "0")]
    min_value: u64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let addr = Address::from_str(&cli.address)?.assume_checked();
    let script_buf = addr.script_pubkey();
    let script = script_buf.as_bytes();

    println!("Address: {}", cli.address);
    println!("Script (hex): {} ({} bytes)\n", hex::encode(script), script.len());

    println!("Loading UTXO index...");
    let index = flat_index::FlatIndex::load_from_snapshot(&cli.snapshot, cli.min_value)?;
    index.print_stats();

    // Binary search
    let mut lo = 0usize;
    let mut hi = index.script_entries.len();
    let mut found_idx = None;

    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let entry = index.script_entries[mid];
        let s_end = entry.script_offset as usize + entry.script_len as usize;
        let s = &index.all_data[entry.script_offset as usize..s_end];
        match s.cmp(script) {
            std::cmp::Ordering::Less => lo = mid + 1,
            std::cmp::Ordering::Greater => hi = mid,
            std::cmp::Ordering::Equal => {
                found_idx = Some(mid);
                break;
            }
        }
    }

    let idx = match found_idx {
        Some(i) => i,
        None => {
            println!("\nScript NOT FOUND in index");
            return Ok(());
        }
    };

    let entry = index.script_entries[idx];
    println!("\nFound at index {}", idx);
    println!("  utxo_offset: {}", entry.utxo_offset);
    println!("  utxo_count: {}", entry.utxo_count);

    // Read each UTXO byte by byte
    let u_start = entry.utxo_offset as usize;
    for i in 0..entry.utxo_count.min(10) {
        let base = u_start + (i as usize) * 44;
        let data = &index.utxo_data[base..base+44];

        let txid = hex::encode(&data[0..32]);
        let vout_bytes: [u8; 4] = data[32..36].try_into().unwrap();
        let vout = u32::from_le_bytes(vout_bytes);
        let value_bytes: [u8; 8] = data[36..44].try_into().unwrap();
        let value_correct = u64::from_le_bytes(value_bytes);

        // What lookup() ACTUALLY reads (chunk[32..40])
        let buggy_bytes: [u8; 8] = data[32..40].try_into().unwrap();
        let value_buggy = u64::from_le_bytes(buggy_bytes);

        println!("\n  UTXO #{} (base offset {}):", i, base);
        println!("    txid:  {}", &txid[..24]);
        println!("    vout:  {} (bytes: {})", vout, hex::encode(&data[32..36]));
        println!("    value (correct, [36..44]): {} sats ({:.8} BTC)", value_correct, value_correct as f64 / 1e8);
        println!("    value (buggy,   [32..40]): {} sats ({:.8} BTC)  ← ce que lookup() lit", value_buggy, value_buggy as f64 / 1e8);

        if value_buggy != value_correct {
            println!("    ⚠️  MISMATCH! Ratio: {}", value_buggy / value_correct.max(1));
            println!("    ⚠️  Difference: {} sats", value_buggy.saturating_sub(value_correct));
        }

        // Show raw bytes for clarity
        println!("    Raw bytes [32..44]: {}", hex::encode(&data[32..44]));
        println!("                      ^^vout^^  ^^^^^^^^^^^^^value^^^^^^^^^^^");
    }

    // Compare with lookup() result
    let lookup_result = index.lookup(script);
    println!("\n\n  lookup() retourne: {} sats ({:.8} BTC)", lookup_result, lookup_result as f64 / 1e8);

    Ok(())
}
