# Player System - Quick Integration Example

This guide shows how to integrate the player system into the main game loop.

## Minimal Integration

```rust
use minecraft_clone::player::Player;
use minecraft_clone::input::InputState;
use minecraft_clone::world::World;
use glam::Vec3;

fn main() {
    // Setup
    let mut player = Player::new(Vec3::new(0.0, 100.0, 0.0));
    let mut input = InputState::new();
    let mut world = World::new();

    // Give player some blocks to place (for testing)
    player.hotbar = minecraft_clone::player::Hotbar::with_starting_items();

    // Game loop
    loop {
        let dt = 1.0 / 60.0; // Fixed timestep

        // 1. Handle input events (update input state)
        input.begin_frame();
        // ... process window events into input state ...

        // 2. Mouse look
        let mouse_delta = input.mouse_look_delta();
        player.update_look(mouse_delta);

        // 3. Movement
        let movement = input.movement_input();
        let is_sprinting = input.is_sprint();
        player.apply_movement_input(movement, is_sprinting);

        if input.is_jump() {
            player.jump();
        }

        // 4. Physics (collision-aware movement)
        player.update_physics(dt, |pos| world.get_block(pos));

        // 5. Block targeting (for highlighting)
        player.update_block_targeting(|pos| world.get_block(pos));

        // 6. Block breaking
        let is_attacking = input.is_attack();
        if let Some(block_pos) = player.update_block_breaking(is_attacking, |p| world.get_block(p)) {
            // Block broken!
            world.set_block(block_pos, minecraft_clone::types::BlockType::Air);
            // TODO: Add block to inventory
        }

        // 7. Block placement
        if input.is_use() {
            if let Some(place_pos) = player.try_place_block(|pos| world.get_block(pos)) {
                if let Some(block_type) = player.selected_block() {
                    world.set_block(place_pos, block_type);
                    player.hotbar.consume_from_slot(player.selected_slot);
                }
            }
        }

        // 8. Hotbar selection (1-9 keys)
        if let Some(slot) = input.hotbar_selection() {
            player.select_slot(slot - 1); // Keys 1-9 map to slots 0-8
        }

        // 9. Hotbar scrolling
        let scroll = input.mouse_scroll_delta();
        if scroll != 0.0 {
            player.scroll_hotbar(scroll.signum() as i32);
        }

        // 10. Death and respawn
        if player.is_dead() {
            // TODO: Show death screen
            if input.is_respawn() {
                let spawn_pos = world.get_spawn_point();
                player.respawn(spawn_pos);
            }
        }

        // 11. Render
        render_world(&world, &player);
        render_ui(&player);
    }
}

fn render_world(world: &World, player: &Player) {
    // Setup camera from player
    let eye = player.eye_position();
    let view_dir = player.view_direction();
    let target = eye + view_dir;
    // let view_matrix = Mat4::look_at_rh(eye, target, Vec3::Y);

    // Render chunks...

    // Render targeted block highlight
    if let Some(block_pos) = player.get_targeted_block() {
        // Draw wireframe cube at block_pos
    }

    // Render breaking progress
    if player.interaction.is_breaking() {
        if let Some(breaking_pos) = player.interaction.breaking_block {
            let stage = player.get_break_stage(); // 0-10
            // Render crack texture overlay at breaking_pos
        }
    }
}

fn render_ui(player: &Player) {
    // Health bar
    let health_percent = player.health.percentage();
    // Draw health_percent * 100% filled red bar

    // Hotbar
    for i in 0..9 {
        if let Some(slot) = player.hotbar.get_slot(i) {
            // Draw slot background
            if i == player.selected_slot {
                // Draw selection highlight
            }
            if let Some(block_type) = slot.block_type {
                // Draw block icon
                // Draw count if > 1
            }
        }
    }

    // Debug info (F3)
    if input.is_debug() {
        println!("Position: {:.2?}", player.position);
        println!("Velocity: {:.2?}", player.velocity);
        println!("On Ground: {}", player.on_ground);
        println!("Health: {}/{}", player.health.current(), player.health.max());
        println!("Facing: {}", player.facing_direction_string());
    }
}
```

## Camera Setup

```rust
use glam::{Mat4, Vec3};

// Method 1: look_at matrix
let eye = player.eye_position();
let target = eye + player.view_direction();
let up = Vec3::Y;
let view_matrix = Mat4::look_at_rh(eye, target, up);

// Method 2: Manual construction from pitch/yaw
let view_rotation = Mat4::from_rotation_y(-player.yaw) * Mat4::from_rotation_x(-player.pitch);
let view_translation = Mat4::from_translation(-player.eye_position());
let view_matrix = view_rotation * view_translation;
```

