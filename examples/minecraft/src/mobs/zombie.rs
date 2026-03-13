// Hostile zombie mob
// Spawns in darkness, pathfinds toward player, deals damage on contact

use glam::Vec3;
use crate::types::{BlockType, WorldPos};
use super::entity::{Entity, DeathDrop};
use super::pathfinding::simple_pathfind;
use crate::inventory::Item;

/// Zombie - hostile mob that attacks the player
pub struct Zombie {
    pub entity: Entity,

    /// Time since last attack
    attack_cooldown: f32,

    /// Time since last pathfinding update
    pathfind_timer: f32,

    /// Current target position to move toward
    target_position: Option<Vec3>,
}

impl Zombie {
    /// Standard zombie dimensions
    const HITBOX_SIZE: Vec3 = Vec3::new(0.3, 0.9, 0.3);
    const MAX_HEALTH: f32 = 20.0;
    const MOVE_SPEED: f32 = 3.5;
    const ATTACK_DAMAGE: f32 = 3.0;
    const ATTACK_COOLDOWN: f32 = 1.0; // 1 second between attacks
    const ATTACK_RANGE: f32 = 1.5;
    const DETECTION_RANGE: f32 = 16.0;
    const PATHFIND_UPDATE_INTERVAL: f32 = 0.5; // Update path twice per second

    /// Create a new zombie at the given position
    pub fn new(position: Vec3) -> Self {
        Self {
            entity: Entity::new(position, Self::HITBOX_SIZE, Self::MAX_HEALTH),
            attack_cooldown: 0.0,
            pathfind_timer: 0.0,
            target_position: None,
        }
    }

    /// Update zombie AI and physics
    pub fn update<F>(
        &mut self,
        delta_time: f32,
        player_position: Vec3,
        get_block: F,
    ) -> Option<ZombieAction>
    where
        F: Fn(&WorldPos) -> BlockType + Copy,
    {
        self.entity.update_cooldown(delta_time);
        self.attack_cooldown -= delta_time;
        self.pathfind_timer -= delta_time;

        if !self.entity.is_alive() {
            return None;
        }

        let distance_to_player = (player_position - self.entity.position).length();

        // Check if player is in range
        if distance_to_player > Self::DETECTION_RANGE {
            // Player too far, idle
            self.target_position = None;
            self.decelerate();
            self.apply_physics(delta_time, get_block);
            return None;
        }

        // Check if player is in attack range
        if distance_to_player <= Self::ATTACK_RANGE {
            // Stop moving and attack
            self.target_position = None;
            self.decelerate();

            // Look at player
            self.entity.look_at(player_position);

            // Attack if cooldown is ready
            if self.attack_cooldown <= 0.0 {
                self.attack_cooldown = Self::ATTACK_COOLDOWN;
                self.apply_physics(delta_time, get_block);
                return Some(ZombieAction::Attack);
            }

            self.apply_physics(delta_time, get_block);
            return None;
        }

        // Update pathfinding periodically
        if self.pathfind_timer <= 0.0 {
            self.target_position = Some(simple_pathfind(
                self.entity.position,
                player_position,
                get_block,
            ));
            self.pathfind_timer = Self::PATHFIND_UPDATE_INTERVAL;
        }

        // Move toward target
        if let Some(target) = self.target_position {
            let direction = (target - self.entity.position).normalize_or_zero();

            // Horizontal movement only
            let move_dir = Vec3::new(direction.x, 0.0, direction.z);

            if move_dir.length_squared() > 0.01 {
                self.entity.velocity.x = move_dir.x * Self::MOVE_SPEED;
                self.entity.velocity.z = move_dir.z * Self::MOVE_SPEED;

                // Look in movement direction
                self.entity.look_at(target);
            }

            // Jump if hitting a wall
            if self.entity.on_ground && self.is_blocked(get_block) {
                self.entity.velocity.y = 8.0; // Jump velocity
            }
        }

        self.apply_physics(delta_time, get_block);
        None
    }

    /// Check if the zombie is blocked by a wall in front
    fn is_blocked<F>(&self, get_block: F) -> bool
    where
        F: Fn(&WorldPos) -> BlockType,
    {
        let forward = self.entity.get_facing_direction();
        let check_pos = self.entity.position + forward * 0.5;
        let block_pos = WorldPos::from_vec3(check_pos);

        get_block(&block_pos).is_solid()
    }

    /// Slow down horizontal movement
    fn decelerate(&mut self) {
        self.entity.velocity.x *= 0.8;
        self.entity.velocity.z *= 0.8;
    }

    /// Apply physics (gravity and ground detection)
    fn apply_physics<F>(&mut self, delta_time: f32, get_block: F)
    where
        F: Fn(&WorldPos) -> BlockType,
    {
        // Apply gravity
        if !self.entity.on_ground {
            self.entity.velocity.y -= 32.0 * delta_time;
        } else {
            if self.entity.velocity.y < 0.0 {
                self.entity.velocity.y = 0.0;
            }
        }

        // Clamp fall speed
        if self.entity.velocity.y < -78.4 {
            self.entity.velocity.y = -78.4; // Terminal velocity
        }

        // Update position
        self.entity.position += self.entity.velocity * delta_time;

        // Ground check
        let feet_pos = WorldPos::from_vec3(
            self.entity.position - Vec3::new(0.0, self.entity.hitbox_size.y, 0.0)
        );
        let block_below = get_block(&feet_pos);
        self.entity.on_ground = block_below.is_solid();

        // Prevent falling through ground
        if self.entity.on_ground && self.entity.velocity.y < 0.0 {
            self.entity.position.y = feet_pos.y as f32 + 1.0 + self.entity.hitbox_size.y;
            self.entity.velocity.y = 0.0;
        }
    }

    /// Get the damage this zombie deals
    pub fn get_attack_damage() -> f32 {
        Self::ATTACK_DAMAGE
    }

    /// Get the items dropped when this zombie dies
    pub fn get_drops() -> Vec<DeathDrop> {
        vec![
            DeathDrop::guaranteed(Item::RottenFlesh, 0, 2),
        ]
    }

    /// Get render color (green)
    pub fn get_color() -> [f32; 3] {
        [0.3, 0.8, 0.3] // Green
    }

    /// Get a simple box mesh for rendering
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

/// Actions a zombie can perform
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZombieAction {
    Attack,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zombie_creation() {
        let zombie = Zombie::new(Vec3::new(0.0, 64.0, 0.0));
        assert_eq!(zombie.entity.health, Zombie::MAX_HEALTH);
        assert!(zombie.entity.is_alive());
    }

    #[test]
    fn test_zombie_attack_range() {
        let mut zombie = Zombie::new(Vec3::new(0.0, 64.0, 0.0));
        let player_pos = Vec3::new(1.0, 64.0, 0.0); // 1 block away

        let action = zombie.update(0.1, player_pos, |_| BlockType::Grass);

        assert_eq!(action, Some(ZombieAction::Attack));
    }

    #[test]
    fn test_zombie_out_of_range() {
        let mut zombie = Zombie::new(Vec3::new(0.0, 64.0, 0.0));
        let player_pos = Vec3::new(100.0, 64.0, 0.0); // Far away

        let action = zombie.update(0.1, player_pos, |_| BlockType::Grass);

        assert_eq!(action, None);
    }

    #[test]
    fn test_zombie_drops() {
        let drops = Zombie::get_drops();
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].item, Item::RottenFlesh);
    }
}
