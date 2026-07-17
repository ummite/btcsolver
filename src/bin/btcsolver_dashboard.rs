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
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
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

    /// Auto-restart bitcoind when process dies (watchdog)
    #[arg(long, default_value_t = true)]
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
        default_batch_size: 256000,
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
        let index_bg = index.clone();
        tokio::spawn(async move {
            tracing::info!("Loading UTXO index in background: {}", path);
            let loaded = tokio::task::spawn_blocking(move || load_index(&path))
                .await
                .ok()
                .flatten();
            if loaded.is_some() {
                tracing::info!("Background UTXO index ready");
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
                    // 4 × 15s = 60s sans avancement de curseur → relance
                    if stuck_ticks >= 4 {
                        tracing::warn!(
                            "auto-scan: brute bloqué (curseur figé) → restart multi-GPU"
                        );
                        let _ = ScanManager::stop(&dash).await;
                        let _ = std::process::Command::new("taskkill")
                            .args(["/IM", "brute_force.exe", "/F"])
                            .output();
                        stuck_ticks = 0;
                        last_pos = None;
                        // retombe dans !brute_on au prochain tick
                    }
                    continue;
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
                cfg.gpus = Some("0,1,2".to_string());
                cfg.random = false;
                cfg.use_range_log = true;
                // threads=0 → utilise cpu_pct (défaut 50 % des cœurs, configurable UI)
                if cfg.cpu_pct == 0 && cfg.threads == 0 {
                    cfg.cpu_pct = 50;
                }
                if cfg.range_step == 0 {
                    cfg.range_step = btcsolver::dashboard::range_log::DEFAULT_RANGE_STEP;
                }
                // Prochaine fenêtre : start_key nul = enchaîner après journal
                // (sauf si l'utilisateur a posé un manual_start via l'UI)
                cfg.start_key = None;
                cfg.end_key = None;
                cfg.count = 0; // → range_step
                cfg.batch_size = cfg.batch_size.max(2_097_152);
                let cpu_n = cfg.resolve_cpu_threads();
                match ScanManager::start(&dash, &cfg).await {
                    Ok(pid) => {
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
                        tracing::debug!("auto-scan: start deferred: {}", e);
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
        .route("/api/scan/config", get(scan_config_handler))
        .route("/api/scan/config", post(scan_config_update_handler))
        .route("/api/scan/ranges", get(scan_ranges_handler))
        .route("/api/scan/ranges", post(scan_ranges_post_handler))
        // Keys
        .route("/api/keys/check", post(check_key_handler))
        .route("/api/keys/batch", post(check_batch_handler))
        .route("/api/keys/archive", get(keys_archive_handler))
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
        // System
        .route("/api/system/health", get(health_handler))
        .route("/api/system/ideas", get(ideas_handler))
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
    // Enrichir pour l’UI (cœurs + threads calculés)
    let mut v = serde_json::to_value(&cfg).unwrap_or_default();
    if let Some(obj) = v.as_object_mut() {
        obj.insert("logical_cores".into(), serde_json::json!(cores));
        obj.insert("resolved_cpu_threads".into(), serde_json::json!(resolved));
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
    match BitcoindManager::generate_snapshot(&state.config).await {
        Ok(msg) => Ok(Json(serde_json::json!({ "success": true, "message": msg }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "success": false, "error": e.to_string() })),
        )),
    }
}

async fn snapshot_reload_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let path = state.config.snapshot_path.clone();
    let loaded = tokio::task::spawn_blocking(move || load_index(&path))
        .await
        .unwrap_or(None);
    let scripts = loaded.as_ref().map(|i| i.num_scripts).unwrap_or(0);
    let ok = loaded.is_some();
    *state.index.write().await = loaded;
    Json(serde_json::json!({
        "success": ok,
        "index_loaded": ok,
        "index_scripts": scripts,
        "path": state.config.snapshot_path,
    }))
}

// ── Health + ideas ────────────────────────────────────────────────────────

async fn health_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let scan = ScanManager::get_stats(&state.dashboard).await;
    let btc = tokio::time::timeout(Duration::from_secs(3), BitcoindManager::get_status(&state.config))
        .await;
    let mut snap = BitcoindManager::get_snapshot_info(&state.config).await;
    let dict = DictScanManager::status(&state.dict);
    let index_loaded = state.index.read().await.is_some();
    let scripts = state
        .index
        .read()
        .await
        .as_ref()
        .map(|i| i.num_scripts)
        .unwrap_or(0);
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

    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "bitcoin_datadir": state.config.bitcoin_datadir,
        "index_loaded": index_loaded,
        "index_scripts": scripts,
        "auto_restart_bitcoind": state.config.auto_restart_bitcoind,
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
                "simple_status": format!("Core: erreur ({})", e),
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
                        "Core: process OK · RPC occupé"
                    } else {
                        "Core: ARRÊTÉ — clique Relancer"
                    },
                })
            },
        },
        "snapshot": match snap {
            Ok(s) => s,
            Err(e) => serde_json::json!({ "exists": false, "message": e.to_string() })
        },
        "dict": dict,
    }))
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
