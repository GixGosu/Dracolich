// Mob AI and rendering system
// Complete mob implementation with spawning, pathfinding, combat, and rendering

pub mod entity;
pub mod pig;
pub mod zombie;
pub mod pathfinding;
pub mod spawning;
pub mod combat;

use glam::Vec3;
use crate::types::{BlockType, WorldPos};
use crate::inventory::Item;

pub use entity::{Entity, DeathDrop};
pub use pig::Pig;
pub use zombie::{Zombie, ZombieAction};
pub use spawning::{SpawnConfig, MobSpawner, SpawnResult, calculate_light_level};
pub use combat::{CombatResult, ItemDrop, DamageEvent, DamageType};

/// Trait for all mob types
pub trait Mob {
    /// Update the mob's AI and physics
    fn update<F>(&mut self, delta_time: f32, player_position: Vec3, get_block: F)
    where
        F: Fn(&WorldPos) -> BlockType;

    /// Get the mob's entity (for position, health, etc.)
    fn entity(&self) -> &Entity;

    /// Get mutable reference to entity
    fn entity_mut(&mut self) -> &mut Entity;

    /// Check if the mob is alive
    fn is_alive(&self) -> bool {
        self.entity().is_alive()
    }

    /// Get the mob's position
    fn position(&self) -> Vec3 {
        self.entity().position
    }

    /// Get render color [r, g, b]
    fn get_color(&self) -> [f32; 3];

    /// Get vertices for rendering
    fn get_render_vertices(&self) -> Vec<[f32; 3]>;

    /// Get the items this mob drops on death
    fn get_drops(&self) -> Vec<DeathDrop>;
}

/// All mob types in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobType {
    Pig,
    Zombie,
}

/// Wrapper for different mob types
pub enum MobInstance {
    Pig(Pig),
    Zombie(Zombie),
}

impl MobInstance {
    /// Create a new mob of the given type at position
    pub fn new(mob_type: MobType, position: Vec3) -> Self {
        match mob_type {
            MobType::Pig => MobInstance::Pig(Pig::new(position)),
            MobType::Zombie => MobInstance::Zombie(Zombie::new(position)),
        }
    }

    /// Update the mob (delegates to specific implementation)
    pub fn update<F>(&mut self, delta_time: f32, player_position: Vec3, get_block: F) -> Option<ZombieAction>
    where
        F: Fn(&WorldPos) -> BlockType + Copy,
    {
        match self {
            MobInstance::Pig(pig) => {
                pig.update(delta_time, get_block);
                None
            }
            MobInstance::Zombie(zombie) => {
                zombie.update(delta_time, player_position, get_block)
            }
        }
    }

    /// Get entity reference
    pub fn entity(&self) -> &Entity {
        match self {
            MobInstance::Pig(pig) => &pig.entity,
            MobInstance::Zombie(zombie) => &zombie.entity,
        }
    }

    /// Get mutable entity reference
    pub fn entity_mut(&mut self) -> &mut Entity {
        match self {
            MobInstance::Pig(pig) => &mut pig.entity,
            MobInstance::Zombie(zombie) => &mut zombie.entity,
        }
    }

    /// Check if alive
    pub fn is_alive(&self) -> bool {
        self.entity().is_alive()
    }

    /// Get position
    pub fn position(&self) -> Vec3 {
        self.entity().position
    }

    /// Get render color
    pub fn get_color(&self) -> [f32; 3] {
        match self {
            MobInstance::Pig(_) => Pig::get_color(),
            MobInstance::Zombie(_) => Zombie::get_color(),
        }
    }

    /// Get render vertices
    pub fn get_render_vertices(&self) -> Vec<[f32; 3]> {
        match self {
            MobInstance::Pig(pig) => pig.get_render_vertices(),
            MobInstance::Zombie(zombie) => zombie.get_render_vertices(),
        }
    }

