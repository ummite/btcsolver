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
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

use btcsolver::dashboard::bitcoind::BitcoindManager;
use btcsolver::dashboard::key_checker::{KeyChecker, KeyFormat};
use btcsolver::dashboard::scan_manager::{ScanConfig, ScanManager};
use btcsolver::dashboard::{DashboardConfig, DashboardState};
use btcsolver::flat_index::FlatIndex;

/// BTC Solver Dashboard — web interface for real-time monitoring and key checking
#[derive(Parser, Debug)]
#[command(name = "btcsolver_dashboard", version, about)]
struct Cli {
    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Path to Bitcoin Core binary
    #[arg(long)]
    bitcoind_path: Option<String>,

    /// Bitcoin RPC URL
    #[arg(long, default_value = "http://127.0.0.1:8332")]
    rpc_url: String,

    /// Bitcoin RPC username
    #[arg(long, default_value = "btcsolver")]
    rpc_user: String,

    /// Bitcoin RPC password
    #[arg(long, default_value = "btcsolver")]
    rpc_password: String,

    /// Path to UTXO snapshot file
    #[arg(long)]
    snapshot_path: Option<String>,

    /// Directory with compiled binaries
    #[arg(long, default_value = r"C:\btcsolver-bin")]
    bin_dir: String,

    /// Cache directory
    #[arg(long, default_value = r"C:\btcsolver-cache")]
    cache_dir: String,

    /// Max snapshot age in seconds (0 = disable check)
    #[arg(long, default_value = "86400")]
    max_snapshot_age: u64,

    /// Auto snapshot refresh interval in hours (0 = disable)
    #[arg(long, default_value = "24")]
    snapshot_interval_hours: u64,

    /// Directory with static files
    #[arg(long, default_value = "static/dashboard")]
    static_dir: String,
}

/// Application state shared across handlers
#[derive(Clone)]
struct AppState {
    dashboard: DashboardState,
    config: DashboardConfig,
    index: Arc<Option<FlatIndex>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let config = DashboardConfig {
        bitcoind_path: cli.bitcoind_path,
        bitcoin_rpc_url: Some(cli.rpc_url),
        bitcoin_rpc_user: Some(cli.rpc_user),
        bitcoin_rpc_password: Some(cli.rpc_password),
        snapshot_path: cli
            .snapshot_path
            .unwrap_or_else(|| format!(r"{}\utxo-index.snapshot", cli.cache_dir)),
        max_snapshot_age_seconds: cli.max_snapshot_age,
        auto_snapshot_interval_hours: cli.snapshot_interval_hours,
        default_threads: 23,
        default_batch_size: 256000,
        bin_dir: cli.bin_dir,
        cache_dir: cli.cache_dir,
    };

    let state = DashboardState::new(config.clone());

    // Load FlatIndex for key checking (min_value=0 = all UTXOs)
    let index = match FlatIndex::load_from_snapshot(&config.snapshot_path, 0) {
        Ok(idx) => {
            tracing::info!(
                "Loaded UTXO index: {} scripts, {:.1} MB",
                idx.num_scripts,
                idx.memory_usage_bytes() as f64 / 1_048_576.0
            );
            Some(idx)
        }
        Err(e) => {
            tracing::warn!(
                "Failed to load UTXO index: {} — key checking will be unavailable",
                e
            );
            None
        }
    };

    // Auto-refresh snapshot task
    if config.auto_snapshot_interval_hours > 0 {
        let cfg = config.clone();
        tokio::spawn(async move {
            let interval = Duration::from_secs(cfg.auto_snapshot_interval_hours * 3600);
            loop {
                tokio::time::sleep(interval).await;
                tracing::info!("Auto-refreshing UTXO snapshot...");
                match BitcoindManager::generate_snapshot(&cfg).await {
                    Ok(msg) => tracing::info!("Snapshot refreshed: {}", msg),
                    Err(e) => tracing::error!("Snapshot refresh failed: {}", e),
                }
            }
        });
    }

