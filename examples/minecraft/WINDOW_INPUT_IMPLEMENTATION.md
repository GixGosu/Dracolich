# Window and Input Implementation

## Overview

Implemented complete windowing, input handling, and game loop infrastructure for the Minecraft clone. All code is written and ready for compilation once Rust toolchain is installed.

## Files Created

### 1. `src/window.rs` (181 lines)

**Purpose:** OpenGL window creation and context management using glutin + winit

**Key Features:**
- OpenGL 3.3+ context creation via glutin
- VSync enabled by default (configurable swap interval)
- Window resize handling with automatic viewport adjustment
- Cursor grab/release for FPS mouse look
- Cursor visibility control
- Cursor centering utility
- Depth testing, backface culling pre-configured
- Clear color set to sky blue (0.53, 0.81, 0.92)

**API:**
```rust
Window::new(event_loop, title, width, height) -> Window
window.swap_buffers()
window.resize(width, height)
window.dimensions() -> (u32, u32)
window.aspect_ratio() -> f32
window.set_cursor_grab(bool)
window.set_cursor_visible(bool)
window.center_cursor()
```

### 2. `src/input.rs` (270 lines)

**Purpose:** Comprehensive input state tracking for keyboard, mouse, and game controls

**Key Features:**
- Per-frame input state (pressed, just_pressed, just_released)
- Mouse position and delta tracking
- Cursor grab state management
- High-level game control queries (WASD movement, jump, sprint, etc.)
- Hotbar selection (1-9 keys)
- Mouse look delta with sensitivity scaling

**Tracked Keys:**
- **Movement:** W, A, S, D
- **Actions:** Space (jump), Shift (sprint/crouch), E (inventory), ESC (pause), F3 (debug)
- **Hotbar:** 1-9 digit keys
- **Mouse:** Left (attack/break), Right (use/place), Middle (pick block)

**API:**
```rust
InputState::new() -> InputState
input.begin_frame()  // Call at start of each frame
input.handle_keyboard(KeyEvent)
input.handle_mouse_button(MouseButton, ElementState)
input.handle_cursor_moved((f64, f64))

// Query methods
input.is_key_down(KeyCode) -> bool
input.is_key_just_pressed(KeyCode) -> bool
input.movement_input() -> (f32, f32)  // Normalized WASD vector
input.mouse_look_delta(sensitivity) -> (f32, f32)
input.is_jump() -> bool
input.is_attack() -> bool
input.hotbar_selection() -> Option<usize>
```

### 3. `src/game_loop.rs` (196 lines)

**Purpose:** Fixed timestep game loop with FPS tracking

**Key Features:**
- **Fixed timestep physics:** 60 ticks/second (configurable)
- **Variable frame rate rendering:** uncapped FPS
- Accumulator-based timestep to prevent spiral of death
- Frame time clamping (max 250ms to handle pauses)
- Safety limit: max 10 physics ticks per frame
- FPS counter with averaging over last 60 frames
- Frame time calculation in milliseconds

**Architecture:**
```
Each frame:
  1. Calculate elapsed time since last frame
  2. Accumulate time into physics accumulator
  3. Run N physics ticks (where N = accumulator / fixed_timestep)
  4. Return (num_ticks, render_delta) to caller
  5. Caller runs physics loop N times with fixed dt
  6. Caller renders once with variable dt for interpolation
```

**API:**
```rust
GameLoop::new() -> GameLoop  // 60 Hz default
GameLoop::with_tick_rate(ticks_per_second) -> GameLoop
game_loop.tick() -> (u32, f32)  // (physics_ticks, render_delta)
game_loop.fps() -> f32
game_loop.average_fps() -> f32
game_loop.frame_time_ms() -> f32
game_loop.fixed_timestep_seconds() -> f32
```

### 4. `src/main.rs` (updated)

**Integration:**
- Creates Window with OpenGL context
- Creates InputState tracker
- Creates GameLoop with fixed timestep
- Event loop handles:
  - Window events (resize, close, focus)
  - Keyboard input
  - Mouse input (buttons + motion)
  - Device events (raw mouse motion for better camera)
- Separates concerns:
  - `handle_input_actions()` - One-time actions (pause, debug, cursor grab)
  - `update_physics()` - Fixed timestep physics loop
  - `update_game_state()` - Variable timestep game logic
  - `render_frame()` - OpenGL rendering

