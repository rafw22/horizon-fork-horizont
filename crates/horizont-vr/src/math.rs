//! # Horizont VR: 3D Spatial & Stereoscopic Frustum Math
//!
//! Provides optimized matrix and quaternion transformations for 6DoF virtual reality:
//! - Quaternion to 4x4 matrix conversion.
//! - OpenXR asymmetric frustum projection matrix generation for dual eye views.
//! - Dual eye view matrix synthesis from HMD tracking poses and Interpupillary Distance (IPD).
//! - 3D raycasting and controller forward vector calculation.

/// 3D Vector with `[x, y, z]` components.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const FORWARD: Self = Self { x: 0.0, y: 0.0, z: -1.0 };
    pub const UP: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const RIGHT: Self = Self { x: 1.0, y: 0.0, z: 0.0 };

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 1e-6 {
            Self { x: self.x / len, y: self.y / len, z: self.z / len }
        } else {
            Self::ZERO
        }
    }

    pub fn dot(&self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

/// Unit Quaternion representing 3D spatial orientation `[x, y, z, w]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const IDENTITY: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Rotates a 3D vector by this quaternion: $v' = q \cdot v \cdot q^{-1}$.
    pub fn rotate_vec3(&self, v: Vec3) -> Vec3 {
        let u = Vec3::new(self.x, self.y, self.z);
        let s = self.w;

        let dot_uv = u.dot(v);
        let dot_uu = u.dot(u);
        let cross_uv = u.cross(v);

        Vec3::new(
            2.0 * dot_uv * u.x + (s * s - dot_uu) * v.x + 2.0 * s * cross_uv.x,
            2.0 * dot_uv * u.y + (s * s - dot_uu) * v.y + 2.0 * s * cross_uv.y,
            2.0 * dot_uv * u.z + (s * s - dot_uu) * v.z + 2.0 * s * cross_uv.z,
        )
    }

    /// Converts the quaternion into a column-major 4x4 rotation matrix.
    pub fn to_mat4(&self) -> Mat4 {
        let x2 = self.x + self.x;
        let y2 = self.y + self.y;
        let z2 = self.z + self.z;

        let xx = self.x * x2;
        let xy = self.x * y2;
        let xz = self.x * z2;
        let yy = self.y * y2;
        let yz = self.y * z2;
        let zz = self.z * z2;
        let wx = self.w * x2;
        let wy = self.w * y2;
        let wz = self.w * z2;

        Mat4([
            [1.0 - (yy + zz), xy + wz, xz - wy, 0.0],
            [xy - wz, 1.0 - (xx + zz), yz + wx, 0.0],
            [xz + wy, yz - wx, 1.0 - (xx + yy), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
}

/// 4x4 column-major matrix for graphics transformations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4(pub [[f32; 4]; 4]);

impl Mat4 {
    pub const IDENTITY: Self = Self([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    /// Matrix multiplication: `self * rhs`.
    pub fn mul(&self, rhs: Self) -> Self {
        let mut out = [[0.0; 4]; 4];
        for col in 0..4 {
            for row in 0..4 {
                out[col][row] = self.0[0][row] * rhs.0[col][0]
                    + self.0[1][row] * rhs.0[col][1]
                    + self.0[2][row] * rhs.0[col][2]
                    + self.0[3][row] * rhs.0[col][3];
            }
        }
        Self(out)
    }

    /// Creates a 4x4 translation matrix.
    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        Self([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [x, y, z, 1.0],
        ])
    }

    /// Creates an OpenXR asymmetric frustum projection matrix for stereoscopic eye views.
    ///
    /// In OpenXR, FOVs are defined by half-angles in radians:
    /// - `angle_left` (negative, e.g. -0.85 rad)
    /// - `angle_right` (positive, e.g. +0.85 rad)
    /// - `angle_up` (positive, e.g. +0.80 rad)
    /// - `angle_down` (negative, e.g. -0.80 rad)
    ///
    /// Generates a standard reversed-Z or standard depth projection compatible with WGPU / Vulkan.
    pub fn create_openxr_projection(
        angle_left: f32,
        angle_right: f32,
        angle_up: f32,
        angle_down: f32,
        near_z: f32,
        far_z: f32,
    ) -> Self {
        let tan_left = angle_left.tan();
        let tan_right = angle_right.tan();
        let tan_up = angle_up.tan();
        let tan_down = angle_down.tan();

        let tan_width = tan_right - tan_left;
        let tan_height = tan_up - tan_down;

        let a = 2.0 / tan_width;
        let b = 2.0 / tan_height;
        let c = (tan_right + tan_left) / tan_width;
        let d = (tan_up + tan_down) / tan_height;

        // Vulkan/WGPU depth range [0.0, 1.0]
        let q = far_z / (near_z - far_z);
        let qn = (far_z * near_z) / (near_z - far_z);

        Self([
            [a, 0.0, 0.0, 0.0],
            [0.0, b, 0.0, 0.0],
            [c, d, q, -1.0],
            [0.0, 0.0, qn, 0.0],
        ])
    }
}
