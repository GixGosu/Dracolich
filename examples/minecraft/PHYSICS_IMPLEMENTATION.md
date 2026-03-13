# Physics Engine Implementation

## Overview

This document describes the physics and collision system implementation for the voxel game. The system provides robust collision detection and response for player movement, raycasting for block targeting, and ground/ceiling detection.

## Architecture

The physics module is split into three main components:

### 1. AABB (`src/physics/aabb.rs`)

Extended functionality for Axis-Aligned Bounding Box collision detection:

- **`sweep_aabb()`** - Continuous collision detection using swept AABB algorithm
  - Detects the time of impact between a moving AABB and a static AABB
  - Returns collision time (0.0 to 1.0), normal, and hit position
  - Uses Minkowski sum to expand the static AABB by the moving AABB's size
  - Performs ray-box intersection test

- **`sweep_blocks()`** - Multi-block sweep test
  - Tests collision against all blocks along the movement path
  - Returns the earliest collision encountered
  - Takes a closure to query block types from the world

- **`get_overlapping_blocks()`** - Spatial query
  - Returns all block positions that the AABB overlaps
  - Used for efficient broad-phase collision detection

- **`get_penetration()`** - Static overlap resolution
  - Calculates penetration depth and separation normal
  - Used for resolving cases where entities clip into blocks

### 2. Collision (`src/physics/collision.rs`)

Implements the **collide-and-slide** algorithm for smooth player movement:

#### Key Functions

- **`collide_and_slide()`** - Main collision resolution
  - Iterative algorithm that resolves collisions while preserving momentum
  - Projects velocity along collision surfaces for smooth sliding
  - Detects ground, ceiling, and wall contacts
  - Returns `CollisionInfo` with final position, velocity, and contact flags

- **`is_on_ground()`** - Ground detection
  - Checks if an AABB is standing on solid ground
  - Tests slightly below the AABB (0.01 units)
  - Required for proper jump mechanics

- **`is_hitting_ceiling()`** - Ceiling detection
  - Checks if an AABB is hitting a ceiling above
  - Required to stop upward movement when jumping into blocks

- **`resolve_penetration()`** - Stuck-in-wall resolution
  - Forces entity out of blocks if they somehow get stuck
  - Useful for spawn points or edge cases

- **`move_with_collision()`** - High-level movement function
  - Combines collide-and-slide with penetration resolution
  - Takes velocity and delta time, returns collision info

#### Collide-and-Slide Algorithm

The algorithm works as follows:

1. **Sweep test** - Cast the AABB along its velocity vector to find the first collision
2. **Move to impact** - Move the AABB to the point of collision
3. **Slide** - Project the remaining velocity along the collision surface
4. **Iterate** - Repeat until velocity is consumed or max iterations reached

This provides smooth movement along walls and prevents getting stuck in corners.

### 3. Raycast (`src/physics/raycast.rs`)

DDA (Digital Differential Analyzer) raycasting through the voxel grid:

#### Key Functions

- **`raycast()`** - Full raycasting with detailed hit info
  - Returns `RaycastHit` with:
    - Block position that was hit
    - Adjacent block position (for placement)
    - Face that was hit (Direction)
    - Distance to hit
    - Exact hit point in world space
  - Uses DDA algorithm for efficient voxel traversal
  - Maximum range of 5 blocks (configurable via `MAX_RAYCAST_DISTANCE`)

- **`raycast_block()`** - Simplified raycast
  - Returns just block position and face
  - Convenience wrapper for simpler use cases

- **`has_line_of_sight()`** - Vision test
  - Checks if two points have clear line of sight
  - Used for mob AI and player detection

- **`get_blocks_along_ray()`** - Ray traversal
  - Returns all blocks along a ray up to max distance
  - Can be used for effects, particle systems, etc.

#### DDA Algorithm

The DDA algorithm efficiently steps through the voxel grid:

1. **Initialize** - Calculate step direction and delta distances for each axis
2. **Step** - Always step along the axis with the smallest t_max value
3. **Check** - Test each voxel for solid blocks
4. **Track face** - Remember which face was crossed to determine hit normal

This is more efficient than naive ray marching (checking every point along the ray).

## Usage Examples

### Player Movement with Collision

