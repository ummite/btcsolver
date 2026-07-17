pub mod bitcoind;
pub mod dict_scan;
pub mod key_checker;
pub mod range_log;
pub mod scan_manager;

pub use bitcoind::BitcoindManager;
pub use dict_scan::DictScanManager;
pub use key_checker::KeyChecker;
pub use scan_manager::{DashboardState, ScanManager};

/// Shared configuration for the dashboard
#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct DashboardConfig {
    pub bin_dir: String,
    pub cache_dir: String,
    pub project_dir: String,
    /// Bitcoin Core datadir (blocks, chainstate, conf)
    pub bitcoin_datadir: String,
    pub bitcoind_path: Option<String>,
    pub bitcoin_cli_path: Option<String>,
    pub bitcoin_rpc_url: Option<String>,
    pub bitcoin_rpc_user: Option<String>,
    pub bitcoin_rpc_password: Option<String>,
    /// Blocks directory for UTXO indexer
    pub blocks_dir: String,
    /// XOR key for blk*.dat (zeros = plaintext blocks on W:)
    pub blocks_obf_key: String,
    pub snapshot_path: String,
    pub redb_path: String,
    pub max_snapshot_age_seconds: u64,
    pub auto_snapshot_interval_hours: u64,
    /// If true, dashboard watchdog restarts bitcoind when it dies
    pub auto_restart_bitcoind: bool,
    /// Seconds between auto-restart health checks
    pub auto_restart_check_secs: u64,
    pub default_threads: usize,
    pub default_batch_size: usize,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            bin_dir: r"Y:\btcsolver".to_string(),
            cache_dir: r"Y:\btcsolver".to_string(),
            project_dir: r"Y:\btcsolver".to_string(),
            bitcoin_datadir: r"W:\Bitcoin".to_string(),
            bitcoind_path: Some(r"W:\Bitcoin\bin\daemon\bitcoind.exe".to_string()),
            bitcoin_cli_path: Some(r"W:\Bitcoin\bin\daemon\bitcoin-cli.exe".to_string()),
            bitcoin_rpc_url: Some("http://127.0.0.1:8332".to_string()),
            bitcoin_rpc_user: Some("btcsolver".to_string()),
            bitcoin_rpc_password: Some("btcsolver_rpc_2026".to_string()),
            blocks_dir: r"W:\Bitcoin\blocks".to_string(),
            // W: blocks are plaintext (blocksxor=0)
            blocks_obf_key: "0000000000000000".to_string(),
            snapshot_path: r"Y:\btcsolver\utxo-index.snapshot".to_string(),
            redb_path: r"Y:\btcsolver\utxo-index.redb".to_string(),
            max_snapshot_age_seconds: 86400 * 7, // 7 days — flexible while node catches up
            auto_snapshot_interval_hours: 0,     // manual by default
            auto_restart_bitcoind: true,
            auto_restart_check_secs: 30,
            default_threads: (num_cpus::get().saturating_sub(1)).max(1),
            default_batch_size: 256000,
        }
    }
}
