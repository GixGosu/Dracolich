// Player block interaction
// Handles block breaking (with progress tracking), block placement, and raycasting

use crate::types::{BlockType, WorldPos};
use crate::physics::raycast::{raycast, RaycastHit, MAX_RAYCAST_DISTANCE};
use glam::Vec3;
use std::time::{Duration, Instant};

/// Tool effectiveness multipliers
pub const BARE_HANDS_MULTIPLIER: f32 = 1.0;
pub const CORRECT_TOOL_MULTIPLIER: f32 = 4.0;

/// Block interaction state
pub struct InteractionState {
    /// Currently targeted block (if any)
    pub targeted_block: Option<RaycastHit>,

    /// Block currently being broken
    pub breaking_block: Option<WorldPos>,

    /// Progress on breaking the current block (0.0 to 1.0)
    pub break_progress: f32,

    /// Time when breaking started
    break_start_time: Option<Instant>,

    /// Expected time to break current block
    break_duration: Duration,
}

impl InteractionState {
    pub fn new() -> Self {
        Self {
            targeted_block: None,
            breaking_block: None,
            break_progress: 0.0,
            break_start_time: None,
            break_duration: Duration::ZERO,
        }
    }

    /// Update the targeted block via raycasting
    pub fn update_targeting<F>(&mut self, eye_position: Vec3, view_direction: Vec3, get_block: F)
    where
        F: Fn(&WorldPos) -> BlockType,
    {
        self.targeted_block = raycast(
            eye_position,
            view_direction,
            MAX_RAYCAST_DISTANCE,
            get_block,
        );
    }

    /// Start breaking a block
    /// Returns true if breaking started successfully
    pub fn start_breaking(&mut self, target: &RaycastHit, block_type: BlockType, tool_multiplier: f32) -> bool {
        if !block_type.is_breakable() {
            return false;
        }

        let base_hardness = block_type.hardness();
        let break_time = base_hardness / tool_multiplier;

        self.breaking_block = Some(target.block_pos);
        self.break_progress = 0.0;
        self.break_start_time = Some(Instant::now());
        self.break_duration = Duration::from_secs_f32(break_time);

        true
    }

    /// Update breaking progress
    /// Returns true if block should be destroyed this frame
    pub fn update_breaking(&mut self) -> bool {
        if let Some(start_time) = self.break_start_time {
            let elapsed = start_time.elapsed();

            if elapsed >= self.break_duration {
                // Block broken!
                self.cancel_breaking();
                return true;
            } else {
                // Update progress
                self.break_progress = elapsed.as_secs_f32() / self.break_duration.as_secs_f32();
                self.break_progress = self.break_progress.clamp(0.0, 1.0);
            }
        }

        false
    }

    /// Cancel breaking the current block
    pub fn cancel_breaking(&mut self) {
        self.breaking_block = None;
        self.break_progress = 0.0;
        self.break_start_time = None;
        self.break_duration = Duration::ZERO;
    }

    /// Check if currently breaking a block
    pub fn is_breaking(&self) -> bool {
        self.breaking_block.is_some()
    }

    /// Get break progress as 0-10 integer (for rendering crack stages)
    pub fn break_stage(&self) -> u8 {
        (self.break_progress * 10.0).floor() as u8
    }
}

impl Default for InteractionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Block breaking and placement logic
impl crate::player::Player {
    /// Update block targeting (call each frame)
    pub fn update_block_targeting<F>(&mut self, get_block: F)
    where
        F: Fn(&WorldPos) -> BlockType,
    {
        let eye_pos = self.eye_position();
        let view_dir = self.view_direction();
        self.interaction.update_targeting(eye_pos, view_dir, get_block);
    }

