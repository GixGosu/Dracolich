// Passive pig mob
// Wanders randomly, drops raw porkchop on death

use glam::Vec3;
use rand::Rng;
use crate::types::{BlockType, WorldPos};
use super::entity::{Entity, DeathDrop};
use crate::inventory::Item;

/// Pig - passive mob that wanders randomly
pub struct Pig {
    pub entity: Entity,

    /// Current wander target (changes periodically)
    wander_target: Option<Vec3>,

    /// Time until next wander target change
    wander_timer: f32,

    /// Time until next random idle action
    idle_timer: f32,
}

impl Pig {
    /// Standard pig dimensions (slightly smaller than 1 block)
    const HITBOX_SIZE: Vec3 = Vec3::new(0.45, 0.45, 0.45);
    const MAX_HEALTH: f32 = 10.0;
    const MOVE_SPEED: f32 = 2.0;
    const WANDER_RANGE: f32 = 8.0;

    /// Create a new pig at the given position
    pub fn new(position: Vec3) -> Self {
        Self {
            entity: Entity::new(position, Self::HITBOX_SIZE, Self::MAX_HEALTH),
            wander_target: None,
            wander_timer: 0.0,
            idle_timer: 0.0,
        }
    }

    /// Update pig AI and physics
    pub fn update<F>(&mut self, delta_time: f32, get_block: F)
    where
        F: Fn(&WorldPos) -> BlockType,
    {
        self.entity.update_cooldown(delta_time);

        if !self.entity.is_alive() {
            return;
        }

        // Update timers
        self.wander_timer -= delta_time;
        self.idle_timer -= delta_time;

        // Decide on new wander target
        if self.wander_timer <= 0.0 {
            self.pick_new_wander_target();
            self.wander_timer = rand::thread_rng().gen_range(3.0..8.0);
        }

        // Occasional idle behavior (stop moving)
        if self.idle_timer <= 0.0 {
            if rand::thread_rng().gen_bool(0.3) {
                // 30% chance to idle
                self.wander_target = None;
                self.idle_timer = rand::thread_rng().gen_range(2.0..5.0);
            }
            self.idle_timer = rand::thread_rng().gen_range(5.0..10.0);
        }

        // Move toward wander target
        if let Some(target) = self.wander_target {
            let direction = (target - self.entity.position).normalize_or_zero();

            // Horizontal movement only
            let move_dir = Vec3::new(direction.x, 0.0, direction.z);

            if move_dir.length_squared() > 0.01 {
                self.entity.velocity.x = move_dir.x * Self::MOVE_SPEED;
                self.entity.velocity.z = move_dir.z * Self::MOVE_SPEED;

                // Look in movement direction
                self.entity.look_at(target);
            }

            // Stop if we reached the target
            let distance = (target - self.entity.position).length();
            if distance < 0.5 {
                self.wander_target = None;
            }
        } else {
            // Decelerate when idle
            self.entity.velocity.x *= 0.8;
            self.entity.velocity.z *= 0.8;
        }

        // Apply gravity
        if !self.entity.on_ground {
            self.entity.velocity.y -= 32.0 * delta_time; // Gravity
        } else {
            self.entity.velocity.y = 0.0;
        }

        // Update position
        self.entity.position += self.entity.velocity * delta_time;

        // Simple ground check
        let feet_pos = WorldPos::from_vec3(self.entity.position - Vec3::new(0.0, self.entity.hitbox_size.y, 0.0));
        let block_below = get_block(&feet_pos);
        self.entity.on_ground = block_below.is_solid();
    }

    /// Pick a random nearby position to wander toward
    fn pick_new_wander_target(&mut self) {
        let mut rng = rand::thread_rng();

        let offset = Vec3::new(
            rng.gen_range(-Self::WANDER_RANGE..Self::WANDER_RANGE),
            0.0,
            rng.gen_range(-Self::WANDER_RANGE..Self::WANDER_RANGE),
        );

        self.wander_target = Some(self.entity.position + offset);
    }

    /// Get the items dropped when this pig dies
    pub fn get_drops() -> Vec<DeathDrop> {
        vec![
            DeathDrop::guaranteed(Item::RawPorkchop, 1, 3),
        ]
    }

    /// Get render color (pink)
    pub fn get_color() -> [f32; 3] {
        [1.0, 0.75, 0.8] // Light pink
    }

    /// Get a simple box mesh for rendering (8 vertices for a box)
    pub fn get_render_vertices(&self) -> Vec<[f32; 3]> {
        let pos = self.entity.position;
        let size = Self::HITBOX_SIZE;

        vec![
            // Bottom face
            [pos.x - size.x, pos.y - size.y, pos.z - size.z],
            [pos.x + size.x, pos.y - size.y, pos.z - size.z],
            [pos.x + size.x, pos.y - size.y, pos.z + size.z],
            [pos.x - size.x, pos.y - size.y, pos.z + size.z],
            // Top face
            [pos.x - size.x, pos.y + size.y, pos.z - size.z],
            [pos.x + size.x, pos.y + size.y, pos.z - size.z],
            [pos.x + size.x, pos.y + size.y, pos.z + size.z],
            [pos.x - size.x, pos.y + size.y, pos.z + size.z],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pig_creation() {
        let pig = Pig::new(Vec3::new(0.0, 64.0, 0.0));
        assert_eq!(pig.entity.health, Pig::MAX_HEALTH);
        assert!(pig.entity.is_alive());
    }

    #[test]
    fn test_pig_wander() {
        let mut pig = Pig::new(Vec3::new(0.0, 64.0, 0.0));
        let start_pos = pig.entity.position;

        // Update multiple times
        for _ in 0..100 {
            pig.update(0.1, |_| BlockType::Grass);
        }

        // Pig should have moved
        let moved_distance = (pig.entity.position - start_pos).length();
        assert!(moved_distance > 0.0);
    }

    #[test]
    fn test_pig_drops() {
        let drops = Pig::get_drops();
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].item, Item::RawPorkchop);
        assert!(drops[0].min_count >= 1);
    }
}
