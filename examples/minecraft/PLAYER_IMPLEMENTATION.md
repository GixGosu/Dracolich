# Player System Implementation

Complete implementation of the player controller and interaction systems for the Minecraft clone.

## Overview

The player system manages all player-related functionality including movement, camera control, health, inventory, and block interactions. It integrates with the physics engine for collision detection and the world system for block breaking/placing.

## Architecture

### Module Structure

```
src/player/
├── mod.rs          - Player struct, core state, helper methods
├── movement.rs     - Movement physics, mouse look, jumping, sprinting
├── health.rs       - Health system, damage, death, respawning
├── interaction.rs  - Block breaking, placement, raycasting
└── hotbar.rs       - 9-slot inventory system
```

## Core Components

### 1. Player Struct (`mod.rs`)

**State:**
- `position: Vec3` - Player feet position in world space
- `velocity: Vec3` - Current velocity (blocks/second)
- `on_ground: bool` - Ground contact state
- `pitch: f32` - Vertical camera rotation (radians, ±89°)
- `yaw: f32` - Horizontal camera rotation (radians, wraps)
- `health: Health` - Health system
- `hotbar: Hotbar` - 9-slot inventory
- `selected_slot: usize` - Currently selected slot (0-8)
- `interaction: InteractionState` - Block breaking/placing state
- `is_sprinting: bool` - Sprint state
- `fall_distance: f32` - Accumulated fall distance for damage

**Constants:**
- Eye height: `1.62` blocks
- Hitbox: `0.6 × 1.8 × 0.6` blocks (width × height × depth)

**Key Methods:**
```rust
// Position queries
fn eye_position(&self) -> Vec3
fn forward(&self) -> Vec3
fn right(&self) -> Vec3
fn view_direction(&self) -> Vec3
fn aabb(&self) -> AABB

// State management
fn respawn(&mut self, spawn_position: Vec3)
fn update_fall_tracking(&mut self)

// Inventory
fn selected_block(&self) -> Option<BlockType>
fn select_slot(&mut self, slot: usize)
fn scroll_hotbar(&mut self, delta: i32)
```

### 2. Movement System (`movement.rs`)

**Physics Constants:**
- Walk speed: `4.3` blocks/second
- Sprint speed: `5.612` blocks/second (1.3× multiplier)
- Jump velocity: `8.5` blocks/second (reaches ~1.25 blocks height)
- Gravity: `-32` blocks/second² (from physics engine)
- Terminal velocity: `-78.4` blocks/second
- Mouse sensitivity: `0.002` radians/pixel

**Camera Control:**
```rust
fn update_look(&mut self, mouse_delta: (f64, f64))
```
- Yaw (horizontal): unlimited rotation, wraps at 2π
- Pitch (vertical): clamped to ±89° to prevent gimbal lock
- Inverted Y-axis (down = look up, like most FPS games)

**Movement:**
```rust
fn apply_movement_input(&mut self, movement_input: Vec3, is_sprinting: bool)
```
- Takes normalized WASD input vector
- Calculates world-space direction from camera yaw
- Applies walk/sprint speed
- Prevents diagonal speed boost via normalization
- Sprint only works when on ground

**Jump:**
```rust
fn jump(&mut self)
```
- Only works when `on_ground == true`
- Sets upward velocity to `JUMP_VELOCITY`
- Clears ground flag

**Physics Update:**
```rust
fn update_physics<F>(&mut self, delta_time: f32, get_block: F)
where F: Fn(&WorldPos) -> BlockType
```
- Applies gravity when airborne
- Clamps falling speed to terminal velocity
- Calls `collide_and_slide` from physics engine
- Updates position and ground state
- Handles ceiling/ground collision response
- Updates fall tracking for damage calculation

### 3. Health System (`health.rs`)

**State:**
- `current: f32` - Current health
- `max: f32` - Maximum health (default 20)
- `last_damage_time: Option<Instant>` - For cooldown tracking
- `is_dead: bool` - Death state

**Damage Cooldown:**
- Duration: `500ms` (0.5 seconds)
- Prevents rapid consecutive damage
- Common in games to prevent instant death from multiple hits

**Key Methods:**
```rust
fn damage(&mut self, amount: f32) -> bool
fn heal(&mut self, amount: f32) -> f32
fn respawn(&mut self)
fn is_dead(&self) -> bool
fn percentage(&self) -> f32  // For UI health bar
```

**Fall Damage:**
- Calculated in `Player::update_fall_tracking()`
- No damage for falls ≤3 blocks
- Damage = `floor(fall_distance - 3.0)`
- Examples:
  - 3 blocks: 0 damage
  - 5 blocks: 2 damage
  - 10 blocks: 7 damage
  - 23+ blocks: instant death (20+ damage)

### 4. Interaction System (`interaction.rs`)

