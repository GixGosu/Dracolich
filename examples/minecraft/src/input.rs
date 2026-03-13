use std::collections::HashSet;
use winit::event::{ElementState, KeyEvent, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};
use glam::Vec3;

/// Tracks the current state of all input devices
pub struct InputState {
    // Keyboard state
    keys_pressed: HashSet<KeyCode>,
    keys_just_pressed: HashSet<KeyCode>,
    keys_just_released: HashSet<KeyCode>,

    // Mouse state
    mouse_buttons: HashSet<MouseButton>,
    mouse_buttons_just_pressed: HashSet<MouseButton>,
    mouse_buttons_just_released: HashSet<MouseButton>,
    mouse_position: (f64, f64),
    mouse_delta: (f64, f64),

    // Mouse capture state
    cursor_grabbed: bool,
    last_cursor_pos: Option<(f64, f64)>,
}

impl InputState {
    /// Create a new input state tracker
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            keys_just_pressed: HashSet::new(),
            keys_just_released: HashSet::new(),
            mouse_buttons: HashSet::new(),
            mouse_buttons_just_pressed: HashSet::new(),
            mouse_buttons_just_released: HashSet::new(),
            mouse_position: (0.0, 0.0),
            mouse_delta: (0.0, 0.0),
            cursor_grabbed: false,
            last_cursor_pos: None,
        }
    }

    /// Call this at the start of each frame to clear per-frame state
    pub fn begin_frame(&mut self) {
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.mouse_buttons_just_pressed.clear();
        self.mouse_buttons_just_released.clear();
        self.mouse_delta = (0.0, 0.0);
    }

    /// Handle keyboard input event
    pub fn handle_keyboard(&mut self, event: KeyEvent) {
        if let PhysicalKey::Code(keycode) = event.physical_key {
            match event.state {
                ElementState::Pressed => {
                    if !self.keys_pressed.contains(&keycode) {
                        self.keys_just_pressed.insert(keycode);
                    }
                    self.keys_pressed.insert(keycode);
                }
                ElementState::Released => {
                    self.keys_pressed.remove(&keycode);
                    self.keys_just_released.insert(keycode);
                }
            }
        }
    }

    /// Handle mouse button event
    pub fn handle_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => {
                if !self.mouse_buttons.contains(&button) {
                    self.mouse_buttons_just_pressed.insert(button);
                }
                self.mouse_buttons.insert(button);
            }
            ElementState::Released => {
                self.mouse_buttons.remove(&button);
                self.mouse_buttons_just_released.insert(button);
            }
        }
    }

    /// Handle cursor movement event (absolute position)
    pub fn handle_cursor_moved(&mut self, position: (f64, f64)) {
        self.mouse_position = position;
        // Note: Don't calculate delta from positions - use handle_raw_mouse_motion instead
        // Position-based delta doesn't work well with cursor grab on Windows
    }

    /// Handle raw mouse motion (relative delta)
    /// This is the preferred method for FPS camera as it works regardless of cursor position
    pub fn handle_raw_mouse_motion(&mut self, delta: (f64, f64)) {
        if self.cursor_grabbed {
            // Accumulate delta (multiple events may arrive per frame)
            self.mouse_delta.0 += delta.0;
            self.mouse_delta.1 += delta.1;
        }
    }

    /// Set cursor grab state
    pub fn set_cursor_grabbed(&mut self, grabbed: bool) {
        // Only clear delta when grab state actually changes
        if self.cursor_grabbed != grabbed {
            self.cursor_grabbed = grabbed;
            // Clear delta when grab state changes to prevent jumps
            self.mouse_delta = (0.0, 0.0);
        }
    }

    // ========== Keyboard query methods ==========

    /// Check if a key is currently held down
    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    /// Check if a key was just pressed this frame
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_just_pressed.contains(&key)
    }

    /// Check if a key was just released this frame
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.keys_just_released.contains(&key)
    }

    // ========== Mouse button query methods ==========

    /// Check if a mouse button is currently held down
    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons.contains(&button)
    }

    /// Check if a mouse button was just pressed this frame
    pub fn is_mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_just_pressed.contains(&button)
    }

    /// Check if a mouse button was just released this frame
    pub fn is_mouse_just_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons_just_released.contains(&button)
    }

    // ========== Mouse position/delta methods ==========

    /// Get current mouse position
    pub fn mouse_position(&self) -> (f64, f64) {
        self.mouse_position
    }

    /// Get mouse movement delta for this frame
    pub fn mouse_delta(&self) -> (f64, f64) {
        self.mouse_delta
    }

    /// Check if cursor is currently grabbed
    pub fn is_cursor_grabbed(&self) -> bool {
        self.cursor_grabbed
    }

    // ========== High-level input queries for game controls ==========

    /// Get movement vector from WASD keys (normalized)
    /// Returns Vec3 with x = strafe (left/right), y = 0, z = forward/back
    pub fn movement_input(&self) -> Vec3 {
        let mut x: f32 = 0.0;
        let mut z: f32 = 0.0;

        if self.is_key_down(KeyCode::KeyW) {
            z += 1.0;  // Forward is positive Z in player space
        }
        if self.is_key_down(KeyCode::KeyS) {
            z -= 1.0;  // Backward is negative Z
        }
        if self.is_key_down(KeyCode::KeyA) {
            x -= 1.0;  // Left is negative X
        }
        if self.is_key_down(KeyCode::KeyD) {
            x += 1.0;  // Right is positive X
        }

        // Normalize diagonal movement
        let length = (x * x + z * z).sqrt();
        if length > 0.0 {
            x /= length;
            z /= length;
        }

        Vec3::new(x, 0.0, z)
    }

    /// Check if jump key is pressed
    pub fn is_jump(&self) -> bool {
        self.is_key_down(KeyCode::Space)
    }

    /// Check if jump was just pressed this frame
    pub fn is_jump_just_pressed(&self) -> bool {
        self.is_key_just_pressed(KeyCode::Space)
    }

    /// Check if sprint/fly-down key is pressed
    pub fn is_sprint(&self) -> bool {
        self.is_key_down(KeyCode::ShiftLeft) || self.is_key_down(KeyCode::ShiftRight)
    }

    /// Check if inventory key was just pressed
    pub fn is_inventory_just_pressed(&self) -> bool {
        self.is_key_just_pressed(KeyCode::KeyE)
    }

    /// Check if pause/menu key was just pressed
    pub fn is_pause_just_pressed(&self) -> bool {
        self.is_key_just_pressed(KeyCode::Escape)
    }

    /// Check if debug overlay toggle was pressed
    pub fn is_debug_just_pressed(&self) -> bool {
        self.is_key_just_pressed(KeyCode::F3)
    }

    /// Get hotbar slot selection (1-9 keys, returns 0-8 or None)
    pub fn hotbar_selection(&self) -> Option<usize> {
        for (i, key) in [
            KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3,
            KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6,
            KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9,
        ].iter().enumerate() {
            if self.is_key_just_pressed(*key) {
                return Some(i);
            }
        }
        None
    }

    /// Get mouse look delta (scaled for camera sensitivity)
    pub fn mouse_look_delta(&self, sensitivity: f32) -> (f32, f32) {
        if !self.cursor_grabbed {
            return (0.0, 0.0);
        }

        let (dx, dy) = self.mouse_delta;
        (
            dx as f32 * sensitivity,
            dy as f32 * sensitivity,
        )
    }

    /// Check if left mouse (attack/break) is down
    pub fn is_attack(&self) -> bool {
        self.is_mouse_down(MouseButton::Left)
    }

    /// Check if left mouse was just pressed
    pub fn is_attack_just_pressed(&self) -> bool {
        self.is_mouse_just_pressed(MouseButton::Left)
    }

    /// Check if right mouse (use/place) was just pressed
    pub fn is_use_just_pressed(&self) -> bool {
        self.is_mouse_just_pressed(MouseButton::Right)
    }

    /// Check if middle mouse (pick block) was just pressed
    pub fn is_pick_block_just_pressed(&self) -> bool {
        self.is_mouse_just_pressed(MouseButton::Middle)
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}
