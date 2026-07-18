//! Historical UTXO Indexer (utxo1) — tracks ALL scripts that have ever appeared in any TX output.
//!
//! Unlike the regular UTXO index (which only tracks currently unspent outputs),
//! this index records every script pubkey that has ever received satoshis,
//! even if all its UTXOs were later spent.
//!
//! Output: sorted binary file for binary search lookup.
//! Format: "BTCSHIST" + version(u32) + count(u64) + [len(u16)+script]* + footer(u64)
//!
//! V2: Uses BTreeSet<ScriptEntry> for memory-efficient storage (~120 bytes per unique script).
//! With 223GB RAM, can hold 100M+ unique scripts comfortably.
//!
//! Usage: historical_indexer build --blocks-dir W:\Bitcoin\blocks --output data\historical-scripts.bin

use anyhow::Result;
use bitcoin::blockdata::block::Block;
use bitcoin::consensus::Decodable;
use clap::Parser;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufReader, Read, Write, BufWriter, Seek, SeekFrom, BufRead};
use std::path::Path;
use std::time::Instant;

/// Fixed-size script entry for memory-efficient storage in BTreeSet.
/// Covers 99.99% of Bitcoin scripts (most are 22-34 bytes).
/// Max Bitcoin script_pubkey = 10,000 bytes, but we cap at 127 for inline storage.
/// Scripts > 127 bytes are truncated (extremely rare in practice).
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct ScriptEntry {
    len: u8,
    data: [u8; 127],
}

impl ScriptEntry {
    fn new(script: &[u8]) -> Self {
        let len = (script.len().min(127)) as u8;
        let mut entry = ScriptEntry {
            len: 0,
            data: [0u8; 127],
        };
        entry.len = len;
        entry.data[..len as usize].copy_from_slice(&script[..len as usize]);
        entry
    }

    fn as_slice(&self) -> &[u8] {
        &self.data[..self.len as usize]
    }
}

#[derive(clap::Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Build historical index from block files (streaming mode — memory efficient)
    Build {
        #[arg(short, long, default_value = "W:\\Bitcoin\\blocks")]
        blocks_dir: String,
        #[arg(short, long, default_value = "data\\historical-scripts.bin")]
        output: String,
        /// Start from file number (resume)
        #[arg(short, long, default_value = "0")]
        start_file: u32,
        /// Checkpoint interval
        #[arg(long, default_value = "50")]
        checkpoint: u32,
        /// Use in-memory mode (old, high RAM usage)
        #[arg(long, action = clap::ArgAction::SetTrue)]
        memory: bool,
    },
    /// Show index statistics
    Stats {
        #[arg(short, long, default_value = "data\\historical-scripts.bin")]
        index: String,
    },
    /// Query if a script hex has ever been active
    Query {
        #[arg(short, long)]
        script: String,
        #[arg(short, long, default_value = "data\\historical-scripts.bin")]
        index: String,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Build { blocks_dir, output, start_file, checkpoint, memory } => {
            if memory {
                cmd_build(&blocks_dir, &output, start_file, checkpoint)
            } else {
                cmd_build_streaming(&blocks_dir, &output, start_file, checkpoint)
            }
        }
        Commands::Query { script, index } => cmd_query(&script, &index),
        Commands::Stats { index } => cmd_stats(&index),
    }
}

fn collect_block_files(dir: &str) -> Result<Vec<String>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("blk") && name.ends_with(".dat") {
            files.push(entry.path().to_string_lossy().to_string());
        }
    }
    files.sort();
    Ok(files)
}

fn file_number(name: &str) -> u32 {
    let stem = std::path::Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    stem.strip_prefix("blk").and_then(|n| n.parse().ok()).unwrap_or(0)
}

