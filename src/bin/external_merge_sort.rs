//! External Merge Sort — tri + dedup de fichiers hex gigantesques (>100 Go).
//!
//! Phase 1: Lit le fichier par chunks, décode hex, déduplique, trie, écrit chunks binaires.
//! Phase 2: Fusion k-way streaming des chunks + index existant → fichier final BTCSHIST.
//!
//! Mémoire: chunk_lines * ~50 octets/script (défaut 50M = ~2.5 GB par chunk).
//! La fusion k-way ne garde qu'un script par fichier dans le heap.
//!
//! Usage:
//!   external_merge_sort --input data\historical-scripts.bin.tmp \
//!     --existing data\historical-scripts.bin \
//!     --output data\historical-scripts-v2.bin \
//!     --chunk-lines 50000000

use anyhow::{Context, Result};
use clap::Parser;
use std::collections::{BinaryHeap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::cmp::Ordering;

#[derive(clap::Parser)]
struct Cli {
    #[arg(long, help = "Input hex file (one hex script per line)")]
    input: String,

    #[arg(long, help = "Existing binary index to merge (BTCSHIST, optional)")]
    existing: Option<String>,

    #[arg(long, help = "Output binary index (BTCSHIST format)")]
    output: String,

    #[arg(long, default_value = "50000000", help = "Lines per chunk (default 50M)")]
    chunk_lines: usize,

    #[arg(long, default_value = "data\\chunks", help = "Temp directory for chunks")]
    temp_dir: String,

    #[arg(long, help = "Skip phase 1, reuse existing chunks")]
    skip_phase1: bool,

    #[arg(long, help = "Only run phase 1 (chunking)")]
    skip_phase2: bool,

    #[arg(long, help = "Keep chunk files after merge")]
    skip_cleanup: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("=== External Merge Sort ===");
    println!("  Input:      {}", cli.input);
    println!("  Output:     {}", cli.output);
    println!("  Chunk size: {} lines", format_num(cli.chunk_lines as u64));
    println!("  Temp dir:   {}", cli.temp_dir);
    println!();

    std::fs::create_dir_all(&cli.temp_dir)?;

    if !cli.skip_phase1 {
        phase1_chunk_sort(&cli)?;
    }

    if cli.skip_phase2 {
        println!("Phase 2 skipped — chunks are in {}", cli.temp_dir);
        return Ok(());
    }

    let result = phase2_streaming_merge(&cli)?;

    if !cli.skip_cleanup {
        println!();
        println!("Cleaning up chunk files...");
        for f in find_chunk_files(&cli.temp_dir) {
            let _ = std::fs::remove_file(&f);
        }
        let _ = std::fs::remove_dir(&cli.temp_dir);
        println!("Done.");
    }

    println!();
    println!("=== COMPLETE ===");
    println!("  Scripts:    {}", format_num(result.total_scripts));
    println!("  Duplicates: {}", format_num(result.duplicates));
    println!("  File size:  {:.1} MB", result.file_size as f64 / 1_024.0 / 1_024.0);

    Ok(())
}

// ============================================================
// Phase 1: Chunk Sort
// ============================================================

fn phase1_chunk_sort(cli: &Cli) -> Result<()> {
    println!("=== Phase 1: Chunk Sort ===");

    let file_size = std::fs::metadata(&cli.input)
        .context("Cannot stat input file")?
        .len();
    println!("  Input size: {:.1} GB", file_size as f64 / 1e9);
    println!();

    let input_file = File::open(&cli.input)?;
    let reader = BufReader::with_capacity(32 * 1024 * 1024, input_file);

    let mut chunk_idx: u64 = 0;
    let mut total_lines: u64 = 0;
    let mut seen: HashSet<Vec<u8>> = HashSet::with_capacity(cli.chunk_lines);
    let mut chunk_unique_total: u64 = 0;


    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        total_lines += 1;

        if let Ok(script) = hex::decode(trimmed) {
            if !(1..=0xFFFF).contains(&script.len()) {
                continue;
            }
            seen.insert(script);
        }

        if seen.len() >= cli.chunk_lines {
            let mut scripts: Vec<Vec<u8>> = seen.drain().collect();
            scripts.sort();
            scripts.dedup();

            let path = Path::new(&cli.temp_dir).join(format!("chunk_{:06}.bin", chunk_idx));
            write_sorted_chunk(&path, &scripts)?;

            let unique = scripts.len() as u64;
            chunk_unique_total += unique;
            println!(
                "  Chunk {:04}: {:>12} unique  ({:.1} MB)",
                chunk_idx, format_num(unique),
                std::fs::metadata(&path)?.len() as f64 / 1_024.0 / 1_024.0
            );

            chunk_idx += 1;
        }

        if total_lines % 200_000_000 == 0 {
            println!(
                "  >>> Progress: {} lines, {} chunks",
                format_num(total_lines), chunk_idx
            );
        }
    }

    // Final chunk
    if !seen.is_empty() {
        let mut scripts: Vec<Vec<u8>> = seen.drain().collect();
        scripts.sort();
        scripts.dedup();

        let path = Path::new(&cli.temp_dir).join(format!("chunk_{:06}.bin", chunk_idx));
        write_sorted_chunk(&path, &scripts)?;

        chunk_unique_total += scripts.len() as u64;
        chunk_idx += 1;
        println!(
            "  Chunk {:04} (final): {:>12} unique",
            chunk_idx - 1,
            format_num(scripts.len() as u64)
        );
    }

    println!();
    println!("  Phase 1 done: {} lines → {} chunks, ~{} unique (per-chunk dedup)",
        format_num(total_lines), chunk_idx, format_num(chunk_unique_total));

    Ok(())
}

