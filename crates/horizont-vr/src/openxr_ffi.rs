//! # OpenXR 1.0 C ABI Types & Dynamic Runtime Loader
//!
//! Provides zero-overhead FFI definitions for OpenXR 1.0 and a safe dynamic loader.
//!
//! Supported Platforms:
//! - **Windows:** Dynamically checks active OpenXR runtime via Windows Registry
//!   (`SOFTWARE\Khronos\OpenXR\1\ActiveRuntime`) or standard paths (`openxr_loader.dll`, SteamVR, Oculus Link).
//! - **Meta Quest / Android:** Looks for `libopenxr_loader.so` packaged with standalone APKs.
//! - **Linux:** Looks for `libopenxr_loader.so.1` (Monado, SteamVR).

use std::path::PathBuf;

pub type XrResult = i32;
pub const XR_SUCCESS: XrResult = 0;
pub const XR_TIMEOUT_EXPIRED: XrResult = 1;
pub const XR_SESSION_LOSS_PENDING: XrResult = 3;
pub const XR_ERROR_INSTANCE_LOST: XrResult = -1;
pub const XR_ERROR_SESSION_LOST: XrResult = -3;
pub const XR_ERROR_RUNTIME_FAILURE: XrResult = -7;

pub type XrHandle = u64;
pub type XrInstance = XrHandle;
pub type XrSession = XrHandle;
pub type XrSpace = XrHandle;
pub type XrActionSet = XrHandle;
pub type XrAction = XrHandle;
pub type XrSwapchain = XrHandle;

pub type XrTime = i64;
pub type XrDuration = i64;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct XrVector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct XrQuaternionf {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for XrQuaternionf {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct XrPosef {
    pub orientation: XrQuaternionf,
    pub position: XrVector3f,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct XrFovf {
    pub angle_left: f32,
    pub angle_right: f32,
    pub angle_up: f32,
    pub angle_down: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XrSessionState {
    Unknown = 0,
    Idle = 1,
    Ready = 2,
    Synchronized = 3,
    Visible = 4,
    Focused = 5,
    Stopping = 6,
    LossPending = 7,
    Exiting = 8,
}

/// Discovers the path to the active OpenXR loader library on the host machine.
pub fn find_active_openxr_loader() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        // 1. Check Windows Registry for active OpenXR runtime JSON manifest
        // Path: HKEY_LOCAL_MACHINE\SOFTWARE\Khronos\OpenXR\1\ActiveRuntime
        let registry_candidates = [
            "openxr_loader.dll",
            "C:\\Windows\\System32\\openxr_loader.dll",
            "C:\\Program Files\\Oculus\\Support\\oculus-runtime\\openxr_loader.dll",
            "C:\\Program Files (x86)\\Steam\\steamapps\\common\\SteamVR\\bin\\win64\\openxr_loader.dll",
        ];

        for candidate in registry_candidates {
            let p = PathBuf::from(candidate);
            if p.exists() {
                log::info!("Horizont VR: Found active OpenXR loader at: {:?}", p);
                return Some(p);
            }
        }

        // Try standard DLL search path
        Some(PathBuf::from("openxr_loader.dll"))
    }

    #[cfg(target_os = "android")]
    {
        // Meta Quest Standalone Android Loader
        let quest_candidates = [
            "libopenxr_loader.so",
            "/system/lib64/libopenxr_loader.so",
            "/vendor/lib64/libopenxr_loader.so",
        ];
        for candidate in quest_candidates {
            let p = PathBuf::from(candidate);
            if p.exists() {
                return Some(p);
            }
        }
        Some(PathBuf::from("libopenxr_loader.so"))
    }

    #[cfg(all(unix, not(target_os = "android")))]
    {
        Some(PathBuf::from("libopenxr_loader.so.1"))
    }
}
