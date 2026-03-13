// Mob spawning system
// Handles mob spawn rules, light level checks, spawn caps, and spawn timing

use glam::Vec3;
use rand::Rng;
use crate::types::{BlockType, WorldPos, ChunkPos};
use super::pathfinding::find_valid_spawn_position;
use super::MobType;

/// Result of a spawn attempt
#[derive(Debug, Clone)]
pub struct SpawnResult {
    pub mob_type: MobType,
    pub position: Vec3,
}

/// Configuration for mob spawning
#[derive(Debug, Clone)]
pub struct SpawnConfig {
    /// Maximum number of mobs of this type in the world
    pub max_count: usize,

    /// Minimum light level required for spawning (0-15)
    pub min_light_level: u8,

    /// Maximum light level allowed for spawning (0-15)
    pub max_light_level: u8,

    /// Can spawn during day
    pub spawn_day: bool,

    /// Can spawn during night
    pub spawn_night: bool,

    /// Minimum distance from player
    pub min_player_distance: f32,

    /// Maximum distance from player
    pub max_player_distance: f32,

    /// Spawn attempts per second
    pub spawn_rate: f32,

    /// Chance of successful spawn per attempt (0.0 to 1.0)
    pub spawn_chance: f32,
}

impl SpawnConfig {
    /// Config for passive mobs (like pigs)
    pub fn passive() -> Self {
        Self {
            max_count: 20,
            min_light_level: 9,
            max_light_level: 15,
            spawn_day: true,
            spawn_night: false,
            min_player_distance: 24.0,
            max_player_distance: 64.0,
            spawn_rate: 0.5, // Try 0.5 times per second
            spawn_chance: 0.1, // 10% success rate
        }
    }

    /// Config for hostile mobs (like zombies)
    pub fn hostile() -> Self {
        Self {
            max_count: 30,
            min_light_level: 0,
            max_light_level: 7, // Only spawn in darkness
            spawn_day: false,
            spawn_night: true,
            min_player_distance: 24.0,
            max_player_distance: 64.0,
            spawn_rate: 1.0, // Try once per second
            spawn_chance: 0.2, // 20% success rate
        }
    }
}

/// Spawner that manages spawn attempts
pub struct MobSpawner {
    /// Accumulated time for spawn attempts
    spawn_timer: f32,

    /// Random number generator
    rng: rand::rngs::ThreadRng,

    /// Maximum passive mobs allowed
    max_passive: usize,

    /// Maximum hostile mobs allowed
    max_hostile: usize,
}

impl MobSpawner {
    pub fn new(max_passive: usize, max_hostile: usize) -> Self {
        Self {
            spawn_timer: 0.0,
            rng: rand::thread_rng(),
            max_passive,
            max_hostile,
        }
    }

    /// Simplified spawn method for single mob type spawning
    /// Returns a SpawnResult if a mob should be spawned this frame
    pub fn try_spawn<F>(
        &mut self,
        player_position: Vec3,
        is_night: bool,
        get_light_level: F,
    ) -> Option<SpawnResult>
    where
        F: Fn(&WorldPos) -> u8,
    {
        // Use hostile config for night, passive for day
        let config = if is_night {
            SpawnConfig::hostile()
        } else {
            SpawnConfig::passive()
        };

        // Check time of day
        if (is_night && !config.spawn_night) || (!is_night && !config.spawn_day) {
            return None;
        }

        // Simple spawn chance check (reduced frequency for single-type spawner)
        if self.rng.gen::<f32>() > 0.01 {
            return None; // 1% chance per frame to attempt spawn
        }

        // Pick random distance from player
        let distance = self.rng.gen_range(config.min_player_distance..config.max_player_distance);

        // Pick random angle
        let angle = self.rng.gen_range(0.0..std::f32::consts::TAU);

        // Calculate spawn position
        let offset = Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance);
        let spawn_pos = player_position + offset;

        // Check light level
        let block_pos = WorldPos::from_vec3(spawn_pos);
        let light_level = get_light_level(&block_pos);