// ============================================================
// Phase 2: Streaming K-Way Merge
// ============================================================

struct MergeStats {
    total_scripts: u64,
    duplicates: u64,
    file_size: u64,
}

fn phase2_streaming_merge(cli: &Cli) -> Result<MergeStats> {
    println!();
    println!("=== Phase 2: Streaming K-Way Merge ===");

    let chunk_files = find_chunk_files(&cli.temp_dir);
    println!("  Chunks: {}", chunk_files.len());
    if chunk_files.is_empty() {
        anyhow::bail!("No chunk files found");
    }

    // Load existing index (small, fits in RAM)
    let mut existing: Vec<Vec<u8>> = Vec::new();
    if let Some(ref ep) = cli.existing {
        if Path::new(ep).exists() {
            println!("  Loading existing index...");
            load_btcshist(ep, &mut existing)?;
            existing.sort();
            existing.dedup();
            println!("    {} unique scripts", format_num(existing.len() as u64));
        }
    }

    // Serialize existing for streaming
    let existing_data = if !existing.is_empty() {
        Some(serialize_sorted_scripts(&existing))
    } else {
        None
    };

    // Open all readers
    let num_chunks = chunk_files.len();
    let has_existing = existing_data.is_some();
    let _total_sources = num_chunks + if has_existing { 1 } else { 0 };

    // Open chunk file readers
    let mut chunk_readers: Vec<Option<BufReader<File>>> = Vec::with_capacity(num_chunks);
    for path in &chunk_files {
        let f = File::open(path)?;
        chunk_readers.push(Some(BufReader::with_capacity(8 * 1024 * 1024, f)));
    }

    // Existing reader
    let mut existing_reader: Option<BufReader<std::io::Cursor<Vec<u8>>>> = None;
    if let Some(data) = existing_data {
        existing_reader = Some(BufReader::new(std::io::Cursor::new(data)));
    }

    // Min-heap
    let mut heap: BinaryHeap<HeapEntry> = BinaryHeap::new();

    // Seed heap
    for idx in 0..num_chunks {
        if let Some(script) = read_script(&mut chunk_readers[idx])? {
            heap.push(HeapEntry { script, source: idx as u32 });
        }
    }
    if has_existing {
        if let Some(script) = read_script_existing(&mut existing_reader)? {
            heap.push(HeapEntry { script, source: num_chunks as u32 });
        }
    }

    // Open output file
    let out = File::create(&cli.output)?;
    let mut out = out;
    // Reserve header space (20 bytes)
    let header_space = [0u8; 20];
    out.write_all(&header_space)?;
    let _body_start = out.stream_position()?;

    let mut w = BufWriter::with_capacity(16 * 1024 * 1024, out);

    let mut total_read: u64 = 0;
    let mut total_written: u64 = 0;
    let mut duplicates: u64 = 0;
    let mut body_bytes: u64 = 0;
    let mut prev: Vec<u8> = Vec::new();
    let mut first = true;

    while let Some(entry) = heap.pop() {
        let script = entry.script;
        let src = entry.source as usize;
        total_read += 1;

        // Dedup
        if !first && script == prev {
            duplicates += 1;
        } else {
            // Write
            w.write_all(&(script.len() as u16).to_le_bytes())?;
            w.write_all(&script)?;
            body_bytes += 2 + script.len() as u64;
            total_written += 1;
            prev = script;
            first = false;
        }

        // Refill from same source
        if src < num_chunks {
            if let Some(s) = read_script(&mut chunk_readers[src])? {
                heap.push(HeapEntry { script: s, source: src as u32 });
            }
        } else if src == num_chunks {
            if let Some(s) = read_script_existing(&mut existing_reader)? {
                heap.push(HeapEntry { script: s, source: src as u32 });
            }
        }

        if total_read % 10_000_000 == 0 {
            println!(
                "  {:>12} read  {:>12} written  {:>12} dupes",
                format_num(total_read),
                format_num(total_written),
                format_num(duplicates)
            );
        }
    }

    // Footer: body_bytes (u64 LE)
    w.write_all(&body_bytes.to_le_bytes())?;
    w.flush()?;

    // Seek back and write header
    let mut out_file = w.into_inner()?;
    out_file.seek(SeekFrom::Start(0))?;
    let mut hw = BufWriter::new(out_file);
    hw.write_all(b"BTCSHIST")?;
    hw.write_all(&5u32.to_le_bytes())?; // version 5
    hw.write_all(&total_written.to_le_bytes())?;
    hw.flush()?;

    let file_size = std::fs::metadata(&cli.output)?.len();

    println!();
    println!("  Merge done: {} written, {} dupes, {:.1} MB",
        format_num(total_written), format_num(duplicates),
        file_size as f64 / 1_024.0 / 1_024.0);

    Ok(MergeStats { total_scripts: total_written, duplicates, file_size })
}

