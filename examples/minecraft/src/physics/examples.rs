// Example usage patterns for the physics engine
// This file is not part of the compiled module but serves as documentation

#![allow(dead_code, unused_imports)]

use crate::physics::{
    aabb::SweepResult,
    collision::{collide_and_slide, is_on_ground, move_with_collision, CollisionInfo},
    raycast::{raycast, RaycastHit, MAX_RAYCAST_DISTANCE},
    GRAVITY, TERMINAL_VELOCITY,
};
use crate::types::{BlockType, WorldPos, AABB};
use glam::Vec3;

/// Example: Player controller with physics
pub struct PlayerPhysicsExample {
    position: Vec3,
    velocity: Vec3,
    on_ground: bool,
}

impl PlayerPhysicsExample {
    /// Update player physics each frame
    pub fn update<F>(&mut self, delta_time: f32, input_direction: Vec3, get_block: F)
    where
        F: Fn(&WorldPos) -> BlockType,
    {
        // Player dimensions (0.6 wide, 1.8 tall)
        let player_size = Vec3::new(0.6, 1.8, 0.6);
        let player_aabb = AABB::from_center_size(self.position, player_size);

        // Apply gravity (with terminal velocity)
        self.velocity.y = (self.velocity.y + GRAVITY * delta_time).max(TERMINAL_VELOCITY);

        // Apply input (walking speed: 4.3 blocks/s)
        let walk_speed = 4.3;
        let walk_velocity = input_direction.normalize_or_zero() * walk_speed;
        self.velocity.x = walk_velocity.x;
        self.velocity.z = walk_velocity.z;

        // Perform collision detection and response
        let collision = move_with_collision(&player_aabb, self.velocity, delta_time, get_block);

        // Update player state
        self.position = collision.position;
        self.on_ground = collision.on_ground;

        // Stop falling when on ground
        if collision.on_ground {
            self.velocity.y = 0.0;
        }

        // Stop rising when hitting ceiling
        if collision.hit_ceiling {
            self.velocity.y = self.velocity.y.min(0.0);
        }
    }

    /// Handle jump input
    pub fn jump(&mut self) {
        if self.on_ground {
            self.velocity.y = 8.0; // Jump velocity (reaches ~3.2 blocks high)
        }
    }
}

/// Example: Block targeting system
pub struct BlockTargetingExample {
    targeted_block: Option<WorldPos>,
    placement_position: Option<WorldPos>,
}

impl BlockTargetingExample {
    /// Update which block the player is looking at
    pub fn update<F>(&mut self, camera_pos: Vec3, camera_dir: Vec3, get_block: F)
    where
        F: Fn(&WorldPos) -> BlockType,
    {
        match raycast(camera_pos, camera_dir, MAX_RAYCAST_DISTANCE, get_block) {
            Some(hit) => {
                self.targeted_block = Some(hit.block_pos);
                self.placement_position = Some(hit.adjacent_pos);
            }
            None => {
                self.targeted_block = None;
                self.placement_position = None;
            }
        }
    }

    /// Break the targeted block
    pub fn break_block<F>(&self, set_block: F)
    where
        F: Fn(&WorldPos, BlockType),
    {
        if let Some(pos) = self.targeted_block {
            set_block(&pos, BlockType::Air);
        }
    }

    /// Place a block on the targeted surface
    pub fn place_block<F, G>(&self, block_type: BlockType, get_block: F, set_block: G)
    where
        F: Fn(&WorldPos) -> BlockType,
        G: Fn(&WorldPos, BlockType),
    {
        if let Some(pos) = self.placement_position {
            // Only place if the position is empty
            if get_block(&pos) == BlockType::Air {
                set_block(&pos, block_type);
            }
        }
    }
}

/// Example: Mob with simple physics
pub struct MobPhysicsExample {
    position: Vec3,
    velocity: Vec3,
    size: Vec3,
}

