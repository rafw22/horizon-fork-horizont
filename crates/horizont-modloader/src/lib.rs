//! # Horizont Native Mod Loader (horizont-modloader)
//!
//! A true native dynamic plugin system for Minecraft / Horizon.
//!
//! ## Why NOT WebAssembly (WASM)?
//! - WASM creates a virtualization barrier that imposes severe serialization/copying overhead
//!   on every frame, tick, and packet crossing the boundary.
//! - WASM sandbox prevents direct low-level SIMD, multi-threading with Rayon, direct GPU wgpu
//!   access, and native C/C++ libraries.
//!
//! ## Why NOT Upstream Horizon Mixin-Recompilation?
//! - Upstream Horizon compiles mods into the main game binary, requiring minutes of recompilation
//!   and caching tens of gigabytes of intermediate build artifacts (`native/target/fast`).
//!
//! ## How Horizont Native Mod Loader Works:
//! 1. Mod authors build standard dynamic libraries (`crate-type = ["cdylib"]` -> `.dll` on Windows, `.so` on Linux).
//! 2. Drop the compiled `.dll` into the `mods/` directory.
//! 3. On game startup, `PluginManager::load_all()` uses `libloading` to link the DLL directly into
//!    the process memory space with zero runtime overhead (100% native machine code performance).
//! 4. Each plugin exports an initialization entrypoint:
//!    ```rust,ignore
//!    #[no_mangle]
//!    pub extern "C" fn horizont_mod_init(api: &HorizontPluginApi) -> bool {
//!        // Register hooks, callbacks, and commands!
//!        true
//!    }
//!    ```

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use libloading::{Library, Symbol};

/// Function signature for the plugin initialization entrypoint.
pub type ModInitFn = unsafe extern "C" fn(&HorizontPluginApi) -> bool;
pub type TickHookFn = extern "C" fn(u64);
pub type RenderHookFn = extern "C" fn(f64, f64, f64);
pub type PacketHookFn = extern "C" fn(u32, *const u8, usize) -> bool;

/// Global plugin manager singleton.
pub static PLUGIN_MANAGER: OnceLock<PluginManager> = OnceLock::new();

/// The public C-compatible API passed to native mods upon initialization.
#[repr(C)]
pub struct HorizontPluginApi {
    pub api_version: u32,
    pub register_tick_hook: extern "C" fn(TickHookFn),
    pub register_render_hook: extern "C" fn(RenderHookFn),
    pub register_packet_hook: extern "C" fn(PacketHookFn),
    pub log_info: extern "C" fn(*const std::os::raw::c_char),
    pub log_warn: extern "C" fn(*const std::os::raw::c_char),
}

/// Represents a single dynamically loaded native plugin.
pub struct LoadedPlugin {
    pub name: String,
    pub path: PathBuf,
    // Keep library handle alive in memory so code stays mapped
    _lib: Library,
}

/// Central registry managing all native dynamic plugins.
pub struct PluginManager {
    plugins: Mutex<Vec<LoadedPlugin>>,
    tick_hooks: Arc<Mutex<Vec<TickHookFn>>>,
    render_hooks: Arc<Mutex<Vec<RenderHookFn>>>,
    packet_hooks: Arc<Mutex<Vec<PacketHookFn>>>,
}

// Global hook vectors for FFI trampolines
static GLOBAL_TICK_HOOKS: OnceLock<Arc<Mutex<Vec<TickHookFn>>>> = OnceLock::new();
static GLOBAL_RENDER_HOOKS: OnceLock<Arc<Mutex<Vec<RenderHookFn>>>> = OnceLock::new();
static GLOBAL_PACKET_HOOKS: OnceLock<Arc<Mutex<Vec<PacketHookFn>>>> = OnceLock::new();

extern "C" fn tramp_register_tick(hook: TickHookFn) {
    if let Some(hooks) = GLOBAL_TICK_HOOKS.get() {
        if let Ok(mut lock) = hooks.lock() {
            lock.push(hook);
        }
    }
}

extern "C" fn tramp_register_render(hook: RenderHookFn) {
    if let Some(hooks) = GLOBAL_RENDER_HOOKS.get() {
        if let Ok(mut lock) = hooks.lock() {
            lock.push(hook);
        }
    }
}

extern "C" fn tramp_register_packet(hook: PacketHookFn) {
    if let Some(hooks) = GLOBAL_PACKET_HOOKS.get() {
        if let Ok(mut lock) = hooks.lock() {
            lock.push(hook);
        }
    }
}

extern "C" fn tramp_log_info(msg: *const std::os::raw::c_char) {
    if !msg.is_null() {
        unsafe {
            let s = std::ffi::CStr::from_ptr(msg).to_string_lossy();
            log::info!("[NativeMod] {}", s);
        }
    }
}

