//! # Horizont VR: Stereoscopic Rendering Pipeline & Dual-Eye Camera
//!
//! Handles dual-eye view matrices, IPD offsets, and asymmetric frustum projections
//! for rendering into OpenXR swapchain textures or side-by-side viewports.

use crate::math::{Mat4, Quat, Vec3};
use crate::tracking::Pose6DoF;

/// Enumeration of stereoscopic eyes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StereoEye {
    Left,
    Right,
}

impl StereoEye {
    /// Eye sign: `-1.0` for Left Eye, `+1.0` for Right Eye.
    pub fn sign(&self) -> f32 {
        match self {
            StereoEye::Left => -1.0,
            StereoEye::Right => 1.0,
        }
    }
}

/// Perspective projection angles for a single VR eye (as reported by OpenXR).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EyeFov {
    pub angle_left: f32,
    pub angle_right: f32,
    pub angle_up: f32,
    pub angle_down: f32,
}

impl Default for EyeFov {
    fn default() -> Self {
        // Standard ~100 degree FOV defaults for PCVR and Meta Quest
        Self {
            angle_left: -0.85,
            angle_right: 0.85,
            angle_up: 0.80,
            angle_down: -0.80,
        }
    }
}

impl EyeFov {
    /// Builds the 4x4 projection matrix for this eye.
    pub fn to_projection_matrix(&self, near_z: f32, far_z: f32) -> Mat4 {
        Mat4::create_openxr_projection(
            self.angle_left,
            self.angle_right,
            self.angle_up,
            self.angle_down,
            near_z,
            far_z,
        )
    }
}

/// Computes the complete 4x4 View Matrix for a specific VR eye.
///
/// Takes the player's world position, the 6DoF head tracking pose, and the eye offset
/// along the head's local right vector based on the Interpupillary Distance (IPD).
pub fn compute_eye_view_matrix(
    _player_world_pos: [f64; 3],
    head_pose: &Pose6DoF,
    eye: StereoEye,
    ipd_meters: f32,
) -> Mat4 {
    let head_rot = Quat::new(
        head_pose.orientation[0],
        head_pose.orientation[1],
        head_pose.orientation[2],
        head_pose.orientation[3],
    );

    // Compute the head's local right vector
    let head_right = head_rot.rotate_vec3(Vec3::RIGHT);

    // Half of IPD shift (negative for left, positive for right)
    let half_ipd = (ipd_meters * 0.5) * eye.sign();
    let eye_offset_x = head_right.x * half_ipd;
    let eye_offset_y = head_right.y * half_ipd;
    let eye_offset_z = head_right.z * half_ipd;

    // Combined eye position relative to world camera
    let eye_pos_x = head_pose.position[0] + eye_offset_x;
    let eye_pos_y = head_pose.position[1] + eye_offset_y;
    let eye_pos_z = head_pose.position[2] + eye_offset_z;

    // Translation matrix to eye origin
    let trans = Mat4::translation(-eye_pos_x, -eye_pos_y, -eye_pos_z);

    // Inverse rotation matrix (transposed rotation for view space)
    let rot_mat = head_rot.to_mat4();

    // View matrix = Rotation * Translation
    rot_mat.mul(trans)
}
