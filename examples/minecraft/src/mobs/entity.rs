// Base entity struct for all mobs
// Provides common properties like position, velocity, health, and collision

use glam::Vec3;
use crate::types::AABB;
use crate::inventory::Item;

/// Base entity properties shared by all mobs
#[derive(Debug, Clone)]
pub struct Entity {
    /// World position (center of mob)
    pub position: Vec3,

    /// Current velocity vector
    pub velocity: Vec3,

    /// Current health (0 = dead)
    pub health: f32,

    /// Maximum health
    pub max_health: f32,

    /// Collision box dimensions (half-extents from center)
    pub hitbox_size: Vec3,

    /// Rotation in radians (yaw around Y axis)
    pub yaw: f32,

    /// Whether the entity is on the ground
    pub on_ground: bool,

    /// Time since last damage (for damage cooldown)
    pub damage_cooldown: f32,
}

impl Entity {
    /// Create a new entity with given position and hitbox
    pub fn new(position: Vec3, hitbox_size: Vec3, max_health: f32) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            health: max_health,
            max_health,
            hitbox_size,
            yaw: 0.0,
            on_ground: false,
            damage_cooldown: 0.0,
        }
    }

    /// Get the AABB for collision detection
    pub fn get_aabb(&self) -> AABB {
        AABB::from_center_size(self.position, self.hitbox_size * 2.0)
    }

    /// Check if entity is alive
    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    /// Apply damage to entity (respects cooldown)
    pub fn take_damage(&mut self, amount: f32) -> bool {
        if self.damage_cooldown > 0.0 {
            return false; // Still in cooldown
        }

        self.health -= amount;
        self.damage_cooldown = 0.5; // 0.5 second damage cooldown

        if self.health < 0.0 {
            self.health = 0.0;
        }

        true
    }

    /// Heal the entity
    pub fn heal(&mut self, amount: f32) {
        self.health += amount;
        if self.health > self.max_health {
            self.health = self.max_health;
        }
    }

    /// Apply knockback in a direction
    pub fn apply_knockback(&mut self, direction: Vec3, strength: f32) {
        let knockback = direction.normalize() * strength;
        self.velocity += Vec3::new(knockback.x, 0.4, knockback.z); // Add upward component
    }

    /// Update damage cooldown timer
    pub fn update_cooldown(&mut self, delta_time: f32) {
        if self.damage_cooldown > 0.0 {
            self.damage_cooldown -= delta_time;
            if self.damage_cooldown < 0.0 {
                self.damage_cooldown = 0.0;
            }
        }
    }

    /// Get facing direction vector (horizontal only)
    pub fn get_facing_direction(&self) -> Vec3 {
        Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos())
    }

    /// Look at a target position (sets yaw)
    pub fn look_at(&mut self, target: Vec3) {
        let delta = target - self.position;
        self.yaw = delta.z.atan2(delta.x) - std::f32::consts::FRAC_PI_2;
    }
}

/// Death drops - items that spawn when a mob dies
#[derive(Debug, Clone)]
pub struct DeathDrop {
    pub item: Item,
    pub min_count: u32,
    pub max_count: u32,
    pub chance: f32, // 0.0 to 1.0
}

impl DeathDrop {
    pub fn new(item: Item, min_count: u32, max_count: u32, chance: f32) -> Self {
        Self {
            item,
            min_count,
            max_count,
            chance,
        }
    }

    /// Create a guaranteed drop
    pub fn guaranteed(item: Item, min_count: u32, max_count: u32) -> Self {
        Self::new(item, min_count, max_count, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let entity = Entity::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(0.4, 0.9, 0.4),
            20.0,
        );

        assert_eq!(entity.health, 20.0);
        assert_eq!(entity.max_health, 20.0);
        assert!(entity.is_alive());
    }

    #[test]
    fn test_damage() {
        let mut entity = Entity::new(Vec3::ZERO, Vec3::ONE, 10.0);

        assert!(entity.take_damage(3.0));
        assert_eq!(entity.health, 7.0);

        // Cooldown prevents immediate damage
        assert!(!entity.take_damage(2.0));
        assert_eq!(entity.health, 7.0);

        // After cooldown expires
        entity.damage_cooldown = 0.0;
        assert!(entity.take_damage(2.0));
        assert_eq!(entity.health, 5.0);
    }

    #[test]
    fn test_death() {
        let mut entity = Entity::new(Vec3::ZERO, Vec3::ONE, 10.0);

        entity.take_damage(15.0);
        assert_eq!(entity.health, 0.0);
        assert!(!entity.is_alive());
    }

    #[test]
    fn test_healing() {
        let mut entity = Entity::new(Vec3::ZERO, Vec3::ONE, 10.0);
        entity.health = 5.0;

        entity.heal(3.0);
        assert_eq!(entity.health, 8.0);

        entity.heal(10.0); // Over-heal
        assert_eq!(entity.health, 10.0); // Capped at max
    }

    #[test]
    fn test_knockback() {
        let mut entity = Entity::new(Vec3::ZERO, Vec3::ONE, 10.0);

        entity.apply_knockback(Vec3::new(1.0, 0.0, 0.0), 5.0);
        assert!(entity.velocity.x > 0.0);
        assert!(entity.velocity.y > 0.0); // Upward component
    }

    #[test]
    fn test_look_at() {
        let mut entity = Entity::new(Vec3::ZERO, Vec3::ONE, 10.0);

        entity.look_at(Vec3::new(10.0, 0.0, 0.0));
        let facing = entity.get_facing_direction();

        // Should be facing roughly in +X direction
        assert!(facing.x > 0.5);
    }
}
