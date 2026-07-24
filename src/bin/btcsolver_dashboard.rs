use anyhow::Result;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Json};
use axum::routing::{get, post};
use axum::Router;
use clap::Parser;
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::io::BufRead;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

use btcsolver::dashboard::bitcoind::BitcoindManager;
use btcsolver::dashboard::dict_scan::{DictScanManager, DictScanRequest, DictScanState};
use btcsolver::dashboard::key_checker::{BrainwalletOptions, KeyChecker, KeyFormat};
use btcsolver::dashboard::scan_manager::{ScanConfig, ScanManager};
use btcsolver::dashboard::{DashboardConfig, DashboardState};
use btcsolver::flat_index::FlatIndex;
use btcsolver::key_archive::{pubkeys_from_priv_hex, ArchivedKey, KeyArchive};

/// BTC Solver Dashboard — pro control center
#[derive(Parser, Debug)]
#[command(name = "btcsolver_dashboard", version, about)]
struct Cli {
    #[arg(short, long, default_value = "3000")]
    port: u16,

    #[arg(long, default_value = r"W:\Bitcoin")]
    bitcoin_datadir: String,

    #[arg(long)]
    bitcoind_path: Option<String>,

    #[arg(long, default_value = "http://127.0.0.1:8332")]
    rpc_url: String,

    #[arg(long, default_value = "btcsolver")]
    rpc_user: String,

    #[arg(long, default_value = "btcsolver_rpc_2026")]
    rpc_password: String,

    #[arg(long, default_value = r"W:\Bitcoin\blocks")]
    blocks_dir: String,

    /// XOR key for blocks (000… for plaintext W: blocks)
    #[arg(long, default_value = "0000000000000000")]
    blocks_obf_key: String,

    #[arg(long)]
    snapshot_path: Option<String>,

    #[arg(long, default_value = r"Y:\btcsolver")]
    bin_dir: String,

    #[arg(long, default_value = r"Y:\btcsolver")]
    cache_dir: String,

    #[arg(long, default_value = r"Y:\btcsolver")]
    project_dir: String,

    #[arg(long, default_value = "604800")]
    max_snapshot_age: u64,

    #[arg(long, default_value = "0")]
    snapshot_interval_hours: u64,

    /// Auto-restart bitcoind when process dies (watchdog).
    /// Pass `--auto-restart-bitcoind false` on machines without Bitcoin Core.
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    auto_restart_bitcoind: bool,

    #[arg(long, default_value = "30")]
    auto_restart_check_secs: u64,

    #[arg(long, default_value = "static/dashboard")]
    static_dir: String,
}

#[derive(Clone)]
struct AppState {
    dashboard: DashboardState,
    config: DashboardConfig,
    index: Arc<RwLock<Option<Arc<FlatIndex>>>>,
    dict: DictScanState,
    corpus: CorpusScanState,
    benchmark: BenchmarkState,
    utxo_rebuild: UtxoRebuildState,
}

/// Background corpus scan state — tracks easy keys scan progress
#[derive(Clone)]
struct CorpusScanState {
    running: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    keys_tested: Arc<AtomicU64>,
    keys_total: Arc<AtomicU64>,
    matches_found: Arc<AtomicU64>,
    status_text: Arc<Mutex<String>>,
}

impl CorpusScanState {
    fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            stop: Arc::new(AtomicBool::new(false)),
            keys_tested: Arc::new(AtomicU64::new(0)),
            keys_total: Arc::new(AtomicU64::new(0)),
            matches_found: Arc::new(AtomicU64::new(0)),
            status_text: Arc::new(Mutex::new(String::new())),
        }
    }
}

/// Benchmark result for a single configuration
#[derive(serde::Serialize, Clone)]
struct BenchmarkEntry {
    /// Label (e.g. "GPU×3 + CPU 8 threads")
    label: String,
    /// CPU threads used (0 = GPU only)
    cpu_threads: usize,
    /// GPU batch size in millions
    gpu_batch_m: usize,
    /// Keys per second achieved
    keys_per_sec: u64,
    /// Keys per second per GPU (if GPU used)
    keys_per_sec_per_gpu: f64,
    /// Duration of the test in seconds
    test_duration_secs: u64,
    /// Total keys tested
    total_keys: u64,
}

/// Benchmark runner state
#[derive(Clone)]
struct BenchmarkState {
    running: Arc<AtomicBool>,
    progress: Arc<Mutex<Option<serde_json::Value>>>,
    results: Arc<Mutex<Vec<BenchmarkEntry>>>,
}

