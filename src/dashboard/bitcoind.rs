use anyhow::Result;
use bitcoincore_rpc::RpcApi;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::dashboard::DashboardConfig;

/// Bitcoin Core sync status
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BitcoindStatus {
    pub running: bool,
    pub blocks: u64,
    pub headers: u64,
    pub sync_percentage: f64,
    pub chain: String,
    pub verification_progress: f64,
    pub warnings: String,
    pub connections: u64,
}

pub struct BitcoindManager;

impl BitcoindManager {
    /// Start bitcoind (detached)
    pub async fn start(config: &DashboardConfig) -> Result<()> {
        if Self::is_running(config).await {
            return Ok(());
        }

        let bitcoind_path = config
            .bitcoind_path
            .clone()
            .unwrap_or_else(|| r"C:\Program Files\Bitcoin\bin\bitcoind.exe".to_string());

        let rpc_user = config
            .bitcoin_rpc_user
            .as_deref()
            .unwrap_or("btcsolver");
        let rpc_password = config
            .bitcoin_rpc_password
            .as_deref()
            .unwrap_or("btcsolver");

        // Launch detached on Windows
        #[cfg(windows)]
        {
            let child = Command::new(&bitcoind_path)
                .args([
                    "-server=1",
                    &format!("-rpcuser={}", rpc_user),
                    &format!("-rpcpassword={}", rpc_password),
                    "-txindex=0",
                ])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()?;
            // Detach: don't wait for it
            drop(child);
        }

        #[cfg(not(windows))]
        {
            let _child = Command::new(&bitcoind_path)
                .args([
                    "-daemon",
                    "-server=1",
                    &format!("-rpcuser={}", rpc_user),
                    &format!("-rpcpassword={}", rpc_password),
                    "-txindex=0",
                ])
                .spawn()?;
        }

        // Wait for RPC to become available
        for i in 0..30 {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            if Self::get_status(config).await.is_ok() {
                eprintln!("bitcoind started and RPC available after {}s", i * 2);
                return Ok(());
            }
        }

        Err(anyhow::anyhow!(
            "bitcoind did not become available within 60s"
        ))
    }

    /// Stop bitcoind via RPC
    pub async fn stop(config: &DashboardConfig) -> Result<()> {
        let client = Self::get_rpc_client(config)?;
        let _ = client.stop()?;
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        Ok(())
    }

    /// Get Bitcoin Core sync status
    pub async fn get_status(config: &DashboardConfig) -> Result<BitcoindStatus> {
        let client = Self::get_rpc_client(config)?;

        let info = client.get_blockchain_info()?;
        let net = client.get_network_info()?;

        let total_blocks = info.headers.max(info.blocks);
        let sync_percentage = if total_blocks > 0 {
            (info.blocks as f64 / total_blocks as f64) * 100.0
        } else {
            0.0
        };

        Ok(BitcoindStatus {
            running: true,
            blocks: info.blocks,
            headers: info.headers,
            sync_percentage: (sync_percentage * 100.0).round() / 100.0,
            chain: info.chain.to_string(),
            verification_progress: info.verification_progress * 100.0,
            warnings: format!("{:?}", info.warnings),
            connections: net.connections as u64,
        })
    }

    /// Check if bitcoind is running and RPC is available
    pub async fn is_running(config: &DashboardConfig) -> bool {
        Self::get_status(config).await.is_ok()
    }

    /// Generate UTXO snapshot using full_utxo_indexer
    pub async fn generate_snapshot(config: &DashboardConfig) -> Result<String> {
        let indexer_path = format!("{}/full_utxo_indexer.exe", config.bin_dir);

        let output = Command::new(&indexer_path)
            .args([
                "--cache-dir".to_string(),
                config.cache_dir.clone(),
                "--snapshot-path".to_string(),
                config.snapshot_path.clone(),
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Indexer failed: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Get snapshot info (age, size, freshness)
    pub async fn get_snapshot_info(config: &DashboardConfig) -> Result<serde_json::Value> {
        let path = std::path::Path::new(&config.snapshot_path);
        let meta = std::fs::metadata(path)?;
        let age_secs = meta.modified()?.elapsed()?.as_secs();

        Ok(serde_json::json!({
            "path": config.snapshot_path,
            "size_bytes": meta.len(),
            "size_mb": ((meta.len() as f64 / 1_048_576.0) * 100.0).round() / 100.0,
            "age_seconds": age_secs,
            "age_hours": ((age_secs as f64 / 3600.0) * 100.0).round() / 100.0,
            "fresh": age_secs <= config.max_snapshot_age_seconds,
            "max_age_hours": ((config.max_snapshot_age_seconds as f64 / 3600.0) * 100.0).round() / 100.0,
        }))
    }

    fn get_rpc_client(config: &DashboardConfig) -> Result<bitcoincore_rpc::Client> {
        let rpc_url = config
            .bitcoin_rpc_url
            .clone()
            .unwrap_or_else(|| "http://127.0.0.1:8332".to_string());

        let auth = bitcoincore_rpc::Auth::UserPass(
            config
                .bitcoin_rpc_user
                .clone()
                .unwrap_or_else(|| "btcsolver".to_string()),
            config
                .bitcoin_rpc_password
                .clone()
                .unwrap_or_else(|| "btcsolver".to_string()),
        );

        bitcoincore_rpc::Client::new(&rpc_url, auth)
            .map_err(|e| anyhow::anyhow!("RPC client error: {}", e))
    }
}
