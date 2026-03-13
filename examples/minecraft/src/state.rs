//! Game state management
//!
//! This module defines the various states the game can be in and manages
//! transitions between them. Each state affects which systems are active
//! and how input is processed.

/// Represents the current state of the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    /// Normal gameplay - player can move, interact, etc.
    Playing,

    /// Game is paused - ESC menu is shown, gameplay frozen
    Paused,

    /// Inventory screen is open - movement disabled, inventory UI shown
    Inventory,

    /// Player has died - waiting for respawn
    Dead,

    /// Loading screen (chunks generating, resources loading)
    Loading,
}

impl GameState {
    /// Check if player input should be processed
    pub fn accepts_player_input(&self) -> bool {
        matches!(self, GameState::Playing)
    }

    /// Check if the world should be updated
    pub fn updates_world(&self) -> bool {
        matches!(self, GameState::Playing)
    }

    /// Check if physics should be simulated
    pub fn updates_physics(&self) -> bool {
        matches!(self, GameState::Playing)
    }

    /// Check if mobs should be updated
    pub fn updates_mobs(&self) -> bool {
        matches!(self, GameState::Playing)
    }

    /// Check if cursor should be grabbed (FPS mode)
    pub fn grabs_cursor(&self) -> bool {
        matches!(self, GameState::Playing)
    }

    /// Check if UI overlays should be rendered
    pub fn shows_ui_overlay(&self) -> bool {
        !matches!(self, GameState::Playing)
    }

    /// Check if the game is in a menu state
    pub fn is_menu(&self) -> bool {
        matches!(self, GameState::Paused | GameState::Inventory | GameState::Dead)
    }
}

/// Manages game state transitions
pub struct StateManager {
    current: GameState,
    previous: GameState,
}

impl StateManager {
    /// Create a new state manager starting in Loading state
    pub fn new() -> Self {
        Self {
            current: GameState::Loading,
            previous: GameState::Loading,
        }
    }

    /// Get the current state
    pub fn current(&self) -> GameState {
        self.current
    }

    /// Get the previous state
    pub fn previous(&self) -> GameState {
        self.previous
    }

    /// Transition to a new state
    pub fn transition_to(&mut self, new_state: GameState) {
        if self.current != new_state {
            self.previous = self.current;
            self.current = new_state;
        }
    }

    /// Toggle between Playing and Paused states
    pub fn toggle_pause(&mut self) {
        match self.current {
            GameState::Playing => self.transition_to(GameState::Paused),
            GameState::Paused => self.transition_to(GameState::Playing),
            _ => {} // Can't pause from other states
        }
    }

    /// Toggle inventory screen
    pub fn toggle_inventory(&mut self) {
        match self.current {
            GameState::Playing => self.transition_to(GameState::Inventory),
            GameState::Inventory => self.transition_to(GameState::Playing),
            _ => {} // Can't open inventory from other states
        }
    }

    /// Handle player death
    pub fn player_died(&mut self) {
        self.transition_to(GameState::Dead);
    }

    /// Respawn player (from Dead state back to Playing)
    pub fn respawn(&mut self) {
        if self.current == GameState::Dead {
            self.transition_to(GameState::Playing);
        }
    }

    /// Mark loading as complete and start gameplay
    pub fn finish_loading(&mut self) {
        if self.current == GameState::Loading {
            self.transition_to(GameState::Playing);
        }
    }

    /// Check if state changed on the last transition
    pub fn state_changed(&self) -> bool {
        self.current != self.previous
    }

    /// Check if we just entered the current state
    pub fn just_entered(&self, state: GameState) -> bool {
        self.current == state && self.previous != state
    }

    /// Check if we just exited a state
    pub fn just_exited(&self, state: GameState) -> bool {
        self.previous == state && self.current != state
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Input state requirements for different game states
pub struct StateInputRequirements {
    /// Cursor should be grabbed
    pub grab_cursor: bool,

    /// Cursor should be visible
    pub show_cursor: bool,

    /// Process movement input
    pub process_movement: bool,

    /// Process mouse look
    pub process_mouse_look: bool,

    /// Process action buttons (attack, use, etc.)
    pub process_actions: bool,

    /// Process UI interactions (clicks, etc.)
    pub process_ui_input: bool,
}

impl StateInputRequirements {
    /// Get input requirements for a given state
    pub fn for_state(state: GameState) -> Self {
        match state {
            GameState::Playing => Self {
                grab_cursor: true,
                show_cursor: false,
                process_movement: true,
                process_mouse_look: true,
                process_actions: true,
                process_ui_input: false,
            },
            GameState::Paused => Self {
                grab_cursor: false,
                show_cursor: true,
                process_movement: false,
                process_mouse_look: false,
                process_actions: false,
                process_ui_input: true,
            },
            GameState::Inventory => Self {
                grab_cursor: false,
                show_cursor: true,
                process_movement: false,
                process_mouse_look: false,
                process_actions: false,
                process_ui_input: true,
            },
            GameState::Dead => Self {
                grab_cursor: false,
                show_cursor: true,
                process_movement: false,
                process_mouse_look: false,
                process_actions: false,
                process_ui_input: true,
            },
            GameState::Loading => Self {
                grab_cursor: false,
                show_cursor: true,
                process_movement: false,
                process_mouse_look: false,
                process_actions: false,
                process_ui_input: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_manager_creation() {
        let manager = StateManager::new();
        assert_eq!(manager.current(), GameState::Loading);
    }

    #[test]
    fn test_state_transitions() {
        let mut manager = StateManager::new();

        manager.transition_to(GameState::Playing);
        assert_eq!(manager.current(), GameState::Playing);
        assert_eq!(manager.previous(), GameState::Loading);
    }

    #[test]
    fn test_toggle_pause() {
        let mut manager = StateManager::new();
        manager.transition_to(GameState::Playing);

        manager.toggle_pause();
        assert_eq!(manager.current(), GameState::Paused);

        manager.toggle_pause();
        assert_eq!(manager.current(), GameState::Playing);
    }

    #[test]
    fn test_toggle_inventory() {
        let mut manager = StateManager::new();
        manager.transition_to(GameState::Playing);

        manager.toggle_inventory();
        assert_eq!(manager.current(), GameState::Inventory);

        manager.toggle_inventory();
        assert_eq!(manager.current(), GameState::Playing);
    }

    #[test]
    fn test_death_and_respawn() {
        let mut manager = StateManager::new();
        manager.transition_to(GameState::Playing);

        manager.player_died();
        assert_eq!(manager.current(), GameState::Dead);

        manager.respawn();
        assert_eq!(manager.current(), GameState::Playing);
    }

    #[test]
    fn test_state_checks() {
        assert!(GameState::Playing.accepts_player_input());
        assert!(!GameState::Paused.accepts_player_input());

        assert!(GameState::Playing.grabs_cursor());
        assert!(!GameState::Inventory.grabs_cursor());
    }

    #[test]
    fn test_just_entered() {
        let mut manager = StateManager::new();

        manager.transition_to(GameState::Playing);
        assert!(manager.just_entered(GameState::Playing));

        manager.transition_to(GameState::Paused);
        assert!(manager.just_entered(GameState::Paused));
        assert!(!manager.just_entered(GameState::Playing));
    }

    #[test]
    fn test_input_requirements() {
        let playing = StateInputRequirements::for_state(GameState::Playing);
        assert!(playing.grab_cursor);
        assert!(playing.process_movement);

        let paused = StateInputRequirements::for_state(GameState::Paused);
        assert!(!paused.grab_cursor);
        assert!(!paused.process_movement);
        assert!(paused.process_ui_input);
    }
}
