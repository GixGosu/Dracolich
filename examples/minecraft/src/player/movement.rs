// Player movement and camera controls
// Handles input processing, velocity updates, mouse look, jumping, and sprinting

use glam::Vec3;
use crate::physics::{GRAVITY, TERMINAL_VELOCITY, collide_and_slide, is_on_ground};
use crate::types::{BlockType, WorldPos};
use super::{Player, PLAYER_HEIGHT};

/// Movement constants
pub const WALK_SPEED: f32 = 4.3;                    // blocks per second
pub const SPRINT_SPEED: f32 = 5.612;                // blocks per second (1.3x walk)
pub const JUMP_VELOCITY: f32 = 8.5;                 // initial upward velocity (~1.25 blocks)
pub const MOUSE_SENSITIVITY: f32 = 0.002;           // radians per pixel
pub const MAX_PITCH: f32 = 89.0_f32.to_radians();   // ±89 degrees

impl Player {
    /// Update camera rotation from mouse delta
    /// delta is (dx, dy) pre-scaled by sensitivity (radians)
    pub fn update_look(&mut self, mouse_delta: (f64, f64)) {
        let dx = mouse_delta.0 as f32;
        let dy = mouse_delta.1 as f32;

        // Yaw (horizontal) - wraps around
        self.yaw += dx;
        self.yaw = self.yaw.rem_euclid(std::f32::consts::TAU);

        // Pitch (vertical) - clamped to ±89 degrees
        self.pitch -= dy; // Inverted Y
        self.pitch = self.pitch.clamp(-MAX_PITCH, MAX_PITCH);
    }

    /// Apply movement input to player velocity
    /// movement_input is (forward, strafe) normalized vector from WASD
    /// is_sprinting indicates if sprint key is held
    pub fn apply_movement_input(&mut self, movement_input: Vec3, is_sprinting: bool) {
        if movement_input.length_squared() < 0.01 {
            // No input - decelerate horizontally
            self.velocity.x *= 0.5;
            self.velocity.z *= 0.5;
            return;
        }

        // Calculate movement direction in world space
        let forward = self.forward();
        let right = self.right();

        let mut direction = forward * movement_input.z + right * movement_input.x;

        // Normalize to prevent diagonal speed boost
        if direction.length_squared() > 0.01 {
            direction = direction.normalize();
        }

        // Apply speed
        let speed = if is_sprinting && self.on_ground {
            self.is_sprinting = true;
            SPRINT_SPEED
        } else {
            self.is_sprinting = false;
            WALK_SPEED
        };

        // Set horizontal velocity
        self.velocity.x = direction.x * speed;
        self.velocity.z = direction.z * speed;
    }

    /// Attempt to jump if on ground
    pub fn jump(&mut self) {
        if self.on_ground {
            self.velocity.y = JUMP_VELOCITY;
            self.on_ground = false;
        }
    }

    /// Update physics simulation
    /// delta_time is in seconds
    /// get_block is a closure that returns the block type at a given position
    pub fn update_physics<F>(&mut self, delta_time: f32, get_block: F)
    where
        F: Fn(&WorldPos) -> BlockType,
    {
        // Apply gravity
        if !self.on_ground {
            self.velocity.y += GRAVITY * delta_time;
            // Clamp to terminal velocity
            self.velocity.y = self.velocity.y.max(TERMINAL_VELOCITY);
        }

        // Calculate desired movement
        let movement = self.velocity * delta_time;

        // Get current AABB
        let aabb = self.aabb();

        // Perform collision-aware movement
        let result = collide_and_slide(&aabb, movement, &get_block);

        // Update position - result.position is AABB center, convert back to feet level
        self.position = result.position - Vec3::new(0.0, PLAYER_HEIGHT * 0.5, 0.0);

        // Get the updated AABB at new position for ground check
        let new_aabb = self.aabb();

        // Update ground state - use collision result OR explicit ground check
        // The explicit check handles cases where we're standing still or moving horizontally
        self.on_ground = result.on_ground || is_on_ground(&new_aabb, &get_block);

        // Stop vertical velocity if we hit ground or ceiling
        if self.on_ground && self.velocity.y < 0.0 {
            self.velocity.y = 0.0;
        }
        if result.hit_ceiling && self.velocity.y > 0.0 {
            self.velocity.y = 0.0;
        }

        // Update fall tracking for damage calculation
        self.update_fall_tracking();
    }

