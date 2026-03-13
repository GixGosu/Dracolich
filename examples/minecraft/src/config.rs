//! Game configuration constants and settings
//!
//! This module centralizes all configurable game parameters including
//! rendering, physics, player controls, world generation, and gameplay.

// ===== Rendering Configuration =====

/// Maximum number of chunks to render in each horizontal direction from player
pub const RENDER_DISTANCE: i32 = 8;

/// Vertical field of view in degrees
pub const FOV_DEGREES: f32 = 70.0;

/// Near clipping plane distance
pub const NEAR_PLANE: f32 = 0.1;

/// Far clipping plane distance (should match render distance)
pub const FAR_PLANE: f32 = (RENDER_DISTANCE as f32 * 16.0) + 64.0;

/// Distance fog start (blocks)
pub const FOG_START: f32 = (RENDER_DISTANCE as f32 * 16.0) * 0.6;

/// Distance fog end (blocks) - should match far plane
pub const FOG_END: f32 = FAR_PLANE;

/// Target frames per second for vsync
pub const TARGET_FPS: u32 = 60;

/// Enable vsync
pub const VSYNC_ENABLED: bool = true;

// ===== Camera & Input Configuration =====

/// Mouse sensitivity for camera rotation
pub const MOUSE_SENSITIVITY: f32 = 0.002;

/// Camera pitch clamp in radians (slightly less than 90 degrees)
pub const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

/// Invert mouse Y-axis
pub const INVERT_MOUSE_Y: bool = false;

// ===== Player Physics Configuration =====

/// Player walk speed (blocks per second)
pub const WALK_SPEED: f32 = 4.3;

/// Player sprint speed (blocks per second)
pub const SPRINT_SPEED: f32 = 5.612;

/// Player jump initial velocity (blocks per second)
pub const JUMP_VELOCITY: f32 = 8.5;

/// Gravity acceleration (blocks per second squared)
pub const GRAVITY: f32 = 32.0;

/// Terminal velocity when falling (blocks per second)
pub const TERMINAL_VELOCITY: f32 = 78.4;

/// Player eye height above feet (blocks)
pub const PLAYER_EYE_HEIGHT: f32 = 1.62;

/// Player hitbox width (blocks)
pub const PLAYER_WIDTH: f32 = 0.6;

/// Player hitbox height (blocks)
pub const PLAYER_HEIGHT: f32 = 1.8;

// ===== Player Gameplay Configuration =====

/// Maximum player health
pub const MAX_HEALTH: i32 = 20;

/// Starting player health
pub const STARTING_HEALTH: i32 = 20;

/// Damage cooldown in seconds (invulnerability after taking damage)
pub const DAMAGE_COOLDOWN: f32 = 0.5;

/// Fall damage threshold (blocks fallen before taking damage)
pub const FALL_DAMAGE_THRESHOLD: f32 = 3.0;

/// Fall damage multiplier (damage per block fallen beyond threshold)
pub const FALL_DAMAGE_MULTIPLIER: f32 = 1.0;

// ===== Block Interaction Configuration =====

/// Maximum reach distance for block interaction (blocks)
pub const REACH_DISTANCE: f32 = 5.0;

/// Raycast step size for block targeting
pub const RAYCAST_STEP: f32 = 0.1;

// ===== World Generation Configuration =====

/// World seed (0 = random seed)
pub const WORLD_SEED: u32 = 12345;

/// Chunk size in blocks (X and Z dimensions)
pub const CHUNK_SIZE: i32 = 16;

/// World height in blocks
pub const WORLD_HEIGHT: i32 = 256;

/// Sea level Y coordinate
pub const SEA_LEVEL: i32 = 64;

/// Bedrock layer Y coordinate
pub const BEDROCK_LEVEL: i32 = 0;

// ===== Chunk Management Configuration =====

/// Maximum number of chunks to generate per frame
pub const MAX_CHUNKS_GENERATED_PER_FRAME: usize = 2;

/// Maximum number of chunks to mesh per frame
pub const MAX_CHUNKS_MESHED_PER_FRAME: usize = 4;

/// Distance in chunks beyond render distance before unloading
pub const UNLOAD_DISTANCE: i32 = RENDER_DISTANCE + 2;

// ===== Game Loop Configuration =====

/// Fixed physics timestep in seconds
pub const FIXED_TIMESTEP: f32 = 1.0 / 60.0;

/// Maximum accumulated time to prevent spiral of death
pub const MAX_ACCUMULATED_TIME: f32 = 0.25;

// ===== Day/Night Cycle Configuration =====

/// Length of full day/night cycle in seconds
pub const DAY_LENGTH_SECONDS: f32 = 600.0; // 10 minutes

/// Time when day starts (0.0 = midnight, 0.5 = noon)
pub const DAY_START: f32 = 0.25;