    /// Get death drops
    pub fn get_drops(&self) -> Vec<DeathDrop> {
        match self {
            MobInstance::Pig(_) => Pig::get_drops(),
            MobInstance::Zombie(_) => Zombie::get_drops(),
        }
    }

    /// Get the mob type
    pub fn mob_type(&self) -> MobType {
        match self {
            MobInstance::Pig(_) => MobType::Pig,
            MobInstance::Zombie(_) => MobType::Zombie,
        }
    }
}

/// Manages all mobs in the world
pub struct MobManager {
    /// All active mobs
    mobs: Vec<MobInstance>,

    /// Spawners for different mob types
    pig_spawner: MobSpawner,
    zombie_spawner: MobSpawner,

    /// Spawn configurations
    pig_config: SpawnConfig,
    zombie_config: SpawnConfig,
}

impl MobManager {
    /// Create a new mob manager
    pub fn new() -> Self {
        Self {
            mobs: Vec::new(),
            pig_spawner: MobSpawner::new(20, 30),  // Using default passive/hostile counts
            zombie_spawner: MobSpawner::new(20, 30),
            pig_config: SpawnConfig::passive(),
            zombie_config: SpawnConfig::hostile(),
        }
    }

    /// Spawn a mob of the given type at position
    pub fn spawn_mob(&mut self, mob_type: MobType, position: Vec3) {
        self.mobs.push(MobInstance::new(mob_type, position));
    }

    /// Update all mobs
    pub fn update<F>(
        &mut self,
        delta_time: f32,
        player_position: Vec3,
        is_night: bool,
        get_block: F,
    ) -> Vec<MobAttackEvent>
    where
        F: Fn(&WorldPos) -> BlockType + Copy,
    {
        let mut attack_events = Vec::new();

        // Update existing mobs
        for mob in &mut self.mobs {
            if let Some(action) = mob.update(delta_time, player_position, get_block) {
                // Zombie attacked
                if let ZombieAction::Attack = action {
                    attack_events.push(MobAttackEvent {
                        attacker_position: mob.position(),
                        damage: Zombie::get_attack_damage(),
                    });
                }
            }
        }

        // Handle spawning
        self.handle_spawning(delta_time, player_position, is_night, get_block);

        // Remove dead mobs and generate drops
        self.remove_dead_mobs();

        attack_events
    }

    /// Handle mob spawning
    fn handle_spawning<F>(
        &mut self,
        delta_time: f32,
        player_position: Vec3,
        is_night: bool,
        get_block: F,
    )
    where
        F: Fn(&WorldPos) -> BlockType + Copy,
    {
        // Count mobs by type
        let pig_count = self.mobs.iter().filter(|m| m.mob_type() == MobType::Pig).count();
        let zombie_count = self.mobs.iter().filter(|m| m.mob_type() == MobType::Zombie).count();

        // Try to spawn pigs
        let pig_spawns = self.pig_spawner.update(
            delta_time,
            player_position,
            pig_count,
            is_night,
            &self.pig_config,
            get_block,
            |pos| calculate_light_level(pos, !is_night),
        );

        for spawn_pos in pig_spawns {
            self.spawn_mob(MobType::Pig, spawn_pos);
        }

        // Try to spawn zombies
        let zombie_spawns = self.zombie_spawner.update(
            delta_time,
            player_position,
            zombie_count,
            is_night,
            &self.zombie_config,
            get_block,
            |pos| calculate_light_level(pos, !is_night),
        );

        for spawn_pos in zombie_spawns {
            self.spawn_mob(MobType::Zombie, spawn_pos);
        }
    }

    /// Remove dead mobs and handle their drops
    fn remove_dead_mobs(&mut self) -> Vec<ItemDrop> {
        let mut all_drops = Vec::new();

        self.mobs.retain(|mob| {
            if !mob.is_alive() {
                // Generate drops
                let drops = combat::handle_death(mob.entity(), &mob.get_drops());
                all_drops.extend(drops);
                false // Remove this mob
            } else {
                true // Keep this mob
            }
        });

        all_drops
    }