impl BenchmarkState {
    fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            progress: Arc::new(Mutex::new(None)),
            results: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// Background UTXO rebuild state — tracks long-running rebuild operations
#[derive(Clone)]
struct UtxoRebuildState {
    running: Arc<AtomicBool>,
    progress: Arc<Mutex<Option<serde_json::Value>>>,
}

impl UtxoRebuildState {
    fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            progress: Arc::new(Mutex::new(None)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    let project = cli.project_dir.clone();

    // Prefer working directory project for static files
    let static_dir = if Path::new(&cli.static_dir).exists() {
        cli.static_dir.clone()
    } else {
        format!(r"{}\static\dashboard", project)
    };

    let snapshot_path = cli.snapshot_path.unwrap_or_else(|| {
        // Prefer largest existing snapshot (more scripts ≈ better coverage for key hunting)
        let candidates = [
            format!(r"{}\data\utxo-day-935000.snapshot", project),
            format!(r"{}\utxo-index.snapshot", project),
            format!(r"{}\utxo-index.snapshot", cli.cache_dir),
            format!(r"{}\data\utxo-index.snapshot", project),
        ];
        let mut best: Option<(u64, String)> = None;
        for c in candidates {
            if let Ok(meta) = std::fs::metadata(&c) {
                let len = meta.len();
                if best.as_ref().map(|(l, _)| len > *l).unwrap_or(true) {
                    best = Some((len, c));
                }
            }
        }
        best.map(|(_, p)| p)
            .unwrap_or_else(|| format!(r"{}\utxo-index.snapshot", project))
    });
    tracing::info!("Using UTXO snapshot: {}", snapshot_path);

    let bitcoind_path = cli.bitcoind_path.or_else(|| {
        let c = r"W:\Bitcoin\bin\daemon\bitcoind.exe";
        if Path::new(c).exists() {
            Some(c.to_string())
        } else {
            None
        }
    });

    let config = DashboardConfig {
        bin_dir: cli.bin_dir,
        cache_dir: cli.cache_dir,
        project_dir: project.clone(),
        bitcoin_datadir: cli.bitcoin_datadir,
        bitcoind_path,
        bitcoin_cli_path: Some(r"W:\Bitcoin\bin\daemon\bitcoin-cli.exe".to_string()),
        bitcoin_rpc_url: Some(cli.rpc_url),
        bitcoin_rpc_user: Some(cli.rpc_user),
        bitcoin_rpc_password: Some(cli.rpc_password),
        blocks_dir: cli.blocks_dir,
        blocks_obf_key: cli.blocks_obf_key,
        snapshot_path: snapshot_path.clone(),
        redb_path: format!(r"{}\utxo-index.redb", project),
        max_snapshot_age_seconds: cli.max_snapshot_age,
        auto_snapshot_interval_hours: cli.snapshot_interval_hours,
        auto_restart_bitcoind: cli.auto_restart_bitcoind,
        auto_restart_check_secs: cli.auto_restart_check_secs.max(10),
        default_threads: (num_cpus::get().saturating_sub(1)).max(1),
        default_batch_size: 25_600_000,
    };

    let state = DashboardState::new(config.clone());

    // Boot: start bitcoind immediately if down (best effort, non-blocking)
    {
        let cfg = config.clone();
        tokio::spawn(async move {
            match BitcoindManager::ensure_running(&cfg).await {
                Ok(Some(msg)) => tracing::info!("boot ensure: {}", msg),
                Ok(None) => tracing::info!("boot ensure: bitcoind already running"),
                Err(e) => tracing::error!("boot ensure failed: {}", e),
            }
        });
    }

    // Watchdog: relance bitcoind s'il tombe (sans toucher un process encore vivant)
    if config.auto_restart_bitcoind {
        let cfg = config.clone();
        let interval = Duration::from_secs(cfg.auto_restart_check_secs);
        tokio::spawn(async move {
            tracing::info!(
                "bitcoind watchdog ON (every {}s)",
                cfg.auto_restart_check_secs
            );
            loop {
                tokio::time::sleep(interval).await;
                match BitcoindManager::ensure_running(&cfg).await {
                    Ok(Some(msg)) => tracing::warn!("{}", msg),
                    Ok(None) => {}
                    Err(e) => tracing::error!("watchdog ensure_running: {}", e),
                }
            }
        });
    }

    // Load index in background so HTTP comes up immediately
    let index: Arc<RwLock<Option<Arc<FlatIndex>>>> = Arc::new(RwLock::new(None));
    {
        let path = config.snapshot_path.clone();
        let project_dir = config.project_dir.clone();
        let index_bg = index.clone();
        tokio::spawn(async move {
            let need = btcsolver::sys_info::utxo_size_from_path(&path);
            let (ok, msg) = btcsolver::sys_info::ram_gate_message(need);
            if !ok {
                tracing::warn!("UTXO index load deferred: {}", msg);
                let _ = std::fs::create_dir_all(format!("{}/data", project_dir));
                let _ = std::fs::write(
                    format!("{}/data/scan-ram-pause.txt", project_dir),
                    format!("{}\nutxo_bytes={}\n", msg, need),
                );
                return;
            }
            tracing::info!("Loading UTXO index in background: {}", path);
            let loaded = tokio::task::spawn_blocking(move || load_index(&path))
                .await
                .ok()
                .flatten();
            if loaded.is_some() {
                tracing::info!("Background UTXO index ready");
                let _ = std::fs::remove_file(format!("{}/data/scan-ram-pause.txt", project_dir));
            } else {
                tracing::warn!("Background UTXO index failed or missing");
            }
            *index_bg.write().await = loaded;
        });
    }

    if config.auto_snapshot_interval_hours > 0 {
        let cfg = config.clone();
        tokio::spawn(async move {
            let interval = Duration::from_secs(cfg.auto_snapshot_interval_hours * 3600);
            loop {
                tokio::time::sleep(interval).await;
                match BitcoindManager::generate_snapshot(&cfg).await {
                    Ok(msg) => tracing::info!("Auto snapshot: {}", msg),
                    Err(e) => tracing::error!("Auto snapshot failed: {}", e),
                }
            }
        });
    }

    let app_state = AppState {
        dashboard: state,
        config,
        index,
        dict: DictScanState::new(),
        corpus: CorpusScanState::new(),
        benchmark: BenchmarkState::new(),
        utxo_rebuild: UtxoRebuildState::new(),
    };

    // Scan auto (brute GPU) : tourne dès que le dict n’utilise pas les GPU
    {
        let dash = app_state.dashboard.clone();
        let dict = app_state.dict.clone();
        let project = app_state.config.project_dir.clone();
        tokio::spawn(async move {
            tracing::info!("auto-scan idle: brute GPU when dict not running");
            tokio::time::sleep(Duration::from_secs(25)).await;
            let mut last_pos: Option<String> = None;
            let mut stuck_ticks: u32 = 0;
            // Crash loop detection: track spawn/death times to detect rapid crashes
            let mut rapid_crash_count: u32 = 0;
            let mut last_spawn_time: Option<std::time::Instant> = None;
            let mut last_death_time: Option<std::time::Instant> = None;
            loop {
                tokio::time::sleep(Duration::from_secs(15)).await;
                let dict_busy = dict.running.load(Ordering::SeqCst);
                let stats = ScanManager::get_stats(&dash).await.ok();
                let brute_on = stats.as_ref().map(|s| s.running).unwrap_or(false)
                    || std::process::Command::new("tasklist")
                        .args(["/FI", "IMAGENAME eq brute_force.exe", "/NH"])
                        .output()
                        .ok()
                        .map(|o| {
                            let s = String::from_utf8_lossy(&o.stdout).to_lowercase();
                            s.contains("brute_force") && !s.contains("no tasks")
                        })
                        .unwrap_or(false);

                if dict_busy {
                    stuck_ticks = 0;
                    last_pos = None;
                    if brute_on {
                        tracing::info!("auto-scan: dict busy → stop brute (libère GPU)");
                        let _ = ScanManager::stop(&dash).await;
                        // tuer aussi un brute externe (watchdog)
                        let _ = std::process::Command::new("taskkill")
                            .args(["/IM", "brute_force.exe", "/F"])
                            .output();
                    }
                    continue;
                }

                let prio = std::path::Path::new(&project)
                    .join("data")
                    .join("PRIORITY-SYNC.flag");
                if prio.exists() {
                    continue;
                }

                // Watchdog : process « running » mais curseur figé → zombie (ex. hang FULL load)
                if brute_on {
                    let pos = stats
                        .as_ref()
                        .and_then(|s| s.current_position.clone())
                        .or_else(|| {
                            stats.as_ref().and_then(|s| s.range_end.clone())
                        });
                    let kps = stats.as_ref().map(|s| s.keys_per_sec).unwrap_or(0);
                    if let Some(ref p) = pos {
                        if last_pos.as_ref() == Some(p) && kps == 0 {
                            stuck_ticks += 1;
                        } else if last_pos.as_ref() == Some(p) && stuck_ticks > 0 {
                            // même position mais kps non nul (stats stale) — encore collé
                            stuck_ticks += 1;
                        } else {
                            stuck_ticks = 0;
                        }
                        last_pos = Some(p.clone());
                    }
                    // 8 × 15s = 120s sans avancement → relance (laisse le temps au chargement index + GPU init)
                    if stuck_ticks >= 8 {
                        tracing::warn!(
                            "auto-scan: brute bloqué (curseur figé) → restart multi-GPU"
                        );
                        let _ = ScanManager::stop(&dash).await;
                        let _ = std::process::Command::new("taskkill")
                            .args(["/IM", "brute_force.exe", "/F"])
                            .output();
                        stuck_ticks = 0;
                        last_pos = None;
                        last_death_time = Some(std::time::Instant::now());
                        // retombe dans !brute_on au prochain tick
                    }
                    continue;
                }

                // Process just died — detect crash (lived < 120s = didn't finish loading+init)
                if let Some(spawn_t) = last_spawn_time {
                    let death_t = last_death_time.get_or_insert_with(std::time::Instant::now);
                    let lifetime = death_t.duration_since(spawn_t).as_secs().max(1);
                    if lifetime < 120 {
                        rapid_crash_count += 1;
                        let backoff_secs = rapid_crash_count.min(8) as u64 * 15;
                        tracing::warn!(
                            "auto-scan: crash #{} (process lived {}s) → backoff {}s before retry",
                            rapid_crash_count, lifetime, backoff_secs
                        );
                        tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                        if dict.running.load(Ordering::SeqCst) {
                            last_spawn_time = None;
                            last_death_time = None;
                            continue;
                        }
                    } else {
                        rapid_crash_count = 0;
                    }
                    last_spawn_time = None;
                    last_death_time = None;
                }

                stuck_ticks = 0;
                last_pos = None;

                // Si une fenêtre était ouverte et le curseur a atteint À → journaliser
                {
                    use btcsolver::dashboard::range_log::RangeLog;
                    let mut log = RangeLog::load(&project);
                    if let Some(ref cur) = log.current {
                        let cursor = stats
                            .as_ref()
                            .and_then(|s| s.current_position.clone())
                            .or_else(|| cur.cursor.clone());
                        let finished = match (&cursor, RangeLog::normalize_hex(&cur.end)) {
                            (Some(c), Ok(end)) => RangeLog::normalize_hex(c)
                                .map(|ck| ck.as_slice() >= end.as_slice())
                                .unwrap_or(false),
                            // pas de curseur fiable : si process mort après un run, considérer done
                            // seulement si started_at > 30s (évite race au boot)
                            _ => {
                                chrono::DateTime::parse_from_rfc3339(&cur.started_at)
                                    .ok()
                                    .map(|t| {
                                        let age = chrono::Local::now()
                                            .signed_duration_since(t)
                                            .num_seconds();
                                        age > 60
                                    })
                                    .unwrap_or(true)
                            }
                        };
                        if finished {
                            if let Some(entry) = log.complete_current("sequential") {
                                let _ = log.save(&project);
                                tracing::info!(
                                    "auto-scan: plage terminée De={} À={} ({} clés) — journal {} plages",
                                    entry.start,
                                    entry.end,
                                    entry.keys,
                                    log.ranges.len()
                                );
                            }
                        }
                    }
                }

                let mut cfg = ScanManager::get_config(&dash).await;
                cfg.use_gpu = true;
                // None = all CUDA devices present at start time (resolved in ScanManager)
                if cfg.gpus.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
                    cfg.gpus = None;
                }
                cfg.random = false;
                cfg.use_range_log = true;
                // GPU-heavy: use all available CPU cores alongside GPU
                // (threads=0 = auto-detect; cpu_pct=0 means no cap)
                if cfg.cpu_pct == 0 && cfg.threads == 0 {
                    cfg.threads = 0; // Auto-detect (uses all cores)
                }
                if cfg.range_step == 0 {
                    cfg.range_step = btcsolver::dashboard::range_log::DEFAULT_RANGE_STEP;
                }
                // Prochaine fenêtre : start_key nul = enchaîner après journal
                // (sauf si l'utilisateur a posé un manual_start via l'UI)
                cfg.start_key = None;
                cfg.end_key = None;
                cfg.count = u64::MAX; // Unlimited — process runs continuously without exiting
                cfg.batch_size = cfg.batch_size.max(1_048_576).min(16_777_216); // 1M-16M batches
                let cpu_n = cfg.resolve_cpu_threads();
                match ScanManager::start(&dash, &cfg).await {
                    Ok(pid) => {
                        last_spawn_time = Some(std::time::Instant::now());
                        last_death_time = None;
                        ScanManager::update_config(&dash, cfg.clone()).await;
                        tracing::info!(
                            "auto-scan: brute multi-GPU pid={:?} cpu_workers={} ({}% / threads={}) step={}",
                            pid,
                            cpu_n,
                            cfg.cpu_pct,
                            cfg.threads,
                            cfg.range_step
                        );
                    }
                    Err(e) => {
                        let es = e.to_string();
                        if es.contains("Insufficient free RAM") || es.contains("UTXO required") {
                            tracing::warn!("auto-scan: RAM pause - {}", es);
                        } else {
                            tracing::debug!("auto-scan: start deferred: {}", e);
                        }
                    }
                }
            }
        });
    }

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(ws_handler))
        // Scan hex brute-force
        .route("/api/scan/stats", get(scan_stats_handler))
        .route("/api/scan/start", post(scan_start_handler))
        .route("/api/scan/stop", post(scan_stop_handler))
        .route("/api/scan/pause", post(scan_pause_handler))
        .route("/api/scan/resume", post(scan_resume_handler))
        .route("/api/scan/export", get(scan_export_handler))
        .route("/api/scan/easy-keys", post(scan_easy_keys_handler))
        .route("/api/scan/corpus/progress", get(corpus_progress_handler))
        .route("/api/scan/corpus/stop", post(corpus_stop_handler))
        .route("/api/scan/config", get(scan_config_handler))
        .route("/api/scan/config", post(scan_config_update_handler))
        .route("/api/scan/ranges", get(scan_ranges_handler))
        .route("/api/scan/ranges", post(scan_ranges_post_handler))
        // Keys
        .route("/api/keys/check", post(check_key_handler))
        .route("/api/keys/batch", post(check_batch_handler))
        .route("/api/keys/archive", get(keys_archive_handler))
        .route("/api/keys/archive/export", get(keys_archive_export_handler))
        .route("/api/keys/pubkeys", post(keys_pubkeys_handler))
        // Dict
        .route("/api/dict/corpora", get(dict_corpora_handler))
        .route("/api/dict/status", get(dict_status_handler))
        .route("/api/dict/start", post(dict_start_handler))
        .route("/api/dict/stop", post(dict_stop_handler))
        // Bitcoin Core
        .route("/api/bitcoind/status", get(bitcoind_status_handler))
        .route("/api/bitcoind/start", post(bitcoind_start_handler))
        .route("/api/bitcoind/stop", post(bitcoind_stop_handler))
        .route("/api/bitcoind/restart", post(bitcoind_restart_handler))
        // Snapshot
        .route("/api/snapshot/info", get(snapshot_info_handler))
        .route("/api/snapshot/refresh", post(snapshot_refresh_handler))
        .route("/api/snapshot/reload", post(snapshot_reload_handler))
        .route("/api/snapshot/rebuild-status", get(snapshot_rebuild_status_handler))
        // System
        .route("/api/system/health", get(health_handler))
        .route("/api/system/ideas", get(ideas_handler))
        // Historical UTXO index (utxo1)
        .route("/api/utxo1/stats", get(utxo1_stats_handler))
        .route("/api/utxo1/query", post(utxo1_query_handler))
        // Scan device toggle (enable/disable GPU or set CPU threads)
        .route("/api/scan/toggle-device", post(scan_toggle_device_handler))
        // Performance benchmark
        .route("/api/benchmark/run", post(benchmark_run_handler))
        .route("/api/benchmark/status", get(benchmark_status_handler))
        .route("/api/benchmark/reset", post(benchmark_reset_handler))
        .nest_service("/static", ServeDir::new(&static_dir))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], cli.port));
    tracing::info!("Dashboard http://localhost:{}", cli.port);
    tracing::info!("Bitcoin datadir default: W:\\Bitcoin");
    tracing::info!("Snapshot: {}", snapshot_path);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn load_index(path: &str) -> Option<Arc<FlatIndex>> {
    match FlatIndex::load_from_snapshot(path, 0) {
        Ok(idx) => {
            tracing::info!(
                "UTXO index loaded: {} scripts, {:.1} MB from {}",
                idx.num_scripts,
                idx.memory_usage_bytes() as f64 / 1_048_576.0,
                path
            );
            Some(Arc::new(idx))
        }
        Err(e) => {
            tracing::warn!("UTXO index not loaded ({}): {}", path, e);
            None
        }
    }
}

