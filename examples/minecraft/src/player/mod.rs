// Player controller and interaction systems
// Manages player state, movement, health, and block interactions

pub mod movement;
pub mod health;
pub mod interaction;
pub mod hotbar;

pub use movement::*;
pub use health::*;
pub use interaction::*;
pub use hotbar::*;

use glam::Vec3;
use crate::types::{BlockType, WorldPos, AABB};

/// Player constants
pub const PLAYER_WIDTH: f32 = 0.6;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_EYE_HEIGHT: f32 = 1.62;
pub const PLAYER_HITBOX: Vec3 = Vec3::new(PLAYER_WIDTH, PLAYER_HEIGHT, PLAYER_WIDTH);

/// The player entity
pub struct Player {
    // Core state
    pub position: Vec3,         // Player center position (feet)
    pub velocity: Vec3,
    pub on_ground: bool,

    // Camera/orientation
    pub pitch: f32,             // Rotation around X axis (looking up/down) in radians
    pub yaw: f32,               // Rotation around Y axis (looking left/right) in radians

    // Health system
    pub health: Health,

    // Inventory
    pub hotbar: Hotbar,
    pub selected_slot: usize,   // Currently selected hotbar slot (0-8)

    // Interaction state
    pub interaction: InteractionState,

    // Movement state
    pub is_sprinting: bool,
    pub fall_distance: f32,     // Distance fallen for calculating fall damage
    last_ground_y: f32,         // Last Y position when on ground
}

impl Player {
    /// Create a new player at the given spawn position
    pub fn new(spawn_position: Vec3) -> Self {
        Self {
            position: spawn_position,
            velocity: Vec3::ZERO,
            on_ground: false,
            pitch: 0.0,
            yaw: 0.0,
            health: Health::new(20.0),
            hotbar: Hotbar::new(),
            selected_slot: 0,
            interaction: InteractionState::new(),
            is_sprinting: false,
            fall_distance: 0.0,
            last_ground_y: spawn_position.y,
        }
    }

    /// Get the player's eye position (for camera and raycasting)
    pub fn eye_position(&self) -> Vec3 {
        self.position + Vec3::new(0.0, PLAYER_EYE_HEIGHT, 0.0)
    }

    /// Get the player's forward direction vector (horizontal plane only)
    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.sin(),
            0.0,
            -self.yaw.cos(),
        ).normalize()
    }

    /// Get the player's right direction vector (horizontal plane only)
    pub fn right(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos(),
            0.0,
            self.yaw.sin(),
        ).normalize()
    }

    /// Get the player's view direction vector (includes pitch)
    pub fn view_direction(&self) -> Vec3 {
        Vec3::new(
            self.pitch.cos() * self.yaw.sin(),
            self.pitch.sin(),
            -self.pitch.cos() * self.yaw.cos(),
        ).normalize()
    }

    /// Get the player's AABB for collision detection
    /// Player position is stored at feet level, so add half height to get center
    pub fn aabb(&self) -> AABB {
        let center = self.position + Vec3::new(0.0, PLAYER_HEIGHT * 0.5, 0.0);
        AABB::from_center_size(center, PLAYER_HITBOX)
    }

    /// Check if the player is dead
    pub fn is_dead(&self) -> bool {
        self.health.is_dead()
    }

    /// Respawn the player at the given position
    pub fn respawn(&mut self, spawn_position: Vec3) {
        self.position = spawn_position;
        self.velocity = Vec3::ZERO;
        self.health.respawn();
        self.fall_distance = 0.0;
        self.last_ground_y = spawn_position.y;
        self.on_ground = false;
    }

    /// Update fall distance tracking
    pub fn update_fall_tracking(&mut self) {
        if self.on_ground {
            // Just landed - calculate fall damage if applicable
            if self.fall_distance > 3.0 {
                let damage = (self.fall_distance - 3.0).floor();
                self.health.damage(damage);
            }
            self.fall_distance = 0.0;
            self.last_ground_y = self.position.y;
        } else if self.position.y < self.last_ground_y {
            // Falling - accumulate distance
            self.fall_distance = (self.last_ground_y - self.position.y).max(self.fall_distance);
        } else {
            // Moving upward or same level - reset fall distance
            self.fall_distance = 0.0;
            self.last_ground_y = self.position.y;
        }
    }

    /// Get the currently selected block type from hotbar
    pub fn selected_block(&self) -> Option<BlockType> {
        self.hotbar.get_block(self.selected_slot)
    }

    /// Select a hotbar slot (0-8)
    pub fn select_slot(&mut self, slot: usize) {
        if slot < 9 {
            self.selected_slot = slot;
        }
    }

    /// Scroll hotbar selection (delta is typically -1 or +1)
    pub fn scroll_hotbar(&mut self, delta: i32) {
        let new_slot = (self.selected_slot as i32 + delta).rem_euclid(9) as usize;
        self.selected_slot = new_slot;
    }
}

