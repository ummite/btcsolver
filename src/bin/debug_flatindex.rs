//! Debug tool: inspect FlatIndex entries near a given script hash
//!
//! Usage:
//!   debug_flatindex --snapshot utxo-index.snapshot --address "1JJopWpJ5ZmXazk3kiisS8iVencznpnWrc"

use anyhow::Result;
use bitcoin::Address;
use clap::Parser;
use std::str::FromStr;

mod flat_index;

#[derive(Parser)]
struct Cli {
    /// Path to the UTXO index snapshot
    #[arg(short, long, default_value = "utxo-index.snapshot")]
    snapshot: String,

    /// Bitcoin address to investigate
    #[arg(long)]
    address: String,

    /// Number of nearby entries to show
    #[arg(long, default_value = "5")]
    nearby: usize,

    /// Minimum UTXO value in satoshis
    #[arg(long, default_value = "0")]
    min_value: u64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Parse address
    let addr = Address::from_str(&cli.address)?.assume_checked();
    let script_buf = addr.script_pubkey();
    let script = script_buf.as_bytes();

    println!("Address: {}", cli.address);
    println!("Script (hex): {}", hex::encode(&script));
    println!("Script length: {} bytes\n", script.len());

    // Load index
    println!("Loading UTXO index from {}...", cli.snapshot);
    let index = flat_index::FlatIndex::load_from_snapshot(&cli.snapshot, cli.min_value)?;
    index.print_stats();

    // Binary search for the script
    let result = binary_search(&index, &script);

    println!("\nBinary search result:");
    match &result {
        SearchResult::Found(idx) => {
            let entry = index.script_entries[*idx];
            println!("  FOUND at index {}", idx);
            println!("  script_offset: {}, script_len: {}", entry.script_offset, entry.script_len);
            println!("  utxo_offset: {}, utxo_count: {}", entry.utxo_offset, entry.utxo_count);

            // Show the stored script
            let s_start = entry.script_offset as usize;
            let s_end = s_start + entry.script_len as usize;
            let stored_script = &index.all_data[s_start..s_end];
            println!("  Stored script (hex): {}", hex::encode(stored_script));
            println!("  Scripts match: {}", stored_script == script);

            // Show UTXOs
            let u_start = entry.utxo_offset as usize;
            let u_end = u_start + (entry.utxo_count as usize) * 44;
            println!("\n  UTXOs ({} entries, {} bytes):", entry.utxo_count, u_end - u_start);

            let mut total = 0u64;
            for i in 0..entry.utxo_count.min(20) {
                let base = u_start + (i as usize) * 44;
                if base + 44 > index.utxo_data.len() {
                    println!("    [!] UTXO {} would read beyond utxo_data (base={}, data_len={})",
                        i, base, index.utxo_data.len());
                    break;
                }
                let txid = hex::encode(&index.utxo_data[base..base+32]);
                let vout = u32::from_le_bytes(index.utxo_data[base+32..base+36].try_into().unwrap());
                let value = u64::from_le_bytes(index.utxo_data[base+36..base+44].try_into().unwrap());
                total += value;
                println!("    UTXO {}: {}:{} = {} sats ({:.8} BTC)",
                    i, &txid[..16], vout, value, value as f64 / 1e8);
            }
            if entry.utxo_count > 20 {
                println!("    ... and {} more", entry.utxo_count - 20);
            }
            println!("  Total (first 20): {} sats ({:.8} BTC)", total, total as f64 / 1e8);

            // Also compute full lookup
            let full_total = index.lookup(&script);
            println!("\n  Full lookup() result: {} sats ({:.8} BTC)",
                full_total, full_total as f64 / 1e8);
        }
        SearchResult::NotFound(insert_pos) => {
            println!("  NOT FOUND (would be inserted at index {})", insert_pos);
            println!("  lookup() returns: {} sats", index.lookup(&script));
        }
    }