async fn index_handler(State(state): State<AppState>) -> impl IntoResponse {
    let candidates = [
        format!(r"{}\static\dashboard\index.html", state.config.project_dir),
        "static/dashboard/index.html".to_string(),
    ];
    for c in candidates {
        if let Ok(html) = std::fs::read_to_string(&c) {
            return Html(html).into_response();
        }
    }
    (
        StatusCode::NOT_FOUND,
        Html("<h1>index.html missing</h1>".to_string()),
    )
        .into_response()
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_stream(socket, state))
}

async fn ws_stream(socket: WebSocket, state: AppState) {
    let (mut sender, mut _receiver) = socket.split();
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    loop {
        interval.tick().await;
        let scan = ScanManager::get_stats(&state.dashboard)
            .await
            .ok()
            .and_then(|s| serde_json::to_value(s).ok())
            .unwrap_or(serde_json::json!({}));
        let dict = DictScanManager::status(&state.dict);
        let btc = match tokio::time::timeout(
            Duration::from_secs(2),
            BitcoindManager::get_status(&state.config),
        )
        .await
        {
            Ok(Ok(s)) => serde_json::to_value(s).unwrap_or_default(),
            Ok(Err(e)) => serde_json::json!({"running": false, "error": e.to_string()}),
            Err(_) => serde_json::json!({"running": true, "error": "RPC timeout"}),
        };

        let payload = serde_json::json!({
            "type": "tick",
            "scan": scan,
            "dict": dict,
            "bitcoind": btc,
            "index_loaded": state.index.read().await.is_some(),
        });
        if sender
            .send(Message::Text(payload.to_string()))
            .await
            .is_err()
        {
            break;
        }
    }
}

// ── Scan hex ──────────────────────────────────────────────────────────────

async fn scan_stats_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    match ScanManager::get_stats(&state.dashboard).await {
        Ok(stats) => Json(serde_json::to_value(stats).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn scan_start_handler(
    State(state): State<AppState>,
    Json(config): Json<ScanConfig>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match ScanManager::start(&state.dashboard, &config).await {
        Ok(pid) => {
            ScanManager::update_config(&state.dashboard, config).await;
            Ok(Json(serde_json::json!({ "success": true, "pid": pid })))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": e.to_string() })),
        )),
    }
}

async fn scan_stop_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    match ScanManager::stop(&state.dashboard).await {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}

async fn scan_config_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let cfg = ScanManager::get_config(&state.dashboard).await;
    let cores = ScanConfig::logical_cores();
    let resolved = cfg.resolve_cpu_threads();
    let present = btcsolver::sys_info::present_gpu_ids();
    let gpus_effective = btcsolver::sys_info::resolve_gpus(&cfg.gpus);
    // Enrichir pour l’UI (cœurs + threads + GPUs présentes)
    let mut v = serde_json::to_value(&cfg).unwrap_or_default();
    if let Some(obj) = v.as_object_mut() {
        obj.insert("logical_cores".into(), serde_json::json!(cores));
        obj.insert("resolved_cpu_threads".into(), serde_json::json!(resolved));
        obj.insert("gpus_present".into(), serde_json::json!(present));
        obj.insert("gpus_effective".into(), serde_json::json!(gpus_effective));
        // Convenience: if config.gpus is null, UI should treat effective as default
        if cfg.gpus.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
            obj.insert("gpus".into(), serde_json::Value::Null);
        }
    }
    Json(v)
}

async fn scan_config_update_handler(
    State(state): State<AppState>,
    Json(config): Json<ScanConfig>,
) -> Json<serde_json::Value> {
    // Propager range_step / manual start vers le journal
    {
        use btcsolver::dashboard::range_log::RangeLog;
        let mut log = RangeLog::load(&state.config.project_dir);
        if config.range_step > 0 {
            log.range_step = config.range_step;
        }
        if let Some(ref sk) = config.start_key {
            if !sk.trim().is_empty() {
                let _ = log.set_manual_start(sk);
            }
        }
        let _ = log.save(&state.config.project_dir);
    }
    ScanManager::update_config(&state.dashboard, config).await;
    Json(serde_json::json!({ "success": true }))
}

async fn scan_ranges_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    use btcsolver::dashboard::range_log::RangeLog;
    let log = RangeLog::load(&state.config.project_dir);
    Json(serde_json::to_value(&log).unwrap_or_default())
}

#[derive(serde::Deserialize)]
struct RangeAction {
    /// set_start | set_step | complete_current | clear_current
    action: String,
    #[serde(default)]
    start: Option<String>,
    #[serde(default)]
    range_step: Option<u64>,
    /// Si true, stoppe le brute et laisse l'auto-scan repartir sur le nouveau départ
    #[serde(default)]
    restart: bool,
}

async fn scan_ranges_post_handler(
    State(state): State<AppState>,
    Json(body): Json<RangeAction>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    use btcsolver::dashboard::range_log::RangeLog;
    let mut log = RangeLog::load(&state.config.project_dir);
    match body.action.as_str() {
        "set_start" => {
            let s = body
                .start
                .as_deref()
                .ok_or_else(|| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "start requis"})),
                    )
                })?;
            let h = log.set_manual_start(s).map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
            })?;
            let mut cfg = ScanManager::get_config(&state.dashboard).await;
            cfg.start_key = Some(h.clone());
            ScanManager::update_config(&state.dashboard, cfg).await;
            let _ = log.save(&state.config.project_dir);
            if body.restart {
                let _ = ScanManager::stop(&state.dashboard).await;
                let _ = std::process::Command::new("taskkill")
                    .args(["/IM", "brute_force.exe", "/F"])
                    .output();
            }
            Ok(Json(serde_json::json!({
                "success": true,
                "manual_start": h,
                "restart": body.restart
            })))
        }
        "set_step" => {
            let step = body.range_step.unwrap_or(RangeLog::default().range_step);
            if step == 0 {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "range_step > 0 requis"})),
                ));
            }
            log.range_step = step;
            let mut cfg = ScanManager::get_config(&state.dashboard).await;
            cfg.range_step = step;
            ScanManager::update_config(&state.dashboard, cfg).await;
            let _ = log.save(&state.config.project_dir);
            Ok(Json(serde_json::json!({ "success": true, "range_step": step })))
        }
        "complete_current" => {
            let entry = log.complete_current("manual");
            let _ = log.save(&state.config.project_dir);
            Ok(Json(serde_json::json!({ "success": true, "entry": entry })))
        }
        "clear_current" => {
            log.current = None;
            let _ = log.save(&state.config.project_dir);
            Ok(Json(serde_json::json!({ "success": true })))
        }
        other => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("action inconnue: {}", other)})),
        )),
    }
}

// ── Scan Device Toggle ────────────────────────────────────────────────────

/// POST /api/scan/toggle-device — enable/disable a GPU or set CPU threads
/// Body: { "device": "gpu0" | "gpu1" | "gpu2" | "cpu", "threads": N }
/// threads=0 means disable that device
async fn scan_toggle_device_handler(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let device = body.get("device").and_then(|d| d.as_str()).unwrap_or("");
    let threads = body.get("threads").and_then(|t| t.as_u64()).map(|n| n as usize).unwrap_or(1);

    let mut cfg = ScanManager::get_config(&state.dashboard).await;

    if device.starts_with("gpu") {
        // Parse GPU ID from device name (e.g., "gpu0" → 0)
        let gpu_id: u32 = device[3..].parse().unwrap_or(0);

        // Current GPU list; empty/None means "all detected" until user toggles
        let current_gpus = cfg.gpus.clone().unwrap_or_default();
        let mut gpu_list: Vec<u32> = current_gpus
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        // First explicit toggle when list was "all": seed with the device being toggled
        if gpu_list.is_empty() && threads == 0 {
            // Disabling one GPU while on "all" → keep others would need enumeration;
            // fall back to single-GPU mode excluding this id is not possible without count.
            // Seed with just this id then remove it → empty → CPU-only until re-enable.
            gpu_list.push(gpu_id);
        }

        if threads == 0 {
            // Disable this GPU: remove from list
            gpu_list.retain(|&g| g != gpu_id);
            if gpu_list.is_empty() {
                // No GPUs left, disable GPU scanning entirely
                cfg.use_gpu = false;
                cfg.gpus = None; // re-enable will mean all detected devices
            } else {
                cfg.gpus = Some(
                    gpu_list
                        .iter()
                        .map(|g| g.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                );
            }
        } else {
            // Enable this GPU: add to list if not present
            if !gpu_list.contains(&gpu_id) {
                gpu_list.push(gpu_id);
                gpu_list.sort();
            }
            cfg.use_gpu = true;
            cfg.gpus = Some(
                gpu_list
                    .iter()
                    .map(|g| g.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }
    } else if device == "cpu" {
        // Set CPU threads
        cfg.threads = threads;
        if threads == 0 {
            cfg.cpu_pct = 0; // CPU off
        }
    } else {
        return Json(serde_json::json!({
            "success": false,
            "error": format!("Unknown device: {}. Use 'gpu0', 'gpu1', 'gpu2', or 'cpu'", device),
        }));
    }

    ScanManager::update_config(&state.dashboard, cfg.clone()).await;

    // Restart scan with new config if scan was running
    let was_running = state.dashboard.scan_running.load(Ordering::SeqCst);
    if was_running {
        let _ = ScanManager::stop(&state.dashboard).await;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        if let Err(e) = ScanManager::start(&state.dashboard, &cfg).await {
            tracing::warn!("Auto-restart scan after device toggle failed: {}", e);
        }
    }

    let status = if threads == 0 { "disabled" } else { "enabled" };
    Json(serde_json::json!({
        "success": true,
        "device": device,
        "status": status,
        "threads": threads,
        "gpus": cfg.gpus,
        "cpu_threads": cfg.threads,
        "use_gpu": cfg.use_gpu,
        "scan_restarted": was_running,
    }))
}

// ── Scan Export ───────────────────────────────────────────────────────────

async fn scan_export_handler(State(state): State<AppState>) -> impl IntoResponse {
    let stats = match ScanManager::get_stats(&state.dashboard).await {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };
    let archive = KeyArchive::new(&state.config.project_dir)
        .stats()
        .unwrap_or_else(|_| serde_json::json!({"count": 0}));

    let export_data = serde_json::json!({
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "scan": serde_json::to_value(&stats).unwrap_or_default(),
        "keys_archive": archive,
        "bitcoind": match tokio::time::timeout(
            Duration::from_secs(3),
            BitcoindManager::get_status(&state.config),
        )
        .await
        {
            Ok(Ok(s)) => serde_json::to_value(s).unwrap_or_default(),
            _ => serde_json::json!({"error": "unavailable"}),
        },
    });

    (
        StatusCode::OK,
        [("Content-Type", "application/json"), ("Content-Disposition", "attachment; filename=\"btcsolver-scan-export.json\"")],
        export_data.to_string(),
    )
        .into_response()
}

// ── Scan Pause / Resume ──────────────────────────────────────────────────

/// Pause the scan by saving current position and stopping the process
async fn scan_pause_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    // Save current position before stopping
    let stats = ScanManager::get_stats(&state.dashboard).await.ok();
    let position = stats
        .as_ref()
        .and_then(|s| s.current_position.clone())
        .or_else(|| stats.as_ref().and_then(|s| s.range_end.clone()));

    // Save position to a file for resume
    if let Some(pos) = &position {
        let pos_file = format!(
            "{}/data/scan-paused-position.txt",
            state.config.project_dir
        );
        let _ = std::fs::write(&pos_file, pos.as_bytes());
    }

    match ScanManager::stop(&state.dashboard).await {
        Ok(()) => Json(serde_json::json!({
            "success": true,
            "paused": true,
            "position_saved": position.is_some(),
            "position": position,
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string(),
        })),
    }
}

/// Resume the scan from the saved position
async fn scan_resume_handler(State(state): State<AppState>) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Read saved position
    let pos_file = format!(
        "{}/data/scan-paused-position.txt",
        state.config.project_dir
    );
    let resume_pos = std::fs::read_to_string(&pos_file).ok();

    let mut cfg = ScanManager::get_config(&state.dashboard).await;
    let resumed_from = if let Some(ref pos) = resume_pos {
        let p = pos.trim().to_string();
        if !p.is_empty() {
            cfg.start_key = Some(p.clone());
        }
        // Clean up pause file
        let _ = std::fs::remove_file(&pos_file);
        Some(p)
    } else {
        None
    };

    // Ensure reasonable defaults for resume (keep gpus selection; None = all CUDA)
    cfg.use_gpu = true;
    cfg.count = u64::MAX;

    match ScanManager::start(&state.dashboard, &cfg).await {
        Ok(pid) => {
            ScanManager::update_config(&state.dashboard, cfg).await;
            Ok(Json(serde_json::json!({
                "success": true,
                "pid": pid,
                "resumed_from": resumed_from,
            })))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": e.to_string() })),
        )),
    }
}

