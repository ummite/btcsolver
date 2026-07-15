use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;

use crate::dashboard::DashboardConfig;

/// Real-time stats from the brute-force scan
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanStats {
    pub running: bool,
    pub pid: Option<u32>,
    pub elapsed_seconds: u64,
    pub keys_per_sec: u64,
    pub keys_tested: u64,
    pub matches_found: u64,
    pub current_position: Option<String>,
    pub gpu_util: Option<f64>,
    pub gpu_temp: Option<f64>,
    pub gpu_vram_used: Option<u64>,
    pub gpu_vram_total: Option<u64>,
    pub ram_mb: Option<f64>,
    pub snapshot_age_hours: Option<f64>,
}

/// Scan configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanConfig {
    pub use_gpu: bool,
    pub threads: usize,
    pub batch_size: usize,
    pub start_key: Option<String>,
    pub count: u64,
    pub addr_types: String,
    pub stats_interval: u64,
    pub progress_interval: u64,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            use_gpu: true,
            threads: 23,
            batch_size: 256000,
            start_key: None,
            count: 0,
            addr_types: "legacy,segwit,wrapped,taproot".to_string(),
            stats_interval: 10,
            progress_interval: 30,
        }
    }
}

/// Shared dashboard state
#[derive(Clone)]
pub struct DashboardState {
    pub config: DashboardConfig,
    pub scan_running: Arc<AtomicBool>,
    pub scan_process: Arc<Mutex<Option<tokio::process::Child>>>,
    pub scan_config: Arc<Mutex<ScanConfig>>,
}

impl DashboardState {
    pub fn new(config: DashboardConfig) -> Self {
        Self {
            config,
            scan_running: Arc::new(AtomicBool::new(false)),
            scan_process: Arc::new(Mutex::new(None)),
            scan_config: Arc::new(Mutex::new(ScanConfig::default())),
        }
    }
}

pub struct ScanManager;

impl ScanManager {
    /// Start the brute-force scan
    pub async fn start(state: &DashboardState, config: &ScanConfig) -> Result<u32> {
        if state.scan_running.load(Ordering::SeqCst) {
            return Err(anyhow::anyhow!("Scan already running"));
        }

        let bin_path = format!("{}/brute_force.exe", state.config.bin_dir);
        if !Path::new(&bin_path).exists() {
            return Err(anyhow::anyhow!("brute_force.exe not found at {}", bin_path));
        }

        // Check snapshot freshness
        if state.config.max_snapshot_age_seconds > 0 {
            let meta = std::fs::metadata(&state.config.snapshot_path)?;
            let age = meta.modified()?.elapsed()?.as_secs();
            if age > state.config.max_snapshot_age_seconds {
                return Err(anyhow::anyhow!(
                    "Snapshot too old ({}s > {}s). Regenerate or use --max-snapshot-age 0",
                    age, state.config.max_snapshot_age_seconds
                ));
            }
        }

        let mut args = vec![
            "--snapshot-path".to_string(),
            state.config.snapshot_path.clone(),
            "--threads".to_string(),
            config.threads.to_string(),
            "--batch-size".to_string(),
            config.batch_size.to_string(),
            "--count".to_string(),
            config.count.to_string(),
            "--stats-interval".to_string(),
            config.stats_interval.to_string(),
            "--progress-interval".to_string(),
            config.progress_interval.to_string(),
            "--progress-file".to_string(),
            format!("{}/brute-force-progress.json", state.config.bin_dir),
        ];

        if config.use_gpu {
            args.push("--use-gpu".to_string());
        }

        if let Some(ref start) = config.start_key {
            args.push("--start".to_string());
            args.push(start.clone());
        }

        args.push("--addr-types".to_string());
        args.push(config.addr_types.clone());

        let mut cmd = Command::new(&bin_path);
        cmd.args(&args)
            .kill_on_drop(true)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn()?;
        let pid = child.id();

        // Spawn background task to read stdout/stderr
        let stdout = child.stdout.take().unwrap();
        tokio::spawn(async move {
            use tokio::io::AsyncBufReadExt;
            let reader = tokio::io::BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                eprintln!("[brute_force] {}", line);
            }
        });