    let app_state = AppState {
        dashboard: state,
        config,
        index: Arc::new(index),
    };

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(ws_handler))
        // Scan API
        .route("/api/scan/stats", get(scan_stats_handler))
        .route("/api/scan/start", post(scan_start_handler))
        .route("/api/scan/stop", post(scan_stop_handler))
        .route("/api/scan/config", get(scan_config_handler))
        .route("/api/scan/config", post(scan_config_update_handler))
        // Key checking
        .route("/api/keys/check", post(check_key_handler))
        .route("/api/keys/batch", post(check_batch_handler))
        // Bitcoin Core
        .route("/api/bitcoind/status", get(bitcoind_status_handler))
        .route("/api/bitcoind/start", post(bitcoind_start_handler))
        .route("/api/bitcoind/stop", post(bitcoind_stop_handler))
        // Snapshot
        .route("/api/snapshot/info", get(snapshot_info_handler))
        .route("/api/snapshot/refresh", post(snapshot_refresh_handler))
        // System
        .route("/api/system/health", get(health_handler))
        // Static assets
        .nest_service("/static", ServeDir::new(&cli.static_dir))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], cli.port));
    tracing::info!("Dashboard starting on http://{}", addr);
    tracing::info!("Open your browser at http://localhost:{}", cli.port);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Static index
// ---------------------------------------------------------------------------

async fn index_handler() -> impl IntoResponse {
    match std::fs::read_to_string("static/dashboard/index.html") {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Html(
                r#"<!DOCTYPE html><html><body style="background:#0d1117;color:#c9d1d9;font-family:sans-serif;padding:2rem">
                <h1>BTC Solver Dashboard</h1>
                <p>static/dashboard/index.html not found. Place frontend files there.</p>
                <p>API is available at <code>/api/system/health</code></p>
                </body></html>"#
                    .to_string(),
            ),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// WebSocket — real-time stats
// ---------------------------------------------------------------------------

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_stream(socket, state))
}

async fn ws_stream(socket: WebSocket, state: AppState) {
    let (mut sender, mut _receiver) = socket.split();

    if let Ok(stats) = ScanManager::get_stats(&state.dashboard).await {
        let msg = serde_json::json!({ "type": "scan_stats", "data": stats });
        if sender.send(Message::Text(msg.to_string())).await.is_err() {
            return;
        }
    }

    let mut interval = tokio::time::interval(Duration::from_secs(3));
    loop {
        interval.tick().await;

        let payload = match ScanManager::get_stats(&state.dashboard).await {
            Ok(stats) => serde_json::json!({ "type": "scan_stats", "data": stats }),
            Err(e) => serde_json::json!({ "type": "error", "message": e.to_string() }),
        };

        if sender
            .send(Message::Text(payload.to_string()))
            .await
            .is_err()
        {
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// Scan API
// ---------------------------------------------------------------------------

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
            Ok(Json(serde_json::json!({
                "success": true,
                "pid": pid,
                "message": format!("Scan started with PID {}", pid),
            })))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            })),
        )),
    }
}

async fn scan_stop_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    match ScanManager::stop(&state.dashboard).await {
        Ok(()) => Json(serde_json::json!({ "success": true, "message": "Scan stopped" })),
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}

async fn scan_config_handler(State(state): State<AppState>) -> Json<ScanConfig> {
    Json(ScanManager::get_config(&state.dashboard).await)
}

async fn scan_config_update_handler(
    State(state): State<AppState>,
    Json(config): Json<ScanConfig>,
) -> Json<serde_json::Value> {
    ScanManager::update_config(&state.dashboard, config).await;
    Json(serde_json::json!({ "success": true, "message": "Config updated" }))
}

// ---------------------------------------------------------------------------
// Key checking
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct CheckKeyRequest {
    key: String,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    passphrase: Option<String>,
}