// ── Easy Keys Corpus Scan (Background) ────────────────────────────────────

#[derive(serde::Deserialize)]
struct EasyKeysRequest {
    #[serde(default)]
    use_gpu: bool,
    #[serde(default)]
    threads: Option<usize>,
    /// Optional: specific corpus file(s) to scan. If omitted, auto-discovers all *-keys*.txt in data/
    #[serde(default)]
    corpus_files: Option<Vec<String>>,
}

async fn scan_easy_keys_handler(
    State(state): State<AppState>,
    Json(req): Json<EasyKeysRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Check if already running
    if state.corpus.running.load(Ordering::Relaxed) {
        return Ok(Json(serde_json::json!({
            "success": false,
            "error": "Corpus scan already running — check /api/scan/corpus/progress"
        })));
    }

    // Get UTXO index
    let guard = state.index.read().await;
    let index = match guard.as_ref() {
        Some(idx) => idx.clone(),
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": "UTXO index not loaded"
                })),
            ));
        }
    };
    drop(guard);

    // Discover corpus files
    let data_dir = format!("{}/data", state.config.project_dir);
    let corpus_files = if let Some(files) = req.corpus_files {
        files
    } else {
        // Auto-discover all *-keys*.txt files
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&data_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if (name.contains("keys") || name.contains("corpus")) && name.ends_with(".txt") {
                    files.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
        files.sort();
        files
    };

    if corpus_files.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "No corpus files found in data/ (looking for *keys*.txt or *corpus*.txt)"
            })),
        ));
    }

    // Count total lines (streaming — don't load entire file into memory)
    let mut total_lines = 0u64;
    for path in &corpus_files {
        if let Ok(file) = std::fs::File::open(path) {
            let reader = std::io::BufReader::new(file);
            for line in reader.lines() {
                if let Ok(l) = line {
                    let l = l.trim();
                    if !l.is_empty() && !l.starts_with('#') {
                        total_lines += 1;
                    }
                }
            }
        }
    }

    if total_lines == 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": "No valid keys found in corpus files"
            })),
        ));
    }

    // Reset state
    state.corpus.running.store(true, Ordering::Relaxed);
    state.corpus.stop.store(false, Ordering::Relaxed);
    state.corpus.keys_tested.store(0, Ordering::Relaxed);
    state.corpus.keys_total.store(total_lines, Ordering::Relaxed);
    state.corpus.matches_found.store(0, Ordering::Relaxed);
    if let Ok(mut s) = state.corpus.status_text.lock() {
        *s = format!("Starting corpus scan: {} files, {} keys", corpus_files.len(), total_lines);
    }

    // Clone for background task
    let corpus = state.corpus.clone();
    let index = index.clone();
    let project_dir = state.config.project_dir.clone();
    let file_count = corpus_files.len();

    // Spawn background task (blocking — CPU-intensive key checking)
    tokio::task::spawn_blocking(move || {
        let start = std::time::Instant::now();
        let mut scanned = 0u64;
        let mut matches = 0u64;
        let mut total_balance = 0u64;
        let archive = KeyArchive::new(&project_dir);
        let batch_size = 10000; // Update progress every N keys

        for file_path in &corpus_files {
            if corpus.stop.load(Ordering::Relaxed) {
                if let Ok(mut s) = corpus.status_text.lock() {
                    *s = "Scan stopped by user".to_string();
                }
                break;
            }

            let file = match std::fs::File::open(file_path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Corpus scan: failed to open {}: {}", file_path, e);
                    continue;
                }
            };
            let reader = std::io::BufReader::new(file);

            let fname = std::path::Path::new(file_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| file_path.clone());

            if let Ok(mut s) = corpus.status_text.lock() {
                *s = format!("Scanning {}...", fname);
            }

            let mut batch = Vec::new();
            for line_result in reader.lines() {
                if corpus.stop.load(Ordering::Relaxed) { break; }

                let line = match line_result {
                    Ok(l) => l,
                    Err(_) => continue,
                };
                let line = line.trim().to_string();
                if line.is_empty() || line.starts_with('#') { continue; }

                batch.push(line);

                if batch.len() >= batch_size {
                    // Process batch
                    let (m, b) = process_key_batch(&index, &batch, &archive, "corpus_scan");
                    matches += m;
                    total_balance += b;
                    scanned += batch.len() as u64;
                    corpus.keys_tested.store(scanned, Ordering::Relaxed);
                    corpus.matches_found.store(matches, Ordering::Relaxed);
                    batch.clear();
                }
            }

            // Process remaining
            if !batch.is_empty() {
                let (m, b) = process_key_batch(&index, &batch, &archive, "corpus_scan");
                matches += m;
                total_balance += b;
                scanned += batch.len() as u64;
                corpus.keys_tested.store(scanned, Ordering::Relaxed);
                corpus.matches_found.store(matches, Ordering::Relaxed);
            }
        }

        let elapsed = start.elapsed();
        let rate = if elapsed.as_secs_f64() > 0.0 {
            scanned as f64 / elapsed.as_secs_f64()
        } else { 0.0 };

        if let Ok(mut s) = corpus.status_text.lock() {
            if corpus.stop.load(Ordering::Relaxed) {
                *s = format!("Stopped: {} keys in {:.1}s ({:.0} k/s)", scanned, elapsed.as_secs_f64(), rate);
            } else {
                *s = format!("Complete: {} keys, {} matches, {:.1}s ({:.0} k/s), {} BTC",
                    scanned, matches, elapsed.as_secs_f64(), rate, total_balance as f64 / 1e8);
            }
        }

        corpus.running.store(false, Ordering::Relaxed);

        if matches > 0 {
            eprintln!("CORPUS SCAN: {} matches found! {} BTC total", matches, total_balance as f64 / 1e8);
        }
    });

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Corpus scan started in background",
        "total_keys": total_lines,
        "files": file_count,
    })))
}

/// Process a batch of keys. Returns (matches, total_balance_sats).
fn process_key_batch(
    index: &FlatIndex,
    keys: &[String],
    archive: &KeyArchive,
    source: &str,
) -> (u64, u64) {
    let mut matches = 0u64;
    let mut total_balance = 0u64;

    for key_hex in keys {
        let bytes_vec = match hex::decode(key_hex.trim()) {
            Ok(b) if b.len() == 32 => b,
            _ => continue,
        };
        let bytes: [u8; 32] = bytes_vec.try_into().unwrap();

        // Lookup scripts in index (sync version for batch processing)
        let (addrs, balance) = KeyChecker::lookup_key_sync(index, bytes);

        if balance > 0 {
            matches += 1;
            total_balance += balance;

            let addr_vec = vec![
                addrs.legacy, addrs.segwit, addrs.wrapped, addrs.taproot,
            ];

            let entry = ArchivedKey::from_utxo_hit(
                key_hex, None, addr_vec, balance, source,
                Some("identity".to_string()), Some(key_hex.clone()),
            );
            let _ = archive.record(entry);
            btcsolver::alert_beep::alert_balance_found();
        }
    }

    (matches, total_balance)
}

async fn corpus_progress_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let running = state.corpus.running.load(Ordering::Relaxed);
    let tested = state.corpus.keys_tested.load(Ordering::Relaxed);
    let total = state.corpus.keys_total.load(Ordering::Relaxed);
    let matches = state.corpus.matches_found.load(Ordering::Relaxed);
    let status = state.corpus.status_text.lock().map(|s| (*s).clone()).unwrap_or_default();
    let pct = if total > 0 { (tested as f64 / total as f64) * 100.0 } else { 0.0 };

    Json(serde_json::json!({
        "running": running,
        "keys_tested": tested,
        "keys_total": total,
        "matches_found": matches,
        "progress_pct": pct,
        "status": status,
    }))
}

async fn corpus_stop_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    state.corpus.stop.store(true, Ordering::Relaxed);
    Json(serde_json::json!({
        "success": true,
        "message": "Corpus scan stop requested",
    }))
}

// ── Keys ──────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct CheckKeyRequest {
    key: String,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    passphrase: Option<String>,
    #[serde(default)]
    options: Option<BrainwalletOptions>,
    /// If true, only return entries with balance > 0 (still computes all)
    #[serde(default)]
    matches_only: bool,
}

#[derive(serde::Deserialize)]
struct CheckBatchRequest {
    keys: Vec<String>,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    passphrase: Option<String>,
    #[serde(default)]
    options: Option<BrainwalletOptions>,
}

fn parse_format(s: &str) -> Option<KeyFormat> {
    match s.to_lowercase().as_str() {
        "hex" => Some(KeyFormat::Hex),
        "wif" => Some(KeyFormat::WIF),
        "bip39" => Some(KeyFormat::BIP39),
        "brainwallet" | "brain" => Some(KeyFormat::Brainwallet),
        _ => None,
    }
}

