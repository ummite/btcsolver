// merge_corpus.rs — Merge k sorted files into one sorted deduplicated file
// Uses a min-heap (BinaryHeap) for k-way merge — O(n log k) time, O(k) memory

use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write, BufWriter};

struct Reader {
    line: Option<String>,
    reader: BufReader<File>,
}

impl Reader {
    fn new(path: &str) -> io::Result<Self> {
        let f = File::open(path)?;
        let r = BufReader::new(f);
        Ok(Self { line: None, reader: r })
    }

    fn next_line(&mut self) -> Option<String> {
        let mut buf = String::new();
        match self.reader.read_line(&mut buf) {
            Ok(0) => None,
            Ok(_) => {
                let trimmed = buf.trim_end_matches(|c: char| c.is_whitespace()).to_string();
                if trimmed.is_empty() { None } else { Some(trimmed) }
            }
            Err(_) => None,
        }
    }
}

struct HeapEntry {
    line: String,
    index: usize,
}

impl Eq for HeapEntry {}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.line == other.line && self.index == other.index
    }
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap
        other.line.cmp(&self.line).then(other.index.cmp(&self.index))
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn main() -> io::Result<()> {
    let files = vec![
        "data/easy-keys-corpus.txt",
        "data/easy-keys-v2.txt",
        "data/human-2009-keys.txt",
        "data/math-constants-keys.txt",
        "data/pop-culture-2009-keys.txt",
        "data/forgotten-mega-corpus.txt",
    ];

    let output = "data/merged-corpus.txt";
    println!("=== Corpus Merge (K-way sorted merge) ===");
    println!("Input files: {}", files.len());
    println!("Output: {}", output);

    // Open all files
    let mut readers: Vec<Option<Reader>> = Vec::new();
    let mut heap = BinaryHeap::new();

    for (i, f) in files.iter().enumerate() {
        match Reader::new(f) {
            Ok(mut r) => {
                let size = std::fs::metadata(f).unwrap().len();
                println!("  {} ({:.1} MB)", f, size as f64 / 1_000_000.0);
                if let Some(line) = r.next_line() {
                    heap.push(HeapEntry { line, index: i });
                }
                readers.push(Some(r));
            }
            Err(e) => {
                eprintln!("  {} (skipped: {})", f, e);
                readers.push(None);
            }
        }
    }

    // Merge
    let out = File::create(output)?;
    let mut writer = BufWriter::new(out);
    let mut total = 0u64;
    let mut deduped = 0u64;
    let mut last_line = String::new();

    let start = std::time::Instant::now();

    while let Some(mut entry) = heap.pop() {
        total += 1;
        if entry.line != last_line {
            writeln!(writer, "{}", entry.line)?;
            deduped += 1;
            last_line = entry.line.clone();
        }
        // Read next line from same reader
        if let Some(Some(ref mut r)) = readers.get_mut(entry.index) {
            if let Some(line) = r.next_line() {
                entry.line = line;
                heap.push(entry);
            }
        }
        // Progress every 1M lines
        if total % 1_000_000 == 0 {
            let elapsed = start.elapsed();
            let rate = total as f64 / elapsed.as_secs_f64();
            println!("  {}M lines merged, {:.0} lines/sec", total / 1_000_000, rate);
        }
    }

    writer.flush()?;
    let elapsed = start.elapsed();
    let size = std::fs::metadata(output).unwrap().len();

    println!("\n=== Results ===");
    println!("Total lines read: {}", total);
    println!("Unique lines written: {}", deduped);
    println!("Duplicates removed: {}", total - deduped);
    println!("Output size: {:.1} MB", size as f64 / 1_000_000.0);
    println!("Time: {:.1}s", elapsed.as_secs_f64());
    if elapsed.as_secs_f64() > 0.0 {
        println!("Rate: {:.0} lines/sec", total as f64 / elapsed.as_secs_f64());
    }

    Ok(())
}
