//! # Horizont VR: Meta Quest Standalone 6DoF Architecture
//!
//! Provides the specialized runtime integration and architecture for running
//! Horizont natively on Meta Quest headsets (Quest 2, Quest 3, Quest 3S, Quest Pro)
//! powered by Qualcomm Snapdragon XR2 / XR2 Gen 2 chipsets.
//!
//! ## Standalone Capabilities:
//! - **Native OpenXR Android Loader:** Loads `/system/lib64/libopenxr_loader.so`.
//! - **Inside-Out 6DoF Tracking:** High-frequency optical tracking with zero external sensors.
//! - **Fixed Foveated Rendering (FFR):** Reduces peripheral GPU pixel shading load by up to 35%.
//! - **Dynamic Refresh Rates:** Supports 72 Hz, 90 Hz, and 120 Hz display modes.
//! - **Proximity Sensor Lifecycle:** Automatically pauses simulation when the headset is taken off.

use serde::{Deserialize, Serialize};

/// Target Meta Quest hardware generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestHardwareModel {
    /// Meta Quest 2 (Snapdragon XR2 Gen 1, 1832x1920 per eye).
    Quest2,
    /// Meta Quest Pro (Snapdragon XR2+ Gen 1, Local Dimming, Eye & Face Tracking).
    QuestPro,
    /// Meta Quest 3 / Quest 3S (Snapdragon XR2 Gen 2, 2064x2208 per eye, High Fill-Rate).
    Quest3,
    /// Generic Android OpenXR device (PICO 4, Vive Focus, etc.).
    GenericAndroidXr,
}

/// Fixed Foveated Rendering (FFR) intensity levels for Meta Quest mobile GPU optimization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoveationLevel {
    /// Full resolution across the entire field of view (PCVR quality).
    Off,
    /// Subtle peripheral reduction; imperceptible during gameplay (+15% GPU headroom).
    Low,
    /// Moderate reduction (+25% GPU headroom, recommended default for Quest 2).
    Medium,
    /// Aggressive reduction (+35% GPU headroom, recommended for high render distances or shaders).
    High,
}

/// Standalone Quest configuration settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuestConfig {
    /// Target hardware generation detected.
    pub hardware: QuestHardwareModel,
    /// Desired display refresh rate in Hz (72, 90, 120).
    pub refresh_rate_hz: u32,
    /// Fixed Foveated Rendering tier.
    pub foveation: FoveationLevel,
    /// Eye texture resolution scale multiplier (default: 1.0 = native panel).
    pub resolution_scale: f32,
    /// Whether passthrough mixed-reality mode is enabled.
    pub passthrough_enabled: bool,
}

impl Default for QuestConfig {
    fn default() -> Self {
        Self {
            hardware: QuestHardwareModel::Quest3,
            refresh_rate_hz: 90,
            foveation: FoveationLevel::Medium,
            resolution_scale: 1.0,
            passthrough_enabled: false,
        }
    }
}

impl QuestConfig {
    /// Recommend optimal settings based on detected hardware model.
    pub fn for_hardware(model: QuestHardwareModel) -> Self {
        match model {
            QuestHardwareModel::Quest3 => Self {
                hardware: model,
                refresh_rate_hz: 90,
                foveation: FoveationLevel::Low,
                resolution_scale: 1.15,
                passthrough_enabled: false,
            },
            QuestHardwareModel::QuestPro => Self {
                hardware: model,
                refresh_rate_hz: 90,
                foveation: FoveationLevel::Medium,
                resolution_scale: 1.0,
                passthrough_enabled: false,
            },
            QuestHardwareModel::Quest2 => Self {
                hardware: model,
                refresh_rate_hz: 72,
                foveation: FoveationLevel::High,
                resolution_scale: 0.9,
                passthrough_enabled: false,
            },
            QuestHardwareModel::GenericAndroidXr => Self::default(),
        }
    }
}