**InteractionState:**
```rust
pub struct InteractionState {
    targeted_block: Option<RaycastHit>,
    breaking_block: Option<WorldPos>,
    break_progress: f32,          // 0.0 to 1.0
    break_start_time: Option<Instant>,
    break_duration: Duration,
}
```

**Block Targeting:**
```rust
fn update_targeting<F>(&mut self, eye_position: Vec3, view_direction: Vec3, get_block: F)
```
- Uses DDA raycasting from physics engine
- Max range: 5 blocks
- Updates every frame
- Provides block position, face, and adjacent position for placement

**Block Breaking:**
```rust
fn update_block_breaking<F>(&mut self, is_attacking: bool, get_block: F) -> Option<WorldPos>
```

**Break Time Calculation:**
```
break_time = block.hardness() / tool_multiplier
```

**Tool Multipliers:**
- Bare hands: `1.0×`
- Correct tool: `4.0×` (stone pickaxe for stone, etc.)

**Example Break Times (bare hands):**
- Dirt: 0.5s
- Wood: 2.0s
- Stone: 4.0s
- Diamond ore: 8.0s
- Bedrock: ∞ (unbreakable)

**Progress Tracking:**
- `break_progress`: 0.0 to 1.0 (for UI)
- `break_stage()`: 0 to 10 integer (for rendering crack textures)
- Auto-cancels if:
  - Player stops attacking
  - Player looks away (targets different block)
  - Block changes (someone else broke it)

**Block Placement:**
```rust
fn try_place_block<F>(&mut self, get_block: F) -> Option<WorldPos>
```

**Placement Rules:**
1. Must have a targeted block (within 5 blocks)
2. Must have a selected block in hotbar
3. Adjacent position must be Air
4. Cannot place inside player hitbox
5. Returns position where block was placed

**Player Collision Check:**
- Uses AABB intersection test
- Prevents player from suffocating in blocks
- Common edge case: placing block below feet while jumping

### 5. Hotbar System (`hotbar.rs`)

**HotbarSlot:**
```rust
pub struct HotbarSlot {
    block_type: Option<BlockType>,
    count: u32,
}
```
- Max stack size: 64 (Minecraft standard)
- Empty when `block_type == None` or `count == 0`

**Hotbar:**
```rust
pub struct Hotbar {
    slots: [HotbarSlot; 9],
}
```

**Key Operations:**
```rust
fn add_item(&mut self, block_type: BlockType, count: u32) -> u32
fn remove_item(&mut self, block_type: BlockType, count: u32) -> u32
fn consume_from_slot(&mut self, slot: usize) -> bool
fn swap_slots(&mut self, slot_a: usize, slot_b: usize)
```

**Add Item Logic:**
1. Try to stack into existing slots of same type (up to 64)
2. Fill empty slots if overflow
3. Return count of items that didn't fit

**Starting Items:**
```rust
fn with_starting_items() -> Self
```
- For creative mode / testing
- Fills all 9 slots with common blocks (64 each)
- Order: Grass, Dirt, Stone, Cobblestone, Planks, Wood, Glass, Sand, Leaves

## Integration Guide

### Game Loop Integration

```rust
// In main game loop
let delta_time = 1.0 / 60.0; // Fixed timestep

// 1. Update input state
input_state.begin_frame();
// ... process window events ...

// 2. Update camera
let mouse_delta = input_state.mouse_look_delta();
player.update_look(mouse_delta);

// 3. Update movement
let movement = input_state.movement_input();
let is_sprinting = input_state.is_sprint();
player.apply_movement_input(movement, is_sprinting);

if input_state.is_jump() {
    player.jump();
}

// 4. Update physics (uses world for collision)
player.update_physics(delta_time, |pos| world.get_block(pos));

// 5. Update block targeting
player.update_block_targeting(|pos| world.get_block(pos));

// 6. Handle block breaking
let is_attacking = input_state.is_attack();
if let Some(block_pos) = player.update_block_breaking(is_attacking, |pos| world.get_block(pos)) {
    world.set_block(block_pos, BlockType::Air);
    // TODO: Add block to inventory
}

// 7. Handle block placement
if input_state.is_use() {
    if let Some(place_pos) = player.try_place_block(|pos| world.get_block(pos)) {
        if let Some(block_type) = player.selected_block() {
            world.set_block(place_pos, block_type);
            player.hotbar.consume_from_slot(player.selected_slot);
        }
    }
}

// 8. Handle hotbar selection
if let Some(slot) = input_state.hotbar_selection() {
    player.select_slot(slot - 1); // Keys 1-9 map to slots 0-8
}

let scroll = input_state.mouse_scroll_delta();
if scroll != 0.0 {
    player.scroll_hotbar(scroll.signum() as i32);
}

// 9. Check for death and respawn
if player.is_dead() {
    // Show death screen, wait for respawn input
    if input_state.is_respawn() {
        let spawn_pos = world.get_spawn_point();
        player.respawn(spawn_pos);
    }
}
```

