//! # Horizont VR: 6DoF Tracking & Motion Controller Input System
//!
//! Provides real-time 6DoF tracking structures, gesture detection (swinging/mining),
//! raycast targeting from hands, and haptic feedback triggers.

use crate::math::{Quat, Vec3};
use serde::{Deserialize, Serialize};

/// 6DoF spatial pose (Position [X, Y, Z] + Unit Quaternion Orientation [X, Y, Z, W]).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pose6DoF {
    pub position: [f32; 3],
    pub orientation: [f32; 4],
}

impl Default for Pose6DoF {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            orientation: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

impl Pose6DoF {
    pub fn new(pos: [f32; 3], rot: [f32; 4]) -> Self {
        Self {
            position: pos,
            orientation: rot,
        }
    }

    /// Computes the forward directional unit vector from the orientation quaternion.
    pub fn forward_vector(&self) -> Vec3 {
        let q = Quat::new(
            self.orientation[0],
            self.orientation[1],
            self.orientation[2],
            self.orientation[3],
        );
        q.rotate_vec3(Vec3::FORWARD)
    }

    /// Computes the upward directional unit vector from the orientation quaternion.
    pub fn up_vector(&self) -> Vec3 {
        let q = Quat::new(
            self.orientation[0],
            self.orientation[1],
            self.orientation[2],
            self.orientation[3],
        );
        q.rotate_vec3(Vec3::UP)
    }
}

/// 6DoF Motion Controller state (triggers, buttons, thumbsticks, tracking).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ControllerState {
    /// 6DoF Aim pose: used for pointing at blocks, UI laser pointer, and crosshairs.
    pub aim_pose: Pose6DoF,
    /// 6DoF Grip pose: used for holding tools (swords, pickaxes), torches, and items.
    pub grip_pose: Pose6DoF,
    /// Linear velocity [m/s] for swing detection.
    pub linear_velocity: [f32; 3],
    /// Analog trigger pull (0.0 = released, 1.0 = fully pressed).
    pub trigger: f32,
    /// Digital trigger click (attack / break / select).
    pub trigger_click: bool,
    /// Analog grip squeeze (0.0 = released, 1.0 = grabbed).
    pub grip: f32,
    /// Thumbstick axes: X [-1.0 (left) .. 1.0 (right)], Y [-1.0 (down) .. 1.0 (up)].
    pub thumbstick: [f32; 2],
    /// Thumbstick click button.
    pub thumbstick_click: bool,
    /// Primary action button (A on Right controller, X on Left controller).
    pub button_primary: bool,
    /// Secondary action button (B on Right controller, Y on Left controller).
    pub button_secondary: bool,
    /// Menu / Pause button.
    pub button_menu: bool,
}

impl ControllerState {
    /// Speed of controller motion in meters per second (magnitude of velocity vector).
    pub fn speed(&self) -> f32 {
        let [vx, vy, vz] = self.linear_velocity;
        (vx * vx + vy * vy + vz * vz).sqrt()
    }

    /// Detects whether the player performed a physical swing gesture (e.g. swinging a sword or pickaxe).
    /// Returns true when linear speed exceeds threshold (default: 2.2 m/s).
    pub fn is_swinging(&self, threshold_meters_per_sec: f32) -> bool {
        self.speed() >= threshold_meters_per_sec
    }
}

/// Complete instantaneous snapshot of all VR tracking inputs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VrTrackingSnapshot {
    /// 6DoF Head Mounted Display (HMD) pose.
    pub head: Pose6DoF,
    /// Left motion controller state.
    pub left_controller: ControllerState,
    /// Right motion controller state.
    pub right_controller: ControllerState,
    /// Interpupillary Distance (IPD) in meters (typically ~0.063m = 63mm).
    pub ipd: f32,
    /// Whether tracking is currently active and valid.
    pub tracking_valid: bool,
}

impl VrTrackingSnapshot {
    /// Generates a 3D ray starting from the hand controller aim pose.
    ///
    /// Returns `(origin, direction_unit_vector)`.
    pub fn controller_ray(&self, is_right_hand: bool) -> ([f32; 3], [f32; 3]) {
        let controller = if is_right_hand {
            &self.right_controller
        } else {
            &self.left_controller
        };

        let origin = controller.aim_pose.position;
        let fwd = controller.aim_pose.forward_vector();
        (origin, [fwd.x, fwd.y, fwd.z])
    }
}

/// Request for haptic vibration feedback delivered to a motion controller motor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HapticPulse {
    /// Duration of vibration in milliseconds.
    pub duration_ms: u32,
    /// Frequency in Hz (e.g. 160.0 Hz for crisp haptic clicks).
    pub frequency_hz: f32,
    /// Amplitude from 0.0 (no vibration) to 1.0 (maximum motor force).
    pub amplitude: f32,
}

impl HapticPulse {
    /// Light click (for menu navigation or block highlight).
    pub const CLICK: Self = Self {
        duration_ms: 15,
        frequency_hz: 200.0,
        amplitude: 0.3,
    };

    /// Solid impact (for striking a mob or breaking a block).
    pub const IMPACT: Self = Self {
        duration_ms: 60,
        frequency_hz: 160.0,
        amplitude: 0.85,
    };
}