/// Parse a block from a reader. Returns None on EOF or error.
/// Block file format: magic(4) + header(80) + size(4) + data(size)
fn try_read_block(reader: &mut BufReader<File>) -> Option<Block> {
    let magic = 0xd9b4bef9u32; // Bitcoin mainnet

    // Read magic (4 bytes)
    let mut magic_buf = [0u8; 4];
    if reader.read_exact(&mut magic_buf).is_err() {
        return None; // EOF
    }
    if u32::from_le_bytes(magic_buf) != magic {
        return None; // Not a valid block or obfuscated
    }

    // Read block size (4 bytes) — blk*.dat format: magic + size + (header + data)
    // size = total bytes of header (80) + block data that follow
    let mut size_buf = [0u8; 4];
    if reader.read_exact(&mut size_buf).is_err() {
        return None;
    }
    let block_total_size = u32::from_le_bytes(size_buf) as usize;

    // Sanity check: header(80) to ~150MB (max with witness)
    if block_total_size < 80 || block_total_size > 150_000_000 {
        return None;
    }

    // Read entire block: header (80 bytes) + data (tx count + transactions)
    let mut full_block = vec![0u8; block_total_size];
    if reader.read_exact(&mut full_block).is_err() {
        return None;
    }

    // Parse using consensus deserialization
    let mut cursor = std::io::Cursor::new(&full_block);
    Block::consensus_decode(&mut cursor).ok()
}

fn extract_scripts_from_file(
    path: &str,
    scripts: &mut BTreeSet<ScriptEntry>,
) -> Result<(u32, usize)> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut blocks_parsed = 0u32;
    let initial_count = scripts.len();

    loop {
        match try_read_block(&mut reader) {
            Some(block) => {
                for tx in block.txdata.iter() {
                    for txout in tx.output.iter() {
                        let script = txout.script_pubkey.as_bytes();
                        scripts.insert(ScriptEntry::new(script));
                    }
                }
                blocks_parsed += 1;
            }
            None => break,
        }
    }

    let new_scripts = scripts.len() - initial_count;
    Ok((blocks_parsed, new_scripts))
}

