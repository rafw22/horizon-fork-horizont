//! # Horizont VR: Crossplay Networking & 6DoF Pose Synchronization
//!
//! Enables seamless multiplayer crossplay between:
//! - **PCVR Players** (SteamVR, Oculus Link, Virtual Desktop)
//! - **Meta Quest Standalone Players** (Android native 6DoF)
//! - **Desktop Players** (Mouse & Keyboard / Gamepad)
//!
//! ## Crossplay Architecture:
//! 1. **Zero Server Modification Required:**
//!    - Standard movement, rotations, block breaks, and chat packets are sent over standard
//!      Minecraft protocol (`ClientboundPlayPacket` / `ServerboundPlayPacket`).
//! 2. **High-Precision 6DoF Pose Channel (`horizont:vr_pose`):**
//!    - Sent as custom payload packets at 20-60 Hz.
//!    - Serializes head orientation/position and dual hand poses + finger gestures.
//!    - Allows all other players on the server (both desktop and VR) to see the VR player's
//!      avatar with full 6DoF articulated head tracking and interactive 3D moving hands!

use crate::tracking::{ControllerState, Pose6DoF};
use serde::{Deserialize, Serialize};

/// Custom plugin message channel used for Horizont VR 6DoF pose broadcasts.
pub const VR_POSE_CHANNEL: &str = "horizont:vr_pose";

/// Hand gesture state for avatar visualization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandGesture {
    /// Open relaxed hand.
    Open,
    /// Clenched fist (e.g. grip squeezed).
    Fist,
    /// Pointing index finger (e.g. trigger released, grip held).
    Point,
    /// Holding an item/weapon.
    HoldItem,
}

/// Compact network packet transmitted across multiplayer sessions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VrPlayerPosePacket {
    /// Protocol version identifier.
    pub version: u8,
    /// Player UUID string.
    pub player_uuid: String,
    /// 6DoF Head pose in world coordinates.
    pub head_pose: Pose6DoF,
    /// 6DoF Left hand pose in world coordinates.
    pub left_hand_pose: Pose6DoF,
    /// 6DoF Right hand pose in world coordinates.
    pub right_hand_pose: Pose6DoF,
    /// Current gesture of left hand.
    pub left_gesture: HandGesture,
    /// Current gesture of right hand.
    pub right_gesture: HandGesture,
    /// Whether the player is in roomscale crouching/sneaking.
    pub is_crouching: bool,
}

impl VrPlayerPosePacket {
    /// Constructs a network packet from local VR tracking inputs.
    pub fn create_local(
        player_uuid: &str,
        head: &Pose6DoF,
        left_ctrl: &ControllerState,
        right_ctrl: &ControllerState,
    ) -> Self {
        let left_gesture = if left_ctrl.grip > 0.5 {
            if left_ctrl.trigger > 0.5 {
                HandGesture::Fist
            } else {
                HandGesture::Point
            }
        } else {
            HandGesture::Open
        };

        let right_gesture = if right_ctrl.grip > 0.5 {
            if right_ctrl.trigger > 0.5 {
                HandGesture::Fist
            } else {
                HandGesture::Point
            }
        } else {
            HandGesture::HoldItem
        };

        Self {
            version: 1,
            player_uuid: player_uuid.to_string(),
            head_pose: *head,
            left_hand_pose: left_ctrl.grip_pose,
            right_hand_pose: right_ctrl.grip_pose,
            left_gesture,
            right_gesture,
            is_crouching: head.position[1] < 1.3, // Detected when head is lower than standard standing height
        }
    }

    /// Serializes the packet into binary format for transmission over Minecraft custom payload channel.
    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Deserializes a received binary payload into a `VrPlayerPosePacket`.
    pub fn deserialize(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| format!("Deserialization error: {e}"))
    }
}