## Input Mappings

The player system expects these inputs from InputState:

```rust
// Movement (WASD)
input.movement_input() -> Vec3  // Normalized (forward, 0, strafe)

// Actions
input.is_jump() -> bool         // Space
input.is_sprint() -> bool       // Left Shift
input.is_attack() -> bool       // Left Mouse (hold to break)
input.is_use() -> bool          // Right Mouse (place block)

// Camera
input.mouse_look_delta() -> (f64, f64)  // (dx, dy) in pixels

// Hotbar
input.hotbar_selection() -> Option<usize>  // 1-9 keys (returns 1-9)
input.mouse_scroll_delta() -> f64          // Scroll wheel

// UI
input.is_debug() -> bool        // F3
input.is_respawn() -> bool      // Enter (on death screen)
```

## World Interface

The player needs these methods from the world:

```rust
impl World {
    // Block queries (used by player physics and interaction)
    fn get_block(&self, pos: &WorldPos) -> BlockType;

    // Block modification
    fn set_block(&mut self, pos: WorldPos, block: BlockType);

    // Spawn point
    fn get_spawn_point(&self) -> Vec3;
}
```

## Common Patterns

### Creative Mode Toggle
```rust
if input.is_toggle_creative() {
    if creative_mode {
        player.hotbar = Hotbar::new(); // Empty
    } else {
        player.hotbar = Hotbar::with_starting_items(); // Full
    }
}
```

### God Mode
```rust
if god_mode {
    player.health.restore(); // Always full health
    // Skip damage in update_fall_tracking
}
```

### No-Clip Mode
```rust
if no_clip {
    // Skip collision detection
    player.position += player.velocity * dt;
} else {
    player.update_physics(dt, |pos| world.get_block(pos));
}
```

### Flying
```rust
if flying {
    // No gravity
    if input.is_jump() {
        player.velocity.y = 10.0; // Fly up
    } else if input.is_crouch() {
        player.velocity.y = -10.0; // Fly down
    } else {
        player.velocity.y = 0.0; // Hover
    }
    // Still use physics for horizontal collision
}
```

## Performance Tips

1. **Raycast only when needed:**
   ```rust
   // Only update targeting when not in inventory screen
   if !ui_open {
       player.update_block_targeting(|pos| world.get_block(pos));
   }
   ```

2. **Cache block lookups:**
   ```rust
   // If world.get_block is expensive, cache results
   let mut block_cache = HashMap::new();
   let get_block = |pos: &WorldPos| {
       *block_cache.entry(*pos).or_insert_with(|| world.get_block(pos))
   };
   player.update_physics(dt, get_block);
   ```

3. **Skip updates when paused:**
   ```rust
   if !paused && !ui_open {
       player.update_physics(dt, |pos| world.get_block(pos));
   }
   ```

## Troubleshooting

### Player falls through floor
- Check that `world.get_block()` returns solid blocks correctly
- Verify physics engine `collide_and_slide` is working
- Check that `on_ground` flag is being set

### Can't break blocks
- Verify `is_attacking` is true when holding left mouse
- Check that raycast is hitting blocks (print `targeted_block`)
- Ensure block hardness is not infinity (bedrock)

### Can't place blocks
- Check that hotbar has blocks (`selected_block()` returns Some)
- Verify placement position is Air
- Check player collision (might be too close to block)

### Movement feels wrong
- Verify delta_time is correct (should be ~0.016 for 60 FPS)
- Check that movement input is normalized
- Ensure collision is enabled (not no-clip mode)

### Camera spins too fast/slow
- Adjust `MOUSE_SENSITIVITY` constant in movement.rs
- Check that mouse delta is in pixels (not already scaled)
- Verify mouse capture is enabled

## Next Steps

After integrating the player:
1. Add rendering for targeted block highlight
2. Add rendering for break progress (crack textures)
3. Implement full inventory screen (beyond hotbar)
4. Add tool system with proper multipliers
5. Add more player actions (crouch, sprint FOV change, etc.)

## See Also

- `PLAYER_IMPLEMENTATION.md` - Full technical documentation
- `PLAYER_SUMMARY.md` - Delivery summary and statistics
- `src/player/` - Source code with inline comments