// ============================================================
// Data structures
// ============================================================

#[derive(Eq, PartialEq)]
struct HeapEntry {
    script: Vec<u8>,
    source: u32,
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap: reverse comparison
        other.script.cmp(&self.script)
            .then(self.source.cmp(&other.source))
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// ============================================================
// I/O helpers
// ============================================================

fn read_script(opt: &mut Option<BufReader<File>>) -> Result<Option<Vec<u8>>> {
    if let Some(ref mut r) = opt {
        read_script_generic(r)
    } else {
        Ok(None)
    }
}

fn read_script_existing(opt: &mut Option<BufReader<std::io::Cursor<Vec<u8>>>>) -> Result<Option<Vec<u8>>> {
    if let Some(ref mut r) = opt {
        read_script_generic(r)
    } else {
        Ok(None)
    }
}

fn read_script_generic<R: Read>(r: &mut R) -> Result<Option<Vec<u8>>> {
    let mut lb = [0u8; 2];
    match r.read_exact(&mut lb) {
        Ok(()) => {},
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
    }
    let len = u16::from_le_bytes(lb) as usize;
    if len == 0 || len > 0xFFFF {
        return Ok(None);
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    Ok(Some(buf))
}

fn write_sorted_chunk(path: &Path, scripts: &[Vec<u8>]) -> Result<()> {
    let f = File::create(path)?;
    let mut w = BufWriter::new(f);
    for s in scripts {
        w.write_all(&(s.len() as u16).to_le_bytes())?;
        w.write_all(s)?;
    }
    w.flush()?;
    Ok(())
}

fn serialize_sorted_scripts(scripts: &[Vec<u8>]) -> Vec<u8> {
    let total: usize = scripts.iter().map(|s| 2 + s.len()).sum();
    let mut buf = Vec::with_capacity(total);
    for s in scripts {
        buf.extend_from_slice(&(s.len() as u16).to_le_bytes());
        buf.extend_from_slice(s);
    }
    buf
}

fn load_btcshist(path: &str, scripts: &mut Vec<Vec<u8>>) -> Result<()> {
    let f = File::open(path)?;
    let mut r = BufReader::new(f);

    let mut magic = [0u8; 8];
    r.read_exact(&mut magic)?;
    if &magic != b"BTCSHIST" {
        anyhow::bail!("Not a BTCSHIST file");
    }

    let mut ver = [0u8; 4];
    r.read_exact(&mut ver)?;
    println!("    Version: {}", u32::from_le_bytes(ver));

    let mut cb = [0u8; 8];
    r.read_exact(&mut cb)?;
    let count = u64::from_le_bytes(cb);
    println!("    Declared: {}", format_num(count));

    for i in 0..count {
        let mut lb = [0u8; 2];
        if r.read_exact(&mut lb).is_err() { break; }
        let len = u16::from_le_bytes(lb) as usize;
        let mut s = vec![0u8; len];
        if r.read_exact(&mut s).is_err() { break; }
        scripts.push(s);
        if (i + 1) % 1_000_000 == 0 {
            println!("    Loaded {}/{}", format_num((i+1) as u64), format_num(count));
        }
    }
    Ok(())
}

fn find_chunk_files(dir: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("bin") {
                if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                    if stem.starts_with("chunk_") {
                        files.push(p);
                    }
                }
            }
        }
    }
    files.sort();
    files
}

fn format_num(n: u64) -> String {
    if n >= 1_000_000_000 { format!("{:.1}B", n as f64 / 1e9) }
    else if n >= 1_000_000 { format!("{:.1}M", n as f64 / 1e6) }
    else if n >= 1_000 { format!("{:.1}K", n as f64 / 1e3) }
    else { format!("{}", n) }
}