    /// Attempt to break the currently targeted block
    /// Returns the block position if it should be destroyed
    /// is_attacking indicates if left mouse button is held
    pub fn update_block_breaking<F>(&mut self, is_attacking: bool, get_block: F) -> Option<WorldPos>
    where
        F: Fn(&WorldPos) -> BlockType,
    {
        if !is_attacking {
            // Not attacking - cancel any in-progress breaking
            if self.interaction.is_breaking() {
                self.interaction.cancel_breaking();
            }
            return None;
        }

        // Check if we have a valid target
        let target = match &self.interaction.targeted_block {
            Some(hit) => *hit,
            None => {
                self.interaction.cancel_breaking();
                return None;
            }
        };

        // Get block type
        let block_type = get_block(&target.block_pos);
        if block_type == BlockType::Air || !block_type.is_breakable() {
            self.interaction.cancel_breaking();
            return None;
        }

        // Check if we're starting to break a new block
        if self.interaction.breaking_block != Some(target.block_pos) {
            // TODO: Get actual tool multiplier from equipped item
            let tool_multiplier = BARE_HANDS_MULTIPLIER;

            if !self.interaction.start_breaking(&target, block_type, tool_multiplier) {
                return None;
            }
        }

        // Update breaking progress
        if self.interaction.update_breaking() {
            // Block broken!
            return Some(target.block_pos);
        }

        None
    }

    /// Attempt to place a block
    /// Returns the position where the block was placed, or None if placement failed
    pub fn try_place_block<F>(&mut self, get_block: F) -> Option<WorldPos>
    where
        F: Fn(&WorldPos) -> BlockType,
    {
        // Check if we have a selected block to place
        let block_to_place = match self.selected_block() {
            Some(block) => block,
            None => return None,
        };

        // Check if we have a valid target
        let target = match &self.interaction.targeted_block {
            Some(hit) => hit,
            None => return None,
        };

        // Get the adjacent position (where we want to place the block)
        let place_pos = target.adjacent_pos;

        // Check if the position is air
        if get_block(&place_pos) != BlockType::Air {
            return None;
        }

        // Check if placing the block would intersect with player
        if self.would_collide_with_block(&place_pos) {
            return None;
        }

        // Valid placement!
        Some(place_pos)
    }

    /// Check if placing a block at the given position would collide with the player
    fn would_collide_with_block(&self, block_pos: &WorldPos) -> bool {
        let player_aabb = self.aabb();
        let block_aabb = crate::types::AABB::from_block(block_pos);
        player_aabb.intersects(&block_aabb)
    }

    /// Get the currently targeted block position (for rendering highlight)
    pub fn get_targeted_block(&self) -> Option<WorldPos> {
        self.interaction.targeted_block.as_ref().map(|hit| hit.block_pos)
    }

    /// Get break progress for rendering (0.0 to 1.0)
    pub fn get_break_progress(&self) -> f32 {
        self.interaction.break_progress
    }