async fn check_key_handler(
    State(state): State<AppState>,
    Json(req): Json<CheckKeyRequest>,
) -> Json<serde_json::Value> {
    let guard = state.index.read().await;
    let index = match guard.as_ref() {
        Some(idx) => idx.clone(),
        None => {
            return Json(serde_json::json!({
                "error": "UTXO index not loaded. Refresh snapshot then Reload index."
            }));
        }
    };
    drop(guard);

    let format = match &req.format {
        Some(f) => match parse_format(f) {
            Some(fmt) => fmt,
            None => {
                return Json(serde_json::json!({
                    "error": "Invalid format. Use: hex, wif, bip39, brainwallet"
                }));
            }
        },
        None => match KeyChecker::detect_format(&req.key) {
            Ok(f) => f,
            Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
        },
    };

    let opts = req.options.unwrap_or_default();
    let derived = match KeyChecker::expand_keys(
        &req.key,
        format,
        req.passphrase.as_deref(),
        &opts,
    ) {
        Ok(k) => k,
        Err(e) => {
            return Json(serde_json::json!({ "error": format!("parse: {}", e) }));
        }
    };

    let format_str = format.to_string();
    let mut results = Vec::new();
    let mut total_balance = 0u64;
    let candidates = derived.len();

    let archive = KeyArchive::new(&state.config.project_dir);
    let mut archived_count = 0u32;

    for d in derived {
        let result = KeyChecker::check_key_with_method(
            &index,
            d.bytes,
            req.key.clone(),
            format_str.clone(),
            d.method.clone(),
        )
        .await;
        total_balance += result.total_balance_sats;

        // Archive: solde UTXO OU activité historique (scanblocks si dispo)
        let mut has_activity = result.total_balance_sats > 0;
        let mut activity_notes: Option<String> = None;
        if !has_activity && result.error.is_none() {
            let addrs = [
                result.addresses.legacy.as_str(),
                result.addresses.segwit.as_str(),
                result.addresses.wrapped.as_str(),
                result.addresses.taproot.as_str(),
            ];
            if let Some(note) = try_detect_address_activity(&state.config, &addrs) {
                has_activity = true;
                activity_notes = Some(note);
            }
        }

        if has_activity {
            let addrs: Vec<String> = result
                .matches
                .iter()
                .map(|m| m.address.clone())
                .chain(
                    [
                        result.addresses.legacy.clone(),
                        result.addresses.segwit.clone(),
                        result.addresses.wrapped.clone(),
                        result.addresses.taproot.clone(),
                    ]
                    .into_iter(),
                )
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            let entry = if result.total_balance_sats > 0 && activity_notes.is_some() {
                ArchivedKey::from_history_hit(
                    &result.privkey_hex,
                    None,
                    addrs,
                    result.total_balance_sats,
                    "dashboard_key_check",
                    Some(result.method.clone()),
                    Some(req.key.clone()),
                    activity_notes.clone(),
                )
            } else if result.total_balance_sats > 0 {
                ArchivedKey::from_utxo_hit(
                    &result.privkey_hex,
                    None,
                    addrs,
                    result.total_balance_sats,
                    "dashboard_key_check",
                    Some(result.method.clone()),
                    Some(req.key.clone()),
                )
            } else {
                ArchivedKey::from_history_hit(
                    &result.privkey_hex,
                    None,
                    addrs,
                    0,
                    "dashboard_key_check",
                    Some(result.method.clone()),
                    Some(req.key.clone()),
                    activity_notes.clone(),
                )
            };
            if archive.record(entry).unwrap_or(false) {
                archived_count += 1;
            }
            // Bip si solde UTXO réel (pas seulement historique)
            if result.total_balance_sats > 0 {
                btcsolver::alert_beep::alert_balance_found();
            }
        }

        if req.matches_only && !has_activity && result.error.is_none() {
            continue;
        }
        // Attach activity flags for UI
        let mut v = serde_json::to_value(&result).unwrap_or_default();
        if let Some(obj) = v.as_object_mut() {
            obj.insert("has_activity".into(), serde_json::json!(has_activity));
            obj.insert(
                "archived".into(),
                serde_json::json!(has_activity),
            );
            if let Some(n) = activity_notes {
                obj.insert("activity_note".into(), serde_json::json!(n));
            }
        }
        results.push(v);
    }

    // Always surface matches first
    results.sort_by(|a, b| {
        let sa = a
            .get("total_balance_sats")
            .and_then(|x| x.as_u64())
            .unwrap_or(0);
        let sb = b
            .get("total_balance_sats")
            .and_then(|x| x.as_u64())
            .unwrap_or(0);
        sb.cmp(&sa)
    });

    Json(serde_json::json!({
        "success": true,
        "format": format_str,
        "candidates": candidates,
        "count": results.len(),
        "results": results,
        "total_balance_sats": total_balance,
        "total_balance_btc": total_balance as f64 / 1e8,
        "archived_count": archived_count,
        "archive_note": "Clés avec solde OU activité on-chain conservées dans data/keys-archive.json",
    }))
}

/// Best-effort: detect past activity via Core `scanblocks` (needs blockfilterindex).
/// Returns a short note if activity likely; None if unknown / unavailable.
fn try_detect_address_activity(config: &DashboardConfig, addresses: &[&str]) -> Option<String> {
    let cli = config
        .bitcoin_cli_path
        .as_deref()
        .unwrap_or(r"W:\Bitcoin\bin\daemon\bitcoin-cli.exe");
    if !Path::new(cli).exists() {
        return None;
    }
    // Skip during heavy IBD — scanblocks not useful / slow
    let status = std::process::Command::new(cli)
        .args([
            &format!("-datadir={}", config.bitcoin_datadir),
            "-rpcclienttimeout=8",
            "getblockchaininfo",
        ])
        .output()
        .ok()?;
    if !status.status.success() {
        return None;
    }
    let info: serde_json::Value = serde_json::from_slice(&status.stdout).ok()?;
    if info
        .get("initialblockdownload")
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
    {
        return None;
    }
    // Require block filter index
    let idx = std::process::Command::new(cli)
        .args([
            &format!("-datadir={}", config.bitcoin_datadir),
            "-rpcclienttimeout=8",
            "getindexinfo",
        ])
        .output()
        .ok()?;
    let idx_json: serde_json::Value = serde_json::from_slice(&idx.stdout).ok()?;
    let filter_synced = idx_json
        .get("basic block filter index")
        .and_then(|v| v.get("synced"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !filter_synced {
        return None;
    }

    let mut descs: Vec<String> = Vec::new();
    for a in addresses {
        if a.is_empty() || *a == "error" {
            continue;
        }
        descs.push(format!("addr({})", a));
    }
    if descs.is_empty() {
        return None;
    }
    let scan_obj = serde_json::to_string(&descs).ok()?;
    let out = std::process::Command::new(cli)
        .args([
            &format!("-datadir={}", config.bitcoin_datadir),
            "-rpcclienttimeout=120",
            "scanblocks",
            "start",
            &scan_obj,
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let res: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    let from_h = res.get("from_height").and_then(|v| v.as_u64()).unwrap_or(0);
    let to_h = res.get("to_height").and_then(|v| v.as_u64()).unwrap_or(0);
    let relevant = res
        .get("relevant_blocks")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    if relevant > 0 {
        Some(format!(
            "scanblocks: {} bloc(s) pertinent(s) (h {}-{})",
            relevant, from_h, to_h
        ))
    } else {
        None
    }
}

async fn keys_archive_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let archive = KeyArchive::new(&state.config.project_dir);
    match archive.list_json() {
        Ok(v) => Json(v),
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}

/// Export keys archive as CSV
async fn keys_archive_export_handler(State(state): State<AppState>) -> impl IntoResponse {
    let archive = KeyArchive::new(&state.config.project_dir);
    let keys = match archive.load_all() {
        Ok(k) => k,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    let mut csv = String::from("privkey_hex,pubkey_hex,wif,input,method,source,addresses,balance_sats,balance_btc,has_activity,has_balance,reason,first_seen,last_seen,peak_balance_sats,notes\n");

    for k in &keys {
        let addrs = k.addresses.join(";");
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            csv_escape(&k.privkey_hex),
            csv_escape(&k.pubkey_hex.as_deref().unwrap_or("")),
            csv_escape(&k.wif.as_deref().unwrap_or("")),
            csv_escape(&k.input.as_deref().unwrap_or("")),
            csv_escape(&k.method.as_deref().unwrap_or("")),
            csv_escape(&k.source.as_deref().unwrap_or("")),
            csv_escape(&addrs),
            k.balance_sats,
            k.balance_btc,
            k.has_activity,
            k.has_balance,
            k.reason.as_str(),
            k.first_seen,
            k.last_seen,
            k.peak_balance_sats,
            csv_escape(&k.notes.as_deref().unwrap_or("")),
        ));
    }

    (
        StatusCode::OK,
        [
            ("Content-Type", "text/csv; charset=utf-8"),
            ("Content-Disposition", "attachment; filename=\"btcsolver-keys-archive.csv\""),
        ],
        csv,
    )
        .into_response()
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Dérive les clés publiques (compressée + non compressée) depuis des priv hex.
/// Body: `{ "privkeys": ["…", …] }` ou `{ "privkey_hex": "…" }`
async fn keys_pubkeys_handler(Json(body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    let mut list: Vec<String> = Vec::new();
    if let Some(arr) = body.get("privkeys").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(s) = v.as_str() {
                list.push(s.to_string());
            }
        }
    }
    if let Some(s) = body
        .get("privkey_hex")
        .or_else(|| body.get("key_hex"))
        .and_then(|v| v.as_str())
    {
        list.push(s.to_string());
    }
    if list.is_empty() {
        return Json(serde_json::json!({
            "success": false,
            "error": "privkeys ou privkey_hex requis"
        }));
    }
    let mut pubkeys = serde_json::Map::new();
    let mut results = Vec::new();
    for p in list {
        let key = p.trim().trim_start_matches("0x").to_lowercase();
        match pubkeys_from_priv_hex(&key) {
            Some((comp, uncomp)) => {
                let obj = serde_json::json!({
                    "privkey_hex": key,
                    "pubkey_hex": comp,
                    "pubkey_uncompressed_hex": uncomp,
                    "compressed": comp,
                    "uncompressed": uncomp,
                });
                pubkeys.insert(key.clone(), obj.clone());
                results.push(obj);
            }
            None => {
                results.push(serde_json::json!({
                    "privkey_hex": key,
                    "error": "privkey invalide"
                }));
            }
        }
    }
    Json(serde_json::json!({
        "success": true,
        "count": results.len(),
        "pubkeys": pubkeys,
        "results": results,
    }))
}

async fn check_batch_handler(
    State(state): State<AppState>,
    Json(req): Json<CheckBatchRequest>,
) -> Json<serde_json::Value> {
    let guard = state.index.read().await;
    let index = match guard.as_ref() {
        Some(idx) => idx.clone(),
        None => return Json(serde_json::json!({ "error": "UTXO index not loaded" })),
    };
    drop(guard);

    let opts = req.options.unwrap_or_default();
    let mut results = Vec::new();
    let mut total_balance = 0u64;

    for key_str in &req.keys {
        let format = match &req.format {
            Some(f) => match parse_format(f) {
                Some(fmt) => fmt,
                None => continue,
            },
            None => match KeyChecker::detect_format(key_str) {
                Ok(f) => f,
                Err(_) => continue,
            },
        };
        let derived = match KeyChecker::expand_keys(
            key_str,
            format,
            req.passphrase.as_deref(),
            &opts,
        ) {
            Ok(k) => k,
            Err(_) => continue,
        };
        let format_str = format.to_string();
        let archive = KeyArchive::new(&state.config.project_dir);
        for d in derived {
            let result = KeyChecker::check_key_with_method(
                &index,
                d.bytes,
                key_str.clone(),
                format_str.clone(),
                d.method.clone(),
            )
            .await;
            let mut has_activity = result.total_balance_sats > 0;
            if !has_activity && result.error.is_none() {
                let addrs = [
                    result.addresses.legacy.as_str(),
                    result.addresses.segwit.as_str(),
                    result.addresses.wrapped.as_str(),
                    result.addresses.taproot.as_str(),
                ];
                has_activity = try_detect_address_activity(&state.config, &addrs).is_some();
            }
            if has_activity {
                total_balance += result.total_balance_sats;
                let addrs: Vec<String> = vec![
                    result.addresses.legacy.clone(),
                    result.addresses.segwit.clone(),
                    result.addresses.wrapped.clone(),
                    result.addresses.taproot.clone(),
                ];
                let entry = if result.total_balance_sats > 0 {
                    ArchivedKey::from_utxo_hit(
                        &result.privkey_hex,
                        None,
                        addrs,
                        result.total_balance_sats,
                        "dashboard_batch",
                        Some(result.method.clone()),
                        Some(key_str.clone()),
                    )
                } else {
                    ArchivedKey::from_history_hit(
                        &result.privkey_hex,
                        None,
                        addrs,
                        0,
                        "dashboard_batch",
                        Some(result.method.clone()),
                        Some(key_str.clone()),
                        Some("activity without current balance".into()),
                    )
                };
                let _ = archive.record(entry);
                results.push(result);
            }
        }
    }

    Json(serde_json::json!({
        "success": true,
        "count": results.len(),
        "results": results,
        "total_balance_sats": total_balance,
        "total_balance_btc": total_balance as f64 / 1e8,
        "archive_note": "Clés actives archivées dans data/keys-archive.json",
    }))
}

// ── Dict ──────────────────────────────────────────────────────────────────

async fn dict_corpora_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "corpora": DictScanManager::list_corpora(&state.config.project_dir)
    }))
}