// Standalone helper functions for game loop

/// Update player camera rotation from mouse input
pub fn update_camera(player: &mut Player, mouse_delta: (f32, f32)) {
    player.update_look((mouse_delta.0 as f64, mouse_delta.1 as f64));
}

/// Update player movement, physics, and collisions
pub fn update_movement<F>(
    player: &mut Player,
    movement_input: Vec3,
    is_sprinting: bool,
    jump_pressed: bool,
    delta_time: f32,
    get_block: F,
) where
    F: Fn(&WorldPos) -> BlockType,
{
    // Apply movement input
    player.apply_movement_input(movement_input, is_sprinting);

    // Handle jumping
    if jump_pressed {
        player.jump();
    }

    // Update physics and collision
    player.update_physics(delta_time, get_block);
}

/// Update player block interactions (breaking and placing)
pub fn update_interaction<F, S>(
    player: &mut Player,
    is_attacking: bool,
    is_placing: bool,
    _delta_time: f32,
    get_block: F,
    mut set_block: S,
) where
    F: Fn(&WorldPos) -> BlockType,
    S: FnMut(&WorldPos, BlockType),
{
    // Update targeting raycast
    player.update_block_targeting(&get_block);

    // Handle block breaking
    if let Some(broken_pos) = player.update_block_breaking(is_attacking, &get_block) {
        set_block(&broken_pos, BlockType::Air);
    }

    // Handle block placement
    if is_placing {
        if let Some(place_pos) = player.try_place_block(&get_block) {
            if let Some(block_to_place) = player.selected_block() {
                set_block(&place_pos, block_to_place);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = Player::new(Vec3::new(0.0, 100.0, 0.0));
        assert_eq!(player.position, Vec3::new(0.0, 100.0, 0.0));
        assert_eq!(player.velocity, Vec3::ZERO);
        assert!(!player.on_ground);
        assert_eq!(player.health.current(), 20.0);
        assert_eq!(player.selected_slot, 0);
    }

    #[test]
    fn test_eye_position() {
        let player = Player::new(Vec3::new(0.0, 100.0, 0.0));
        assert_eq!(player.eye_position(), Vec3::new(0.0, 100.0 + PLAYER_EYE_HEIGHT, 0.0));
    }

    #[test]
    fn test_view_direction() {
        let mut player = Player::new(Vec3::ZERO);

        // Facing north (negative Z)
        player.yaw = 0.0;
        player.pitch = 0.0;
        let dir = player.view_direction();
        assert!((dir.x - 0.0).abs() < 0.01);
        assert!((dir.y - 0.0).abs() < 0.01);
        assert!((dir.z - (-1.0)).abs() < 0.01);
    }

    #[test]
    fn test_hotbar_selection() {
        let mut player = Player::new(Vec3::ZERO);
        player.select_slot(5);
        assert_eq!(player.selected_slot, 5);

        player.scroll_hotbar(1);
        assert_eq!(player.selected_slot, 6);

        player.scroll_hotbar(-2);
        assert_eq!(player.selected_slot, 4);

        // Test wrapping
        player.select_slot(8);
        player.scroll_hotbar(1);
        assert_eq!(player.selected_slot, 0);

        player.scroll_hotbar(-1);
        assert_eq!(player.selected_slot, 8);
    }

    #[test]
    fn test_fall_damage() {
        let mut player = Player::new(Vec3::new(0.0, 100.0, 0.0));
        player.on_ground = true;
        player.update_fall_tracking();

        // Simulate falling 5 blocks
        player.on_ground = false;
        player.position.y = 95.0;
        player.update_fall_tracking();
        assert!((player.fall_distance - 5.0).abs() < 0.01);

        // Land - should take 2 damage (5 - 3 = 2)
        player.on_ground = true;
        let initial_health = player.health.current();
        player.update_fall_tracking();
        assert_eq!(player.health.current(), initial_health - 2.0);
        assert_eq!(player.fall_distance, 0.0);
    }
}
