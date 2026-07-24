//! System helpers: free RAM, UTXO size, present CUDA GPUs.
//!
//! Rules used by dashboard / auto-scan:
//! - Default GPU selection = all CUDA devices currently present.
//! - Free physical RAM must be >= UTXO size (dynamic) before loading
//!   the index or starting a scan process that reloads it.

use crate::flat_index::FlatIndex;
use crate::gpu;

#[derive(Debug, Clone, Copy)]
pub struct MemInfo {
    pub total_bytes: u64,
    pub free_bytes: u64,
}

/// Physical memory snapshot (Windows via GlobalMemoryStatusEx; fallback via none).
pub fn mem_info() -> Option<MemInfo> {
    #[cfg(windows)]
    {
        #[repr(C)]
        struct MEMORYSTATUSEX {
            dw_length: u32,
            dw_memory_load: u32,
            ull_total_phys: u64,
            ull_avail_phys: u64,
            ull_total_page_file: u64,
            ull_avail_page_file: u64,
            ull_total_virtual: u64,
            ull_avail_virtual: u64,
            ull_avail_extended_virtual: u64,
        }
        #[link(name = "kernel32")]
        extern "system" {
            fn GlobalMemoryStatusEx(lp_buffer: *mut MEMORYSTATUSEX) -> i32;
        }
        unsafe {
            let mut st = std::mem::MaybeUninit::<MEMORYSTATUSEX>::uninit();
            (*st.as_mut_ptr()).dw_length = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
            if GlobalMemoryStatusEx(st.as_mut_ptr()) == 0 {
                return None;
            }
            let st = st.assume_init();
            return Some(MemInfo {
                total_bytes: st.ull_total_phys,
                free_bytes: st.ull_avail_phys,
            });
        }
    }
    #[cfg(not(windows))]
    {
        // Best-effort: MemAvailable from /proc/meminfo
        let s = std::fs::read_to_string("/proc/meminfo").ok()?;
        let mut total_kb = 0u64;
        let mut avail_kb = 0u64;
        for line in s.lines() {
            if let Some(v) = line.strip_prefix("MemTotal:") {
                total_kb = v.split_whitespace().next()?.parse().ok()?;
            } else if let Some(v) = line.strip_prefix("MemAvailable:") {
                avail_kb = v.split_whitespace().next()?.parse().ok()?;
            }
        }
        if total_kb == 0 {
            return None;
        }
        Some(MemInfo {
            total_bytes: total_kb * 1024,
            free_bytes: avail_kb * 1024,
        })
    }
}

/// UTXO size in bytes from snapshot file on disk (dynamic).
pub fn utxo_size_from_path(path: &str) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

/// UTXO size once loaded in process memory.
pub fn utxo_size_from_index(idx: &FlatIndex) -> u64 {
    idx.memory_usage_bytes() as u64
}

/// Best estimate of UTXO footprint: loaded index if present, else snapshot file.
pub fn utxo_required_bytes(snapshot_path: &str, loaded: Option<&FlatIndex>) -> u64 {
    if let Some(idx) = loaded {
        let mem = utxo_size_from_index(idx);
        let file = utxo_size_from_path(snapshot_path);
        return mem.max(file);
    }
    utxo_size_from_path(snapshot_path)
}

/// True when free physical RAM >= required UTXO bytes.
pub fn ram_ok_for_utxo(required_bytes: u64) -> bool {
    if required_bytes == 0 {
        return true;
    }
    match mem_info() {
        Some(m) => m.free_bytes >= required_bytes,
        None => true, // if we cannot measure, do not hard-block
    }
}

/// Human-readable RAM / UTXO gate result.
pub fn ram_gate_message(required_bytes: u64) -> (bool, String) {
    let free = mem_info().map(|m| m.free_bytes).unwrap_or(0);
    let total = mem_info().map(|m| m.total_bytes).unwrap_or(0);
    let ok = required_bytes == 0 || free >= required_bytes;
    let msg = if required_bytes == 0 {
        format!(
            "RAM free {:.1} GB / total {:.1} GB (UTXO size unknown)",
            free as f64 / 1_073_741_824.0,
            total as f64 / 1_073_741_824.0
        )
    } else if ok {
        format!(
            "RAM OK: {:.1} GB free >= {:.1} GB UTXO",
            free as f64 / 1_073_741_824.0,
            required_bytes as f64 / 1_073_741_824.0
        )
    } else {
        format!(
            "Insufficient free RAM: {:.1} GB free < {:.1} GB UTXO required. Scan paused.",
            free as f64 / 1_073_741_824.0,
            required_bytes as f64 / 1_073_741_824.0
        )
    };
    (ok, msg)
}

/// JSON blob for /api/system/health and UI banners.
pub fn ram_status_json(snapshot_path: &str, loaded: Option<&FlatIndex>) -> serde_json::Value {
    let required = utxo_required_bytes(snapshot_path, loaded);
    let mem = mem_info();
    let free = mem.map(|m| m.free_bytes).unwrap_or(0);
    let total = mem.map(|m| m.total_bytes).unwrap_or(0);
    let (ok, message) = ram_gate_message(required);
    serde_json::json!({
        "free_bytes": free,
        "total_bytes": total,
        "free_gb": free as f64 / 1_073_741_824.0,
        "total_gb": total as f64 / 1_073_741_824.0,
        "utxo_bytes": required,
        "utxo_gb": required as f64 / 1_073_741_824.0,
        "ok": ok,
        "paused": !ok,
        "message": message,
    })
}

/// All CUDA device IDs currently present (default selection).
pub fn present_gpu_ids() -> Vec<i32> {
    let n = gpu::gpu_device_count().max(0);
    (0..n).collect()
}

/// Comma-separated list of present GPU IDs, or None if no CUDA device.
pub fn present_gpus_csv() -> Option<String> {
    let ids = present_gpu_ids();
    if ids.is_empty() {
        None
    } else {
        Some(
            ids.iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(","),
        )
    }
}

/// Resolve config GPU list: explicit non-empty string wins; else all present devices.
/// Returns None when no GPU (caller should fall back to CPU).
pub fn resolve_gpus(config_gpus: &Option<String>) -> Option<String> {
    if let Some(s) = config_gpus {
        let t = s.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    present_gpus_csv()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn present_gpus_does_not_panic() {
        let _ = present_gpu_ids();
        let _ = present_gpus_csv();
    }

    #[test]
    fn ram_ok_zero_required() {
        assert!(ram_ok_for_utxo(0));
    }
}