impl MobPhysicsExample {
    pub fn new(position: Vec3, size: Vec3) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            size,
        }
    }

    /// Update mob physics
    pub fn update<F>(&mut self, delta_time: f32, get_block: F)
    where
        F: Fn(&WorldPos) -> BlockType,
    {
        let mob_aabb = AABB::from_center_size(self.position, self.size);

        // Apply gravity
        self.velocity.y += GRAVITY * delta_time;

        // Collision detection
        let collision = move_with_collision(&mob_aabb, self.velocity, delta_time, get_block);

        self.position = collision.position;

        // Reset vertical velocity when on ground or hitting ceiling
        if collision.on_ground || collision.hit_ceiling {
            self.velocity.y = 0.0;
        }
    }

    /// Check if mob can see a target position
    pub fn can_see<F>(&self, target: Vec3, get_block: F) -> bool
    where
        F: Fn(&WorldPos) -> BlockType,
    {
        use crate::physics::raycast::has_line_of_sight;

        // Check from mob's eye height
        let eye_pos = self.position + Vec3::new(0.0, self.size.y * 0.8, 0.0);
        has_line_of_sight(eye_pos, target, get_block)
    }
}

/// Example: Falling block physics (sand, gravel)
pub struct FallingBlockExample {
    position: Vec3,
    velocity: Vec3,
    block_type: BlockType,
}

impl FallingBlockExample {
    pub fn new(position: Vec3, block_type: BlockType) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            block_type,
        }
    }

    /// Update falling block
    pub fn update<F>(&mut self, delta_time: f32, get_block: F) -> bool
    where
        F: Fn(&WorldPos) -> BlockType,
    {
        // Block is exactly 1x1x1
        let block_aabb = AABB::from_center_size(self.position, Vec3::ONE);

        // Apply gravity
        self.velocity.y += GRAVITY * delta_time;

        // Collision detection
        let collision = move_with_collision(&block_aabb, self.velocity, delta_time, get_block);

        self.position = collision.position;

        // Return true if block has landed (should be placed in world)
        collision.on_ground && self.velocity.y.abs() < 0.1
    }
}

/// Example: Projectile physics (arrows, fireballs)
pub struct ProjectileExample {
    position: Vec3,
    velocity: Vec3,
    size: f32,
}

impl ProjectileExample {
    /// Update projectile and check for collision
    /// Returns Some(hit_pos) if projectile hit a block
    pub fn update<F>(&mut self, delta_time: f32, get_block: F) -> Option<WorldPos>
    where
        F: Fn(&WorldPos) -> BlockType,
    {
        use crate::physics::raycast::raycast;

        // Raycast along velocity to check for block hit
        let direction = self.velocity.normalize_or_zero();
        let distance = self.velocity.length() * delta_time;

        if let Some(hit) = raycast(self.position, direction, distance, get_block) {
            // Hit a block
            return Some(hit.block_pos);
        }

        // Move projectile
        self.position += self.velocity * delta_time;

        // Apply gravity (slight arc for arrows)
        self.velocity.y += GRAVITY * 0.05 * delta_time;

        None
    }
}

/// Example: Explosion physics
pub fn explosion_example<F>(center: Vec3, radius: f32, get_block: F) -> Vec<WorldPos>
where
    F: Fn(&WorldPos) -> BlockType,
{
    use crate::physics::raycast::has_line_of_sight;

    let mut blocks_to_destroy = Vec::new();
    let r = radius.ceil() as i32;

    // Check all blocks in radius
    for x in -r..=r {
        for y in -r..=r {
            for z in -r..=r {
                let offset = Vec3::new(x as f32, y as f32, z as f32);
                let distance = offset.length();

                if distance <= radius {
                    let block_pos = WorldPos::from_vec3(center + offset);
                    let block = get_block(&block_pos);

                    // Only destroy breakable blocks with line of sight
                    if block.is_breakable() && has_line_of_sight(center, block_pos.to_vec3(), &get_block) {
                        blocks_to_destroy.push(block_pos);
                    }
                }
            }
        }
    }

    blocks_to_destroy
}