#[derive(serde::Deserialize)]
struct CheckBatchRequest {
    keys: Vec<String>,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    passphrase: Option<String>,
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
    let index = match &*state.index {
        Some(idx) => idx,
        None => {
            return Json(serde_json::json!({ "error": "UTXO index not loaded" }));
        }
    };

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

    let keys = match KeyChecker::parse_private_key(
        &req.key,
        format,
        req.passphrase.as_deref(),
        None,
    ) {
        Ok(k) => k,
        Err(e) => {
            return Json(serde_json::json!({
                "error": format!("Failed to parse key: {}", e)
            }));
        }
    };

    let format_str = format.to_string();
    let mut results = Vec::new();
    let mut total_balance = 0u64;

    for key_bytes in keys {
        let result =
            KeyChecker::check_key(index, key_bytes, req.key.clone(), format_str.clone()).await;
        total_balance += result.total_balance_sats;
        results.push(result);
    }

    Json(serde_json::json!({
        "success": true,
        "count": results.len(),
        "results": results,
        "total_balance_sats": total_balance,
        "total_balance_btc": total_balance as f64 / 1e8,
    }))
}

async fn check_batch_handler(
    State(state): State<AppState>,
    Json(req): Json<CheckBatchRequest>,
) -> Json<serde_json::Value> {
    let index = match &*state.index {
        Some(idx) => idx,
        None => {
            return Json(serde_json::json!({ "error": "UTXO index not loaded" }));
        }
    };

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

        let keys = match KeyChecker::parse_private_key(
            key_str,
            format,
            req.passphrase.as_deref(),
            None,
        ) {
            Ok(k) => k,
            Err(_) => continue,
        };

        let format_str = format.to_string();
        for key_bytes in keys {
            let result =
                KeyChecker::check_key(index, key_bytes, key_str.clone(), format_str.clone()).await;
            total_balance += result.total_balance_sats;
            results.push(result);
        }
    }

    Json(serde_json::json!({
        "success": true,
        "count": results.len(),
        "results": results,
        "total_balance_sats": total_balance,
        "total_balance_btc": total_balance as f64 / 1e8,
    }))
}

// ---------------------------------------------------------------------------
// Bitcoin Core
// ---------------------------------------------------------------------------

async fn bitcoind_status_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    match BitcoindManager::get_status(&state.config).await {
        Ok(status) => Json(serde_json::to_value(status).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "running": false, "error": e.to_string() })),
    }
}

async fn bitcoind_start_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    match BitcoindManager::start(&state.config).await {
        Ok(()) => Json(serde_json::json!({ "success": true, "message": "Bitcoin Core started" })),
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}

async fn bitcoind_stop_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    match BitcoindManager::stop(&state.config).await {
        Ok(()) => Json(serde_json::json!({ "success": true, "message": "Bitcoin Core stopping" })),
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}

// ---------------------------------------------------------------------------
// Snapshot
// ---------------------------------------------------------------------------

async fn snapshot_info_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    match BitcoindManager::get_snapshot_info(&state.config).await {
        Ok(info) => Json(info),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
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

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

async fn health_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let scan_stats = ScanManager::get_stats(&state.dashboard).await;
    let bitcoind_status = BitcoindManager::get_status(&state.config).await;
    let snapshot_info = BitcoindManager::get_snapshot_info(&state.config).await;

    Json(serde_json::json!({
        "status": "ok",
        "scan": match scan_stats {
            Ok(s) => serde_json::to_value(s).unwrap_or_default(),
            Err(e) => serde_json::json!({"error": e.to_string()}),
        },
        "bitcoind": match bitcoind_status {
            Ok(s) => serde_json::to_value(s).unwrap_or_default(),
            Err(e) => serde_json::json!({"running": false, "error": e.to_string()}),
        },
        "snapshot": match snapshot_info {
            Ok(s) => s,
            Err(e) => serde_json::json!({"error": e.to_string()}),
        },
        "index_loaded": state.index.is_some(),
    }))
}
