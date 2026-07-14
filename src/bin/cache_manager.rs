//! BTCSolver Cache Manager
//!
//! Manages a local copy of the UTXO index on the fastest local disk,
//! synced from the SAN (network share). This avoids slow network I/O
//! when loading the index for brute-force operations.
//!
//! Usage:
//!   cache_manager init          # First-time: copy from SAN to local disk
//!   cache_manager sync          # Check for updates and sync if needed
//!   cache_manager status        # Show cache status
//!   cache_manager path          # Print local cache path (for scripts)
//!
//! Configuration:
//!   --san-path    Path to SAN index (default: Y:\btcsolver\utxo-index.redb)
//!   --cache-dir   Override cache directory (auto-detected if not set)

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Parser)]
struct Cli {
    /// Path to SAN index (source of truth)
    #[arg(long, default_value = "Y:\\btcsolver\\utxo-index.redb")]
    san_path: String,

    /// Override cache directory (auto-detected if not set)
    #[arg(long)]
    cache_dir: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize local cache from SAN (first-time setup)
    Init,
    /// Sync local cache with SAN (copy if SAN has newer data)
    Sync,
    /// Show cache status (local vs SAN)
    Status,
    /// Print local cache path (for use in scripts)
    Path,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let san_path = Path::new(&cli.san_path);

    let cache_dir = if let Some(dir) = &cli.cache_dir {
        PathBuf::from(dir)
    } else {
        auto_detect_cache_dir()?
    };

    match cli.command {
        Commands::Init => cmd_init(san_path, &cache_dir)?,
        Commands::Sync => cmd_sync(san_path, &cache_dir)?,
        Commands::Status => cmd_status(san_path, &cache_dir)?,
        Commands::Path => println!("{}", cache_dir.display()),
    }

    Ok(())
}

// ─── Auto-detect fastest local disk ──────────────────────────────────────

/// Known network drive letters to skip
const NETWORK_DRIVES: &[char] = &['Y', 'T', 'Z', 'X', 'W'];

fn auto_detect_cache_dir() -> Result<PathBuf> {
    println!("Detecting fastest local disk with sufficient space...");

    let mut candidates: Vec<(String, u64)> = Vec::new();

    // Check common local drive letters (skip known network drives)
    for drive in ['C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'U', 'V'] {
        let drive_path = format!("{}:\\", drive);

        // Skip known network drive letters
        if NETWORK_DRIVES.contains(&drive) {
            continue;
        }

        // Check if drive exists and is accessible
        if let Ok(free_gb) = get_free_space_gb(&drive_path) {
            if free_gb >= 10 {
                println!("  {} - {} GB free (local)", drive_path, free_gb);
                candidates.push((drive_path, free_gb));
            } else if free_gb > 0 {
                println!("  {} - {} GB free (insufficient, need 10+)", drive_path, free_gb);
            }
        }
    }

    if candidates.is_empty() {
        println!("\nNo local disk with 10+ GB free found.");
        println!("Falling back to SAN directory: Y:\\btcsolver");
        return Ok(PathBuf::from("Y:\\btcsolver"));
    }

    // Sort by free space (descending) — prefer disk with most space
    candidates.sort_by(|a, b| b.1.cmp(&a.1));

    let chosen = &candidates[0];
    let cache_dir = PathBuf::from(&chosen.0).join("btcsolver-cache");

    println!("\nSelected: {} ({} GB free)", chosen.0, chosen.1);
    println!("Cache dir: {}", cache_dir.display());

    // Create directory if needed
    fs::create_dir_all(&cache_dir)?;

    Ok(cache_dir)
}

/// Get free space in GB for a drive using PowerShell
fn get_free_space_gb(drive_path: &str) -> io::Result<u64> {
    let test_file = format!("{}\\.", drive_path);
    if fs::metadata(&test_file).is_err() {
        return Ok(0); // Drive doesn't exist
    }

    let output = std::process::Command::new("powershell")
        .args(&[
            "-NoProfile", "-NonInteractive", "-Command",
            &format!(
                "try {{ $v = Get-PSDrive -Name '{}' -ErrorAction Stop; [math]::Round($v.Free/1GB, 0) }} catch {{ 0 }}",
                &drive_path[..1]
            ),
        ])
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if let Ok(gb) = stdout.parse::<u64>() {
            return Ok(gb);
        }
    }

    // Fallback: assume sufficient space if we can access the drive
    Ok(9999)
}

// ─── Commands ────────────────────────────────────────────────────────────

