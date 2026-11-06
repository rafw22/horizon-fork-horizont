//! # Sample Native VR Mod for Horizont
//!
//! Demonstrates how a native dynamic `.dll` plugin can tap into Horizont's OpenXR
//! 6DoF tracking subsystem, detect physical swing/attack gestures, and interact with the engine.

use horizont_vr::tracking::{HapticPulse, Pose6DoF, VrTrackingSnapshot};

#[unsafe(no_mangle)]
pub extern "C" fn horizont_mod_init() -> bool {
    println!("[Horizont-VR-Mod] Initializing OpenXR 6DoF Virtual Reality Plugin...");
    let vr = horizont_vr::get();
    let runtime_found = vr.is_openxr_available();
    println!("[Horizont-VR-Mod] OpenXR Runtime detected: {}", runtime_found);
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn horizont_on_tick() {
    let vr = horizont_vr::get();
    let snap = vr.current_snapshot();

    // Check if right controller performed a physical swing gesture (> 2.2 m/s)
    if snap.right_controller.is_swinging(2.2) {
        // Trigger haptic rumble on the controller motor
        vr.trigger_haptics(true, HapticPulse::IMPACT);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn horizont_on_render() {
    // Stereoscopic render hook per frame
}

#[unsafe(no_mangle)]
pub extern "C" fn horizont_mod_shutdown() {
    println!("[Horizont-VR-Mod] Shutting down VR plugin cleanly.");
}
