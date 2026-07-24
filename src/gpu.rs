//! GPU acceleration for secp256k1 public key derivation.
//!
//! Pipeline: GPU derives compressed public keys from private keys in batch;
//! CPU performs SHA256+RIPEMD160 hashing and FlatIndex lookups.
//! Also supports combined derive+lookup where GPU does everything.

use std::path::PathBuf;
use std::sync::OnceLock;

/// Function pointer types
type FnInit = unsafe extern "C" fn() -> i32;
type FnDerive = unsafe extern "C" fn(*const u8, *mut u8, i32) -> i32;
type FnDeriveMulti = unsafe extern "C" fn(*const u8, *mut u8, i32, *const i32, i32) -> i32;
type FnDeviceCount = unsafe extern "C" fn() -> i32;
type FnDeviceName = unsafe extern "C" fn(i32, *mut i8, i32) -> ();
type FnCleanup = unsafe extern "C" fn() -> ();
type FnLoadIndex = unsafe extern "C" fn(*const (), *const u8, *const u8, u32, usize, usize) -> i32;
type FnUnloadIndex = unsafe extern "C" fn() -> ();
type FnDeriveLookup = unsafe extern "C" fn(*const u8, *mut u64, i32, u32, *const i32, i32) -> i32;
type FnDeriveLookupSingle = unsafe extern "C" fn(*const u8, *mut u64, i32, u32) -> i32;
type FnDeriveLookupAsync = unsafe extern "C" fn(*const u8, *mut u64, i32, u32, *const i32, i32) -> i32;
type FnSyncAll = unsafe extern "C" fn();

static GPU_FUNCS: OnceLock<GpuFuncs> = OnceLock::new();

/// Keep library handle alive (leaked for 'static lifetime)
static GPU_LIB_HANDLE: OnceLock<std::sync::Mutex<Option<libloading::Library>>> = OnceLock::new();

struct GpuFuncs {
    pub init: FnInit,
    pub derive: FnDerive,
    pub derive_multi: FnDeriveMulti,
    pub device_count: FnDeviceCount,
    pub device_name: FnDeviceName,
    pub cleanup: FnCleanup,
    pub load_index: Option<FnLoadIndex>,
    pub unload_index: Option<FnUnloadIndex>,
    pub derive_lookup: Option<FnDeriveLookup>,
    pub derive_lookup_single: Option<FnDeriveLookupSingle>,
    pub derive_lookup_async: Option<FnDeriveLookupAsync>,
    pub sync_all: Option<FnSyncAll>,
}

/// DLL basenames to try (canonical first, then arch-specific builds from older scripts)
const GPU_DLL_NAMES: &[&str] = &[
    "libsecp_gpu.dll",
    "libsecp_gpu_new.dll",
    "libsecp_gpu_sm86.dll",
    "libsecp_gpu_dual.dll",
];

/// Find the CUDA DLL (exe dir, cwd, project paths, env override, stable bin)
fn find_dll() -> Option<PathBuf> {
    // Explicit override: BTC_GPU_DLL=C:\path\to\libsecp_gpu.dll
    if let Ok(p) = std::env::var("BTC_GPU_DLL") {
        let pb = PathBuf::from(p.trim());
        if pb.exists() {
            return Some(pb);
        }
    }

    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            dirs.push(parent.to_path_buf());
            // target/release -> repo root
            if let Some(grand) = parent.parent().and_then(|p| p.parent()) {
                dirs.push(grand.to_path_buf());
            }
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        dirs.push(cwd.clone());
        dirs.push(cwd.join("target").join("release"));
    }
    dirs.push(PathBuf::from(r"Y:\btcsolver"));
    dirs.push(PathBuf::from(r"Y:\btcsolver\target\release"));
    dirs.push(PathBuf::from(r"C:\btcsolver-bin"));
    dirs.push(PathBuf::from(r"C:\Programmation\BTCSolver"));
    dirs.push(PathBuf::from(r"C:\Programmation\BTCSolver\target\release"));

    for dir in dirs {
        for name in GPU_DLL_NAMES {
            let p = dir.join(name);
            if p.exists() {
                return Some(p);
            }
        }
    }
    None
}