    /// Get facing direction as a string (for debug display)
    pub fn facing_direction_string(&self) -> String {
        let angle = (self.yaw.to_degrees() + 180.0).rem_euclid(360.0);

        let direction = match angle {
            a if a >= 337.5 || a < 22.5 => "South (+Z)",
            a if a >= 22.5 && a < 67.5 => "Southwest",
            a if a >= 67.5 && a < 112.5 => "West (-X)",
            a if a >= 112.5 && a < 157.5 => "Northwest",
            a if a >= 157.5 && a < 202.5 => "North (-Z)",
            a if a >= 202.5 && a < 247.5 => "Northeast",
            a if a >= 247.5 && a < 292.5 => "East (+X)",
            a if a >= 292.5 && a < 337.5 => "Southeast",
            _ => "Unknown",
        };

        format!("{} ({:.1}°)", direction, angle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BlockType;

    fn air_world(_pos: &WorldPos) -> BlockType {
        BlockType::Air
    }

    fn ground_world(pos: &WorldPos) -> BlockType {
        if pos.y <= 0 {
            BlockType::Stone
        } else {
            BlockType::Air
        }
    }

    #[test]
    fn test_mouse_look() {
        let mut player = Player::new(Vec3::new(0.0, 10.0, 0.0));

        // Look right (positive yaw) - delta is pre-scaled (radians)
        player.update_look((0.2, 0.0));
        assert!(player.yaw > 0.0);

        // Look left (negative yaw)
        player.update_look((-0.4, 0.0));
        assert!(player.yaw < 0.0);

        // Look down (negative pitch)
        player.update_look((0.0, 0.2));
        assert!(player.pitch < 0.0);

        // Try to look beyond limit (should clamp)
        player.update_look((0.0, -20.0)); // Large value in radians
        assert!(player.pitch <= MAX_PITCH);
        assert!(player.pitch >= -MAX_PITCH);
    }

    #[test]
    fn test_jump() {
        let mut player = Player::new(Vec3::new(0.0, 1.0, 0.0));

        // Can't jump while in air
        player.on_ground = false;
        player.jump();
        assert_eq!(player.velocity.y, 0.0);

        // Can jump when on ground
        player.on_ground = true;
        player.jump();
        assert_eq!(player.velocity.y, JUMP_VELOCITY);
        assert!(!player.on_ground);
    }

    #[test]
    fn test_gravity() {
        let mut player = Player::new(Vec3::new(0.0, 100.0, 0.0));
        player.on_ground = false;

        let initial_y = player.position.y;

        // Apply physics for 1 second in air
        player.update_physics(1.0, air_world);

        // Should have fallen
        assert!(player.position.y < initial_y);
        assert!(player.velocity.y < 0.0);
    }

    #[test]
    fn test_ground_collision() {
        let mut player = Player::new(Vec3::new(0.0, 5.0, 0.0));
        player.velocity.y = -10.0;
        player.on_ground = false;

        // Update physics several times to fall to ground
        for _ in 0..20 {
            player.update_physics(0.1, ground_world);
        }

        // Should be on ground and stopped falling
        assert!(player.on_ground);
        assert_eq!(player.velocity.y, 0.0);
        // Position should be above ground (player height)
        assert!(player.position.y > 0.0);
    }

    #[test]
    fn test_movement_input() {
        let mut player = Player::new(Vec3::ZERO);
        player.yaw = 0.0; // Face north

        // Move forward (negative Z)
        let input = Vec3::new(0.0, 0.0, 1.0);
        player.apply_movement_input(input, false);

        assert!((player.velocity.x - 0.0).abs() < 0.01);
        assert!((player.velocity.z - (-WALK_SPEED)).abs() < 0.01);
    }

    #[test]
    fn test_sprint() {
        let mut player = Player::new(Vec3::ZERO);
        player.on_ground = true;

        // Walk
        let input = Vec3::new(0.0, 0.0, 1.0);
        player.apply_movement_input(input, false);
        let walk_speed = player.velocity.length();

        // Sprint
        player.apply_movement_input(input, true);
        let sprint_speed = player.velocity.length();

        assert!(sprint_speed > walk_speed);
        assert!((sprint_speed - SPRINT_SPEED).abs() < 0.01);
    }

    #[test]
    fn test_terminal_velocity() {
        let mut player = Player::new(Vec3::new(0.0, 1000.0, 0.0));
        player.on_ground = false;

        // Fall for a very long time
        for _ in 0..1000 {
            player.update_physics(0.1, air_world);
        }

        // Velocity should not exceed terminal velocity
        assert!(player.velocity.y >= TERMINAL_VELOCITY);
    }

    #[test]
    fn test_facing_direction() {
        let mut player = Player::new(Vec3::ZERO);

        player.yaw = 0.0;
        assert!(player.facing_direction_string().contains("North"));

        player.yaw = std::f32::consts::PI / 2.0;
        assert!(player.facing_direction_string().contains("East"));

        player.yaw = std::f32::consts::PI;
        assert!(player.facing_direction_string().contains("South"));

        player.yaw = -std::f32::consts::PI / 2.0;
        assert!(player.facing_direction_string().contains("West"));
    }
}