        if light_level >= config.min_light_level && light_level <= config.max_light_level {
            // Choose mob type based on time of day
            let mob_type = if is_night {
                MobType::Zombie
            } else {
                MobType::Pig
            };

            Some(SpawnResult {
                mob_type,
                position: spawn_pos,
            })
        } else {
            None
        }
    }

    /// Update spawner and return spawn positions if conditions are met
    pub fn update<F, L>(
        &mut self,
        delta_time: f32,
        player_position: Vec3,
        current_mob_count: usize,
        is_night: bool,
        config: &SpawnConfig,
        get_block: F,
        get_light_level: L,
    ) -> Vec<Vec3>
    where
        F: Fn(&WorldPos) -> BlockType,
        L: Fn(&WorldPos) -> u8,
    {
        let mut spawn_positions = Vec::new();

        // Check if we've hit the mob cap
        if current_mob_count >= config.max_count {
            return spawn_positions;
        }

        // Check time of day
        if (is_night && !config.spawn_night) || (!is_night && !config.spawn_day) {
            return spawn_positions;
        }

        // Update timer
        self.spawn_timer += delta_time;

        let spawn_interval = 1.0 / config.spawn_rate;

        while self.spawn_timer >= spawn_interval {
            self.spawn_timer -= spawn_interval;

            // Try to spawn
            if self.rng.gen::<f32>() > config.spawn_chance {
                continue; // Failed spawn chance
            }

            // Pick random position around player
            if let Some(spawn_pos) = self.try_find_spawn_position(
                player_position,
                config,
                &get_block,
                &get_light_level,
            ) {
                spawn_positions.push(spawn_pos);

                // Only spawn one mob per update
                break;
            }
        }

        spawn_positions
    }

    /// Attempt to find a valid spawn position
    fn try_find_spawn_position<F, L>(
        &mut self,
        player_position: Vec3,
        config: &SpawnConfig,
        get_block: F,
        get_light_level: &L,
    ) -> Option<Vec3>
    where
        F: Fn(&WorldPos) -> BlockType,
        L: Fn(&WorldPos) -> u8,
    {
        // Pick random distance from player
        let distance = self.rng.gen_range(config.min_player_distance..config.max_player_distance);

        // Pick random angle
        let angle = self.rng.gen_range(0.0..std::f32::consts::TAU);

        // Calculate spawn position
        let offset = Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance);
        let target_pos = player_position + offset;

        // Find valid position near target
        if let Some(spawn_pos) = find_valid_spawn_position(target_pos, 8.0, &get_block) {
            // Check light level
            let block_pos = WorldPos::from_vec3(spawn_pos);
            let light_level = get_light_level(&block_pos);

            if light_level >= config.min_light_level && light_level <= config.max_light_level {
                return Some(spawn_pos);
            }
        }

        None
    }
}

impl Default for MobSpawner {
    fn default() -> Self {
        Self::new(20, 30) // Default passive and hostile limits
    }
}

/// Simple light level calculation (stub - should be replaced with proper lighting system)
/// For now, assumes:
/// - Sunlight during day: 15
/// - Night: 0
/// - Underground: 0
pub fn calculate_light_level(world_pos: &WorldPos, is_day: bool) -> u8 {
    // Check if position is above ground (sky access)
    let is_surface = world_pos.y > 60; // Approximate surface level

    if is_surface && is_day {
        15 // Full sunlight
    } else if is_surface && !is_day {
        4 // Moonlight
    } else {
        0 // Underground or night
    }
}

/// Check if a chunk position is within spawn range of player
pub fn is_chunk_in_spawn_range(chunk_pos: ChunkPos, player_pos: Vec3, max_distance: f32) -> bool {
    let origin = chunk_pos.to_world_origin();
    let chunk_center = Vec3::new(origin.x as f32, origin.y as f32, origin.z as f32) + Vec3::new(8.0, 0.0, 8.0);
    let distance = (chunk_center - player_pos).length();
    distance <= max_distance
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_world(pos: &WorldPos) -> BlockType {
        if pos.y <= 64 {
            BlockType::Stone
        } else {
            BlockType::Air
        }
    }

    fn mock_light(_pos: &WorldPos) -> u8 {
        15 // Always bright for testing
    }

    #[test]
    fn test_passive_config() {
        let config = SpawnConfig::passive();
        assert!(config.spawn_day);
        assert!(!config.spawn_night);
        assert!(config.min_light_level > 8);
    }

    #[test]
    fn test_hostile_config() {
        let config = SpawnConfig::hostile();
        assert!(!config.spawn_day);
        assert!(config.spawn_night);
        assert!(config.max_light_level < 8);
    }

    #[test]
    fn test_spawner_respects_cap() {
        let mut spawner = MobSpawner::new(20, 30);
        let config = SpawnConfig::passive();
        let player_pos = Vec3::new(0.0, 65.0, 0.0);

        // At max count, should not spawn
        let spawns = spawner.update(
            1.0,
            player_pos,
            config.max_count,
            false,
            &config,
            mock_world,
            mock_light,
        );

        assert!(spawns.is_empty());
    }

    #[test]
    fn test_spawner_respects_time() {
        let mut spawner = MobSpawner::new(20, 30);
        let config = SpawnConfig::hostile();
        let player_pos = Vec3::new(0.0, 65.0, 0.0);

        // Hostile mobs don't spawn during day
        let spawns = spawner.update(
            10.0,
            player_pos,
            0,
            false, // is_night = false
            &config,
            mock_world,
            mock_light,
        );

        assert!(spawns.is_empty());
    }

    #[test]
    fn test_light_level_calculation() {
        let surface_pos = WorldPos::new(0, 70, 0);
        let underground_pos = WorldPos::new(0, 30, 0);

        assert_eq!(calculate_light_level(&surface_pos, true), 15);
        assert_eq!(calculate_light_level(&surface_pos, false), 4);
        assert_eq!(calculate_light_level(&underground_pos, true), 0);
    }

    #[test]
    fn test_chunk_spawn_range() {
        let chunk = ChunkPos::new(0, 0);
        let player_near = Vec3::new(5.0, 64.0, 5.0);
        let player_far = Vec3::new(1000.0, 64.0, 1000.0);

        assert!(is_chunk_in_spawn_range(chunk, player_near, 100.0));
        assert!(!is_chunk_in_spawn_range(chunk, player_far, 100.0));
    }
}
