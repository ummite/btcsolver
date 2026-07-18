use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;

use crate::dashboard::DashboardConfig;

/// Vitesse par carte GPU (bruteforce live)
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ScanGpuRate {
    pub id: i32,
    pub keys_tested: u64,
    pub keys_per_sec: u64,
    #[serde(default)]
    pub keys_per_sec_avg: u64,
    /// GPU temperature in Celsius (from nvidia-smi)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature_c: Option<f64>,
    /// GPU utilization in percent (from nvidia-smi)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utilization_pct: Option<f64>,
    /// VRAM used in MB
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vram_used_mb: Option<f64>,
    /// VRAM total in MB
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vram_total_mb: Option<f64>,
}

/// Real-time stats from the brute-force scan
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanStats {
    pub running: bool,
    pub pid: Option<u32>,
    pub elapsed_seconds: u64,
    pub keys_per_sec: u64,
    pub keys_tested: u64,
    pub matches_found: u64,
    /// Keys with balance written to found-keys.json (persisted hits)
    pub keys_with_balance: u64,
    pub current_position: Option<String>,
    /// Lowest key hex currently covered (sequential start, or min of thread windows)
    pub range_start: Option<String>,
    /// Highest key hex currently covered (sequential cursor, or max of thread windows)
    pub range_end: Option<String>,
    /// random | sequential
    pub mode: Option<String>,
    /// Active key transforms (identity, reverse_bits, …)
    pub transforms: Vec<String>,
    /// Sample of last keys per thread (short list)
    pub thread_keys: Vec<String>,
    pub gpu_util: Option<f64>,
    pub gpu_temp: Option<f64>,
    pub gpu_vram_used: Option<u64>,
    pub gpu_vram_total: Option<u64>,
    pub ram_mb: Option<f64>,
    pub snapshot_age_hours: Option<f64>,
    /// Libellé humain pour l’UI (ex. « SÉQUENTIEL : départ → curseur »)
    #[serde(default)]
    pub range_summary: Option<String>,
    /// Départ configuré / start_key (hex 64)
    #[serde(default)]
    pub start_key: Option<String>,
    /// Heure locale HH:MM:SS de la dernière écriture stats (prouve que ce n'est pas figé)
    #[serde(default)]
    pub stats_updated_at: Option<String>,
    /// Timestamp RFC3339 du fichier stats
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Vitesse instantanée (fenêtre stats_interval)
    #[serde(default)]
    pub keys_per_sec_live: Option<u64>,
    /// Moyenne depuis démarrage
    #[serde(default)]
    pub keys_per_sec_avg: Option<u64>,
    /// Vitesses par carte GPU
    #[serde(default)]
    pub gpu_rates: Vec<ScanGpuRate>,
    #[serde(default)]
    pub cpu_keys_tested: u64,
    #[serde(default)]
    pub cpu_keys_per_sec: u64,
    #[serde(default)]
    pub cpu_threads: u32,
    /// Fin exclusive de la fenêtre courante (À cible)
    #[serde(default)]
    pub window_end: Option<String>,
    /// Pas d'extension (2^30 par défaut)
    #[serde(default)]
    pub range_step: Option<u64>,
    /// Nombre de plages déjà journalisées
    #[serde(default)]
    pub ranges_done: u32,
}

/// Scan configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanConfig {
    pub use_gpu: bool,
    /// Nombre fixe de workers CPU. **0** = calculer via `cpu_pct` (recommandé).
    pub threads: usize,
    /// Pourcentage des cœurs logiques utilisés comme workers CPU (défaut **50**).
    /// Utilisé seulement si `threads == 0`. 0 % = GPU only (pas de workers CPU).
    #[serde(default = "default_cpu_pct")]
    pub cpu_pct: u32,
    pub batch_size: usize,
    pub start_key: Option<String>,
    /// Nombre de clés de la fenêtre (0 = utiliser `range_step`)
    pub count: u64,
    pub addr_types: String,
    pub stats_interval: u64,
    pub progress_interval: u64,
    /// Random keys (true) or sequential (false)
    #[serde(default)]
    pub random: bool,
    /// Comma list or array of transforms: identity,reverse_bytes,reverse_bits,rotl8,rotr8,sha256,double_sha256
    #[serde(default = "default_transforms")]
    pub transforms: Vec<String>,
    /// GPU CUDA IDs (ex. "0,1,2") — même multi-GPU que le scan listes. Vide = toutes.
    #[serde(default)]
    pub gpus: Option<String>,
    /// Quand le À (fin de fenêtre) est atteint : taille de la plage suivante (défaut 2^30).
    #[serde(default = "default_range_step")]
    pub range_step: u64,
    /// Hex fin de fenêtre exclusive (optionnel, calculé si absent)
    #[serde(default)]
    pub end_key: Option<String>,
    /// Utiliser le journal de plages (ne pas retester) — défaut true en séquentiel
    #[serde(default = "default_true")]
    pub use_range_log: bool,
}