**Current Behavior:**
- Click window to grab cursor (FPS mode)
- ESC to release cursor (pause)
- F3 to toggle debug overlay flag
- WASD, Space, Shift tracked (printed to console)
- 1-9 hotbar selection tracked
- Window resize handled
- Cursor auto-released on focus loss

## Technical Details

### Coordinate Systems
- **Screen space:** (0,0) = top-left, Y down
- **Mouse delta:** Relative motion, suitable for camera rotation
- **Movement input:** Normalized 2D vector, diagonal movement = same speed as cardinal

### Fixed Timestep Benefits
- Deterministic physics simulation
- No velocity/acceleration errors from variable dt
- Consistent behavior regardless of frame rate
- Network-friendly (same tick rate for all clients)

### Input Handling
- Uses `HashSet<KeyCode>` for O(1) key state lookup
- Separates "held" vs "just pressed" vs "just released" states
- `begin_frame()` clears per-frame state (just_pressed/just_released)
- Mouse delta only tracked when cursor is grabbed

### OpenGL Context
- Uses glutin 0.31 for cross-platform OpenGL context
- Compatible with winit 0.29 event loop
- Depth buffer: 24-bit
- Stencil buffer: 8-bit
- VSync: enabled (1 frame wait)

## Integration Notes for Next Agents

### For RENDERER agent:
- OpenGL context is already current and loaded
- Call `window.dimensions()` and `window.aspect_ratio()` for viewport/projection
- Call `window.swap_buffers()` after rendering
- Depth test and culling already enabled

### For PLAYER agent:
- Query `input.movement_input()` for WASD vector
- Query `input.mouse_look_delta(sensitivity)` for camera rotation
- Query `input.is_jump_just_pressed()` for jump events
- Use `game_loop.fixed_timestep_seconds()` for physics dt

### For PHYSICS agent:
- Run physics in the loop: `for _ in 0..num_physics_ticks`
- Use `game_loop.fixed_timestep_seconds()` as dt (1/60 = 0.0166s)
- Never use render_delta for physics, only for interpolation

### For UI agent:
- Query `game_loop.fps()` for FPS display
- Query `input.hotbar_selection()` for active slot
- Check `show_debug` flag from main.rs for debug overlay

## Testing

**Without Rust toolchain installed, cannot test compilation.**

Once `cargo build --release` is run, expected output:
```
Initializing Voxel Game...
OpenGL initialized:
  Version: "X.X.X"
  Renderer: "GPU Name"
Window created: 1280x720
Fixed timestep: 16.67ms (60 ticks/sec)
```

**Manual testing checklist:**
- [ ] Window opens at 1280x720
- [ ] Window resizes properly
- [ ] Click to grab cursor (cursor disappears)
- [ ] ESC releases cursor (cursor reappears)
- [ ] WASD keys detected (console output)
- [ ] Mouse look delta calculated
- [ ] F3 toggles debug flag
- [ ] 1-9 keys select hotbar slots
- [ ] Window close button works
- [ ] Alt+F4 works (focus loss releases cursor)

## Dependencies Used

```toml
gl = "0.14"           # OpenGL function pointers
winit = "0.29"        # Cross-platform windowing
glutin = "0.31"       # OpenGL context creation
glutin-winit = "0.4"  # glutin + winit integration
raw-window-handle = "0.5"  # Platform window handles
```

## Lines of Code

- `window.rs`: 181 lines
- `input.rs`: 270 lines
- `game_loop.rs`: 196 lines (including tests)
- `main.rs`: 168 lines (updated)
- **Total:** 815 lines of window/input code

## Known Limitations

1. **No gamepad support** (only keyboard + mouse)
2. **No scroll wheel tracking** (easy to add if needed)
3. **No text input** (not needed for Minecraft-style game)
4. **Single window only** (no multi-window support)
5. **No IME support** (international text input)

These limitations are acceptable for the project scope.

## Next Steps

1. **Install Rust toolchain** to test compilation
2. **RENDERER agent:** Implement shader loading, mesh rendering, texture atlas
3. **PLAYER agent:** Implement camera, movement controller, collision response
4. **PHYSICS agent:** Implement raycasting, AABB collision, gravity

The window and input infrastructure is complete and ready for use by other systems.
