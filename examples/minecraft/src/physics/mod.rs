// Physics and collision detection
// Handles AABB collision resolution, raycasting for block targeting,
// gravity, and collision response

pub mod aabb;
pub mod collision;
pub mod raycast;

pub use self::aabb::*;
pub use self::collision::*;
pub use self::raycast::*;

use glam::Vec3;

/// Physics constants
pub const GRAVITY: f32 = -32.0; // blocks per second squared
pub const TERMINAL_VELOCITY: f32 = -78.4; // max falling speed

pub struct PhysicsEngine {
    pub gravity: f32,
}

impl PhysicsEngine {
    pub fn new() -> Self {
        Self {
            gravity: GRAVITY,
        }
    }
}

impl Default for PhysicsEngine {
    fn default() -> Self {
        Self::new()
    }
}