async fn dict_status_handler(State(state): State<AppState>) -> Json<DictScanStatusWrap> {
    Json(DictScanStatusWrap(DictScanManager::status(&state.dict)))
}

#[derive(serde::Serialize)]
struct DictScanStatusWrap(btcsolver::dashboard::dict_scan::DictScanStatus);

async fn dict_start_handler(
    State(state): State<AppState>,
    Json(req): Json<DictScanRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let guard = state.index.read().await;
    let index = match guard.as_ref() {
        Some(idx) => idx.clone(),
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": "UTXO index not loaded"
                })),
            ));
        }
    };
    drop(guard);

    // Libérer les GPU : arrêter le brute auto pendant le scan listes
    if req.use_gpu {
        let _ = ScanManager::stop(&state.dashboard).await;
        tracing::info!("dict start: brute_force stopped to free GPUs");
    }

    match DictScanManager::start(&state.dict, index, state.config.project_dir.clone(), req) {
        Ok(()) => Ok(Json(serde_json::json!({ "success": true }))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": e.to_string() })),
        )),
    }
}

async fn dict_stop_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    DictScanManager::stop(&state.dict);
    Json(serde_json::json!({ "success": true }))
}

// ── Bitcoin Core ──────────────────────────────────────────────────────────

async fn bitcoind_status_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    match tokio::time::timeout(Duration::from_secs(6), BitcoindManager::get_status(&state.config))
        .await
    {
        Ok(Ok(status)) => Json(serde_json::to_value(status).unwrap_or_default()),
        Ok(Err(e)) => Json(serde_json::json!({
            "running": false,
            "process_running": false,
            "rpc_ok": false,
            "can_start": true,
            "can_stop": false,
            "error": e.to_string(),
            "message": e.to_string(),
        })),
        Err(_) => {
            // Timeout: still detect process so Start stays disabled if daemon is up
            let (proc, pid) = btcsolver::dashboard::bitcoind::BitcoindManager::detect_process(
                &state.config.bitcoin_datadir,
            );
            Json(serde_json::json!({
                "running": proc,
                "process_running": proc,
                "rpc_ok": false,
                "can_start": !proc,
                "can_stop": proc,
                "pid": pid,
                "message": if proc {
                    "Process up — RPC timeout (node busy)"
                } else {
                    "RPC timeout"
                },
                "datadir": state.config.bitcoin_datadir,
            }))
        }
    }
}

async fn bitcoind_start_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    match BitcoindManager::start(&state.config).await {
        Ok(msg) => Json(serde_json::json!({ "success": true, "message": msg })),
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}

async fn bitcoind_stop_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    match BitcoindManager::stop(&state.config).await {
        Ok(msg) => Json(serde_json::json!({ "success": true, "message": msg })),
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}

async fn bitcoind_restart_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    match BitcoindManager::restart(&state.config).await {
        Ok(msg) => Json(serde_json::json!({ "success": true, "message": msg })),
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}

// ── Snapshot ──────────────────────────────────────────────────────────────

async fn snapshot_info_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let mut info = match BitcoindManager::get_snapshot_info(&state.config).await {
        Ok(info) => info,
        Err(e) => serde_json::json!({ "error": e.to_string() }),
    };
    let loaded = state.index.read().await.is_some();
    let scripts = state
        .index
        .read()
        .await
        .as_ref()
        .map(|i| i.num_scripts)
        .unwrap_or(0);
    if let Some(obj) = info.as_object_mut() {
        obj.insert("index_loaded".into(), serde_json::json!(loaded));
        obj.insert("index_scripts".into(), serde_json::json!(scripts));
    }
    Json(info)
}

async fn snapshot_refresh_handler(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Check if already rebuilding
    if state.utxo_rebuild.running.swap(true, Ordering::SeqCst) {
        return Ok(Json(serde_json::json!({
            "success": false,
            "error": "UTXO rebuild already in progress",
            "in_progress": true,
        })));
    }

    let config = state.config.clone();
    let rebuild = state.utxo_rebuild.clone();
    let index = state.index.clone();
    let snapshot_path = config.snapshot_path.clone();

    // Write rebuild marker file
    let marker_path = format!("{}/.UTXO_REBUILD_IN_PROGRESS", config.project_dir);
    let _ = std::fs::write(&marker_path, "rebuilding");

    {
        let mut progress = rebuild.progress.lock().unwrap();
        *progress = Some(serde_json::json!({
            "phase": "starting",
            "percent": 0,
            "message": "Starting UTXO rebuild from blocks…",
            "started_at": chrono::Utc::now().to_rfc3339(),
        }));
    }

    // Run rebuild in background
    tokio::spawn(async move {
        let result = BitcoindManager::generate_snapshot(&config).await;

        match result {
            Ok(msg) => {
                // Copy snapshot to local cache for faster access
                let local_cache = format!(r"C:\btcsolver-cache\utxo-index.snapshot");
                let copy_result = std::fs::create_dir_all(r"C:\btcsolver-cache")
                    .and_then(|()| std::fs::copy(&config.snapshot_path, &local_cache));

                let copy_msg = match copy_result {
                    Ok(n) => format!("\nCopied to local cache: {} ({:.1} MB)", local_cache, n as f64 / 1_048_576.0),
                    Err(e) => format!("\nCache copy skipped: {}", e),
                };

                // Reload index into memory automatically
                let loaded = tokio::task::spawn_blocking(move || load_index(&snapshot_path))
                    .await
                    .ok()
                    .flatten();
                if loaded.is_some() {
                    *index.write().await = loaded;
                }

                // Remove marker
                let _ = std::fs::remove_file(&marker_path);

                rebuild.running.store(false, Ordering::SeqCst);
                {
                    let mut progress = rebuild.progress.lock().unwrap();
                    *progress = Some(serde_json::json!({
                        "phase": "complete",
                        "percent": 100,
                        "message": format!("{}{}", msg, copy_msg),
                        "completed_at": chrono::Utc::now().to_rfc3339(),
                    }));
                }
                tracing::info!("UTXO rebuild complete:{}", copy_msg);
            }
            Err(e) => {
                let _ = std::fs::remove_file(&marker_path);
                rebuild.running.store(false, Ordering::SeqCst);
                {
                    let mut progress = rebuild.progress.lock().unwrap();
                    *progress = Some(serde_json::json!({
                        "phase": "error",
                        "percent": 0,
                        "error": e.to_string(),
                    }));
                }
                tracing::error!("UTXO rebuild failed: {}", e);
            }
        }
    });

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "UTXO rebuild started in background. Check /api/snapshot/rebuild-status for progress.",
        "background": true,
    })))
}

/// GET /api/snapshot/rebuild-status — check background rebuild progress
async fn snapshot_rebuild_status_handler(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let running = state.utxo_rebuild.running.load(Ordering::SeqCst);
    let progress = state.utxo_rebuild.progress.lock().unwrap().clone();

    // Also check marker file for external rebuilds (Keep-Core-And-Utxo.ps1)
    let marker_exists = std::path::Path::new(&format!("{}/.UTXO_REBUILD_IN_PROGRESS", state.config.project_dir)).exists();

    Json(serde_json::json!({
        "running": running || marker_exists,
        "progress": progress,
        "marker_file": marker_exists,
    }))
}

async fn snapshot_reload_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let path = state.config.snapshot_path.clone();
    let need = btcsolver::sys_info::utxo_size_from_path(&path);
    let (ram_ok, ram_msg) = btcsolver::sys_info::ram_gate_message(need);
    if !ram_ok {
        let _ = std::fs::create_dir_all(format!("{}/data", state.config.project_dir));
        let _ = std::fs::write(
            format!("{}/data/scan-ram-pause.txt", state.config.project_dir),
            format!("{}\nutxo_bytes={}\n", ram_msg, need),
        );
        return Json(serde_json::json!({
            "success": false,
            "index_loaded": false,
            "index_scripts": 0,
            "path": state.config.snapshot_path,
            "error": ram_msg,
            "ram_paused": true,
            "ram": btcsolver::sys_info::ram_status_json(&state.config.snapshot_path, None),
        }));
    }
    let loaded = tokio::task::spawn_blocking(move || load_index(&path))
        .await
        .unwrap_or(None);
    let scripts = loaded.as_ref().map(|i| i.num_scripts).unwrap_or(0);
    let ok = loaded.is_some();
    *state.index.write().await = loaded;
    if ok {
        let _ = std::fs::remove_file(format!(
            "{}/data/scan-ram-pause.txt",
            state.config.project_dir
        ));
    }
    Json(serde_json::json!({
        "success": ok,
        "index_loaded": ok,
        "index_scripts": scripts,
        "path": state.config.snapshot_path,
        "ram_paused": false,
    }))
}

// ── Health + ideas ────────────────────────────────────────────────────────

