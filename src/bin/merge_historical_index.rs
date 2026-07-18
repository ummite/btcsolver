//! Merge Historical Index — merges a tmp hex file with an existing binary historical index.
//!
//! Reads hex lines from a tmp file, deduplicates with HashSet,
//! then merges with an existing binary index (BTCSHIST format).
//!
//! Usage: merge_historical_index --tmp data\historical-scripts.bin.tmp --index data\historical-scripts.bin --output data\historical-scripts-merged.bin

use anyhow::Result;
use clap::Parser;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write, Seek, SeekFrom};

#[derive(clap::Parser)]
struct Cli {
    /// Path to tmp hex file (unsorted, with duplicates)
    #[arg(long)]
    tmp: String,

    /// Path to existing binary index (BTCSHIST format)
    #[arg(long)]
    index: String,

    /// Output path for merged index
    #[arg(long, default_value = "data\\historical-scripts-merged.bin")]
    output: String,

    /// Batch size for HashSet clearing (memory management)
    #[arg(long, default_value = "10000000")]
    batch_size: usize,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("=== Historical Index Merger ===");
    println!("Tmp file: {}", cli.tmp);
    println!("Existing index: {}", cli.index);
    println!("Output: {}", cli.output);
    println!();

    // Step 1: Read existing index scripts into a HashSet
    println!("Loading existing index...");
    let mut existing_scripts: HashSet<Vec<u8>> = HashSet::with_capacity(2_000_000);
    if std::path::Path::new(&cli.index).exists() {
        load_existing_index(&cli.index, &mut existing_scripts)?;
        println!("  Loaded {} scripts from existing index", existing_scripts.len());
    } else {
        println!("  No existing index found, starting fresh");
    }

    // Step 2: Read tmp file and add unique scripts
    println!("Processing tmp file (deduplicating)...");
    let tmp_file = File::open(&cli.tmp)?;
    let reader = BufReader::with_capacity(1024 * 1024, tmp_file);
    let mut total_lines = 0u64;
    let mut unique_from_tmp = 0u64;
    let mut batch_count = 0u64;

    for line_result in reader.lines() {
        let line = line_result?.trim().to_string();
        if line.is_empty() {
            continue;
        }
        total_lines += 1;

        if let Ok(script) = hex::decode(&line) {
            if !script.is_empty() && script.len() <= 0xFFFF {
                if existing_scripts.insert(script) {
                    unique_from_tmp += 1;
                }
            }
        }

        batch_count += 1;
        if (batch_count as usize) % cli.batch_size == 0 {
            println!(
                "  Batch {}: {} lines processed, {} unique ({} total scripts)",
                batch_count / cli.batch_size as u64,
                format_number(total_lines),
                format_number(unique_from_tmp),
                format_number(existing_scripts.len() as u64)
            );
        }
    }

    println!();
    println!("Tmp processing complete:");
    println!("  Total lines: {}", format_number(total_lines));
    println!("  Unique from tmp: {}", format_number(unique_from_tmp));
    println!("  Total unique scripts: {}", format_number(existing_scripts.len() as u64));
    println!();

    // Step 3: Sort all scripts and write merged binary index
    println!("Sorting and writing merged index...");
    let mut all_scripts: Vec<Vec<u8>> = existing_scripts.into_iter().collect();
    all_scripts.sort();

    let out_file = File::create(&cli.output)?;
    let mut w = BufWriter::new(out_file);

    // Header: "BTCSHIST" + version(u32 LE) + count(u64 LE)
    w.write_all(b"BTCSHIST")?;
    w.write_all(&3u32.to_le_bytes())?; // version 3 (merged)
    let count = all_scripts.len() as u64;
    w.write_all(&count.to_le_bytes())?;

    // Body: len(u16 LE) + script bytes for each entry
    let mut body_bytes = 0u64;
    for (i, script) in all_scripts.iter().enumerate() {
        w.write_all(&(script.len() as u16).to_le_bytes())?;
        w.write_all(script)?;
        body_bytes += 2 + script.len() as u64;

        if (i + 1) % 10_000_000 == 0 {
            println!(
                "  Written {}/{} scripts ({:.1} MB)",
                format_number((i + 1) as u64),
                format_number(count),
                body_bytes as f64 / 1024.0 / 1024.0
            );
        }
    }

    // Footer: body_bytes for verification
    w.write_all(&body_bytes.to_le_bytes())?;
    w.flush()?;

    let file_size = std::fs::metadata(&cli.output)?.len();
    println!();
    println!("Merged index written:");
    println!("  Path: {}", cli.output);
    println!("  Scripts: {}", format_number(count));
    println!("  File size: {:.1} MB", file_size as f64 / 1024.0 / 1024.0);
    println!("  Body bytes: {}", format_number(body_bytes));

    Ok(())
}

fn load_existing_index(path: &str, scripts: &mut HashSet<Vec<u8>>) -> Result<()> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // Read magic
    let mut magic = [0u8; 8];
    reader.read_exact(&mut magic)?;
    if &magic != b"BTCSHIST" {
        anyhow::bail!("Invalid index: wrong magic");
    }

    // Read version
    let mut version = [0u8; 4];
    reader.read_exact(&mut version)?;
    let version = u32::from_le_bytes(version);
    println!("  Index version: {}", version);

    // Read count
    let mut count_buf = [0u8; 8];
    reader.read_exact(&mut count_buf)?;
    let count = u64::from_le_bytes(count_buf);
    println!("  Declared count: {}", format_number(count));

    // Read scripts
    for i in 0..count {
        let mut len_buf = [0u8; 2];
        if reader.read_exact(&mut len_buf).is_err() {
            break;
        }
        let len = u16::from_le_bytes(len_buf) as usize;
        let mut script = vec![0u8; len];
        if reader.read_exact(&mut script).is_err() {
            break;
        }
        scripts.insert(script);

        if (i + 1) % 1_000_000 == 0 {
            println!("  Loaded {}/{} scripts", format_number((i + 1) as u64), format_number(count));
        }
    }

    Ok(())
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}