    /// Get break stage for rendering (0 to 10)
    pub fn get_break_stage(&self) -> u8 {
        self.interaction.break_stage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Player;
    use std::thread::sleep;

    fn test_world(pos: &WorldPos) -> BlockType {
        // Simple test world: ground at y=0, stone block at (5, 1, 0)
        if pos.y == 0 {
            BlockType::Grass
        } else if pos.x == 5 && pos.y == 1 && pos.z == 0 {
            BlockType::Stone
        } else {
            BlockType::Air
        }
    }

    #[test]
    fn test_interaction_state_creation() {
        let state = InteractionState::new();
        assert!(state.targeted_block.is_none());
        assert!(state.breaking_block.is_none());
        assert_eq!(state.break_progress, 0.0);
        assert!(!state.is_breaking());
    }

    #[test]
    fn test_breaking_progress() {
        let mut state = InteractionState::new();

        // Create a fake raycast hit
        let hit = RaycastHit::new(
            WorldPos::new(0, 0, 0),
            WorldPos::new(0, 1, 0),
            crate::types::Direction::Up,
            1.0,
            Vec3::ZERO,
        );

        // Start breaking dirt (0.5 seconds)
        state.start_breaking(&hit, BlockType::Dirt, BARE_HANDS_MULTIPLIER);
        assert!(state.is_breaking());
        assert_eq!(state.break_progress, 0.0);

        // Wait a bit
        sleep(Duration::from_millis(250));
        state.update_breaking();

        // Should be around 50% complete
        assert!(state.break_progress > 0.4 && state.break_progress < 0.6);

        // Wait for completion
        sleep(Duration::from_millis(300));
        let broken = state.update_breaking();

        assert!(broken);
        assert!(!state.is_breaking());
    }

    #[test]
    fn test_cancel_breaking() {
        let mut state = InteractionState::new();

        let hit = RaycastHit::new(
            WorldPos::new(0, 0, 0),
            WorldPos::new(0, 1, 0),
            crate::types::Direction::Up,
            1.0,
            Vec3::ZERO,
        );

        state.start_breaking(&hit, BlockType::Stone, BARE_HANDS_MULTIPLIER);
        assert!(state.is_breaking());

        state.cancel_breaking();
        assert!(!state.is_breaking());
        assert_eq!(state.break_progress, 0.0);
    }

    #[test]
    fn test_break_stage() {
        let mut state = InteractionState::new();
        assert_eq!(state.break_stage(), 0);

        state.break_progress = 0.15;
        assert_eq!(state.break_stage(), 1);

        state.break_progress = 0.55;
        assert_eq!(state.break_stage(), 5);

        state.break_progress = 0.99;
        assert_eq!(state.break_stage(), 9);
    }

    #[test]
    fn test_targeting() {
        let mut state = InteractionState::new();

        // Look at stone block at (5, 1, 0) from position (0, 1.62, 0)
        let eye_pos = Vec3::new(0.0, 1.62, 0.0);
        let view_dir = Vec3::new(1.0, 0.0, 0.0); // Look east

        state.update_targeting(eye_pos, view_dir, test_world);

        // Should hit the stone block
        assert!(state.targeted_block.is_some());
        if let Some(hit) = &state.targeted_block {
            assert_eq!(hit.block_pos, WorldPos::new(5, 1, 0));
        }
    }

    #[test]
    fn test_unbreakable_blocks() {
        let mut state = InteractionState::new();

        let hit = RaycastHit::new(
            WorldPos::new(0, 0, 0),
            WorldPos::new(0, 1, 0),
            crate::types::Direction::Up,
            1.0,
            Vec3::ZERO,
        );

        // Try to break bedrock
        let started = state.start_breaking(&hit, BlockType::Bedrock, BARE_HANDS_MULTIPLIER);
        assert!(!started);
        assert!(!state.is_breaking());
    }

    #[test]
    fn test_player_block_collision() {
        let player = Player::new(Vec3::new(0.0, 1.0, 0.0));

        // Block at player's feet should collide
        assert!(player.would_collide_with_block(&WorldPos::new(0, 0, 0)));

        // Block at player's head should collide
        assert!(player.would_collide_with_block(&WorldPos::new(0, 1, 0)));

        // Block far away should not collide
        assert!(!player.would_collide_with_block(&WorldPos::new(10, 1, 0)));
    }

    #[test]
    fn test_tool_multiplier() {
        let mut state = InteractionState::new();

        let hit = RaycastHit::new(
            WorldPos::new(0, 0, 0),
            WorldPos::new(0, 1, 0),
            crate::types::Direction::Up,
            1.0,
            Vec3::ZERO,
        );

        // Stone takes 4 seconds with bare hands
        state.start_breaking(&hit, BlockType::Stone, BARE_HANDS_MULTIPLIER);
        let bare_hands_duration = state.break_duration;

        // With correct tool, should be 4x faster
        state.start_breaking(&hit, BlockType::Stone, CORRECT_TOOL_MULTIPLIER);
        let correct_tool_duration = state.break_duration;

        assert!(correct_tool_duration < bare_hands_duration);
        assert!((bare_hands_duration.as_secs_f32() / correct_tool_duration.as_secs_f32() - 4.0).abs() < 0.1);
    }
}