```rust
use crate::physics::collision::move_with_collision;
use glam::Vec3;

// Player AABB (centered on player position, 0.6x1.8x0.6 blocks)
let player_aabb = AABB::from_center_size(
    player_position,
    Vec3::new(0.6, 1.8, 0.6)
);

// Apply gravity and player input
let mut velocity = player_velocity;
velocity.y += GRAVITY * delta_time;

// Resolve collision
let collision = move_with_collision(
    &player_aabb,
    velocity,
    delta_time,
    |pos| world.get_block(pos)
);

// Update player state
player_position = collision.position;
player_velocity = collision.velocity;

// Check if on ground for jumping
if collision.on_ground {
    player_velocity.y = 0.0;
    can_jump = true;
}

// Stop upward movement if hit ceiling
if collision.hit_ceiling {
    player_velocity.y = player_velocity.y.min(0.0);
}
```

### Block Targeting

```rust
use crate::physics::raycast::{raycast, MAX_RAYCAST_DISTANCE};

let camera_pos = player.position + Vec3::new(0.0, 1.6, 0.0); // Eye height
let camera_dir = player.get_look_direction();

if let Some(hit) = raycast(
    camera_pos,
    camera_dir,
    MAX_RAYCAST_DISTANCE,
    |pos| world.get_block(pos)
) {
    // Player is looking at a block
    targeted_block = Some(hit.block_pos);

    // For breaking
    if left_click {
        world.set_block(&hit.block_pos, BlockType::Air);
    }

    // For placing
    if right_click {
        // Place on the adjacent face
        if world.get_block(&hit.adjacent_pos) == BlockType::Air {
            world.set_block(&hit.adjacent_pos, selected_block_type);
        }
    }
}
```

### Ground Detection for Jumping

```rust
use crate::physics::collision::is_on_ground;

let player_aabb = AABB::from_center_size(
    player_position,
    Vec3::new(0.6, 1.8, 0.6)
);

if is_on_ground(&player_aabb, |pos| world.get_block(pos)) {
    if jump_pressed {
        player_velocity.y = 8.0; // Jump velocity
    }
}
```

## Configuration

### Physics Constants

Defined in `src/physics/mod.rs`:

- `GRAVITY`: -32.0 blocks/s² (realistic falling speed)
- `TERMINAL_VELOCITY`: -78.4 blocks/s (prevents infinite acceleration)

### Collision Parameters

Defined in `src/physics/collision.rs`:

- `MAX_COLLISION_ITERATIONS`: 4 (prevents infinite loops in corners)
- `MIN_VELOCITY_THRESHOLD`: 0.001 (stops micro-movements)

### Raycast Parameters

Defined in `src/physics/raycast.rs`:

- `MAX_RAYCAST_DISTANCE`: 5.0 blocks (Minecraft-like reach)

## Performance Considerations

### Efficient Broad-Phase

The `get_overlapping_blocks()` method limits collision checks to only nearby blocks:
- For a typical player (0.6x1.8x0.6), this is ~4-8 blocks max
- Much faster than checking every block in the world

### DDA vs Naive Raymarching

DDA algorithm checks exactly one voxel per grid crossing:
- For 5 block range, DDA checks ~5-15 voxels
- Naive raymarching would check hundreds of points

### Iteration Limits

The collide-and-slide algorithm limits iterations to prevent worst-case performance in complex geometry.

## Testing

Each module includes unit tests:

- `aabb.rs`: Tests intersection, point containment, overlapping blocks
- `collision.rs`: Tests free movement, ground collision, ground detection
- `raycast.rs`: Tests hitting blocks, missing blocks, line of sight

Run tests with `cargo test` (requires Rust toolchain).

## Integration Points

The physics system interfaces with:

1. **World module** - Via closure `F: Fn(&WorldPos) -> BlockType`
   - Allows physics to query block types without direct coupling

2. **Player module** - Uses collision detection for movement
   - Updates position based on `CollisionInfo`

3. **UI module** - Uses raycast results for block highlighting
   - Displays which block is targeted

## Anti-Clipping Guarantees

The implementation prevents all clipping scenarios:

1. **Floor clipping**: Collide-and-slide stops downward movement on ground contact
2. **Ceiling clipping**: Ceiling detection stops upward movement
3. **Wall clipping**: Sweep tests prevent moving through walls
4. **Corner clipping**: Multiple iterations resolve complex corner cases
5. **Stuck recovery**: `resolve_penetration()` forces entities out if they glitch inside

## Future Enhancements

Potential improvements for later:

- **Water physics** - Buoyancy and swimming
- **Sloped surfaces** - Ramps and stairs (requires non-AABB geometry)
- **Swept character controller** - Capsule collision instead of box
- **Dynamic friction** - Different slide behavior per block type
- **Entity-entity collision** - Currently only handles entity-world