### Camera Matrix Setup

```rust
// View matrix (for rendering)
let eye = player.eye_position();
let target = eye + player.view_direction();
let up = Vec3::Y;
let view_matrix = Mat4::look_at_rh(eye, target, up);

// Or construct manually from pitch/yaw
let view_matrix = Mat4::from_rotation_y(-player.yaw) * Mat4::from_rotation_x(-player.pitch);
view_matrix = view_matrix * Mat4::from_translation(-eye);
```

### World Access Pattern

The player system uses closure-based world access to avoid tight coupling:

```rust
// Good: Closure captures world reference
player.update_physics(dt, |pos| world.get_block(pos));

// Also works: Direct world reference
let get_block = |pos: &WorldPos| world.get_block(pos);
player.update_physics(dt, get_block);

// For testing: Mock world
let air_world = |_pos: &WorldPos| BlockType::Air;
player.update_physics(dt, air_world);
```

## Testing

All modules include comprehensive unit tests:

**movement.rs:**
- Mouse look (yaw/pitch clamping)
- Jump mechanics (ground requirement)
- Gravity and terminal velocity
- Ground collision
- Sprint speed
- Facing direction calculation

**health.rs:**
- Damage and healing
- Death state
- Respawn
- Damage cooldown
- Health percentage calculation
- Max health modification

**interaction.rs:**
- Block targeting
- Break progress tracking
- Break cancellation
- Placement collision checks
- Tool multiplier effects
- Unbreakable blocks

**hotbar.rs:**
- Slot management
- Item stacking
- Add/remove operations
- Overflow handling
- Slot swapping

Run tests with:
```bash
cargo test --lib player
```

## Performance Characteristics

**Per-frame overhead:**
- Mouse look: ~5 trig operations (negligible)
- Movement input: ~10 vector operations (negligible)
- Physics update: Dominated by collision detection (see physics module)
- Block targeting: DDA raycast (~5-15 voxel checks)
- Block breaking: Simple timer comparison (negligible)

**Memory footprint:**
- Player struct: ~400 bytes
- Health: ~32 bytes
- Hotbar: ~320 bytes (9 slots × 36 bytes)
- InteractionState: ~80 bytes

**Total: ~830 bytes per player (single-player only needs one)**

## Constants Reference

```rust
// Player dimensions
PLAYER_WIDTH: 0.6        // blocks
PLAYER_HEIGHT: 1.8       // blocks
PLAYER_EYE_HEIGHT: 1.62  // blocks

// Movement
WALK_SPEED: 4.3          // blocks/second
SPRINT_SPEED: 5.612      // blocks/second
JUMP_VELOCITY: 8.5       // blocks/second (initial)

// Camera
MOUSE_SENSITIVITY: 0.002 // radians/pixel
MAX_PITCH: ±89°          // degrees

// Health
MAX_HEALTH: 20.0
DAMAGE_COOLDOWN: 500ms

// Interaction
MAX_RAYCAST_DISTANCE: 5.0 // blocks
BARE_HANDS_MULTIPLIER: 1.0
CORRECT_TOOL_MULTIPLIER: 4.0

// Hotbar
HOTBAR_SIZE: 9
MAX_STACK_SIZE: 64
```

## Known Limitations & Future Work

**Current Implementation:**
- ✅ Full physics and collision
- ✅ Mouse look with proper clamping
- ✅ Sprint and jump mechanics
- ✅ Fall damage
- ✅ Health system with damage cooldown
- ✅ Block breaking with progress tracking
- ✅ Block placement with collision checks
- ✅ 9-slot hotbar with stacking
- ✅ Comprehensive test coverage

**Not Yet Implemented:**
- ⏳ Tool system (currently only bare hands multiplier)
- ⏳ Hunger/food system
- ⏳ Full inventory (only hotbar implemented)
- ⏳ Crouching/sneaking
- ⏳ Swimming mechanics
- ⏳ Flying (creative mode)
- ⏳ Status effects (poison, regeneration, etc.)
- ⏳ Item dropping/picking up

**Integration Dependencies:**
- Requires `physics` module for collision and raycasting
- Requires `world` module for block queries
- Requires `input` module for control state
- Requires `types` module for BlockType, WorldPos, AABB

## File Summary

| File | Lines | Purpose |
|------|-------|---------|
| mod.rs | 237 | Player struct, core state, helper methods |
| movement.rs | 246 | Movement physics, mouse look, jumping |
| health.rs | 240 | Health system, damage, respawning |
| interaction.rs | 329 | Block breaking, placement, raycasting |
| hotbar.rs | 358 | 9-slot inventory with stacking |
| **TOTAL** | **1,410** | Complete player controller system |

All files include comprehensive unit tests and documentation.