extern "C" fn tramp_log_warn(msg: *const std::os::raw::c_char) {
    if !msg.is_null() {
        unsafe {
            let s = std::ffi::CStr::from_ptr(msg).to_string_lossy();
            log::warn!("[NativeMod] {}", s);
        }
    }
}

impl PluginManager {
    /// Initialize the plugin manager and scan the `mods/` directory.
    pub fn new() -> Self {
        let tick_hooks = Arc::new(Mutex::new(Vec::new()));
        let render_hooks = Arc::new(Mutex::new(Vec::new()));
        let packet_hooks = Arc::new(Mutex::new(Vec::new()));

        let _ = GLOBAL_TICK_HOOKS.set(tick_hooks.clone());
        let _ = GLOBAL_RENDER_HOOKS.set(render_hooks.clone());
        let _ = GLOBAL_PACKET_HOOKS.set(packet_hooks.clone());

        Self {
            plugins: Mutex::new(Vec::new()),
            tick_hooks,
            render_hooks,
            packet_hooks,
        }
    }

    /// Load all `.dll` / `.so` plugins found in the specified mods folder.
    pub fn load_all(&self, mods_dir: &Path) {
        if !mods_dir.exists() {
            let _ = std::fs::create_dir_all(mods_dir);
            return;
        }

        let Ok(entries) = std::fs::read_dir(mods_dir) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let is_dynamic_lib = path.extension().is_some_and(|ext| {
                ext.eq_ignore_ascii_case("dll") || ext.eq_ignore_ascii_case("so")
            });

            if is_dynamic_lib {
                self.load_plugin(&path);
            }
        }
    }

    /// Dynamically load a single native library into the process.
    pub fn load_plugin(&self, path: &Path) -> bool {
        let mod_name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown".into());

        log::info!("Horizont Mod Loader: loading native plugin from {:?}", path);

        unsafe {
            let lib = match Library::new(path) {
                Ok(lib) => lib,
                Err(err) => {
                    log::error!(
                        "Horizont Mod Loader: failed to load library {:?}: {}",
                        path,
                        err
                    );
                    return false;
                }
            };

            // Locate entrypoint symbol: horizont_mod_init
            let init_symbol: Symbol<ModInitFn> = match lib.get(b"horizont_mod_init") {
                Ok(sym) => sym,
                Err(err) => {
                    log::warn!(
                        "Horizont Mod Loader: plugin {:?} lacks 'horizont_mod_init' entrypoint: {}",
                        path,
                        err
                    );
                    return false;
                }
            };

            let api = HorizontPluginApi {
                api_version: 1,
                register_tick_hook: tramp_register_tick,
                register_render_hook: tramp_register_render,
                register_packet_hook: tramp_register_packet,
                log_info: tramp_log_info,
                log_warn: tramp_log_warn,
            };

            if !init_symbol(&api) {
                log::warn!(
                    "Horizont Mod Loader: plugin {:?} returned false from init",
                    path
                );
                return false;
            }

            log::info!(
                "Horizont Mod Loader: plugin '{}' successfully initialized!",
                mod_name
            );

            if let Ok(mut list) = self.plugins.lock() {
                list.push(LoadedPlugin {
                    name: mod_name,
                    path: path.to_path_buf(),
                    _lib: lib,
                });
            }

            true
        }
    }

    /// Dispatches game tick to all registered native mod tick listeners.
    pub fn dispatch_tick(&self, tick_count: u64) {
        if let Ok(hooks) = self.tick_hooks.lock() {
            for hook in hooks.iter() {
                hook(tick_count);
            }
        }
    }

    /// Dispatches render frame to all registered native mod render listeners.
    pub fn dispatch_render(&self, camera_x: f64, camera_y: f64, camera_z: f64) {
        if let Ok(hooks) = self.render_hooks.lock() {
            for hook in hooks.iter() {
                hook(camera_x, camera_y, camera_z);
            }
        }
    }

    /// Dispatches incoming/outgoing network packets to mod listeners.
    /// Returns true if packet should continue processing, false if cancelled.
    pub fn dispatch_packet(&self, packet_id: u32, data: &[u8]) -> bool {
        if let Ok(hooks) = self.packet_hooks.lock() {
            for hook in hooks.iter() {
                if !hook(packet_id, data.as_ptr(), data.len()) {
                    return false;
                }
            }
        }
        true
    }
}

/// Public access helper to the global plugin manager.
pub fn manager() -> &'static PluginManager {
    PLUGIN_MANAGER.get_or_init(PluginManager::new)
}