fn default_transforms() -> Vec<String> {
    vec!["identity".to_string()]
}

fn default_cpu_pct() -> u32 {
    50
}

fn default_range_step() -> u64 {
    crate::dashboard::range_log::DEFAULT_RANGE_STEP
}

fn default_true() -> bool {
    true
}

impl ScanConfig {
    /// Résout le nombre de workers CPU : `threads` si > 0, sinon `cpu_pct` des cœurs.
    pub fn resolve_cpu_threads(&self) -> usize {
        if self.threads > 0 {
            return self.threads;
        }
        let cores = num_cpus::get().max(1);
        let pct = self.cpu_pct.min(100) as usize;
        if pct == 0 {
            return 0;
        }
        let n = cores * pct / 100;
        n.max(1)
    }

    /// Cœurs logiques détectés (pour l’UI).
    pub fn logical_cores() -> usize {
        num_cpus::get().max(1)
    }
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            use_gpu: true,
            threads: 0, // 0 = auto via cpu_pct
            cpu_pct: 50, // 50 % des cœurs par défaut
            batch_size: 4_194_304, // gros lot / GPU (~90 % util avec double-buffer)
            start_key: None,
            count: 0, // 0 + séquentiel = range_step (2^30)
            addr_types: "legacy,segwit,wrapped,taproot".to_string(),
            stats_interval: 5,
            progress_interval: 15,
            // Séquentiel par défaut : « De → À » lisible (curseur qui avance)
            random: false,
            transforms: default_transforms(),
            gpus: Some("0,1,2".to_string()),
            range_step: default_range_step(),
            end_key: None,
            use_range_log: true,
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

fn brute_process_alive() -> bool {
    std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq brute_force.exe", "/NH"])
        .output()
        .ok()
        .map(|o| {
            let s = String::from_utf8_lossy(&o.stdout).to_lowercase();
            s.contains("brute_force") && !s.contains("no tasks")
        })
        .unwrap_or(false)
}

impl ScanManager {
    /// Start the brute-force scan
    pub async fn start(state: &DashboardState, config: &ScanConfig) -> Result<u32> {
        // Flag « running » orphelin (process mort sans stop) → débloquer
        if state.scan_running.load(Ordering::SeqCst) {
            if brute_process_alive() {
                return Err(anyhow::anyhow!("Scan already running"));
            }
            eprintln!("[scan] flag running orphelin (pas de brute_force.exe) → reset");
            state.scan_running.store(false, Ordering::SeqCst);
            let mut process = state.scan_process.lock().await;
            *process = None;
        }
        if brute_process_alive() {
            return Err(anyhow::anyhow!(
                "brute_force.exe déjà actif (externe). Stoppez-le d'abord."
            ));
        }

        let candidates = [
            format!(r"{}\brute_force.exe", state.config.bin_dir),
            format!(r"{}\brute_force.exe", state.config.project_dir),
            format!(r"{}\target\release\brute_force.exe", state.config.project_dir),
            format!(r"{}\brute_force_v10_gpu.exe", state.config.project_dir),
        ];
        let bin_path = candidates
            .iter()
            .find(|p| Path::new(p).exists())
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "brute_force.exe not found in bin_dir/project (tried {:?})",
                    candidates
                )
            })?;

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

        // threads=0 + cpu_pct → nombre résolu ici (et passé en clair à brute_force)
        let cpu_workers = config.resolve_cpu_threads();

        // Fenêtre De→À : journal de plages + range_step (défaut 2^30)
        let mut start_hex = config.start_key.clone();
        let mut end_hex = config.end_key.clone();
        let mut count = config.count;
        let step = if config.range_step > 0 {
            config.range_step
        } else {
            crate::dashboard::range_log::DEFAULT_RANGE_STEP
        };

        if !config.random && config.use_range_log {
            use crate::dashboard::range_log::RangeLog;
            let mut log = RangeLog::load(&state.config.project_dir);
            if log.range_step != step {
                log.range_step = step;
            }
            // Préférence : start_key config, sinon manual_start du log
            let preferred = start_hex
                .clone()
                .or_else(|| log.manual_start.clone());
            // Conserver De original si on reprend une fenêtre ouverte
            let prev_start = log.current.as_ref().map(|c| c.start.clone());
            let prev_end = log.current.as_ref().map(|c| c.end.clone());
            let (s, e, keys) = log
                .next_window(preferred.as_deref(), step)
                .map_err(|e| anyhow::anyhow!("range window: {}", e))?;
            start_hex = Some(s.clone());
            end_hex = Some(e.clone());
            if count == 0 {
                count = keys;
            }
            let de = if prev_end.as_deref() == Some(e.as_str()) {
                prev_start.unwrap_or_else(|| s.clone())
            } else {
                s.clone()
            };
            log.open_window(&de, &e, count);
            if let Some(ref mut cur) = log.current {
                cur.cursor = Some(s.clone());
            }
            let _ = log.save(&state.config.project_dir);
            eprintln!(
                "[scan] fenêtre De={} À={} départ_effectif={} ({} clés, step={})",
                de, e, s, count, step
            );
        } else if !config.random && count == 0 {
            // Séquentiel sans journal : une fenêtre de range_step quand même
            count = step;
            if let Some(ref s) = start_hex {
                if let Ok(sk) = crate::dashboard::range_log::RangeLog::normalize_hex(s) {
                    let ek = crate::dashboard::range_log::RangeLog::add_u64(&sk, step);
                    end_hex = Some(crate::dashboard::range_log::RangeLog::hex_encode(&ek));
                }
            }
        }

        let mut args = vec![
            "--snapshot-path".to_string(),
            state.config.snapshot_path.clone(),
            "--threads".to_string(),
            cpu_workers.to_string(),
            "--batch-size".to_string(),
            config.batch_size.to_string(),
            "--count".to_string(),
            count.to_string(),
            "--stats-interval".to_string(),
            config.stats_interval.to_string(),
            "--progress-interval".to_string(),
            config.progress_interval.to_string(),
            "--progress-file".to_string(),
            format!("{}/brute-force-progress.json", state.config.project_dir),
        ];

        if config.use_gpu {
            args.push("--use-gpu".to_string());
            // Même multi-GPU que dict (toutes les cartes, ou liste explicite)
            let gpus = config
                .gpus
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "0,1,2".to_string());
            args.push("--gpus".to_string());
            args.push(gpus);
        }

        if config.random {
            args.push("--random".to_string());
        }

        if let Some(ref start) = start_hex {
            args.push("--start".to_string());
            args.push(start.clone());
        }
        // Note: --end is not a brute_force.exe arg; range length is controlled by --count

        args.push("--addr-types".to_string());
        args.push(config.addr_types.clone());

        let transforms = if config.transforms.is_empty() {
            "identity".to_string()
        } else {
            config.transforms.join(",")
        };
        args.push("--transforms".to_string());
        args.push(transforms);

        // Always disable age gate for UI-launched scans (user controls snapshot)
        args.push("--max-snapshot-age".to_string());
        args.push("0".to_string());

        args.push("--output-file".to_string());
        args.push(format!("{}/found-keys.json", state.config.project_dir));
        args.push("--stats-file".to_string());
        args.push(format!("{}/brute-force-stats.json", state.config.project_dir));

        let mut cmd = Command::new(&bin_path);
        cmd.args(&args)
            .env("BTC_GPU_LAUNCH", "33554432") // 32M keys/GPU call for max throughput
            // CPU workers: stride=15 (3 GPU + 12 CPU) gave 180M/s; stride=3 (GPU only) gave 159M/s
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

        // Quand le process meurt (fin de fenêtre ou crash), libérer le flag
        let flag = Arc::clone(&state.scan_running);
        let proc_slot = Arc::clone(&state.scan_process);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                if !brute_process_alive() {
                    flag.store(false, Ordering::SeqCst);
                    let mut g = proc_slot.lock().await;
                    *g = None;
                    break;
                }
            }
        });

        pid.ok_or_else(|| anyhow::anyhow!("Failed to get PID"))
    }

    /// Stop the running scan
    pub async fn stop(state: &DashboardState) -> Result<()> {
        let mut process = state.scan_process.lock().await;
        if let Some(mut child) = process.take() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        // Tuer aussi un brute externe / orphelin
        let _ = std::process::Command::new("taskkill")
            .args(["/IM", "brute_force.exe", "/F"])
            .output();
        state.scan_running.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Get current scan stats by reading the stats JSON and position file
    pub async fn get_stats(state: &DashboardState) -> Result<ScanStats> {
        // Prefer project_dir (where keep-alive writes), fall back to bin_dir
        let roots = [
            state.config.project_dir.as_str(),
            state.config.bin_dir.as_str(),
        ];
        let stats_file = roots
            .iter()
            .map(|r| format!("{}/brute-force-stats.json", r))
            .find(|p| Path::new(p).exists())
            .unwrap_or_else(|| format!("{}/brute-force-stats.json", state.config.project_dir));
        let progress_file = roots
            .iter()
            .map(|r| format!("{}/brute-force-progress.json", r))
            .find(|p| Path::new(p).exists())
            .unwrap_or_else(|| format!("{}/brute-force-progress.json", state.config.project_dir));
        let position_file = progress_file.replace(".json", ".position");

        // Source de vérité = process réel (évite UI « ON » avec k/s = 0)
        let alive = brute_process_alive();
        if state.scan_running.load(Ordering::SeqCst) && !alive {
            state.scan_running.store(false, Ordering::SeqCst);
        }
        let is_running = alive || state.scan_running.load(Ordering::SeqCst);

        let mut stats = ScanStats {
            running: is_running,
            pid: None,
            elapsed_seconds: 0,
            keys_per_sec: 0,
            keys_tested: 0,
            matches_found: 0,
            keys_with_balance: 0,
            current_position: None,
            range_start: None,
            range_end: None,
            mode: None,
            transforms: Vec::new(),
            thread_keys: Vec::new(),
            gpu_util: None,
            gpu_temp: None,
            gpu_vram_used: None,
            gpu_vram_total: None,
            ram_mb: None,
            snapshot_age_hours: None,
            range_summary: None,
            start_key: None,
            stats_updated_at: None,
            timestamp: None,
            keys_per_sec_live: None,
            keys_per_sec_avg: None,
            gpu_rates: Vec::new(),
            cpu_keys_tested: 0,
            cpu_keys_per_sec: 0,
            cpu_threads: 0,
            window_end: None,
            range_step: None,
            ranges_done: 0,
        };

        // Read stats JSON (may include range/transforms if new brute_force)
        if let Ok(content) = std::fs::read_to_string(&stats_file) {
            if let Ok(s) = serde_json::from_str::<serde_json::Value>(&content) {
                stats.elapsed_seconds = s.get("elapsed_seconds").and_then(|v| v.as_u64()).unwrap_or(0);
                stats.keys_per_sec = s.get("keys_per_sec").and_then(|v| v.as_u64()).unwrap_or(0);
                stats.keys_tested = s.get("keys_tested").and_then(|v| v.as_u64()).unwrap_or(0);
                stats.matches_found = s.get("matches_found").and_then(|v| v.as_u64()).unwrap_or(0);
                if let Some(m) = s.get("mode").and_then(|v| v.as_str()) {
                    stats.mode = Some(m.to_string());
                }
                if let Some(sk) = s.get("start_key").and_then(|v| v.as_str()) {
                    stats.start_key = Some(sk.to_lowercase());
                }
                if let Some(rs) = s.get("range_start").and_then(|v| v.as_str()) {
                    stats.range_start = Some(rs.to_lowercase());
                }
                if let Some(re) = s.get("range_end").and_then(|v| v.as_str()) {
                    stats.range_end = Some(re.to_lowercase());
                }
                if let Some(cp) = s.get("current_position").and_then(|v| v.as_str()) {
                    stats.current_position = Some(cp.to_lowercase());
                }
                if let Some(arr) = s.get("transforms").and_then(|v| v.as_array()) {
                    stats.transforms = arr
                        .iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect();
                } else if let Some(t) = s.get("transforms").and_then(|v| v.as_str()) {
                    stats.transforms = t
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                if let Some(t) = s.get("stats_updated_at").and_then(|v| v.as_str()) {
                    stats.stats_updated_at = Some(t.to_string());
                }
                if let Some(t) = s.get("timestamp").and_then(|v| v.as_str()) {
                    stats.timestamp = Some(t.to_string());
                }
                stats.keys_per_sec_live = s.get("keys_per_sec_live").and_then(|v| v.as_u64());
                stats.keys_per_sec_avg = s.get("keys_per_sec_avg").and_then(|v| v.as_u64());
                stats.cpu_keys_tested = s.get("cpu_keys_tested").and_then(|v| v.as_u64()).unwrap_or(0);
                stats.cpu_keys_per_sec = s
                    .get("cpu_keys_per_sec")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                stats.cpu_threads = s
                    .get("cpu_threads")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                if let Some(arr) = s.get("gpu_rates").and_then(|v| v.as_array()) {
                    stats.gpu_rates = arr
                        .iter()
                        .filter_map(|g| {
                            Some(ScanGpuRate {
                                id: g.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                keys_tested: g
                                    .get("keys_tested")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                                keys_per_sec: g
                                    .get("keys_per_sec")
                                    .and_then(|v| v.as_u64())
                                    .or_else(|| {
                                        g.get("keys_per_sec")
                                            .and_then(|v| v.as_f64())
                                            .map(|f| f.round() as u64)
                                    })
                                    .unwrap_or(0),
                                keys_per_sec_avg: g
                                    .get("keys_per_sec_avg")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                                temperature_c: g.get("temperature_c").and_then(|v| v.as_f64()),
                                utilization_pct: g.get("utilization_pct").and_then(|v| v.as_f64()),
                                vram_used_mb: g.get("vram_used_mb").and_then(|v| v.as_f64()),
                                vram_total_mb: g.get("vram_total_mb").and_then(|v| v.as_f64()),
                            })
                        })
                        .collect();
                }
                // thread_keys from stats if progress lags
                if let Some(arr) = s.get("thread_keys").and_then(|v| v.as_array()) {
                    let keys: Vec<String> = arr
                        .iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_lowercase()))
                        .filter(|s| s.len() == 64)
                        .collect();
                    if !keys.is_empty() {
                        stats.thread_keys = keys.into_iter().take(12).collect();
                    }
                }
            }
        }

        // Read rich progress JSON → range + thread keys
        if let Ok(content) = std::fs::read_to_string(&progress_file) {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&content) {
                if stats.mode.is_none() {
                    if let Some(m) = p.get("mode").and_then(|v| v.as_str()) {
                        stats.mode = Some(m.to_string());
                    }
                }
                if let Some(threads) = p.get("threads").and_then(|v| v.as_array()) {
                    let mut keys: Vec<String> = threads
                        .iter()
                        .filter_map(|t| {
                            t.get("last_key_hex")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_lowercase())
                                .filter(|s| s.len() == 64)
                        })
                        .collect();
                    keys.sort();
                    if let (Some(first), Some(last)) = (keys.first(), keys.last()) {
                        if stats.range_start.is_none() {
                            stats.range_start = Some(first.clone());
                        }
                        if stats.range_end.is_none() {
                            stats.range_end = Some(last.clone());
                        }
                        if stats.current_position.is_none() {
                            stats.current_position = Some(last.clone());
                        }
                    }
                    stats.thread_keys = keys.into_iter().take(8).collect();
                }
                // Sequential start may be stored
                if let Some(sk) = p.get("start_key").and_then(|v| v.as_str()) {
                    if stats.mode.as_deref() == Some("sequential") || stats.mode.is_none() {
                        stats.range_start = Some(sk.to_lowercase());
                    }
                }
            }
        }

        // Read position file (sequential: "sequential <hex> <count>")
        if let Ok(content) = std::fs::read_to_string(&position_file) {
            let lines: Vec<&str> = content.lines().collect();
            if let Some(line) = lines.first() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                // formats: "sequential HEX count" or raw hex
                if parts.len() >= 2 && parts[0].eq_ignore_ascii_case("sequential") {
                    stats.mode = Some("sequential".to_string());
                    stats.current_position = Some(parts[1].to_lowercase());
                    stats.range_end = Some(parts[1].to_lowercase());
                    if stats.range_start.is_none() {
                        stats.range_start = Some(
                            "0000000000000000000000000000000000000000000000000000000000000001"
                                .to_string(),
                        );
                    }
                } else if !line.is_empty() {
                    stats.current_position = Some(line.trim().to_string());
                }
            }
        }

        // Default transforms display if brute didn't report yet
        if stats.transforms.is_empty() {
            let cfg = state.scan_config.lock().await;
            stats.transforms = if cfg.transforms.is_empty() {
                vec!["identity".to_string()]
            } else {
                cfg.transforms.clone()
            };
        }

        // Journal de plages : De/À = fenêtre courante (hex complets), curseur à part
        {
            use crate::dashboard::range_log::RangeLog;
            let log = RangeLog::load(&state.config.project_dir);
            stats.range_step = Some(log.range_step);
            stats.ranges_done = log.ranges.len() as u32;
            if let Some(ref cur) = log.current {
                stats.range_start = Some(cur.start.clone());
                stats.window_end = Some(cur.end.clone());
                // À affiché = fin de fenêtre (cible), pas le curseur
                stats.range_end = Some(cur.end.clone());
                if stats.start_key.is_none() {
                    stats.start_key = Some(cur.start.clone());
                }
                if let Some(ref c) = cur.cursor {
                    if stats.current_position.is_none() {
                        stats.current_position = Some(c.clone());
                    }
                }
            } else if let Some(ref ms) = log.manual_start {
                if stats.range_start.is_none() {
                    stats.range_start = Some(ms.clone());
                }
            }
        }

        // Séquentiel : De = départ fenêtre, À = fin fenêtre (step)
        let mode = stats.mode.as_deref().unwrap_or("");
        if mode == "sequential" {
            let live = stats
                .keys_per_sec_live
                .unwrap_or(stats.keys_per_sec);
            let maj = stats
                .stats_updated_at
                .as_deref()
                .unwrap_or("—");
            let step = stats.range_step.unwrap_or(0);
            stats.range_summary = Some(format!(
                "SÉQUENTIEL · fenêtre 2^{:.0} ({} clés) · {} k/s live · {} testées · {} plages done · MAJ {}",
                (step as f64).log2().max(0.0),
                step,
                live,
                stats.keys_tested,
                stats.ranges_done,
                maj
            ));
        } else if mode == "random" {
            let live = stats
                .keys_per_sec_live
                .unwrap_or(stats.keys_per_sec);
            let maj = stats
                .stats_updated_at
                .as_deref()
                .unwrap_or("—");
            stats.range_summary = Some(format!(
                "RANDOM · fenêtre min→max ({}) · {} k/s live · {} testées · MAJ {}",
                stats.thread_keys.len(),
                live,
                stats.keys_tested,
                maj
            ));
        } else if stats.running {
            stats.range_summary = Some("Scan actif — attente premières stats…".into());
        } else {
            stats.range_summary = Some("Scan arrêté".into());
        }

        // Count archived keys (activity + balance) then fall back to found-keys.json
        let archive_path = format!(
            "{}/data/keys-archive.json",
            state.config.project_dir
        );
        let mut counted = false;
        if let Ok(content) = std::fs::read_to_string(&archive_path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                let n = v
                    .get("count")
                    .and_then(|x| x.as_u64())
                    .or_else(|| v.get("keys").and_then(|x| x.as_array()).map(|a| a.len() as u64))
                    .unwrap_or(0);
                let with_bal = v
                    .get("with_balance")
                    .and_then(|x| x.as_u64())
                    .unwrap_or(n);
                stats.keys_with_balance = with_bal;
                if stats.matches_found < n {
                    stats.matches_found = n;
                }
                counted = true;
            }
        }
        if !counted {
            let found_paths = [
                format!("{}/found-keys.json", state.config.project_dir),
                format!("{}/data/found-keys.json", state.config.project_dir),
                format!("{}/found-keys.json", state.config.bin_dir),
                format!("{}/target/release/found-keys.json", state.config.project_dir),
            ];
            for fp in &found_paths {
                if let Ok(content) = std::fs::read_to_string(fp) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                        let n = if let Some(arr) = v.as_array() {
                            arr.len() as u64
                        } else if let Some(arr) = v.get("keys").and_then(|x| x.as_array()) {
                            arr.len() as u64
                        } else if let Some(arr) = v.get("found").and_then(|x| x.as_array()) {
                            arr.len() as u64
                        } else {
                            0
                        };
                        stats.keys_with_balance = n;
                        if stats.matches_found < n {
                            stats.matches_found = n;
                        }
                        break;
                    }
                }
            }
        }
        if stats.keys_with_balance == 0 {
            stats.keys_with_balance = stats.matches_found;
        }

        // Check snapshot age
        if let Ok(meta) = std::fs::metadata(&state.config.snapshot_path) {
            if let Ok(modified) = meta.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    stats.snapshot_age_hours = Some(elapsed.as_secs() as f64 / 3600.0);
                }
            }
        }

        // nvidia-smi only when a scan is actually running (avoid spawn every poll)
        if is_running {
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
                    // Aggregate: max util / sum VRAM across all GPUs
                    let mut max_util = 0.0f64;
                    let mut max_temp = 0.0f64;
                    let mut mem_used = 0.0f64;
                    let mut mem_total = 0.0f64;
                    // Per-GPU data
                    let mut gpu_temps: Vec<(i32, f64, f64, f64, f64)> = Vec::new();
                    let mut gpu_idx: i32 = 0;
                    for line in stdout.lines() {
                        let parts: Vec<&str> = line.split(',').collect();
                        if parts.len() >= 4 {
                            let u: f64 = parts[0].trim().parse().unwrap_or(0.0);
                            let t: f64 = parts[1].trim().parse().unwrap_or(0.0);
                            let mu: f64 = parts[2].trim().parse().unwrap_or(0.0);
                            let mt: f64 = parts[3].trim().parse().unwrap_or(0.0);
                            if u > max_util {
                                max_util = u;
                            }
                            if t > max_temp {
                                max_temp = t;
                            }
                            mem_used += mu;
                            mem_total += mt;
                            gpu_temps.push((gpu_idx, t, u, mu, mt));
                            gpu_idx += 1;
                        }
                    }
                    stats.gpu_util = Some(max_util);
                    stats.gpu_temp = Some(max_temp);
                    stats.gpu_vram_used = Some(mem_used as u64);
                    stats.gpu_vram_total = Some(mem_total as u64);

                    // Merge per-GPU temps into gpu_rates
                    for (idx, temp, util, vram_u, vram_t) in gpu_temps {
                        if let Some(gr) = stats.gpu_rates.iter_mut().find(|g| g.id == idx) {
                            gr.temperature_c = Some(temp);
                            gr.utilization_pct = Some(util);
                            gr.vram_used_mb = Some(vram_u);
                            gr.vram_total_mb = Some(vram_t);
                        } else {
                            // GPU exists in nvidia-smi but not in stats gpu_rates — add it
                            stats.gpu_rates.push(ScanGpuRate {
                                id: idx,
                                keys_tested: 0,
                                keys_per_sec: 0,
                                keys_per_sec_avg: 0,
                                temperature_c: Some(temp),
                                utilization_pct: Some(util),
                                vram_used_mb: Some(vram_u),
                                vram_total_mb: Some(vram_t),
                            });
                        }
                    }
                }
            }

            #[cfg(windows)]
            {
                // Lightweight: no PowerShell spawn
                if let Ok(output) = Command::new("tasklist")
                    .args(["/FI", "IMAGENAME eq brute_force.exe", "/FO", "CSV", "/NH"])
                    .output()
                    .await
                {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        // CSV: "brute_force.exe","pid","session","n","1,234 K"
                        if let Some(mem_field) = stdout.split(',').nth(4) {
                            let digits: String = mem_field
                                .chars()
                                .filter(|c| c.is_ascii_digit())
                                .collect();
                            if let Ok(kb) = digits.parse::<u64>() {
                                stats.ram_mb = Some(kb as f64 / 1024.0);
                            }
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