    // Show nearby entries
    let start = result.insert_pos().saturating_sub(cli.nearby);
    let end = (result.insert_pos() + cli.nearby).min(index.script_entries.len());

    println!("\n\nNearby script entries (indices {} to {}):", start, end - 1);
    println!("{:<6} {:<10} {:<10} {:<10} {:<12} {}",
        "Idx", "s_offset", "s_len", "u_offset", "u_count", "script_hex");

    for i in start..end {
        let e = index.script_entries[i];
        let s_start = e.script_offset as usize;
        let s_end = s_start + e.script_len as usize;
        let script_hex = if s_end <= index.all_data.len() {
            let s = &index.all_data[s_start..s_end];
            let mut h = hex::encode(s);
            if h.len() > 50 { h.truncate(47); h.push_str("..."); }
            h
        } else {
            format!("[CORRUPT offset={} len={}]", e.script_offset, e.script_len)
        };

        let marker = if result.is_found() && result.insert_pos() == i {
            " <-- MATCH"
        } else {
            ""
        };

        println!("{:<6} {:<10} {:<10} {:<10} {:<12} {}{}",
            i, e.script_offset, e.script_len, e.utxo_offset, e.utxo_count, script_hex, marker);
    }

    // Write raw data dump for further analysis
    let dump_path = "flatindex-debug-dump.json";
    let mut dump = Vec::new();
    for i in start..end {
        let e = index.script_entries[i];
        let s_start = e.script_offset as usize;
        let s_end = s_start + e.script_len as usize;
        let script_hex = if s_end <= index.all_data.len() {
            hex::encode(&index.all_data[s_start..s_end])
        } else {
            "CORRUPT".to_string()
        };

        // Get UTXO details
        let mut utxos = Vec::new();
        let u_start = e.utxo_offset as usize;
        for j in 0..e.utxo_count.min(5) {
            let base = u_start + (j as usize) * 44;
            if base + 44 <= index.utxo_data.len() {
                let txid = hex::encode(&index.utxo_data[base..base+32]);
                let vout = u32::from_le_bytes(index.utxo_data[base+32..base+36].try_into().unwrap());
                let value = u64::from_le_bytes(index.utxo_data[base+36..base+44].try_into().unwrap());
                utxos.push(format!("{}:{} = {} sats", &txid[..16], vout, value));
            }
        }

        dump.push(serde_json::json!({
            "index": i,
            "script_offset": e.script_offset,
            "script_len": e.script_len,
            "utxo_offset": e.utxo_offset,
            "utxo_count": e.utxo_count,
            "script_hex": script_hex,
            "sample_utxos": utxos,
        }));
    }

    std::fs::write(dump_path, serde_json::to_string_pretty(&dump)?)?;
    println!("\n\nDetailed dump written to {}", dump_path);

    Ok(())
}

enum SearchResult {
    Found(usize),
    NotFound(usize), // insertion position
}

impl SearchResult {
    fn insert_pos(&self) -> usize {
        match self {
            SearchResult::Found(i) => *i,
            SearchResult::NotFound(i) => *i,
        }
    }
    fn is_found(&self) -> bool {
        matches!(self, SearchResult::Found(_))
    }
}

fn binary_search(index: &flat_index::FlatIndex, script: &[u8]) -> SearchResult {
    if index.script_entries.is_empty() {
        return SearchResult::NotFound(0);
    }

    let mut lo: usize = 0;
    let mut hi: usize = index.script_entries.len();

    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        // Inline cmp_script_at
        let entry = index.script_entries[mid];
        let s_len = entry.script_len as usize;
        let s_start = entry.script_offset as usize;
        let s_end = s_start + s_len;
        let stored = &index.all_data[s_start..s_end];

        match stored.cmp(script) {
            std::cmp::Ordering::Less => lo = mid + 1,
            std::cmp::Ordering::Greater => hi = mid,
            std::cmp::Ordering::Equal => return SearchResult::Found(mid),
        }
    }

    SearchResult::NotFound(lo)
}