/// Load the GPU library and extract function pointers
fn load_gpu() -> Result<(), String> {
    let dll_path = find_dll().ok_or_else(|| "libsecp_gpu.dll not found".to_string())?;

    let lib = unsafe {
        libloading::Library::new(&dll_path)
            .map_err(|e| format!("Failed to load {}: {}", dll_path.display(), e))?
    };

    // Extract function pointers BEFORE moving lib
    let funcs = GpuFuncs {
        init: unsafe { *lib.get::<FnInit>(b"secp_gpu_init").map_err(|e| e.to_string())? },
        derive: unsafe { *lib.get::<FnDerive>(b"secp_gpu_derive").map_err(|e| e.to_string())? },
        derive_multi: unsafe { *lib.get::<FnDeriveMulti>(b"secp_gpu_derive_multi").map_err(|e| e.to_string())? },
        device_count: unsafe { *lib.get::<FnDeviceCount>(b"secp_gpu_device_count").map_err(|e| e.to_string())? },
        device_name: unsafe { *lib.get::<FnDeviceName>(b"secp_gpu_device_name").map_err(|e| e.to_string())? },
        cleanup: unsafe { *lib.get::<FnCleanup>(b"secp_gpu_cleanup").map_err(|e| e.to_string())? },
        load_index: unsafe { lib.get::<FnLoadIndex>(b"secp_gpu_load_index").ok().map(|p| *p) },
        unload_index: unsafe { lib.get::<FnUnloadIndex>(b"secp_gpu_unload_index").ok().map(|p| *p) },
        derive_lookup: unsafe { lib.get::<FnDeriveLookup>(b"secp_gpu_derive_lookup").ok().map(|p| *p) },
        derive_lookup_single: unsafe { lib.get::<FnDeriveLookupSingle>(b"secp_gpu_derive_lookup_single").ok().map(|p| *p) },
        derive_lookup_async: unsafe { lib.get::<FnDeriveLookupAsync>(b"secp_gpu_derive_lookup_async").ok().map(|p| *p) },
        sync_all: unsafe { lib.get::<FnSyncAll>(b"secp_gpu_sync_all").ok().map(|p| *p) },
    };

    // Keep library alive (function pointers remain valid while lib is loaded)
    GPU_LIB_HANDLE.set(std::sync::Mutex::new(Some(lib))).map_err(|_| "init".to_string())?;
    GPU_FUNCS.set(funcs).map_err(|_| "Already initialized".to_string())?;
    Ok(())
}

fn ensure_loaded() -> bool {
    if GPU_FUNCS.get().is_none() {
        if load_gpu().is_err() {
            return false;
        }
    }
    true
}

/// Initialize GPUs and return the number of devices
pub fn gpu_init() -> i32 {
    if !ensure_loaded() { return -1; }
    let funcs = GPU_FUNCS.get().unwrap();
    unsafe { (funcs.init)() }
}

/// Derive compressed public keys from private keys using GPU.
pub fn gpu_derive(privkeys: &[u8], pubkeys: &mut [u8], count: usize) -> i32 {
    if !ensure_loaded() { return -1; }
    let funcs = GPU_FUNCS.get().unwrap();
    unsafe { (funcs.derive)(privkeys.as_ptr(), pubkeys.as_mut_ptr(), count as i32) }
}

/// Derive using specific GPUs
pub fn gpu_derive_multi(privkeys: &[u8], pubkeys: &mut [u8], count: usize, device_ids: &[i32]) -> i32 {
    if !ensure_loaded() { return -1; }
    let funcs = GPU_FUNCS.get().unwrap();
    unsafe {
        (funcs.derive_multi)(
            privkeys.as_ptr(), pubkeys.as_mut_ptr(), count as i32,
            device_ids.as_ptr(), device_ids.len() as i32,
        )
    }
}

/// Get the number of available CUDA devices
pub fn gpu_device_count() -> i32 {
    if !ensure_loaded() { return 0; }
    let funcs = GPU_FUNCS.get().unwrap();
    unsafe { (funcs.device_count)() }
}

/// Get the name of a CUDA device
pub fn gpu_device_name(idx: i32) -> String {
    if !ensure_loaded() { return format!("Device {}", idx); }
    let funcs = GPU_FUNCS.get().unwrap();
    let mut buf = [0i8; 128];
    unsafe { (funcs.device_name)(idx, buf.as_mut_ptr(), buf.len() as i32) };
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..end].iter().map(|&b| b as u8).collect::<Vec<u8>>()).into_owned()
}

/// Cleanup GPU resources
pub fn gpu_cleanup() {
    if let Some(funcs) = GPU_FUNCS.get() {
        unsafe { (funcs.cleanup)() };
    }
}

