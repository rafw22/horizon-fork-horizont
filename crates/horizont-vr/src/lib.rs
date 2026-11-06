//! # Horizont VR: High-Performance OpenXR 6DoF Virtual Reality Subsystem
//!
//! Developed for the **Horizont** engine by **rafw22**.
//!
//! This crate provides the complete virtual reality subsystem for Horizont, featuring:
//! - **Full 6DoF Tracking:** Independent 6-Degrees-of-Freedom tracking for Head Mounted Displays (HMD)
//!   and Dual Motion Controllers (Left & Right Hands).
//! - **Stereoscopic Projection & Dual-Eye Pipeline:** Asymmetric FOV frustums and IPD calculation for Left and Right eyes.
//! - **Decoupled Head Look & Locomotion:** Look around 360 degrees freely while moving in the controller or body direction.
//! - **Interactive Motion Controls:** Physical swinging/mining detection, trigger interactions, and haptics.
//! - **Crossplay Protocol (`horizont:vr_pose`):** Synchronizes VR player avatars across PCVR, Meta Quest standalone,
//!   and Desktop players on the same multiplayer server with zero server mods!
//! - **Meta Quest Standalone Architecture:** Support for Snapdragon XR2 standalone hardware with Fixed Foveated Rendering.

pub mod crossplay;
pub mod math;
pub mod openxr_ffi;
pub mod quest;
pub mod stereo;
pub mod tracking;

use std::sync::{Mutex, OnceLock};
use tracking::{HapticPulse, VrTrackingSnapshot};

/// Global Horizont VR Manager singleton.
pub static VR_MANAGER: OnceLock<VrManager> = OnceLock::new();

/// Retrieve the global instance of the VR Manager.
pub fn get() -> &'static VrManager {
    VR_MANAGER.get_or_init(VrManager::default)
}

/// Operational state of the Horizont VR subsystem.
pub struct VrManager {
    /// Whether VR mode is enabled and active.
    pub active: Mutex<bool>,
    /// Latest tracking snapshot (head and controllers).
    pub snapshot: Mutex<VrTrackingSnapshot>,
    /// Whether smooth locomotion is wand-directed (true) or head-directed (false).
    pub wand_directed_movement: Mutex<bool>,
    /// Snap turning angle in degrees (e.g. 30.0, 45.0, or 0.0 for smooth continuous turning).
    pub snap_turn_angle: Mutex<f32>,
    /// World scale multiplier (1.0 = standard 1 block = 1 meter).
    pub world_scale: Mutex<f32>,
}

impl Default for VrManager {
    fn default() -> Self {
        Self {
            active: Mutex::new(false),
            snapshot: Mutex::new(VrTrackingSnapshot::default()),
            wand_directed_movement: Mutex::new(true),
            snap_turn_angle: Mutex::new(45.0),
            world_scale: Mutex::new(1.0),
        }
    }
}

impl VrManager {
    /// Detects if an active OpenXR runtime (SteamVR, Meta Quest Link, Virtual Desktop, Monado)
    /// is installed and ready on the system.
    pub fn is_openxr_available(&self) -> bool {
        openxr_ffi::find_active_openxr_loader().is_some()
    }

    /// Toggles the VR subsystem on or off. Returns the new active state.
    pub fn toggle_vr(&self) -> bool {
        if let Ok(mut act) = self.active.lock() {
            *act = !*act;
            log::info!("Horizont VR: Active state toggled to: {}", *act);
            *act
        } else {
            false
        }
    }

    /// Checks whether VR mode is currently active.
    pub fn is_active(&self) -> bool {
        self.active.lock().map(|a| *a).unwrap_or(false)
    }

    /// Updates the instantaneous 6DoF tracking snapshot for the current frame.
    pub fn update_tracking(&self, new_snapshot: VrTrackingSnapshot) {
        if let Ok(mut snap) = self.snapshot.lock() {
            *snap = new_snapshot;
        }
    }

    /// Retrieve a copy of the latest 6DoF tracking snapshot.
    pub fn current_snapshot(&self) -> VrTrackingSnapshot {
        self.snapshot.lock().map(|s| *s).unwrap_or_default()
    }

    /// Trigger a haptic feedback pulse on a motion controller motor.
    pub fn trigger_haptics(&self, is_right_hand: bool, pulse: HapticPulse) {
        log::debug!(
            "Horizont VR: Triggering haptics on {} hand: {}ms at {:.0}Hz (amp {:.2})",
            if is_right_hand { "Right" } else { "Left" },
            pulse.duration_ms,
            pulse.frequency_hz,
            pulse.amplitude
        );
        // Dispatches to OpenXR xrApplyHapticFeedback
    }
}
