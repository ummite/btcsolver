//! GPU acceleration for secp256k1 public key derivation.
//!
//! Pipeline: GPU derives compressed public keys from private keys in batch;
//! CPU performs SHA256+RIPEMD160 hashing and FlatIndex lookups.

use std::path::PathBuf;
use std::sync::OnceLock;

/// Function pointer types
type FnInit = unsafe extern "C" fn() -> i32;
type FnDerive = unsafe extern "C" fn(*const u8, *mut u8, i32) -> i32;
type FnDeriveMulti = unsafe extern "C" fn(*const u8, *mut u8, i32, *const i32, i32) -> i32;
type FnDeviceCount = unsafe extern "C" fn() -> i32;
type FnDeviceName = unsafe extern "C" fn(i32, *mut i8, i32) -> ();
type FnCleanup = unsafe extern "C" fn() -> ();

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
}

/// Find the CUDA DLL
fn find_dll() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let p = parent.join("libsecp_gpu.dll");
            if p.exists() { return Some(p); }
        }
    }
    let cwd_dll = std::env::current_dir().ok()?.join("libsecp_gpu.dll");
    if cwd_dll.exists() { return Some(cwd_dll); }
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