fn cmd_init(san_path: &Path, cache_dir: &Path) -> Result<()> {
    if !san_path.exists() {
        anyhow::bail!("SAN index not found: {}", san_path.display());
    }

    let local_path = cache_dir.join("utxo-index.redb");

    if local_path.exists() {
        let local_size = fs::metadata(&local_path)?.len();
        let san_size = san_path.metadata()?.len();
        if local_size == san_size {
            println!("Local cache already exists and matches SAN ({:.1} MB).", local_size as f64 / 1_048_576.0);
            println!("Use 'sync' to force update, or delete {} to re-init.", local_path.display());
            return Ok(());
        }
    }

    fs::create_dir_all(cache_dir)?;

    let san_size = san_path.metadata()?.len();
    println!("Copying index from SAN to local disk...");
    println!("  Source: {} ({:.1} MB)", san_path.display(), san_size as f64 / 1_048_576.0);
    println!("  Target: {}", local_path.display());

    let start = Instant::now();
    copy_file_with_progress(san_path, &local_path, san_size)?;
    let elapsed = start.elapsed();

    let local_size = local_path.metadata()?.len();
    println!("\nDone in {:?}!", elapsed);
    println!("  Local cache: {} ({:.1} MB)", local_path.display(), local_size as f64 / 1_048_576.0);
    println!("  Speed: {:.1} MB/s", (san_size as f64 / 1_048_576.0) / elapsed.as_secs_f64());

    // Save metadata
    save_cache_metadata(&local_path, san_path)?;

    Ok(())
}

fn cmd_sync(san_path: &Path, cache_dir: &Path) -> Result<()> {
    let local_path = cache_dir.join("utxo-index.redb");

    if !san_path.exists() {
        anyhow::bail!("SAN index not found: {}", san_path.display());
    }

    if !local_path.exists() {
        println!("No local cache found. Running init...");
        return cmd_init(san_path, cache_dir);
    }

    let san_meta = san_path.metadata()?;
    let local_meta = local_path.metadata()?;

    let san_size = san_meta.len();
    let local_size = local_meta.len();
    let san_modified = san_meta.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs();
    let local_modified = local_meta.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs();

    println!("Cache status:");
    println!("  SAN:  {} ({:.1} MB, modified {})",
        san_path.display(), san_size as f64 / 1_048_576.0, san_modified);
    println!("  Local: {} ({:.1} MB, modified {})",
        local_path.display(), local_size as f64 / 1_048_576.0, local_modified);

    if san_size == local_size && san_modified <= local_modified {
        println!("\nLocal cache is up to date. No sync needed.");
        return Ok(());
    }

    if san_size > local_size || san_modified > local_modified {
        println!("\nSAN has newer data. Syncing...");
        let start = Instant::now();
        copy_file_with_progress(san_path, &local_path, san_size)?;
        let elapsed = start.elapsed();
        println!("Synced in {:?}!", elapsed);
        save_cache_metadata(&local_path, san_path)?;
    }

    Ok(())
}

fn cmd_status(san_path: &Path, cache_dir: &Path) -> Result<()> {
    let local_path = cache_dir.join("utxo-index.redb");

    println!("BTCSolver Cache Status");
    println!("  SAN path:  {}", san_path.display());
    println!("  Cache dir: {}", cache_dir.display());

    if san_path.exists() {
        let meta = san_path.metadata()?;
        println!("  SAN size: {:.1} MB", meta.len() as f64 / 1_048_576.0);
    } else {
        println!("  SAN: NOT FOUND");
    }

    if local_path.exists() {
        let meta = local_path.metadata()?;
        println!("  Local size: {:.1} MB", meta.len() as f64 / 1_048_576.0);
    } else {
        println!("  Local: NOT FOUND (run 'init' to create)");
    }

    // Show metadata if exists
    let meta_path = cache_dir.join("cache-meta.json");
    if meta_path.exists() {
        let content = fs::read_to_string(&meta_path)?;
        println!("  Metadata: {}", content.trim());
    }

    Ok(())
}

// ─── Utilities ───────────────────────────────────────────────────────────

fn copy_file_with_progress(src: &Path, dst: &Path, total_size: u64) -> Result<()> {
    let src_file = fs::File::open(src)?;
    let mut dst_file = fs::File::create(dst)?;

    let mut buffer = std::io::BufReader::with_capacity(1024 * 1024, src_file);
    let mut copied = 0u64;
    let mut last_percent = 0u64;

    loop {
        let buf = buffer.fill_buf()?;
        if buf.is_empty() {
            break;
        }
        let bytes_read = buf.len();
        dst_file.write_all(buf)?;
        buffer.consume(bytes_read);

        let bytes_read = bytes_read as u64;
        copied += bytes_read;

        let percent = (copied * 100) / total_size.max(1);
        if percent > last_percent {
            eprintln!("  {}% ({:.1}/{:.1} MB)",
                percent,
                copied as f64 / 1_048_576.0,
                total_size as f64 / 1_048_576.0);
            last_percent = percent;
        }
    }

    Ok(())
}

fn save_cache_metadata(local_path: &Path, san_path: &Path) -> Result<()> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct Metadata {
        local_path: String,
        san_path: String,
        synced_at: String,
    }

    let meta = Metadata {
        local_path: local_path.display().to_string(),
        san_path: san_path.display().to_string(),
        synced_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    let meta_path = local_path.parent().unwrap().join("cache-meta.json");
    let json = serde_json::to_string_pretty(&meta)?;
    fs::write(&meta_path, json)?;

    Ok(())
}