async fn health_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let scan = ScanManager::get_stats(&state.dashboard).await;
    let btc = tokio::time::timeout(Duration::from_secs(3), BitcoindManager::get_status(&state.config))
        .await;
    let mut snap = BitcoindManager::get_snapshot_info(&state.config).await;
    let dict = DictScanManager::status(&state.dict);
    let idx_guard = state.index.read().await;
    let index_loaded = idx_guard.is_some();
    let scripts = idx_guard.as_ref().map(|i| i.num_scripts).unwrap_or(0);
    let ram = btcsolver::sys_info::ram_status_json(
        &state.config.snapshot_path,
        idx_guard.as_ref().map(|a| a.as_ref()),
    );
    let ram_paused = ram.get("paused").and_then(|v| v.as_bool()).unwrap_or(false)
        || std::path::Path::new(&format!(
            "{}/data/scan-ram-pause.txt",
            state.config.project_dir
        ))
        .exists();
    let gpus_present = btcsolver::sys_info::present_gpu_ids();
    drop(idx_guard);
    // Enrich snapshot payload with live index_scripts for the UI strip
    if let Ok(ref mut v) = snap {
        if let Some(obj) = v.as_object_mut() {
            obj.insert("index_loaded".into(), serde_json::json!(index_loaded));
            obj.insert("index_scripts".into(), serde_json::json!(scripts));
            if !obj.contains_key("num_scripts") || obj.get("num_scripts").and_then(|x| x.as_u64()).unwrap_or(0) == 0 {
                if scripts > 0 {
                    obj.insert("num_scripts".into(), serde_json::json!(scripts));
                }
            }
        }
    }

    // Detect external brute_force process (24/7 keep-alive) even if not started from this UI
    let external_brute = std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq brute_force.exe", "/NH"])
        .output()
        .ok()
        .map(|o| {
            let s = String::from_utf8_lossy(&o.stdout).to_lowercase();
            s.contains("brute_force") && !s.contains("no tasks")
        })
        .unwrap_or(false);

    // Always-on status written by Keep-Core-And-Utxo.ps1
    let mut core_utxo_status = {
        let p = format!(r"{}\data\CORE-UTXO-STATUS.json", state.config.project_dir);
        std::fs::read_to_string(&p)
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
    };

    // Rebuild UTXO en cours ? (marker + artefacts dump / conversion)
    let rebuild_marker = std::path::Path::new(&state.config.project_dir)
        .join(".UTXO_REBUILD_IN_PROGRESS")
        .exists();
    let rebuild_artifacts = [
        format!(r"{}\data\utxo-tip.building.snapshot", state.config.project_dir),
        r"W:\Temp\utxo-tip.dat.incomplete".to_string(),
    ]
    .iter()
    .any(|p| {
        std::path::Path::new(p)
            .metadata()
            .map(|m| m.len() > 0 || p.ends_with("incomplete"))
            .unwrap_or(false)
            && std::path::Path::new(p).exists()
    });
    let dump_to_flat_alive = std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq dump_to_flat.exe", "/NH"])
        .output()
        .ok()
        .map(|o| {
            let s = String::from_utf8_lossy(&o.stdout).to_lowercase();
            s.contains("dump_to_flat") && !s.contains("no tasks")
        })
        .unwrap_or(false);
    let utxo_rebuild_in_progress = rebuild_marker || dump_to_flat_alive || rebuild_artifacts;
    if let Some(ref mut v) = core_utxo_status {
        if let Some(obj) = v.as_object_mut() {
            obj.insert(
                "utxo_rebuild_in_progress".into(),
                serde_json::json!(utxo_rebuild_in_progress),
            );
            if utxo_rebuild_in_progress {
                obj.insert(
                    "last_refresh".into(),
                    serde_json::json!({"ok": false, "reason": "in_progress"}),
                );
            }
        }
    }

    let keys_archive = KeyArchive::new(&state.config.project_dir)
        .stats()
        .unwrap_or_else(|_| serde_json::json!({"count": 0}));

    // GPU stats from nvidia-smi
    let gpu_stats = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=index,name,utilization.gpu,memory.used,memory.total,temperature.gpu,power.draw,power.limit",
               "--format=csv,noheader,nounits"])
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).to_string();
            let mut list: Vec<serde_json::Value> = Vec::new();
            for line in s.lines() {
                let fields: Vec<&str> = line.split(',').map(|f| f.trim()).collect();
                if fields.len() >= 7 {
                    list.push(serde_json::json!({
                        "index": fields[0].parse::<i32>().ok(),
                        "name": fields[1],
                        "util_pct": fields[2].parse::<i32>().ok(),
                        "mem_used_mb": fields[3].parse::<i32>().ok().map(|v| v / 1024),
                        "mem_total_mb": fields[4].parse::<i32>().ok().map(|v| v / 1024),
                        "temp_c": fields[5].parse::<i32>().ok(),
                        "power_w": fields[6].parse::<f64>().ok(),
                    }));
                }
            }
            if list.is_empty() { None } else { Some(serde_json::Value::Array(list)) }
        })
        .unwrap_or_else(|| serde_json::Value::Array(Vec::new()));

    // Historical indexer status
    let hi_running = std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq historical_indexer.exe", "/NH"])
        .output()
        .ok()
        .map(|o| {
            let s = String::from_utf8_lossy(&o.stdout).to_lowercase();
            s.contains("historical_indexer") && !s.contains("no tasks")
        })
        .unwrap_or(false);
    let hi_checkpoint = std::fs::read_to_string(
        format!("{}/data/historical-indexer.checkpoint", state.config.project_dir)
    ).ok();
    let hi_status = serde_json::json!({ "running": hi_running, "checkpoint": hi_checkpoint });

    // Process list
    let proc_names = ["brute_force.exe", "btcsolver_dashboard.exe", "bitcoind.exe",
                      "historical_indexer.exe", "dump_to_flat.exe", "llama-server.exe"];
    let mut proc_list: Vec<serde_json::Value> = Vec::new();
    for pn in &proc_names {
        let running = std::process::Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {}", pn), "/NH"])
            .output()
            .ok()
            .map(|o| {
                let s = String::from_utf8_lossy(&o.stdout).to_lowercase();
                s.contains(pn) && !s.contains("no tasks")
            })
            .unwrap_or(false);
        if running {
            // Get PID and memory
            let details = std::process::Command::new("tasklist")
                .args(["/FI", &format!("IMAGENAME eq {}", pn), "/NH", "/FO", "CSV"])
                .output()
                .ok();
            if let Some(ref o) = details {
                let s = String::from_utf8_lossy(&o.stdout).to_string();
                if let Some(line) = s.lines().next() {
                    let fields: Vec<&str> = line.split('"').collect();
                    let mem_kb = fields.get(8).and_then(|f| f.trim().parse::<u64>().ok()).unwrap_or(0);
                    let pid = fields.get(4).and_then(|f| f.trim().parse::<u32>().ok()).unwrap_or(0);
                    proc_list.push(serde_json::json!({ "name": pn, "running": true, "pid": pid, "mem_mb": mem_kb / 1024 }));
                    continue;
                }
            }
            proc_list.push(serde_json::json!({ "name": pn, "running": true }));
        } else {
            proc_list.push(serde_json::json!({ "name": pn, "running": false }));
        }
    }

    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "bitcoin_datadir": state.config.bitcoin_datadir,
        "index_loaded": index_loaded,
        "index_scripts": scripts,
        "auto_restart_bitcoind": state.config.auto_restart_bitcoind,
        "ram": ram,
        "ram_paused": ram_paused,
        "gpus_present": gpus_present,
        "core_utxo": core_utxo_status,
        "utxo_rebuild_in_progress": utxo_rebuild_in_progress,
        "keys_archive": keys_archive,
        "scan_bg": external_brute,
        "scan": match scan {
            Ok(mut s) => {
                // If UI scan not running but external brute is, mark running for status bar
                if !s.running && external_brute {
                    s.running = true;
                }
                serde_json::to_value(s).unwrap_or_default()
            }
            Err(e) => serde_json::json!({"error": e.to_string(), "running": external_brute}),
        },
        "bitcoind": match btc {
            // Light payload for UI (no raw RPC blob / full peer dump — was crashing some browsers)
            Ok(Ok(s)) => serde_json::json!({
                "running": s.running,
                "process_running": s.process_running,
                "rpc_ok": s.rpc_ok,
                "can_start": s.can_start,
                "can_stop": s.can_stop,
                "pid": s.pid,
                "blocks": s.blocks,
                "headers": s.headers,
                "blocks_behind": s.blocks_behind,
                "sync_percentage": s.sync_percentage,
                "verification_progress": s.verification_progress,
                "initialblockdownload": s.initialblockdownload,
                "is_synced": s.is_synced,
                "connections": s.connections,
                "simple_status": s.simple_status,
                "message": s.message,
                "datadir": s.datadir,
                "block_time_utc": s.block_time_utc,
                "mediantime_utc": s.mediantime_utc,
                "bestblockhash": s.bestblockhash,
                "uptime_human": s.uptime_human,
                "process_rss_mb": s.process_rss_mb,
            }),
            Ok(Err(e)) => serde_json::json!({
                "running": false,
                "is_synced": false,
                "rpc_ok": false,
                "simple_status": format!("Core: error ({})", e),
                "message": e.to_string()
            }),
            Err(_) => {
                let (proc, pid) = BitcoindManager::detect_process(&state.config.bitcoin_datadir);
                serde_json::json!({
                    "running": proc,
                    "process_running": proc,
                    "rpc_ok": false,
                    "is_synced": false,
                    "pid": pid,
                    "simple_status": if proc {
                        "Core: process OK · RPC busy"
                    } else {
                        "Core: STOPPED - click Restart"
                    },
                })
            },
        },
        "snapshot": match snap {
            Ok(s) => s,
            Err(e) => serde_json::json!({ "exists": false, "message": e.to_string() })
        },
        "dict": dict,
        "gpu": gpu_stats,
        "historical_indexer": hi_status,
        "processes": serde_json::Value::Array(proc_list),
    }))
}

/// GET /api/utxo1/stats — returns stats about the historical UTXO index
async fn utxo1_stats_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let project_dir = &state.config.project_dir;
    let mut paths = vec![
        format!("{}/data/historical-scripts-merged.bin", project_dir),
        format!("{}/data/historical-scripts.bin", project_dir),
    ];

    for path in paths {
        if let Ok(metadata) = std::fs::metadata(&path) {
            let file_size = metadata.len();
            // Try to read header for script count
            let mut count = 0u64;
            let mut version = 0u32;
            if let Ok(mut file) = std::fs::File::open(&path) {
                use std::io::Read;
                let mut header = [0u8; 20]; // magic(8) + version(4) + count(8)
                if file.read_exact(&mut header).is_ok() {
                    if &header[..8] == b"BTCSHIST" {
                        version = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
                        count = u64::from_le_bytes([
                            header[12], header[13], header[14], header[15],
                            header[16], header[17], header[18], header[19],
                        ]);
                    }
                }
            }

            return Json(serde_json::json!({
                "exists": true,
                "path": path,
                "file_size": file_size,
                "file_size_mb": (file_size as f64) / 1_048_576.0,
                "scripts": count,
                "version": version,
                "format": "BTCSHIST",
            }));
        }
    }

    // Check for tmp/intermediate files
    let tmp_path = format!("{}/data/historical-scripts.bin.tmp", project_dir);
    let tmp_size = std::fs::metadata(&tmp_path).map(|m| m.len()).unwrap_or(0);

    Json(serde_json::json!({
        "exists": false,
        "message": "No historical index found",
        "tmp_file_size": tmp_size,
        "tmp_file_size_gb": (tmp_size as f64) / 1_073_741_824.0,
    }))
}