/// Streaming version: write scripts to a temp file as hex lines, then sort externally.
/// Much more memory efficient than accumulating in a HashSet.
fn cmd_build_streaming(
    blocks_dir: &str,
    output: &str,
    start_file: u32,
    checkpoint: u32,
) -> Result<()> {
    use std::process::Command;

    let block_files = collect_block_files(blocks_dir)?;
    let total_files = block_files.len();

    eprintln!("=== Historical Indexer (Streaming Mode) ===");
    eprintln!("Blocks dir: {}", blocks_dir);
    eprintln!("Output: {}", output);
    eprintln!("Block files: {}", total_files);
    eprintln!("Start from file #: {}", start_file);
    eprintln!();

    // Temp file for raw scripts (hex lines)
    let temp_file = format!("{}.tmp", output);
    let mut raw_file = BufWriter::new(File::create(&temp_file)?);
    let start_time = Instant::now();
    let mut total_blocks = 0u64;
    let mut total_scripts = 0u64;

    for (i, path) in block_files.into_iter().enumerate() {
        let file_num = file_number(&path);
        if file_num < start_file {
            eprintln!("[{0}/{1}] Skipping (before start_file)", file_num, total_files);
            continue;
        }

        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        let mut file_blocks = 0u32;

        while let Some(block) = try_read_block(&mut reader) {
            for tx in block.txdata.iter() {
                for txout in tx.output.iter() {
                    let script = txout.script_pubkey.as_bytes();
                    // Write as hex line (length-prefixed for variable length)
                    writeln!(raw_file, "{}", hex::encode(script)).ok();
                    total_scripts += 1;
                }
            }
            file_blocks += 1;
            total_blocks += 1;
        }

        raw_file.flush().ok();
        let elapsed = start_time.elapsed().as_secs_f32().max(0.001);
        let rate = if elapsed > 0.0 { total_blocks as f32 / elapsed } else { 0.0 };
        eprintln!(
            "[{0}/{1}] {2} blocks — {3} total blocks, {4} scripts written {5:.0} blk/s",
            file_num, total_files, file_blocks, total_blocks, total_scripts, rate
        );

        // Checkpoint
        if i % (checkpoint as usize) == 0 {
            raw_file.flush().ok();
            let ck = CheckpointFile {
                file_number: file_num,
                script_count: total_scripts,
                block_count: total_blocks,
            };
            let ck_path = format!("{}.ckpt", output);
            serde_json::to_writer(File::create(&ck_path)?, &ck).ok();
        }
    }

    raw_file.flush().ok();
    drop(raw_file);

    eprintln!();
    eprintln!("Extraction complete: {} blocks, {} scripts (with duplicates)", total_blocks, total_scripts);
    eprintln!("Sorting and deduplicating...");

    // Sort and deduplicate using system sort command
    let sort_result = if cfg!(target_os = "windows") {
        // Windows: use PowerShell Sort-Object -Unique
        Command::new("powershell")
            .args(&[
                "-NoProfile", "-Command",
                &format!("[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; Get-Content '{}' | Sort-Object -Unique | Out-File -Encoding utf8 '{}'", temp_file, format!("{}.sorted", output)),
            ])
            .status()
    } else {
        // Linux/Mac: use sort -u
        Command::new("sort")
            .args(&["-u", "-o", &format!("{}.sorted", output), &temp_file])
            .status()
    };

    if sort_result.map_or(true, |s| !s.success()) {
        eprintln!("WARNING: sort failed, falling back to in-memory dedup");
        // Fallback: read and dedup in memory
        let content = std::fs::read_to_string(&temp_file).unwrap_or_default();
        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        lines.sort();
        lines.dedup();

        // Write sorted unique to temp sorted file
        let mut sorted_file = File::create(format!("{}.sorted", output))?;
        for line in &lines {
            writeln!(sorted_file, "{}", line).ok();
        }
        drop(sorted_file);
    }

    // Build final binary index from sorted unique file
    eprintln!("Building binary index...");
    build_binary_index(&format!("{}.sorted", output), output)?;

    // Cleanup temp files
    std::fs::remove_file(&temp_file).ok();
    std::fs::remove_file(format!("{}.sorted", output)).ok();

    let elapsed = start_time.elapsed();
    eprintln!();
    eprintln!("Done in {:.1}s", elapsed.as_secs_f32());
    Ok(())
}

