/// Camera system with view/projection matrices and frustum culling
/// Handles first-person camera transformations and visibility testing

use glam::{Mat4, Vec3, Vec4};

/// Frustum plane for culling
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    /// Create plane from equation coefficients (ax + by + cz + d = 0)
    pub fn from_coefficients(a: f32, b: f32, c: f32, d: f32) -> Self {
        let normal = Vec3::new(a, b, c);
        let length = normal.length();
        Self {
            normal: normal / length,
            distance: d / length,
        }
    }

    /// Calculate signed distance from point to plane
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }

    /// Check if point is on positive side of plane
    pub fn is_point_in_front(&self, point: Vec3) -> bool {
        self.distance_to_point(point) >= 0.0
    }
}

/// Viewing frustum for culling
pub struct Frustum {
    pub planes: [Plane; 6], // Near, Far, Left, Right, Top, Bottom
}

impl Frustum {
    /// Extract frustum planes from view-projection matrix
    pub fn from_matrix(vp: &Mat4) -> Self {
        let m = vp.to_cols_array();

        // Extract planes using Gribb-Hartmann method
        let planes = [
            // Near plane
            Plane::from_coefficients(
                m[3] + m[2],
                m[7] + m[6],
                m[11] + m[10],
                m[15] + m[14],
            ),
            // Far plane
            Plane::from_coefficients(
                m[3] - m[2],
                m[7] - m[6],
                m[11] - m[10],
                m[15] - m[14],
            ),
            // Left plane
            Plane::from_coefficients(
                m[3] + m[0],
                m[7] + m[4],
                m[11] + m[8],
                m[15] + m[12],
            ),
            // Right plane
            Plane::from_coefficients(
                m[3] - m[0],
                m[7] - m[4],
                m[11] - m[8],
                m[15] - m[12],
            ),
            // Top plane
            Plane::from_coefficients(
                m[3] - m[1],
                m[7] - m[5],
                m[11] - m[9],
                m[15] - m[13],
            ),
            // Bottom plane
            Plane::from_coefficients(
                m[3] + m[1],
                m[7] + m[5],
                m[11] + m[9],
                m[15] + m[13],
            ),
        ];

        Self { planes }
    }

    /// Test if a sphere is inside or intersecting the frustum
    pub fn is_sphere_visible(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            if plane.distance_to_point(center) < -radius {
                return false;
            }
        }
        true
    }

    /// Test if an axis-aligned bounding box is inside or intersecting the frustum
    pub fn is_aabb_visible(&self, min: Vec3, max: Vec3) -> bool {
        for plane in &self.planes {
            // Get positive vertex (furthest point in direction of plane normal)
            let p = Vec3::new(
                if plane.normal.x >= 0.0 { max.x } else { min.x },
                if plane.normal.y >= 0.0 { max.y } else { min.y },
                if plane.normal.z >= 0.0 { max.z } else { min.z },
            );

            // If positive vertex is behind plane, box is completely outside
            if !plane.is_point_in_front(p) {
                return false;
            }
        }
        true
    }

    /// Test if a chunk at (chunk_x, chunk_z) is visible
    /// Chunks are 16x256x16 blocks
    pub fn is_chunk_visible(&self, chunk_x: i32, chunk_z: i32) -> bool {
        let min = Vec3::new(
            chunk_x as f32 * 16.0,
            0.0,
            chunk_z as f32 * 16.0,
        );
        let max = Vec3::new(
            (chunk_x + 1) as f32 * 16.0,
            256.0,
            (chunk_z + 1) as f32 * 16.0,
        );
        self.is_aabb_visible(min, max)
    }
}

/// First-person camera
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,   // Rotation around Y axis (radians)
    pub pitch: f32, // Rotation around X axis (radians)
    pub fov: f32,   // Field of view in radians
    pub aspect_ratio: f32, // Aspect ratio (width / height)
    pub near: f32,  // Near clip plane
    pub far: f32,   // Far clip plane
}

impl Camera {
    /// Create a new camera
    pub fn new(position: Vec3, yaw: f32, pitch: f32) -> Self {
        Self {
            position,
            yaw,
            pitch,
            fov: 70.0_f32.to_radians(),
            aspect_ratio: 16.0 / 9.0, // Default 16:9 aspect ratio
            near: 0.1,
            far: 1000.0,
        }
    }

    /// Get forward direction vector
    /// Matches player view direction convention: yaw=0 faces -Z (North)
    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.pitch.cos() * self.yaw.sin(),
            self.pitch.sin(),
            -self.pitch.cos() * self.yaw.cos(),
        ).normalize()
    }

    /// Get right direction vector
    /// Matches player right direction convention
    pub fn right(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos(),
            0.0,
            self.yaw.sin(),
        ).normalize()
    }

    /// Get up direction vector
    pub fn up(&self) -> Vec3 {
        self.right().cross(self.forward()).normalize()
    }

    /// Generate view matrix
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(
            self.position,
            self.position + self.forward(),
            Vec3::Y,
        )
    }

    /// Generate projection matrix
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov, aspect_ratio, self.near, self.far)
    }

    /// Get view-projection matrix
    pub fn view_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        self.projection_matrix(aspect_ratio) * self.view_matrix()
    }

    /// Generate frustum from current camera state
    pub fn frustum(&self, aspect_ratio: f32) -> Frustum {
        let vp = self.view_projection_matrix(aspect_ratio);
        Frustum::from_matrix(&vp)
    }

    /// Move camera in local space
    pub fn move_local(&mut self, forward: f32, right: f32, up: f32) {
        self.position += self.forward() * forward;
        self.position += self.right() * right;
        self.position += Vec3::Y * up;
    }

    /// Rotate camera
    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch += delta_pitch;

        // Clamp pitch to prevent gimbal lock
        self.pitch = self.pitch.clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01,
            std::f32::consts::FRAC_PI_2 - 0.01,
        );

        // Normalize yaw to 0-2π
        self.yaw = self.yaw.rem_euclid(std::f32::consts::TAU);
    }

    /// Set camera position
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    /// Set camera rotation
    pub fn set_rotation(&mut self, yaw: f32, pitch: f32) {
        self.yaw = yaw;
        self.pitch = pitch.clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01,
            std::f32::consts::FRAC_PI_2 - 0.01,
        );
    }

    /// Set the camera's aspect ratio
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }

    /// Check if a chunk is visible in the camera frustum
    pub fn is_chunk_visible(&self, chunk_pos: &crate::types::ChunkPos) -> bool {
        let frustum = self.frustum(self.aspect_ratio);
        frustum.is_chunk_visible(chunk_pos.x, chunk_pos.z)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(Vec3::new(0.0, 100.0, 0.0), 0.0, 0.0)
    }
}