/// Detect GPUs
pub fn detect_gpus() -> Vec<GpuInfo> {
    let _ = ensure_loaded();
    let count = gpu_device_count();
    (0..count).map(|i| GpuInfo { id: i, name: gpu_device_name(i) }).collect()
}

#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub id: i32,
    pub name: String,
}

/// Load FlatIndex data onto all GPU devices.
/// script_entries: packed array of (offset:u32, len:u16, utxo_offset:u32, utxo_count:u32)
/// all_data: raw script bytes
/// utxo_data: raw UTXO entry bytes (44 bytes each: txid[32] + vout[4] + value[8])
pub fn gpu_load_index(
    script_entries: &[u8],
    all_data: &[u8],
    utxo_data: &[u8],
    num_entries: u32,
) -> i32 {
    if !ensure_loaded() { return -1; }
    let funcs = GPU_FUNCS.get().unwrap();
    match (funcs.load_index, funcs.unload_index) {
        (Some(load), _) => unsafe {
            (load)(
                script_entries.as_ptr() as *const (),
                all_data.as_ptr(),
                utxo_data.as_ptr(),
                num_entries,
                all_data.len(),
                utxo_data.len(),
            )
        },
        _ => -1,
    }
}

/// Unload FlatIndex data from all GPU devices
pub fn gpu_unload_index() {
    if let Some(funcs) = GPU_FUNCS.get() {
        if let Some(unload) = funcs.unload_index {
            unsafe { (unload)() };
        }
    }
}

/// Derive pubkey + FlatIndex lookup in one kernel launch.
/// privkeys: input private keys (32 bytes each, LE)
/// total_values: output total UTXO value per key (8 bytes each, LE)
/// addr_types: bitmask (1=legacy, 2=segwit, 4=wrapped, 8=taproot)
pub fn gpu_derive_lookup(
    privkeys: &[u8],
    total_values: &mut [u64],
    count: usize,
    addr_types: u32,
    device_ids: &[i32],
) -> i32 {
    if !ensure_loaded() { return -1; }
    let funcs = GPU_FUNCS.get().unwrap();
    match funcs.derive_lookup {
        Some(f) => unsafe {
            (f)(
                privkeys.as_ptr(),
                total_values.as_mut_ptr(),
                count as i32,
                addr_types,
                device_ids.as_ptr(),
                device_ids.len() as i32,
            )
        },
        None => -1,
    }
}

/// Simple wrapper for single-GPU derive+lookup (uses all available GPUs)
pub fn gpu_derive_lookup_single(
    privkeys: &[u8],
    total_values: &mut [u64],
    count: usize,
    addr_types: u32,
) -> i32 {
    if !ensure_loaded() { return -1; }
    let funcs = GPU_FUNCS.get().unwrap();
    match funcs.derive_lookup_single {
        Some(f) => unsafe {
            (f)(
                privkeys.as_ptr(),
                total_values.as_mut_ptr(),
                count as i32,
                addr_types,
            )
        },
        None => -1,
    }
}

/// Async version: launch derive+lookup on GPU without blocking.
/// Call `gpu_sync_all()` before reading `total_values`.
/// Enables double-buffering: CPU can prepare the next batch while GPU processes the current one.
pub fn gpu_derive_lookup_async(
    privkeys: &[u8],
    total_values: &mut [u64],
    count: usize,
    addr_types: u32,
    device_ids: &[i32],
) -> i32 {
    if !ensure_loaded() { return -1; }
    let funcs = GPU_FUNCS.get().unwrap();
    match funcs.derive_lookup_async {
        Some(f) => unsafe {
            (f)(
                privkeys.as_ptr(),
                total_values.as_mut_ptr(),
                count as i32,
                addr_types,
                device_ids.as_ptr(),
                device_ids.len() as i32,
            )
        },
        None => -1,
    }
}

/// Synchronize all GPU devices — wait for pending async operations to complete.
/// Must be called before reading results from `gpu_derive_lookup_async`.
pub fn gpu_sync_all() {
    if let Some(funcs) = GPU_FUNCS.get() {
        if let Some(sync) = funcs.sync_all {
            unsafe { (sync)() };
        }
    }
}

/// Check if async GPU API is available (pinned memory + non-blocking transfers).
pub fn gpu_async_available() -> bool {
    if let Some(funcs) = GPU_FUNCS.get() {
        funcs.derive_lookup_async.is_some() && funcs.sync_all.is_some()
    } else {
        false
    }
}
