//! Example Native Dynamic Plugin Mod for Horizont
//!
//! Compiles directly to `sample_native_mod.dll` (Windows) or `sample_native_mod.so` (Linux).
//! Simply drop into the `mods/` folder and run Horizont — no WASM overhead,
//! no recompiling the entire game engine!

use std::ffi::CString;

/// C-ABI plugin API mirror from `horizont-modloader`
#[repr(C)]
pub struct HorizontPluginApi {
    pub api_version: u32,
    pub register_tick_hook: extern "C" fn(extern "C" fn(u64)),
    pub register_render_hook: extern "C" fn(extern "C" fn(f64, f64, f64)),
    pub register_packet_hook: extern "C" fn(extern "C" fn(u32, *const u8, usize) -> bool),
    pub log_info: extern "C" fn(*const std::os::raw::c_char),
    pub log_warn: extern "C" fn(*const std::os::raw::c_char),
}

/// Called every game tick (20 Hz)
extern "C" fn on_game_tick(tick: u64) {
    if tick % 100 == 0 {
        println!("[SampleNativeMod] Native tick callback running! Current tick: {}", tick);
    }
}

/// Main entrypoint exported by the DLL
#[unsafe(no_mangle)]
pub extern "C" fn horizont_mod_init(api: &HorizontPluginApi) -> bool {
    let msg = CString::new("Sample Native DLL Mod loaded successfully into Horizont!").unwrap();
    (api.log_info)(msg.as_ptr());

    // Register our high-speed native tick hook
    (api.register_tick_hook)(on_game_tick);

    true
}