/// POST /api/utxo1/query — query if a script hex has ever been active
async fn utxo1_query_handler(
    State(state): State<AppState>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let script_hex = body.get("script").and_then(|s| s.as_str()).unwrap_or("");
    if script_hex.is_empty() {
        return Json(serde_json::json!({"error": "Missing 'script' parameter (hex expected)"}));
    }

    let script_bytes = match hex::decode(script_hex) {
        Ok(b) => b,
        Err(e) => return Json(serde_json::json!({"error": format!("Invalid hex: {}", e)})),
    };

    let project_dir = &state.config.project_dir;
    let mut paths = vec![
        format!("{}/data/historical-scripts-merged.bin", project_dir),
        format!("{}/data/historical-scripts.bin", project_dir),
    ];

    for path in paths {
        if !std::path::Path::new(&path).exists() {
            continue;
        }

        // Use the historical_indexer CLI to query
        let cli_path = format!("{}/target/release/historical_indexer.exe", project_dir);
        if let Ok(output) = std::process::Command::new(&cli_path)
            .args(["query", "--script", &script_hex, "--index", &path])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            let stdout_lower = stdout.to_lowercase();
            if stdout_lower.contains("found") && !stdout_lower.contains("not found") {
                return Json(serde_json::json!({
                    "found": true,
                    "script": script_hex,
                    "index": path,
                }));
            }
            if stdout_lower.contains("not found") {
                // Check next index file
                continue;
            }
            // Fallback: return raw output
            return Json(serde_json::json!({
                "found": false,
                "script": script_hex,
                "index": path,
                "raw_output": format!("{} {}", stdout.trim(), stderr.trim()),
            }));
        }
    }

    Json(serde_json::json!({
        "found": false,
        "script": script_hex,
        "error": "No historical index available for query",
    }))
}

// ── Performance Benchmark ─────────────────────────────────────────────────

/// POST /api/benchmark/run — run CPU thread benchmark
async fn benchmark_run_handler(
    State(state): State<AppState>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    // Check if already running
    if state.benchmark.running.swap(true, Ordering::SeqCst) {
        return Json(serde_json::json!({
            "error": "Benchmark already running",
        }));
    }

    // Parse config
    let test_duration: u64 = body.get("duration_secs")
        .and_then(|v| v.as_u64()).unwrap_or(15);
    let gpu_batch_m: usize = body.get("gpu_batch_m")
        .and_then(|v| v.as_u64()).map(|v| v as usize).unwrap_or(32);
    let cpu_threads_list: Vec<usize> = body.get("cpu_threads")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_u64().map(|n| n as usize)).collect())
        .unwrap_or_else(|| vec![0, 2, 4, 8, 12, 16, 24, 32]);

    let project_dir = state.config.project_dir.clone();
    let snapshot_path = state.config.snapshot_path.clone();
    let bench = state.benchmark.clone();

    // Clear previous results
    {
        let mut results = bench.results.lock().unwrap();
        results.clear();
    }
    {
        let mut progress = bench.progress.lock().unwrap();
        *progress = Some(serde_json::json!({
            "phase": "starting",
            "current": 0,
            "total": cpu_threads_list.len(),
            "message": "Preparing benchmark...",
        }));
    }

    let config_count = cpu_threads_list.len();

    tokio::spawn(async move {
        let bin_path = find_brute_force_bin(&project_dir);
        let test_keys: u64 = (180_000_000_u64 * test_duration).max(500_000_000); // enough for stable measurement

        for (i, cpu_threads) in cpu_threads_list.iter().enumerate() {
            let total = cpu_threads_list.len();
            {
                let mut progress = bench.progress.lock().unwrap();
                *progress = Some(serde_json::json!({
                    "phase": "testing",
                    "current": i + 1,
                    "total": total,
                    "cpu_threads": cpu_threads,
                    "gpu_batch_m": gpu_batch_m,
                    "message": format!("Testing CPU={} threads… ({}/{})", cpu_threads, i + 1, total),
                }));
            }

            let label = if *cpu_threads == 0 {
                format!("GPU only")
            } else {
                format!("GPU + CPU {} threads", cpu_threads)
            };

            let kps = run_single_benchmark(&bin_path, &snapshot_path, *cpu_threads, gpu_batch_m, test_keys, test_duration, &project_dir).await;

            // Detect GPU count from nvidia-smi
            let gpu_count = count_gpus();
            let kps_per_gpu = if gpu_count > 0 { kps as f64 / gpu_count as f64 } else { 0.0 };

            let entry = BenchmarkEntry {
                label,
                cpu_threads: *cpu_threads,
                gpu_batch_m,
                keys_per_sec: kps,
                keys_per_sec_per_gpu: (kps_per_gpu / 1_000_000.0).round() * 1_000_000.0,
                test_duration_secs: test_duration,
                total_keys: test_keys,
            };

            let mut results = bench.results.lock().unwrap();
            results.push(entry);
        }

        // Mark complete
        bench.running.store(false, Ordering::SeqCst);
        {
            let mut progress = bench.progress.lock().unwrap();
            *progress = Some(serde_json::json!({
                "phase": "complete",
                "message": "Benchmark complete!",
            }));
        }

        tracing::info!("Benchmark complete: {} configs tested", cpu_threads_list.len());
    });

    Json(serde_json::json!({
        "success": true,
        "message": format!("Benchmark started: {} configs, {}s each", config_count, test_duration),
    }))
}

/// GET /api/benchmark/status — get benchmark progress and results
async fn benchmark_status_handler(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let running = state.benchmark.running.load(Ordering::SeqCst);
    let progress = state.benchmark.progress.lock().unwrap().clone();
    let results = state.benchmark.results.lock().unwrap().clone();

    // Find best config
    let best = results.iter().max_by_key(|e| e.keys_per_sec);

    Json(serde_json::json!({
        "running": running,
        "progress": progress,
        "results": results,
        "best": best,
    }))
}

/// POST /api/benchmark/reset — clear benchmark results
async fn benchmark_reset_handler(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    {
        let mut results = state.benchmark.results.lock().unwrap();
        results.clear();
    }
    {
        let mut progress = state.benchmark.progress.lock().unwrap();
        *progress = None;
    }
    Json(serde_json::json!({
        "success": true,
        "message": "Benchmark results cleared",
    }))
}

/// Run a single benchmark test: spawn brute_force, wait, read stats
async fn run_single_benchmark(
    bin_path: &str,
    snapshot_path: &str,
    cpu_threads: usize,
    gpu_batch_m: usize,
    test_keys: u64,
    test_duration_secs: u64,
    project_dir: &str,
) -> u64 {
    let stats_file = format!("{}/bench-stats-{}.json", project_dir, cpu_threads);

    let mut args = vec![
        "--snapshot-path".to_string(), snapshot_path.to_string(),
        "--threads".to_string(), cpu_threads.to_string(),
        "--batch-size".to_string(), "4194304".to_string(),
        "--count".to_string(), test_keys.to_string(),
        "--stats-interval".to_string(), "3".to_string(),
        "--use-gpu".to_string(),
        "--addr-types".to_string(), "legacy,segwit,wrapped,taproot".to_string(),
        "--transforms".to_string(), "identity".to_string(),
        "--max-snapshot-age".to_string(), "0".to_string(),
        "--output-file".to_string(), format!("{}/found-keys.json", project_dir),
        "--stats-file".to_string(), stats_file.clone(),
    ];

    let gpu_batch = (gpu_batch_m as u64) * 1_000_000;

    let mut cmd = std::process::Command::new(bin_path);
    cmd.args(&args)
        .env("BTC_GPU_LAUNCH", gpu_batch.to_string())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let Ok(mut child) = cmd.spawn() else {
        tracing::warn!("Benchmark spawn failed for CPU={} threads", cpu_threads);
        return 0;
    };

    let pid = child.id();

    // Wait for test duration
    tokio::time::sleep(Duration::from_secs(test_duration_secs)).await;

    // Kill the process
    let _ = child.kill();
    let _ = child.wait();

    // Fallback: ensure process is dead (Windows zombie cleanup)
    if let Ok(kill_output) = std::process::Command::new("taskkill")
        .args(["/F", "/IM", "brute_force.exe", "/PID", &pid.to_string()])
        .output()
    {
        let _ = kill_output;
    }

    // Read stats file
    let kps = read_benchmark_stats(&stats_file).await;
    let _ = std::fs::remove_file(&stats_file); // cleanup

    tracing::info!("Benchmark CPU={} threads: {} keys/sec", cpu_threads, kps);
    kps
}

async fn read_benchmark_stats(stats_file: &str) -> u64 {
    for _ in 0..10 {
        tokio::time::sleep(Duration::from_millis(200)).await;
        if let Ok(content) = std::fs::read_to_string(stats_file) {
            if let Ok(stats) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(kps) = stats.get("keys_per_sec").and_then(|v| v.as_u64()) {
                    return kps;
                }
                if let Some(kps) = stats.get("keys_per_sec_avg").and_then(|v| v.as_u64()) {
                    return kps;
                }
            }
        }
    }
    0
}

fn find_brute_force_bin(project_dir: &str) -> String {
    let candidates = [
        format!("{}/target/release/brute_force.exe", project_dir),
        format!("{}/brute_force.exe", project_dir),
        format!("{}/brute_force_gpu.exe", project_dir),
        r"C:\btcsolver-bin\brute_force.exe".to_string(),
    ];
    for c in &candidates {
        if std::path::Path::new(c).exists() {
            return c.clone();
        }
    }
    format!("{}/target/release/brute_force.exe", project_dir)
}

fn count_gpus() -> usize {
    let Ok(output) = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=index", "--format=csv,noheader,nounits"])
        .output() else {
        return 0;
    };
    let s = String::from_utf8_lossy(&output.stdout);
    s.lines().filter(|l| !l.trim().is_empty()).count()
}

async fn ideas_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "ideas": KEY_HUNT_IDEAS,
    }))
}

const KEY_HUNT_IDEAS: &[&str] = &[
    "Brainwallet classique: SHA256(phrase) → clé — tester endroit, envers caractères, envers mots, lower/upper, sans espaces",
    "Double hash: SHA256d(phrase) et SHA256(UTF-16LE) (wallets Windows/Android anciens)",
    "BIP39 partiel: 11 mots connus + bruteforce du 12e (2048 essais) — déjà un bat find-12th-word dans le repo",
    "BIP39 + passphrase oubliée: dictionnaire de passphrases courtes (pet, dates, villes) sur seed valide",
    "Chemins de dérivation non standards: m/0, m/0'/0', m/44'/0'/0'/0 (sans change), account 1–5",
    "Timestamps: epoch Unix / dates de naissance / anniversaires → SHA256 (timestamp_scan)",
    "Clés faibles RNG: patterns low-entropy (bytes croissants, clés proches de 1, séquences clavier qwerty)",
    "Minikeys Casascius (S… base58 22/30 chars) — encore des UTXO dust historiques",
    "Electrum seeds anciens (v1 seed → old electrum derivation, pas BIP39)",
    "Warpwallet / scrypt brainwallets (coûteux CPU — batch prioritaire sur phrases à fort signal)",
    "Phrases multi-langues: FR/EN mélange, accents strip, leet speak (a→4, e→3)",
    "Suffixes d’époque: « bitcoin 2011 », « satoshi », « mtgox », « silkroad »",
    "Corrélations sociales: usernames + années (sans doxxer — corpus public uniquement)",
    "Collision d’adresses P2PK non compressées (early 2009–2012) en plus du compressé",
    "Scan par plage de bits de clé privée autour d’une seed partielle (si bits connus)",
    "Utiliser dumptxoutset une fois tip atteint pour un snapshot UTXO exact tip (plus fiable que reparse blocks)",
    "Prioriser les UTXO non dépensés > dust pour réduire le bruit des matchs historiques",
    "Pipeline: Core tip → rebuild UTXO → reload index → dict waves → manual verify on-chain",
];
