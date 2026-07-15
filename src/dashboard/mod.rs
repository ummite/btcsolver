pub mod scan_manager;
pub mod bitcoind;
pub mod key_checker;

pub use scan_manager::ScanManager;
pub use bitcoind::BitcoindManager;
pub use key_checker::KeyChecker;

/// Shared configuration for the dashboard
#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct DashboardConfig {
    pub bin_dir: String,
    pub cache_dir: String,
    pub bitcoind_path: Option<String>,
    pub bitcoin_rpc_url: Option<String>,
    pub bitcoin_rpc_user: Option<String>,
    pub bitcoin_rpc_password: Option<String>,
    pub snapshot_path: String,
    pub max_snapshot_age_seconds: u64,
    pub auto_snapshot_interval_hours: u64,
    pub default_threads: usize,
    pub default_batch_size: usize,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            bin_dir: "C:\\btcsolver-bin".to_string(),
            cache_dir: "C:\\btcsolver-cache".to_string(),
            bitcoind_path: None,
            bitcoin_rpc_url: None,
            bitcoin_rpc_user: None,
            bitcoin_rpc_password: None,
            snapshot_path: "C:\\btcsolver-cache\\utxo-index.snapshot".to_string(),
            max_snapshot_age_seconds: 86400,
            auto_snapshot_interval_hours: 24,
            default_threads: 23,
            default_batch_size: 256000,
        }
    }
}

/// Current state of the dashboard, shared across handlers
pub use scan_manager::DashboardState;