/// Time when night starts
pub const NIGHT_START: f32 = 0.75;

// ===== Audio Configuration =====

/// Master volume (0.0 to 1.0)
pub const MASTER_VOLUME: f32 = 0.7;

/// Sound effects volume (0.0 to 1.0)
pub const SFX_VOLUME: f32 = 0.8;

/// Music volume (0.0 to 1.0)
pub const MUSIC_VOLUME: f32 = 0.3;

/// Maximum distance to hear 3D sounds (blocks)
pub const SOUND_MAX_DISTANCE: f32 = 64.0;

// ===== Mob Configuration =====

/// Maximum number of passive mobs in world
pub const MAX_PASSIVE_MOBS: usize = 20;

/// Maximum number of hostile mobs in world
pub const MAX_HOSTILE_MOBS: usize = 30;

/// Mob spawn check interval in seconds
pub const MOB_SPAWN_INTERVAL: f32 = 5.0;

/// Minimum distance from player for mob spawning (blocks)
pub const MOB_SPAWN_MIN_DISTANCE: f32 = 24.0;

/// Maximum distance from player for mob spawning (blocks)
pub const MOB_SPAWN_MAX_DISTANCE: f32 = 64.0;

// ===== Inventory Configuration =====

/// Number of hotbar slots
pub const HOTBAR_SLOTS: usize = 9;

/// Total inventory slots (including hotbar)
pub const INVENTORY_SLOTS: usize = 36;

/// Maximum stack size for blocks/items
pub const MAX_STACK_SIZE: usize = 64;

// ===== UI Configuration =====

/// Crosshair size in pixels
pub const CROSSHAIR_SIZE: f32 = 20.0;

/// Crosshair thickness in pixels
pub const CROSSHAIR_THICKNESS: f32 = 2.0;

/// Hotbar slot size in pixels
pub const HOTBAR_SLOT_SIZE: f32 = 40.0;

/// Hotbar spacing between slots in pixels
pub const HOTBAR_SPACING: f32 = 4.0;

/// Heart size in pixels for health bar
pub const HEART_SIZE: f32 = 16.0;

/// Debug overlay font size
pub const DEBUG_FONT_SIZE: f32 = 16.0;

// ===== Key Bindings =====
// (Actual key codes are handled in InputState, these are just documentation)

/// Movement: WASD
/// Jump: Space
/// Sprint: Left Shift
/// Sneak: Left Control
/// Attack/Break: Left Mouse Button
/// Use/Place: Right Mouse Button
/// Pick Block: Middle Mouse Button
/// Inventory: E
/// Pause: Escape
/// Debug: F3
/// Hotbar: 1-9 keys

// ===== Performance Tuning =====

/// Enable frustum culling
pub const ENABLE_FRUSTUM_CULLING: bool = true;

/// Enable greedy meshing
pub const ENABLE_GREEDY_MESHING: bool = true;

/// Enable ambient occlusion
pub const ENABLE_AMBIENT_OCCLUSION: bool = true;

/// Enable backface culling
pub const ENABLE_BACKFACE_CULLING: bool = true;

// ===== Helper Functions =====

/// Calculate FOV in radians
pub const fn fov_radians() -> f32 {
    FOV_DEGREES * std::f32::consts::PI / 180.0
}

/// Get day time from elapsed game time
pub fn get_day_time(elapsed_seconds: f32) -> f32 {
    (elapsed_seconds % DAY_LENGTH_SECONDS) / DAY_LENGTH_SECONDS
}

/// Check if it's currently night time
pub fn is_night(day_time: f32) -> bool {
    day_time >= NIGHT_START || day_time < DAY_START
}

/// Calculate sun position angle (0 = noon, PI = midnight)
pub fn sun_angle(day_time: f32) -> f32 {
    (day_time * 2.0 * std::f32::consts::PI) - std::f32::consts::FRAC_PI_2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_time_calculation() {
        assert_eq!(get_day_time(0.0), 0.0);
        assert_eq!(get_day_time(DAY_LENGTH_SECONDS), 0.0);
        assert_eq!(get_day_time(DAY_LENGTH_SECONDS / 2.0), 0.5);
    }

    #[test]
    fn test_night_detection() {
        assert!(is_night(0.0)); // Midnight
        assert!(!is_night(0.5)); // Noon
        assert!(is_night(0.8)); // Evening
    }

    #[test]
    fn test_render_distance_reasonable() {
        assert!(RENDER_DISTANCE >= 4);
        assert!(RENDER_DISTANCE <= 32);
    }

    #[test]
    fn test_physics_constants_reasonable() {
        assert!(WALK_SPEED > 0.0);
        assert!(SPRINT_SPEED > WALK_SPEED);
        assert!(GRAVITY > 0.0);
        assert!(JUMP_VELOCITY > 0.0);
    }
}