        state.scan_running.store(true, Ordering::SeqCst);
        let mut process = state.scan_process.lock().await;
        *process = Some(child);

        pid.ok_or_else(|| anyhow::anyhow!("Failed to get PID"))
    }

    /// Stop the running scan
    pub async fn stop(state: &DashboardState) -> Result<()> {
        let mut process = state.scan_process.lock().await;
        if let Some(mut child) = process.take() {
            child.kill().await?;
            let _ = child.wait().await;
        }
        state.scan_running.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Get current scan stats by reading the stats JSON and position file
    pub async fn get_stats(state: &DashboardState) -> Result<ScanStats> {
        let stats_file = format!("{}/brute-force-stats.json", state.config.bin_dir);
        let position_file = format!("{}/brute-force-progress.position", state.config.bin_dir);

        let is_running = state.scan_running.load(Ordering::SeqCst);

        let mut stats = ScanStats {
            running: is_running,
            pid: None,
            elapsed_seconds: 0,
            keys_per_sec: 0,
            keys_tested: 0,
            matches_found: 0,
            current_position: None,
            gpu_util: None,
            gpu_temp: None,
            gpu_vram_used: None,
            gpu_vram_total: None,
            ram_mb: None,
            snapshot_age_hours: None,
        };

        // Read stats JSON
        if let Ok(content) = std::fs::read_to_string(&stats_file) {
            if let Ok(s) = serde_json::from_str::<serde_json::Value>(&content) {
                stats.elapsed_seconds = s.get("elapsed_seconds").and_then(|v| v.as_u64()).unwrap_or(0);
                stats.keys_per_sec = s.get("keys_per_sec").and_then(|v| v.as_u64()).unwrap_or(0);
                stats.keys_tested = s.get("keys_tested").and_then(|v| v.as_u64()).unwrap_or(0);
                stats.matches_found = s.get("matches_found").and_then(|v| v.as_u64()).unwrap_or(0);
            }
        }

        // Read position file
        if let Ok(content) = std::fs::read_to_string(&position_file) {
            let lines: Vec<&str> = content.lines().collect();
            if !lines.is_empty() {
                stats.current_position = Some(lines[0].to_string());
            }
        }

        // Check snapshot age
        if let Ok(meta) = std::fs::metadata(&state.config.snapshot_path) {
            if let Ok(modified) = meta.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    stats.snapshot_age_hours = Some(elapsed.as_secs() as f64 / 3600.0);
                }
            }
        }

        // Query nvidia-smi if available
        if let Ok(output) = Command::new("nvidia-smi")
            .args([
                "--query-gpu=utilization.gpu,temperature.gpu,memory.used,memory.total",
                "--format=csv,noheader,nounits",
            ])
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = stdout.lines().next() {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 4 {
                        stats.gpu_util = parts[0].trim().parse().ok();
                        stats.gpu_temp = parts[1].trim().parse().ok();
                        stats.gpu_vram_used = parts[2].trim().parse().ok();
                        stats.gpu_vram_total = parts[3].trim().parse().ok();
                    }
                }
            }
        }

        // Get RAM usage from process
        if is_running {
            #[cfg(windows)]
            {
                if let Ok(output) = Command::new("powershell")
                    .args([
                        "-NoProfile", "-Command",
                        "Get-Process brute_force -ErrorAction SilentlyContinue | Select-Object -ExpandProperty WorkingSet64",
                    ])
                    .output()
                    .await
                {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if let Ok(bytes) = stdout.trim().parse::<u64>() {
                            stats.ram_mb = Some(bytes as f64 / 1_048_576.0);
                        }
                    }
                }
            }
        }

        Ok(stats)
    }

    /// Get the current scan config
    pub async fn get_config(state: &DashboardState) -> ScanConfig {
        state.scan_config.lock().await.clone()
    }

    /// Update the scan config (applied on next start)
    pub async fn update_config(state: &DashboardState, config: ScanConfig) {
        let mut current = state.scan_config.lock().await;
        *current = config;
    }
}