/// Build binary index from a sorted unique hex file
fn build_binary_index(sorted_path: &str, output: &str) -> Result<()> {
    let input = File::open(sorted_path)?;
    let reader = BufReader::new(input);
    let out_file = File::create(output)?;
    let mut w = BufWriter::new(out_file);

    // Header: "BTCSHIST" + version(u32 LE) + count placeholder(u64 LE)
    w.write_all(b"BTCSHIST")?;
    w.write_all(&1u32.to_le_bytes())?; // version 1
    let count_offset = 12u64; // position of count field
    w.write_all(&0u64.to_le_bytes())?; // placeholder, will update

    let mut count = 0u64;
    for line_result in reader.lines() {
        let line = line_result?;
        let line = line.trim();
        if line.is_empty() { continue; }

        let script = hex::decode(line).unwrap_or_default();
        if script.is_empty() || script.len() > 0xFFFF { continue; }

        // Write: len(u16 LE) + script bytes
        w.write_all(&(script.len() as u16).to_le_bytes())?;
        w.write_all(&script)?;
        count += 1;
    }

    w.flush()?;
    drop(w);

    // Update count field at offset 8
    let mut file = std::fs::OpenOptions::new()
        .read(true).write(true).open(output)?;
    file.seek(std::io::SeekFrom::Start(count_offset))?;
    file.write_all(&count.to_le_bytes())?;

    // Footer: count again (for verification)
    file.seek(std::io::SeekFrom::End(0))?;
    file.write_all(&count.to_le_bytes())?;

    eprintln!("Binary index: {} unique scripts → {}", count, output);
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CheckpointFile {
    file_number: u32,
    script_count: u64,
    block_count: u64,
}

fn cmd_build(
    blocks_dir: &str,
    output: &str,
    start_file: u32,
    checkpoint: u32,
) -> Result<()> {
    let block_files = collect_block_files(blocks_dir)?;
    let total_files = block_files.len();

    eprintln!("=== Historical Indexer ===");
    eprintln!("Blocks dir: {}", blocks_dir);
    eprintln!("Output: {}", output);
    eprintln!("Block files: {}", total_files);
    eprintln!("Start from file #: {}", start_file);
    eprintln!();

    let mut scripts: BTreeSet<ScriptEntry> = BTreeSet::new();
    let start_time = Instant::now();
    let mut total_blocks = 0u32;

    // Checkpoint file
    let checkpoint_file = format!("{}.ckpt", output);

    for (i, path) in block_files.iter().enumerate() {
        let file_num = file_number(path);
        if file_num < start_file {
            continue;
        }

        match extract_scripts_from_file(path, &mut scripts) {
            Ok((blocks, _new_in_file)) => {
                total_blocks += blocks;
            }
            Err(e) => {
                eprintln!("  ERROR processing {}: {}", path, e);
                continue;
            }
        }

        let elapsed = start_time.elapsed();
        let rate = if elapsed.as_secs() > 0 {
            total_blocks as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        let fname = std::path::Path::new(path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        eprintln!(
            "  [{:4}/{}] {} — {} blocks (total: {} blks, {:8} scripts) {:.0} blk/s",
            i, total_files, fname, total_blocks,
            total_blocks, scripts.len(), rate
        );

        // Checkpoint
        if (file_num.saturating_sub(start_file)) > 0 && (file_num.saturating_sub(start_file)) % checkpoint == 0 {
            let ckpt = CheckpointFile {
                file_number: file_num,
                script_count: scripts.len() as u64,
                block_count: total_blocks as u64,
            };
            let json = serde_json::to_string(&ckpt)?;
            std::fs::write(&checkpoint_file, json)?;
        }
    }

    eprintln!();
    eprintln!("Scan complete: {} blocks, {} unique scripts in {:.1}s",
              total_blocks, scripts.len(), start_time.elapsed().as_secs_f64());

    // BTreeSet is already sorted — no need to sort again
    let sorted_count = scripts.len();
    eprintln!("Sorted unique scripts: {}", sorted_count);

    // Write output
    let output_path = Path::new(output);
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = File::create(output)?;
    let mut w = BufWriter::new(file);

    // Header: magic + version + count
    w.write_all(b"BTCSHIST")?;
    w.write_all(&2u32.to_le_bytes())?; // version 2 (BTreeSet format)
    w.write_all(&(sorted_count as u64).to_le_bytes())?; // count

    // Body: len(u16) + script bytes for each entry
    let mut body_bytes = 0u64;
    for entry in &scripts {
        let slice = entry.as_slice();
        w.write_all(&(slice.len() as u16).to_le_bytes())?;
        w.write_all(slice)?;
        body_bytes += 2 + slice.len() as u64;
    }
    w.write_all(&body_bytes.to_le_bytes())?; // footer
    w.flush()?;

    let file_size = output_path.metadata()?.len();
    let total_time = start_time.elapsed();

    eprintln!();
    eprintln!("Historical index written:");
    eprintln!("  Path: {}", output);
    eprintln!("  Scripts: {}", sorted_count);
    eprintln!("  File size: {:.1} MB", file_size as f64 / (1024.0 * 1024.0));
    eprintln!("  Total time: {:.1}s ({:.0} blk/s)",
              total_time.as_secs_f64(),
              if total_time.as_secs_f64() > 0.0 {
                  total_blocks as f64 / total_time.as_secs_f64()
              } else { 0.0 });

    // Remove checkpoint
    let _ = std::fs::remove_file(&checkpoint_file);

    Ok(())
}

fn cmd_stats(index: &str) -> Result<()> {
    let metadata = std::fs::metadata(index)?;
    let file = File::open(index)?;
    let mut r = BufReader::new(file);

    let mut magic = [0u8; 8];
    r.read_exact(&mut magic)?;
    if &magic != b"BTCSHIST" {
        anyhow::bail!("Not a historical index file");
    }

    let mut ver = [0u8; 4];
    r.read_exact(&mut ver)?;
    let mut cnt = [0u8; 8];
    r.read_exact(&mut cnt)?;
    let script_count = u64::from_le_bytes(cnt);

    // Sample average script size
    let mut total_len = 0u64;
    let samples = 1000u64.min(script_count);
    for _ in 0..samples {
        let mut lb = [0u8; 2];
        r.read_exact(&mut lb)?;
        let sl = u16::from_le_bytes(lb) as u64;
        total_len += sl;
        r.seek(SeekFrom::Current(sl as i64))?;
    }
    let avg = if samples > 0 { total_len / samples } else { 0 };

    eprintln!("Historical Index Stats:");
    eprintln!("  File: {}", index);
    eprintln!("  Version: {}", u32::from_le_bytes(ver));
    eprintln!("  Scripts: {} ({:.1} M)", script_count, script_count as f64 / 1e6);
    eprintln!("  File size: {:.1} MB", metadata.len() as f64 / (1024.0 * 1024.0));
    eprintln!("  Avg script size (sample): {} bytes", avg);

    Ok(())
}

fn cmd_query(script_hex: &str, index: &str) -> Result<()> {
    let target = hex::decode(script_hex)?;
    eprintln!("Querying script: {} ({} bytes)", hex::encode(&target), target.len());

    let metadata = std::fs::metadata(index)?;
    let file_len = metadata.len();

    // Memory-map the file for fast random access
    let file = File::open(index)?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };

    // Parse header
    if mmap.len() < 20 {
        anyhow::bail!("File too small");
    }
    if &mmap[0..8] != b"BTCSHIST" {
        anyhow::bail!("Not a BTCSHIST file");
    }
    let version = u32::from_le_bytes([mmap[8], mmap[9], mmap[10], mmap[11]]);
    let script_count = u64::from_le_bytes([
        mmap[12], mmap[13], mmap[14], mmap[15],
        mmap[16], mmap[17], mmap[18], mmap[19],
    ]);

    eprintln!("Index has {} scripts (v{}, {:.1} MB)",
        script_count, version, file_len as f64 / 1_024.0 / 1_024.0);

    // Build position index (byte offset for each script entry)
    let mut positions: Vec<u64> = Vec::with_capacity(script_count as usize);
    let mut off: u64 = 20; // skip header
    for _ in 0..script_count {
        if off + 2 > mmap.len() as u64 { break; }
        positions.push(off);
        let sl = u16::from_le_bytes([mmap[off as usize], mmap[(off + 1) as usize]]) as u64;
        off += 2 + sl;
    }
    // Add sentinel after last entry (before footer)
    positions.push(off);

    let actual_count = positions.len() as u64 - 1;
    eprintln!("Position index built: {} entries", actual_count);

    // Binary search
    let found = binary_search_script(&mmap, &positions, &target, actual_count);

    if let Some(pos) = found {
        eprintln!("FOUND at position {}/{} — script HAS been active on-chain", pos, actual_count);
    } else {
        eprintln!("NOT FOUND — script has NEVER been active");
    }

    Ok(())
}

/// Binary search for a target script in the memory-mapped index.
/// Returns Some(position) if found, None otherwise.
fn binary_search_script(mmap: &[u8], positions: &[u64], target: &[u8], count: u64) -> Option<u64> {
    let mut lo: u64 = 0;
    let mut hi = count;

    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let off = positions[mid as usize];
        let next_off = positions[(mid + 1) as usize];

        // Read script at this position
        let sl = u16::from_le_bytes([mmap[off as usize], mmap[(off + 1) as usize]]) as usize;
        let script = &mmap[(off + 2) as usize..(off + 2 + sl as u64) as usize];

        let cmp = script.cmp(target);
        if cmp == std::cmp::Ordering::Equal {
            return Some(mid);
        } else if cmp < std::cmp::Ordering::Equal {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }

    None
}
