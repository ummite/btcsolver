use anyhow::{Context, Result};
use bitcoincore_rpc::RpcApi;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::process::Command;

use crate::dashboard::DashboardConfig;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LogProgress {
    pub height: u64,
    pub progress: f64,
    pub block_date: String,
    pub cache: String,
    pub headers_presync: u64,
}

fn extract_after<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.split(key).nth(1).map(|s| s.trim())
}

/// Bitcoin Core sync / process status (process + rich RPC)
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BitcoindStatus {
    /// True if process is up OR RPC answers (safe for UI "online")
    pub running: bool,
    pub process_running: bool,
    pub rpc_ok: bool,
    pub can_start: bool,
    pub can_stop: bool,
    pub pid: Option<u32>,
    /// Process working set MB (Windows tasklist when available)
    pub process_rss_mb: Option<f64>,

    // ── Chain ──
    pub blocks: u64,
    pub headers: u64,
    pub blocks_behind: u64,
    pub sync_percentage: f64,
    pub chain: String,
    pub bestblockhash: String,
    pub difficulty: f64,
    pub chainwork: String,
    pub mediantime: u64,
    pub mediantime_utc: String,
    pub block_time: u64,
    pub block_time_utc: String,
    pub verification_progress: f64,
    pub initialblockdownload: bool,
    pub pruned: bool,
    pub pruneheight: Option<u64>,
    pub automatic_pruning: Option<bool>,
    pub warnings: String,
    pub size_on_disk_bytes: u64,
    pub size_on_disk_gb: f64,

    // ── Network ──
    pub connections: u64,
    pub connections_in: u64,
    pub connections_out: u64,
    pub networkactive: bool,
    pub version: u64,
    pub subversion: String,
    pub protocolversion: u64,
    pub localservices: Vec<String>,
    pub relay_fee: f64,
    pub timeoffset: i64,
    pub uptime_seconds: u64,
    pub uptime_human: String,
    pub networkhashps: f64,
    pub bytes_recv: u64,
    pub bytes_sent: u64,
    pub bytes_recv_human: String,
    pub bytes_sent_human: String,

    // ── Mempool ──
    pub mempool_loaded: bool,
    pub mempool_size: u64,
    pub mempool_bytes: u64,
    pub mempool_usage: u64,
    pub mempool_max: u64,
    pub mempool_min_fee: f64,
    pub mempool_total_fee: f64,
    pub mempool_unbroadcast: u64,

    // ── Memory (bitcoind locked pool) ──
    pub mem_locked_used: u64,
    pub mem_locked_total: u64,
    pub mem_locked_free: u64,

    // ── Peers (summary + sample) ──
    pub peers: Vec<PeerSummary>,
    pub peers_by_network: serde_json::Value,

    // ── Meta ──
    pub datadir: String,
    pub message: String,
    pub auth_method: String,
    /// Simple: fully caught up (not IBD, behind <= 2, has progress)
    pub is_synced: bool,
    /// Human one-liner for the sticky status bar
    pub simple_status: String,
    /// Full raw RPC blobs for power users
    pub raw: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PeerSummary {
    pub id: u64,
    pub addr: String,
    pub addrbind: String,
    pub network: String,
    pub inbound: bool,
    pub subver: String,
    pub version: i64,
    pub startingheight: i64,
    pub synced_headers: i64,
    pub synced_blocks: i64,
    pub pingtime_ms: f64,
    pub minping_ms: f64,
    pub bytessent: u64,
    pub bytesrecv: u64,
    pub connection_type: String,
    pub bip152_hb_to: bool,
    pub bip152_hb_from: bool,
}

pub struct BitcoindManager;

impl BitcoindManager {
    fn bitcoind_exe(config: &DashboardConfig) -> String {
        if let Some(ref p) = config.bitcoind_path {
            if Path::new(p).exists() {
                return p.clone();
            }
        }
        for c in [
            r"W:\Bitcoin\bin\daemon\bitcoind.exe",
            r"W:\Bitcoin\bin\bitcoind.exe",
            r"Y:\Bitcoin\bin\daemon\bitcoind.exe",
        ] {
            if Path::new(c).exists() {
                return c.to_string();
            }
        }
        r"W:\Bitcoin\bin\daemon\bitcoind.exe".to_string()
    }

    fn normalize_datadir(p: &str) -> String {
        let p = p.trim().trim_end_matches(['\\', '/']);
        // Case-insensitive compare on Windows
        #[cfg(windows)]
        {
            return p.to_lowercase();
        }
        #[cfg(not(windows))]
        {
            p.to_string()
        }
    }

    /// Detect bitcoind process(es). Prefer those whose cmdline contains our datadir.
    pub fn detect_process(datadir: &str) -> (bool, Option<u32>) {
        let want = Self::normalize_datadir(datadir);

        // 1) bitcoind.pid in datadir (most precise when present)
        let pid_file = Path::new(datadir).join("bitcoind.pid");
        if let Ok(s) = std::fs::read_to_string(&pid_file) {
            if let Ok(pid) = s.trim().parse::<u32>() {
                if Self::pid_alive(pid) {
                    return (true, Some(pid));
                }
            }
        }

        // 2) Scan processes — ONLY real processes count (.lock alone is NOT "running")
        let procs = Self::list_bitcoind_processes();
        if procs.is_empty() {
            // Stale lock/pid from crash: not running
            return (false, None);
        }

        // Prefer match on datadir in command line
        for (pid, cmd) in &procs {
            let cmd_n = Self::normalize_datadir(cmd);
            let want_slash = want.replace('/', "\\");
            if cmd_n.contains(&want) || cmd_n.contains(&want_slash) {
                return (true, Some(*pid));
            }
            if cmd.to_lowercase().contains(&format!("-datadir={}", want))
                || cmd.to_lowercase().contains(&format!("-datadir=\"{}\"", want))
            {
                return (true, Some(*pid));
            }
        }

        // 3) Single bitcoind on machine → assume ours (portable install)
        if procs.len() == 1 {
            return (true, Some(procs[0].0));
        }

        // Multiple bitcoinds, none matched our datadir — conservative: report running
        // so we don't start a third instance
        (true, procs.first().map(|(p, _)| *p))
    }

    /// Parse last UpdateTip / header sync line from debug.log (when RPC is busy or lagging).
    /// Only reads the last ~256 KiB — never the whole file (debug.log can be 100+ MB).
    pub fn parse_debug_log_progress(datadir: &str) -> Option<LogProgress> {
        use std::io::{Read, Seek, SeekFrom};
        let path = Path::new(datadir).join("debug.log");
        let mut f = std::fs::File::open(&path).ok()?;
        let len = f.metadata().ok()?.len();
        const TAIL: u64 = 256 * 1024;
        let start = len.saturating_sub(TAIL);
        f.seek(SeekFrom::Start(start)).ok()?;
        let mut buf = String::new();
        f.read_to_string(&mut buf).ok()?;
        // If we seek mid-line, drop the partial first line
        let tail = if start > 0 {
            buf.split_once('\n').map(|(_, rest)| rest).unwrap_or(&buf)
        } else {
            buf.as_str()
        };

        let mut progress = LogProgress::default();

        for line in tail.lines().rev() {
            if progress.height == 0 {
                if line.contains("UpdateTip:") {
                    // height=44723 ... date='2010-03-10T16:58:08Z' progress=0.000033
                    if let Some(h) = extract_after(line, "height=") {
                        progress.height = h.parse().unwrap_or(0);
                    }
                    if let Some(p) = extract_after(line, "progress=") {
                        progress.progress = p
                            .split_whitespace()
                            .next()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0.0);
                    }
                    if let Some(rest) = line.split("date='").nth(1) {
                        progress.block_date = rest.split('\'').next().unwrap_or("").to_string();
                    }
                    if let Some(rest) = line.split("cache=").nth(1) {
                        progress.cache = rest.split_whitespace().next().unwrap_or("").to_string();
                    }
                }
            }
            if progress.headers_presync == 0 {
                if line.contains("Pre-synchronizing blockheaders")
                    || line.contains("Synchronizing blockheaders")
                {
                    if let Some(h) = extract_after(line, "height: ") {
                        let h = h.split_whitespace().next().unwrap_or("0");
                        progress.headers_presync = h.replace(',', "").parse().unwrap_or(0);
                    }
                }
            }
            if progress.height > 0 && (progress.headers_presync > 0 || progress.block_date.len() > 0)
            {
                break;
            }
            // enough if we have UpdateTip
            if progress.height > 0 && !line.contains("UpdateTip") {
                // keep scanning a bit for headers line, but don't go forever
            }
        }

        if progress.height > 0 || progress.headers_presync > 0 {
            Some(progress)
        } else {
            None
        }
    }

    fn pid_alive(pid: u32) -> bool {
        #[cfg(windows)]
        {
            let out = std::process::Command::new("tasklist")
                .args(["/FI", &format!("PID eq {}", pid), "/NH"])
                .output();
            if let Ok(o) = out {
                let s = String::from_utf8_lossy(&o.stdout);
                return s.contains(&pid.to_string()) && !s.to_lowercase().contains("no tasks");
            }
            false
        }
        #[cfg(not(windows))]
        {
            Path::new(&format!("/proc/{}", pid)).exists()
        }
    }

    fn list_bitcoind_processes() -> Vec<(u32, String)> {
        let mut out = Vec::new();

        #[cfg(windows)]
        {
            // Fast path: tasklist (PID only)
            if let Ok(o) = std::process::Command::new("tasklist")
                .args(["/FI", "IMAGENAME eq bitcoind.exe", "/FO", "CSV", "/NH"])
                .output()
            {
                let text = String::from_utf8_lossy(&o.stdout);
                for line in text.lines() {
                    if line.to_lowercase().contains("no tasks") {
                        continue;
                    }
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 2 {
                        let pid_s = parts[1].trim().trim_matches('"');
                        if let Ok(pid) = pid_s.parse::<u32>() {
                            out.push((pid, String::new()));
                        }
                    }
                }
            }

            // Optional: enrich with CommandLine via WMIC only if multiple PIDs
            if out.len() > 1 {
                if let Ok(o) = std::process::Command::new("wmic")
                    .args([
                        "process",
                        "where",
                        "name='bitcoind.exe'",
                        "get",
                        "ProcessId,CommandLine",
                        "/FORMAT:LIST",
                    ])
                    .output()
                {
                    let text = String::from_utf8_lossy(&o.stdout);
                    let mut enriched = Vec::new();
                    let mut cmd = String::new();
                    let mut pid: Option<u32> = None;
                    for line in text.lines() {
                        let line = line.trim();
                        if line.is_empty() {
                            if let Some(p) = pid.take() {
                                enriched.push((p, std::mem::take(&mut cmd)));
                            }
                            continue;
                        }
                        if let Some(rest) = line.strip_prefix("CommandLine=") {
                            cmd = rest.to_string();
                        } else if let Some(rest) = line.strip_prefix("ProcessId=") {
                            pid = rest.trim().parse().ok();
                        }
                    }
                    if let Some(p) = pid {
                        enriched.push((p, cmd));
                    }
                    if !enriched.is_empty() {
                        out = enriched;
                    }
                }
            }
        }

        #[cfg(not(windows))]
        {
            if let Ok(o) = std::process::Command::new("pgrep")
                .args(["-a", "bitcoind"])
                .output()
            {
                for line in String::from_utf8_lossy(&o.stdout).lines() {
                    let mut it = line.splitn(2, ' ');
                    if let (Some(pid_s), cmd) = (it.next(), it.next().unwrap_or("")) {
                        if let Ok(pid) = pid_s.parse() {
                            out.push((pid, cmd.to_string()));
                        }
                    }
                }
            }
        }

        out
    }

    /// Full status: process detection + optional RPC (never lies if process is up)
    pub async fn get_status(config: &DashboardConfig) -> Result<BitcoindStatus> {
        let datadir = config.bitcoin_datadir.clone();
        let (process_running, pid) = Self::detect_process(&datadir);
        let log_progress = Self::parse_debug_log_progress(&datadir);

        // RPC with timeout handled by caller; try multi-auth here
        let rpc = tokio::task::spawn_blocking({
            let cfg = config.clone();
            move || Self::rpc_blockchain_snapshot(&cfg)
        })
        .await
        .unwrap_or(Err(anyhow::anyhow!("RPC task join failed")));

        match rpc {
            Ok((status, auth_method)) => {
                let mut s = status;
                s.rpc_ok = true;
                s.running = true;
                s.can_start = false;
                s.can_stop = true;
                s.pid = pid.or(s.pid);
                s.process_running = process_running || pid.is_some() || s.rpc_ok;
                s.process_rss_mb = s.pid.and_then(Self::process_rss_mb).or(s.process_rss_mb);
                s.auth_method = auth_method;
                // Enrich with log if RPC still shows height 0 but log is further (lag)
                if let Some(ref lp) = log_progress {
                    if s.blocks < lp.height {
                        // RPC may be stale/busy; trust higher log tip for display
                        if s.blocks == 0 && lp.height > 0 {
                            s.blocks = lp.height;
                            s.verification_progress = lp.progress * 100.0;
                            if !lp.block_date.is_empty() {
                                s.block_time_utc = lp.block_date.clone();
                                s.mediantime_utc = lp.block_date.clone();
                            }
                        }
                    }
                    if s.headers == 0 && lp.headers_presync > 0 {
                        s.headers = lp.headers_presync;
                    }
                    s.blocks_behind = s.headers.saturating_sub(s.blocks);
                    if s.headers > 0 {
                        s.sync_percentage =
                            ((s.blocks as f64 / s.headers.max(1) as f64) * 10000.0).round() / 100.0;
                    }
                    Self::refresh_simple_status(&mut s, Some(lp));
                } else {
                    Self::refresh_simple_status(&mut s, None);
                }
                if s.message.is_empty() {
                    s.message = "RPC OK".into();
                }
                Ok(s)
            }
            Err(e) => {
                let mut s = BitcoindStatus {
                    datadir: datadir.clone(),
                    pid,
                    process_rss_mb: pid.and_then(Self::process_rss_mb),
                    ..Default::default()
                };
                if process_running {
                    s.running = true;
                    s.process_running = true;
                    s.rpc_ok = false;
                    s.can_start = false;
                    s.can_stop = true;
                    s.initialblockdownload = true;
                    s.is_synced = false;
                    s.auth_method = "none".into();
                    if let Some(ref lp) = log_progress {
                        s.blocks = lp.height;
                        s.headers = lp.headers_presync.max(lp.height);
                        s.verification_progress = lp.progress * 100.0;
                        s.block_time_utc = lp.block_date.clone();
                        s.mediantime_utc = lp.block_date.clone();
                        s.blocks_behind = s.headers.saturating_sub(s.blocks);
                        if s.headers > 0 {
                            s.sync_percentage = ((s.blocks as f64 / s.headers as f64) * 10000.0)
                                .round()
                                / 100.0;
                        }
                        s.message = format!(
                            "RPC occupé — progress log: height {} ({}) cache={}",
                            lp.height, lp.block_date, lp.cache
                        );
                        Self::refresh_simple_status(&mut s, Some(lp));
                    } else {
                        s.message = format!(
                            "Process running (PID {:?}) but RPC not ready: {}",
                            pid, e
                        );
                        s.simple_status = "Core: process OK · RPC en chargement…".into();
                    }
                } else {
                    s.running = false;
                    s.can_start = true;
                    s.can_stop = false;
                    s.is_synced = false;
                    s.message = format!("Stopped ({})", e);
                    if let Some(ref lp) = log_progress {
                        s.blocks = lp.height;
                        s.simple_status = format!(
                            "Core: ARRÊTÉ (dernier tip log ~{}) — clique Relancer",
                            lp.height
                        );
                    } else {
                        s.simple_status = "Core: ARRÊTÉ — clique Relancer".into();
                    }
                    s.auth_method = "none".into();
                }
                Ok(s)
            }
        }
    }

    fn refresh_simple_status(s: &mut BitcoindStatus, log: Option<&LogProgress>) {
        s.is_synced = !s.initialblockdownload
            && s.blocks_behind <= 2
            && s.headers > 0
            && s.blocks > 0
            && s.rpc_ok;

        if s.is_synced {
            s.simple_status = format!(
                "Core: À JOUR · bloc {} · {} peers",
                s.blocks, s.connections
            );
            return;
        }

        if s.initialblockdownload || !s.rpc_ok {
            if let Some(lp) = log {
                if lp.height > 0 {
                    s.simple_status = format!(
                        "Core: SYNC en cours · ~bloc {} ({}) · progress {:.4}% · PAS à jour",
                        lp.height,
                        if lp.block_date.is_empty() {
                            "?"
                        } else {
                            &lp.block_date
                        },
                        lp.progress * 100.0
                    );
                    return;
                }
                if lp.headers_presync > 0 {
                    s.simple_status = format!(
                        "Core: pré-sync headers ~{} · PAS à jour",
                        lp.headers_presync
                    );
                    return;
                }
            }
            if s.headers == 0 && s.blocks == 0 {
                s.simple_status = format!(
                    "Core: SYNC en cours · {} peers · PAS à jour",
                    s.connections
                );
            } else {
                s.simple_status = format!(
                    "Core: SYNC {:.2}% · blocs {}/{} · retard {} · PAS à jour",
                    s.sync_percentage, s.blocks, s.headers, s.blocks_behind
                );
            }
        } else {
            s.simple_status = format!(
                "Core: en ligne · blocs {} · retard {} · PAS à jour",
                s.blocks, s.blocks_behind
            );
        }
    }

    fn process_rss_mb(pid: u32) -> Option<f64> {
        #[cfg(windows)]
        {
            let out = std::process::Command::new("tasklist")
                .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
                .output()
                .ok()?;
            let s = String::from_utf8_lossy(&out.stdout);
            // "bitcoind.exe","1234","Session","1","64 624 K" (NBSP / locale)
            let parts: Vec<&str> = s.split(',').collect();
            if parts.len() >= 5 {
                let mem: String = parts[4]
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect();
                if let Ok(kb) = mem.parse::<f64>() {
                    return Some((kb / 1024.0 * 10.0).round() / 10.0);
                }
            }
            None
        }
        #[cfg(not(windows))]
        {
            let _ = pid;
            None
        }
    }

    fn fmt_bytes(n: u64) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;
        let x = n as f64;
        if x >= GB {
            format!("{:.2} GiB", x / GB)
        } else if x >= MB {
            format!("{:.1} MiB", x / MB)
        } else if x >= KB {
            format!("{:.0} KiB", x / KB)
        } else {
            format!("{} B", n)
        }
    }

    fn fmt_uptime(secs: u64) -> String {
        let d = secs / 86400;
        let h = (secs % 86400) / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        if d > 0 {
            format!("{}d {:02}h {:02}m", d, h, m)
        } else if h > 0 {
            format!("{}h {:02}m {:02}s", h, m, s)
        } else {
            format!("{}m {:02}s", m, s)
        }
    }

    fn utc_from_unix(ts: u64) -> String {
        use chrono::{TimeZone, Utc};
        Utc.timestamp_opt(ts as i64, 0)
            .single()
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or_else(|| ts.to_string())
    }

    fn rpc_blockchain_snapshot(
        config: &DashboardConfig,
    ) -> Result<(BitcoindStatus, String)> {
        let (client, auth_method) = Self::get_rpc_client(config)?;
        let info = client.get_blockchain_info()?;

        let net = client.get_network_info().ok();
        let mempool = client.get_mempool_info().ok();
        let net_totals = client.get_net_totals().ok();
        let uptime = client.uptime().unwrap_or(0);
        let networkhashps = client.get_network_hash_ps(Some(120), None).unwrap_or(0.0);
        // Prefer raw JSON — typed GetPeerInfoResult often fails on newer Core fields
        let peers_json: Vec<serde_json::Value> = client
            .call::<Vec<serde_json::Value>>("getpeerinfo", &[])
            .or_else(|_| {
                client
                    .get_peer_info()
                    .map(|v| {
                        v.into_iter()
                            .filter_map(|p| serde_json::to_value(p).ok())
                            .collect()
                    })
            })
            .unwrap_or_default();
        let meminfo: Option<serde_json::Value> = client.call("getmemoryinfo", &[]).ok();
        let chain_tx: Option<serde_json::Value> = client
            .call("getchaintxstats", &[serde_json::json!(null)])
            .ok()
            .or_else(|| client.call("getchaintxstats", &[]).ok());
        let index_info: Option<serde_json::Value> = client.call("getindexinfo", &[]).ok();
        let rpc_info: Option<serde_json::Value> = client.call("getrpcinfo", &[]).ok();
        let mining: Option<serde_json::Value> = client
            .get_mining_info()
            .ok()
            .and_then(|m| serde_json::to_value(m).ok());

        let total = info.headers.max(info.blocks).max(1);
        let sync_percentage = (info.blocks as f64 / total as f64) * 100.0;
        let blocks_behind = info.headers.saturating_sub(info.blocks);

        let (
            connections,
            connections_in,
            connections_out,
            networkactive,
            version,
            subversion,
            protocolversion,
            localservices,
            relay_fee,
            timeoffset,
        ) = if let Some(ref n) = net {
            let services: Vec<String> = n
                .local_services
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            (
                n.connections as u64,
                n.connections_in.unwrap_or(0) as u64,
                n.connections_out.unwrap_or(0) as u64,
                n.network_active,
                n.version as u64,
                n.subversion.clone(),
                n.protocol_version as u64,
                if services.is_empty() {
                    vec![n.local_services.clone()]
                } else {
                    services
                },
                n.relay_fee.to_btc(),
                n.time_offset as i64,
            )
        } else {
            (0, 0, 0, false, 0, String::new(), 0, vec![], 0.0, 0)
        };

        let (
            mempool_loaded,
            mempool_size,
            mempool_bytes,
            mempool_usage,
            mempool_max,
            mempool_min_fee,
            mempool_total_fee,
            mempool_unbroadcast,
        ) = if let Some(ref m) = mempool {
            (
                m.loaded.unwrap_or(true),
                m.size as u64,
                m.bytes as u64,
                m.usage as u64,
                m.max_mempool as u64,
                m.mempool_min_fee.to_btc(),
                m.total_fee.map(|a| a.to_btc()).unwrap_or(0.0),
                m.unbroadcast_count.unwrap_or(0) as u64,
            )
        } else {
            (false, 0, 0, 0, 0, 0.0, 0.0, 0)
        };

        let (bytes_recv, bytes_sent) = if let Some(ref t) = net_totals {
            (t.total_bytes_recv, t.total_bytes_sent)
        } else {
            (0, 0)
        };

        let (mem_locked_used, mem_locked_total, mem_locked_free) = meminfo
            .as_ref()
            .and_then(|v| v.get("locked"))
            .map(|l| {
                (
                    l.get("used").and_then(|x| x.as_u64()).unwrap_or(0),
                    l.get("total").and_then(|x| x.as_u64()).unwrap_or(0),
                    l.get("free").and_then(|x| x.as_u64()).unwrap_or(0),
                )
            })
            .unwrap_or((0, 0, 0));

        fn j_u64(v: &serde_json::Value, k: &str) -> u64 {
            v.get(k)
                .and_then(|x| x.as_u64().or_else(|| x.as_i64().map(|i| i as u64)))
                .unwrap_or(0)
        }
        fn j_i64(v: &serde_json::Value, k: &str) -> i64 {
            v.get(k)
                .and_then(|x| x.as_i64().or_else(|| x.as_u64().map(|u| u as i64)))
                .unwrap_or(0)
        }
        fn j_f64(v: &serde_json::Value, k: &str) -> f64 {
            v.get(k).and_then(|x| x.as_f64()).unwrap_or(0.0)
        }
        fn j_str(v: &serde_json::Value, k: &str) -> String {
            v.get(k)
                .and_then(|x| x.as_str().map(|s| s.to_string()))
                .unwrap_or_default()
        }
        fn j_bool(v: &serde_json::Value, k: &str) -> bool {
            v.get(k).and_then(|x| x.as_bool()).unwrap_or(false)
        }

        let mut peers: Vec<PeerSummary> = peers_json
            .iter()
            .take(50)
            .map(|p| {
                let inbound = j_bool(p, "inbound");
                let net = j_str(p, "network");
                PeerSummary {
                    id: j_u64(p, "id"),
                    addr: j_str(p, "addr"),
                    addrbind: j_str(p, "addrbind"),
                    network: if net.is_empty() {
                        if inbound {
                            "inbound".into()
                        } else {
                            "outbound".into()
                        }
                    } else {
                        net
                    },
                    inbound,
                    subver: j_str(p, "subver"),
                    version: j_i64(p, "version"),
                    startingheight: j_i64(p, "startingheight"),
                    synced_headers: j_i64(p, "synced_headers"),
                    synced_blocks: j_i64(p, "synced_blocks"),
                    pingtime_ms: j_f64(p, "pingtime") * 1000.0,
                    minping_ms: j_f64(p, "minping") * 1000.0,
                    bytessent: j_u64(p, "bytessent"),
                    bytesrecv: j_u64(p, "bytesrecv"),
                    connection_type: j_str(p, "connection_type"),
                    bip152_hb_to: j_bool(p, "bip152_hb_to"),
                    bip152_hb_from: j_bool(p, "bip152_hb_from"),
                }
            })
            .collect();
        peers.sort_by(|a, b| b.bytesrecv.cmp(&a.bytesrecv));

        let mut by_net = serde_json::Map::new();
        for p in &peers {
            let e = by_net
                .entry(p.network.clone())
                .or_insert(serde_json::json!(0));
            if let Some(n) = e.as_u64() {
                *e = serde_json::json!(n + 1);
            }
        }

        let warnings = format!("{:?}", info.warnings);
        let best = info.best_block_hash.to_string();
        let mediantime = info.median_time;
        let chainwork_hex = hex::encode(&info.chain_work);

        let raw = serde_json::json!({
            "blockchain": {
                "chain": info.chain.to_string(),
                "blocks": info.blocks,
                "headers": info.headers,
                "bestblockhash": best,
                "difficulty": info.difficulty,
                "mediantime": mediantime,
                "verificationprogress": info.verification_progress,
                "initialblockdownload": info.initial_block_download,
                "chainwork": chainwork_hex,
                "size_on_disk": info.size_on_disk,
                "pruned": info.pruned,
                "warnings": warnings,
            },
            "network": net.as_ref().and_then(|n| serde_json::to_value(n).ok()),
            "mempool": mempool.as_ref().and_then(|m| serde_json::to_value(m).ok()),
            "nettotals": net_totals.as_ref().and_then(|t| serde_json::to_value(t).ok()),
            "memory": meminfo,
            "chaintxstats": chain_tx,
            "indexinfo": index_info,
            "rpcinfo": rpc_info,
            "mining": mining,
            "uptime": uptime,
            "networkhashps_120": networkhashps,
            "peer_count": peers_json.len(),
        });

        let status = BitcoindStatus {
            running: true,
            process_running: true,
            rpc_ok: true,
            can_start: false,
            can_stop: true,
            pid: None,
            process_rss_mb: None,
            blocks: info.blocks,
            headers: info.headers,
            blocks_behind,
            sync_percentage: (sync_percentage * 100.0).round() / 100.0,
            chain: info.chain.to_string(),
            bestblockhash: best,
            difficulty: info.difficulty,
            chainwork: chainwork_hex,
            mediantime,
            mediantime_utc: Self::utc_from_unix(mediantime),
            block_time: mediantime,
            block_time_utc: Self::utc_from_unix(mediantime),
            verification_progress: info.verification_progress * 100.0,
            initialblockdownload: info.initial_block_download,
            pruned: info.pruned,
            pruneheight: info.prune_height,
            automatic_pruning: info.automatic_pruning,
            warnings,
            size_on_disk_bytes: info.size_on_disk,
            size_on_disk_gb: info.size_on_disk as f64 / 1_073_741_824.0,
            connections,
            connections_in,
            connections_out,
            networkactive,
            version,
            subversion,
            protocolversion,
            localservices,
            relay_fee,
            timeoffset,
            uptime_seconds: uptime,
            uptime_human: Self::fmt_uptime(uptime),
            networkhashps,
            bytes_recv,
            bytes_sent,
            bytes_recv_human: Self::fmt_bytes(bytes_recv),
            bytes_sent_human: Self::fmt_bytes(bytes_sent),
            mempool_loaded,
            mempool_size,
            mempool_bytes,
            mempool_usage,
            mempool_max,
            mempool_min_fee,
            mempool_total_fee,
            mempool_unbroadcast,
            mem_locked_used,
            mem_locked_total,
            mem_locked_free,
            peers,
            peers_by_network: serde_json::Value::Object(by_net),
            datadir: config.bitcoin_datadir.clone(),
            message: if info.initial_block_download {
                format!(
                    "IBD — {} blocs derrière (headers {})",
                    blocks_behind, info.headers
                )
            } else {
                "Synced (IBD false)".into()
            },
            auth_method: String::new(),
            is_synced: false, // set in get_status via refresh_simple_status
            simple_status: String::new(),
            raw,
        };

        Ok((status, auth_method))
    }

    /// True if we must NOT start another instance
    pub async fn is_running(config: &DashboardConfig) -> bool {
        let (proc, _) = Self::detect_process(&config.bitcoin_datadir);
        if proc {
            return true;
        }
        // RPC alone (process detection failed but node answers)
        Self::get_rpc_client(config)
            .and_then(|(c, _)| {
                c.get_blockchain_info()?;
                Ok(())
            })
            .is_ok()
    }

    fn cleanup_locks(datadir: &str) {
        let _ = std::fs::remove_file(Path::new(datadir).join(".lock"));
        let _ = std::fs::remove_file(Path::new(datadir).join("blocks").join(".lock"));
        let _ = std::fs::remove_file(Path::new(datadir).join("bitcoind.pid"));
    }

    fn kill_pid(pid: u32) {
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F", "/T"])
                .output();
        }
        #[cfg(not(windows))]
        {
            let _ = std::process::Command::new("kill")
                .args(["-9", &pid.to_string()])
                .output();
        }
    }

    /// Hard stop: RPC stop if possible, then kill process + clean locks
    pub async fn force_stop(config: &DashboardConfig) -> Result<String> {
        let datadir = config.bitcoin_datadir.clone();
        let (proc, pid) = Self::detect_process(&datadir);
        let mut notes = Vec::new();

        if !proc {
            // Still try kill any orphan bitcoind if lock stuck
            let any = Self::list_bitcoind_processes();
            if any.is_empty() {
                Self::cleanup_locks(&datadir);
                return Ok("Already stopped".into());
            }
        }

        // Graceful RPC first (short)
        if let Ok((client, _)) = Self::get_rpc_client(config) {
            let _ = client.stop();
            notes.push("RPC stop sent".into());
            for _ in 0..8 {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                let (still, _) = Self::detect_process(&datadir);
                if !still {
                    Self::cleanup_locks(&datadir);
                    return Ok(format!(
                        "Stopped gracefully ({})",
                        notes.join("; ")
                    ));
                }
            }
        }

        // Force kill known PID
        if let Some(pid) = pid {
            Self::kill_pid(pid);
            notes.push(format!("killed PID {}", pid));
        }

        // Kill remaining bitcoind.exe (single-node machine assumption)
        for (p, _) in Self::list_bitcoind_processes() {
            Self::kill_pid(p);
            notes.push(format!("killed PID {}", p));
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        Self::cleanup_locks(&datadir);

        let (still, _) = Self::detect_process(&datadir);
        if still {
            anyhow::bail!("force_stop failed — process still alive ({})", notes.join("; "));
        }
        Ok(format!("Force-stopped ({})", notes.join("; ")))
    }

    /// Start only if not already running
    pub async fn start(config: &DashboardConfig) -> Result<String> {
        let (proc, pid) = Self::detect_process(&config.bitcoin_datadir);
        if proc {
            return Ok(format!(
                "Already running (PID {:?}) — start skipped",
                pid
            ));
        }
        if let Ok((client, _)) = Self::get_rpc_client(config) {
            if client.get_blockchain_info().is_ok() {
                return Ok("Already running (RPC OK) — start skipped".into());
            }
        }
        Self::spawn_and_wait(config, false).await
    }

    /// Stop + start — for when the node is wedged / dead / RPC broken
    pub async fn restart(config: &DashboardConfig) -> Result<String> {
        let stop_msg = match Self::force_stop(config).await {
            Ok(m) => m,
            Err(e) => format!("force_stop warn: {}", e),
        };
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        Self::cleanup_locks(&config.bitcoin_datadir);
        let start_msg = Self::spawn_and_wait(config, true).await?;
        Ok(format!("Relancé. [{}] → [{}]", stop_msg, start_msg))
    }

    /// Watchdog helper: start only if fully down
    pub async fn ensure_running(config: &DashboardConfig) -> Result<Option<String>> {
        // Overnight de-XOR rewrites blk*.dat — never start Core during that window
        let dexor_flag = std::path::Path::new(&config.project_dir).join(".DEXOR_IN_PROGRESS");
        if dexor_flag.exists() {
            return Ok(None);
        }
        let (proc, _) = Self::detect_process(&config.bitcoin_datadir);
        if proc {
            // Process alive — even if RPC slow, do not restart (avoid interrupting IBD)
            return Ok(None);
        }
        // Stale locks from crash
        Self::cleanup_locks(&config.bitcoin_datadir);
        // Down → start
        let msg = Self::spawn_and_wait(config, true).await?;
        Ok(Some(format!("auto-restart: {}", msg)))
    }

    /// Spawn bitcoind and wait until process/RPC appears
    async fn spawn_and_wait(config: &DashboardConfig, clean_locks: bool) -> Result<String> {
        let bitcoind_path = Self::bitcoind_exe(config);
        let datadir = &config.bitcoin_datadir;

        if !Path::new(&bitcoind_path).exists() {
            anyhow::bail!("bitcoind not found at {}", bitcoind_path);
        }
        if !Path::new(datadir).exists() {
            anyhow::bail!("datadir not found: {}", datadir);
        }

        // Refuse double-start if process exists
        let (proc, pid) = Self::detect_process(datadir);
        if proc {
            return Ok(format!("Already running (PID {:?})", pid));
        }

        let any_bitcoind = !Self::list_bitcoind_processes().is_empty();
        let lock = Path::new(datadir).join(".lock");
        if lock.exists() {
            if any_bitcoind && !clean_locks {
                anyhow::bail!(
                    ".lock present and a bitcoind process exists — use Relancer"
                );
            }
            tracing::warn!("Removing .lock at {}", lock.display());
            Self::cleanup_locks(datadir);
        } else if clean_locks {
            Self::cleanup_locks(datadir);
        }

        Self::fix_xor_for_plaintext(datadir);

        #[cfg(windows)]
        {
            let child = Command::new(&bitcoind_path)
                .arg(format!("-datadir={}", datadir))
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .with_context(|| format!("spawn {}", bitcoind_path))?;
            let spawned_pid = child.id();
            drop(child);
            tracing::info!("Spawned bitcoind pid={:?} datadir={}", spawned_pid, datadir);
        }

        #[cfg(not(windows))]
        {
            let _child = Command::new(&bitcoind_path)
                .args(["-daemon", &format!("-datadir={}", datadir)])
                .spawn()?;
        }

        for i in 0..45 {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            let (p, p_pid) = Self::detect_process(datadir);
            if let Ok((client, _)) = Self::get_rpc_client(config) {
                if client.get_block_count().is_ok() {
                    return Ok(format!(
                        "bitcoind started — RPC ready after {}s (PID {:?})",
                        i * 2,
                        p_pid
                    ));
                }
            }
            if p && i >= 3 {
                return Ok(format!(
                    "bitcoind process up (PID {:?}) after {}s — RPC still loading",
                    p_pid,
                    i * 2
                ));
            }
        }

        let (p, p_pid) = Self::detect_process(datadir);
        if p {
            Ok(format!(
                "bitcoind process running (PID {:?}) but RPC not ready within 90s — check debug.log",
                p_pid
            ))
        } else {
            Err(anyhow::anyhow!(
                "bitcoind did not start within 90s. Check {}",
                Path::new(datadir).join("debug.log").display()
            ))
        }
    }

    fn fix_xor_for_plaintext(datadir: &str) {
        let conf = Path::new(datadir).join("bitcoin.conf");
        let xor = Path::new(datadir).join("blocks").join("xor.dat");
        if xor.exists() && conf.exists() {
            if let Ok(txt) = std::fs::read_to_string(&conf) {
                if txt.lines().any(|l| {
                    let t = l.trim();
                    t.starts_with("blocksxor=0") || t.starts_with("blocksxor = 0")
                }) {
                    let bak = Path::new(datadir).join("blocks").join(format!(
                        "xor.dat.bak-{}",
                        chrono::Local::now().format("%Y%m%d-%H%M%S")
                    ));
                    let _ = std::fs::rename(&xor, &bak);
                    tracing::warn!("Moved orphan xor.dat → {}", bak.display());
                }
            }
        }
    }

    pub async fn stop(config: &DashboardConfig) -> Result<String> {
        let (proc, pid) = Self::detect_process(&config.bitcoin_datadir);
        if !proc {
            // Maybe RPC only
            if let Ok((client, _)) = Self::get_rpc_client(config) {
                if client.get_blockchain_info().is_ok() {
                    let _ = client.stop();
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    return Ok("stop sent via RPC".into());
                }
            }
            return Ok("Not running — stop skipped".into());
        }

        // Prefer graceful RPC stop
        if let Ok((client, _)) = Self::get_rpc_client(config) {
            let _ = client.stop();
            for _ in 0..20 {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                let (still, _) = Self::detect_process(&config.bitcoin_datadir);
                if !still {
                    return Ok(format!("Stopped gracefully (was PID {:?})", pid));
                }
            }
        }

        // Force kill only our PID if known
        if let Some(pid) = pid {
            #[cfg(windows)]
            {
                let _ = std::process::Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/F"])
                    .output();
            }
            #[cfg(not(windows))]
            {
                let _ = std::process::Command::new("kill")
                    .args(["-TERM", &pid.to_string()])
                    .output();
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            return Ok(format!("Force-stopped PID {}", pid));
        }

        Err(anyhow::anyhow!("Could not stop bitcoind"))
    }

    /// Rebuild UTXO index from local blk*.dat → redb + .snapshot
    pub async fn generate_snapshot(config: &DashboardConfig) -> Result<String> {
        let indexer = Self::find_indexer(config)?;
        let blocks_dir = &config.blocks_dir;
        let db_path = &config.redb_path;

        if !Path::new(blocks_dir).exists() {
            anyhow::bail!("blocks dir missing: {}", blocks_dir);
        }

        tracing::info!(
            "Starting UTXO rebuild: indexer={} blocks={} db={} xor={}",
            indexer,
            blocks_dir,
            db_path,
            config.blocks_obf_key
        );

        let output = Command::new(&indexer)
            .args([
                "build",
                "--blocks-dir",
                blocks_dir,
                "--db-path",
                db_path,
                "--obf-key",
                &config.blocks_obf_key,
                "--start-file",
                "0",
                "--checkpoint-interval",
                "200",
            ])
            .current_dir(&config.project_dir)
            .output()
            .await
            .with_context(|| format!("run {}", indexer))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Indexer failed (code {:?}):\n{}\n{}",
                output.status.code(),
                stdout,
                stderr
            ));
        }

        let produced = db_path.replace(".redb", ".snapshot");
        if Path::new(&produced).exists() && produced != config.snapshot_path {
            std::fs::copy(&produced, &config.snapshot_path)
                .with_context(|| format!("copy {} → {}", produced, config.snapshot_path))?;
        }

        Ok(format!(
            "UTXO index rebuilt. snapshot={} redb={}\n{}",
            config.snapshot_path,
            db_path,
            if stdout.len() > 2000 {
                format!("…{}", &stdout[stdout.len() - 2000..])
            } else {
                stdout
            }
        ))
    }

    fn find_indexer(config: &DashboardConfig) -> Result<String> {
        let candidates = [
            format!(r"{}\full_utxo_indexer.exe", config.bin_dir),
            format!(r"{}\target\release\full_utxo_indexer.exe", config.project_dir),
            format!(r"{}\full_utxo_indexer_v8.exe", config.project_dir),
            format!(r"{}\full_utxo_indexer.exe", config.project_dir),
        ];
        for c in candidates {
            if Path::new(&c).exists() {
                return Ok(c);
            }
        }
        anyhow::bail!(
            "full_utxo_indexer.exe not found under {} or {}",
            config.bin_dir,
            config.project_dir
        )
    }

    pub async fn get_snapshot_info(config: &DashboardConfig) -> Result<serde_json::Value> {
        let path = Path::new(&config.snapshot_path);
        if !path.exists() {
            return Ok(serde_json::json!({
                "exists": false,
                "path": config.snapshot_path,
                "error": "snapshot file not found",
            }));
        }
        let meta = std::fs::metadata(path)?;
        let age_secs = meta.modified()?.elapsed().map(|d| d.as_secs()).unwrap_or(0);
        let modified_utc = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| Self::utc_from_unix(d.as_secs()))
            .unwrap_or_default();

        // Sidecar meta written by dump/indexer (e.g. utxo-day-935000.snapshot.meta.json)
        let mut base_block_hash: Option<String> = None;
        let mut built_at: Option<String> = None;
        let mut num_scripts_meta: Option<u64> = None;
        let mut num_utxos: Option<u64> = None;
        let mut source: Option<String> = None;
        let mut block_height: Option<u64> = None;
        let mut mempool_space: Option<String> = None;
        let mut block_time_utc: Option<String> = None;
        let mut block_time_unix: Option<u64> = None;

        let meta_candidates = [
            format!("{}.meta.json", config.snapshot_path),
            config.snapshot_path.replace(".snapshot", ".snapshot.meta.json"),
        ];
        for mp in &meta_candidates {
            if let Ok(raw) = std::fs::read_to_string(mp) {
                if let Ok(j) = serde_json::from_str::<serde_json::Value>(&raw) {
                    base_block_hash = j
                        .get("base_block_hash")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .or(base_block_hash);
                    built_at = j
                        .get("built_at")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .or(built_at);
                    num_scripts_meta = j
                        .get("num_scripts")
                        .and_then(|v| v.as_u64())
                        .or(num_scripts_meta);
                    num_utxos = j
                        .get("num_utxos_source")
                        .and_then(|v| v.as_u64())
                        .or_else(|| j.get("num_utxos").and_then(|v| v.as_u64()))
                        .or(num_utxos);
                    source = j
                        .get("source")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .or(source);
                    block_height = j
                        .get("block_height")
                        .and_then(|v| v.as_u64())
                        .or_else(|| j.get("height").and_then(|v| v.as_u64()))
                        .or(block_height);
                    mempool_space = j
                        .get("mempool_space")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .or(mempool_space);
                    block_time_utc = j
                        .get("block_time_utc")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .or(block_time_utc);
                    block_time_unix = j
                        .get("block_time_unix")
                        .and_then(|v| v.as_u64())
                        .or(block_time_unix);
                    if block_time_utc.is_none() {
                        if let Some(ts) = block_time_unix {
                            block_time_utc = Some(Self::utc_from_unix(ts));
                        }
                    }
                    break;
                }
            }
        }

        // Infer height from filename patterns: utxo-day-935000, btc-index-935000, utxo-935000
        if block_height.is_none() {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            for part in name.split(|c: char| !c.is_ascii_digit()) {
                if part.len() >= 5 && part.len() <= 7 {
                    if let Ok(h) = part.parse::<u64>() {
                        if (100_000..2_000_000).contains(&h) {
                            block_height = Some(h);
                            break;
                        }
                    }
                }
            }
        }

        // Optional: block time via local RPC only if meta did not provide it
        // (during reindex Core often has only genesis → getblockheader fails for tip hash)
        if block_time_utc.is_none() {
            if let Some(ref hash) = base_block_hash {
                let cfg = config.clone();
                let hash_c = hash.clone();
                if let Ok(Ok((ts, utc))) = tokio::task::spawn_blocking(move || {
                    Self::rpc_block_time(&cfg, &hash_c)
                })
                .await
                {
                    block_time_unix = Some(ts);
                    block_time_utc = Some(utc);
                }
            }
        }

        Ok(serde_json::json!({
            "exists": true,
            "path": config.snapshot_path,
            "size_bytes": meta.len(),
            "size_mb": ((meta.len() as f64 / 1_048_576.0) * 100.0).round() / 100.0,
            "age_seconds": age_secs,
            "age_hours": ((age_secs as f64 / 3600.0) * 100.0).round() / 100.0,
            "fresh": age_secs <= config.max_snapshot_age_seconds,
            "max_age_hours": ((config.max_snapshot_age_seconds as f64 / 3600.0) * 100.0).round() / 100.0,
            "blocks_dir": config.blocks_dir,
            "redb_path": config.redb_path,
            "datadir": config.bitcoin_datadir,
            "file_modified_utc": modified_utc,
            "base_block_hash": base_block_hash,
            "block_height": block_height,
            "block_time_utc": block_time_utc,
            "block_time_unix": block_time_unix,
            "built_at": built_at,
            "num_scripts": num_scripts_meta,
            "num_utxos": num_utxos,
            "source": source,
            "mempool_space": mempool_space,
        }))
    }

    /// Best-effort: getblockheader → time for a known tip hash of the UTXO set.
    fn rpc_block_time(config: &DashboardConfig, block_hash: &str) -> Result<(u64, String)> {
        let (client, _) = Self::get_rpc_client(config)?;
        let hash: bitcoin::BlockHash = block_hash
            .parse()
            .map_err(|e| anyhow::anyhow!("bad block hash: {}", e))?;
        let hdr = client.get_block_header_info(&hash)?;
        let ts = hdr.time as u64;
        Ok((ts, Self::utc_from_unix(ts)))
    }

    /// Build auth candidates (user/pass preferred when configured — avoids 401 spam on cookie).
    /// Probe once with getblockcount (lighter than getblockchaininfo).
    fn get_rpc_client(
        config: &DashboardConfig,
    ) -> Result<(bitcoincore_rpc::Client, String)> {
        let rpc_url = config
            .bitcoin_rpc_url
            .clone()
            .unwrap_or_else(|| "http://127.0.0.1:8332".to_string());

        let mut candidates: Vec<(bitcoincore_rpc::Auth, String)> = Vec::new();

        // 1) Config user/pass (dashboard CLI defaults match W:\Bitcoin\bitcoin.conf)
        let user = config
            .bitcoin_rpc_user
            .clone()
            .unwrap_or_else(|| "btcsolver".to_string());
        let pass = config
            .bitcoin_rpc_password
            .clone()
            .unwrap_or_else(|| "btcsolver_rpc_2026".to_string());
        candidates.push((
            bitcoincore_rpc::Auth::UserPass(user.clone(), pass.clone()),
            format!("userpass:{}", user),
        ));

        // 2) bitcoin.conf
        let conf = Path::new(&config.bitcoin_datadir).join("bitcoin.conf");
        if let Ok(txt) = std::fs::read_to_string(&conf) {
            let mut u = None;
            let mut p = None;
            for line in txt.lines() {
                let line = line.trim();
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                if let Some(rest) = line.strip_prefix("rpcuser=") {
                    u = Some(rest.trim().to_string());
                }
                if let Some(rest) = line.strip_prefix("rpcpassword=") {
                    p = Some(rest.trim().to_string());
                }
            }
            if let (Some(u), Some(p)) = (u, p) {
                if u != user || p != pass {
                    candidates.push((
                        bitcoincore_rpc::Auth::UserPass(u, p),
                        "bitcoin.conf".into(),
                    ));
                }
            }
        }

        // 3) Cookie last (can 401 if mismatched with rpcuser setups / stale file)
        let cookie = PathBuf::from(&config.bitcoin_datadir).join(".cookie");
        if cookie.exists() {
            candidates.push((
                bitcoincore_rpc::Auth::CookieFile(cookie),
                "cookie".into(),
            ));
        }

        let mut errors = Vec::new();
        for (auth, label) in candidates {
            match bitcoincore_rpc::Client::new(&rpc_url, auth) {
                Ok(client) => match client.get_block_count() {
                    Ok(_) => return Ok((client, label)),
                    Err(e) => errors.push(format!("{}: {}", label, e)),
                },
                Err(e) => errors.push(format!("{} build: {}", label, e)),
            }
        }

        Err(anyhow::anyhow!(
            "RPC auth failed ({})",
            errors.join("; ")
        ))
    }
}