    /// Get all active mobs
    pub fn mobs(&self) -> &[MobInstance] {
        &self.mobs
    }

    /// Get mutable reference to all mobs
    pub fn mobs_mut(&mut self) -> &mut [MobInstance] {
        &mut self.mobs
    }

    /// Get count of mobs
    pub fn mob_count(&self) -> usize {
        self.mobs.len()
    }

    /// Get count of mobs by type
    pub fn mob_count_by_type(&self, mob_type: MobType) -> usize {
        self.mobs.iter().filter(|m| m.mob_type() == mob_type).count()
    }

    /// Apply damage to mobs in range of an attack
    pub fn damage_mobs_in_range(
        &mut self,
        attack_position: Vec3,
        attack_range: f32,
        damage: f32,
    ) -> Vec<(usize, CombatResult)> {
        let mut results = Vec::new();

        for (i, mob) in self.mobs.iter_mut().enumerate() {
            let distance = (mob.position() - attack_position).length();

            if distance <= attack_range {
                let result = combat::apply_damage(
                    mob.entity_mut(),
                    attack_position,
                    damage,
                    2.0,
                );
                results.push((i, result));
            }
        }

        results
    }
}

impl Default for MobManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Event fired when a mob attacks
#[derive(Debug, Clone)]
pub struct MobAttackEvent {
    pub attacker_position: Vec3,
    pub damage: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_world(pos: &WorldPos) -> BlockType {
        if pos.y <= 64 {
            BlockType::Grass
        } else {
            BlockType::Air
        }
    }

    #[test]
    fn test_mob_manager_creation() {
        let manager = MobManager::new();
        assert_eq!(manager.mob_count(), 0);
    }

    #[test]
    fn test_spawn_mob() {
        let mut manager = MobManager::new();

        manager.spawn_mob(MobType::Pig, Vec3::new(0.0, 65.0, 0.0));
        assert_eq!(manager.mob_count(), 1);

        manager.spawn_mob(MobType::Zombie, Vec3::new(5.0, 65.0, 0.0));
        assert_eq!(manager.mob_count(), 2);
    }

    #[test]
    fn test_mob_update() {
        let mut manager = MobManager::new();
        manager.spawn_mob(MobType::Pig, Vec3::new(0.0, 65.0, 0.0));

        let player_pos = Vec3::new(10.0, 65.0, 10.0);
        manager.update(0.1, player_pos, false, mock_world);

        assert_eq!(manager.mob_count(), 1);
    }

    #[test]
    fn test_mob_death_removal() {
        let mut manager = MobManager::new();
        manager.spawn_mob(MobType::Zombie, Vec3::new(0.0, 65.0, 0.0));

        // Kill the zombie
        manager.mobs_mut()[0].entity_mut().health = 0.0;

        // Update should remove dead mobs
        manager.update(0.1, Vec3::ZERO, true, mock_world);

        assert_eq!(manager.mob_count(), 0);
    }

    #[test]
    fn test_damage_mobs_in_range() {
        let mut manager = MobManager::new();
        manager.spawn_mob(MobType::Zombie, Vec3::new(0.0, 65.0, 0.0));
        manager.spawn_mob(MobType::Pig, Vec3::new(10.0, 65.0, 0.0));

        let results = manager.damage_mobs_in_range(Vec3::new(0.0, 65.0, 0.0), 2.0, 5.0);

        // Only the zombie should be damaged (pig is too far)
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_mob_count_by_type() {
        let mut manager = MobManager::new();
        manager.spawn_mob(MobType::Pig, Vec3::new(0.0, 65.0, 0.0));
        manager.spawn_mob(MobType::Pig, Vec3::new(1.0, 65.0, 0.0));
        manager.spawn_mob(MobType::Zombie, Vec3::new(2.0, 65.0, 0.0));

        assert_eq!(manager.mob_count_by_type(MobType::Pig), 2);
        assert_eq!(manager.mob_count_by_type(MobType::Zombie), 1);
    }
}
