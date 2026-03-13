# VoxelCraft - System Architecture

## Overview

VoxelCraft is a voxel-based sandbox game built from scratch in Rust using raw OpenGL 3.3+. The architecture follows a modular design with clear separation between world management, rendering, player control, game logic, and UI systems.

## Core Philosophy

### Design Principles

1. **Performance First:** All systems optimized for 60+ FPS with 8+ chunk render distance
2. **Data-Oriented Design:** Cache-friendly data layouts, minimal allocations in hot paths
3. **Modularity:** Clear boundaries between systems with minimal coupling
4. **Simplicity:** Prefer straightforward solutions over premature abstractions
5. **No External Engines:** Raw OpenGL and system-level control for learning and performance

### Key Constraints

- No game engines (Bevy, Macroquad, etc.)
- No voxel libraries (handles all voxel logic manually)
- Raw OpenGL 3.3+ via `gl` crate
- Must compile with `cargo build --release`
- Target: 30+ FPS on integrated graphics

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Main Loop                           │
│  (src/main.rs - 60Hz tick, event processing, frame render) │
└────┬─────────────┬──────────────┬──────────────┬───────────┘
     │             │              │              │
     ▼             ▼              ▼              ▼
┌─────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────┐
│  World  │  │  Player  │  │ Renderer │  │  UI System  │
│ Manager │  │Controller│  │  Engine  │  │   & HUD     │
└────┬────┘  └─────┬────┘  └─────┬────┘  └──────┬──────┘
     │             │              │              │
     ▼             ▼              ▼              ▼
┌─────────────────────────────────────────────────────────────┐
│              Shared Game State & Resources                  │
│  (Block registry, texture atlas, chunk cache, mob pool)    │
└─────────────────────────────────────────────────────────────┘
```

## Module Breakdown

### 1. World System (`src/world/`)

Manages the voxel world, terrain generation, chunk lifecycle, and block data.

#### Files
- `mod.rs` - Public API and world coordinator
- `chunk.rs` - Chunk data structure (16×256×16 blocks)
- `generator.rs` - Procedural terrain generation
- `block.rs` - Block type definitions and properties

#### Chunk Structure

```
Chunk: 16×256×16 blocks = 65,536 voxels
│
├── Block Data: [BlockType; 65536]
│   └── Stored as flat array, indexed as: y*256 + z*16 + x
│
├── Mesh Data: Vec<Vertex>
│   └── Greedy-meshed geometry (only visible faces)
│
└── Metadata
    ├── Position: (chunk_x, chunk_z)
    ├── Dirty flag: needs remeshing
    └── State: Ungenerated | Generating | Generated | Meshed
```

#### Coordinate Systems

**Block Coordinates (Global):**
- Absolute position in world: `(block_x, block_y, block_z)`
- Example: `(150, 64, -200)` = block at X=150, Y=64, Z=-200

**Chunk Coordinates:**
- Chunk position: `(chunk_x, chunk_z)`
- Conversion: `chunk_x = floor(block_x / 16)`
- Example: block X=150 → chunk X=9

**Local Block Coordinates:**
- Position within chunk: `(local_x, local_y, local_z)`
- Range: `0..16` for X/Z, `0..256` for Y
- Conversion: `local_x = block_x % 16` (with proper negative handling)

**World Origin:**
- Center: (0, 0, 0)
- Y=0: Bedrock layer
- Y=64: Sea level
- Y=255: Build height limit

#### Terrain Generation Pipeline

```
Player Position
      │
      ▼
┌────────────────────┐
│ Chunk Load Request │  (within render distance)
└─────────┬──────────┘
          │
          ▼
┌─────────────────────┐
│   Noise Sampling    │
│                     │
│ • 2D Perlin: height │
│ • 3D Simplex: caves │
│ • Multi-octave for  │
│   terrain detail    │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│  Block Placement    │
│                     │
│ • Bedrock (Y=0)     │
│ • Stone (Y<height)  │
│ • Dirt (near surf.) │
│ • Grass (surface)   │
│ • Sand (near water) │
│ • Water (Y≤64)      │
│ • Cave carving      │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Feature Generation  │
│                     │
│ • Tree placement    │
│ • Ore distribution  │
│   (coal, iron,      │
│    gold, diamond)   │
└─────────┬───────────┘
          │
          ▼
     [Chunk Ready]
          │
          ▼
┌─────────────────────┐
│   Mesh Generation   │  (see Rendering section)
└─────────────────────┘
```

#### Chunk Loading Strategy

**Spiral Loading Pattern:**
```
Player at chunk (0,0), render distance = 2

Load order (spiral outward):
 17 10 11 12 13
 16  3  4  5 14
  9  2  1  6 18
  8  7 20 19 22
 15 24 23 21 25

Priority: closest chunks first
```

**Unloading:**
- Chunks beyond render distance + 2 are unloaded
- Mesh data freed immediately
- Block data cached for quick re-generation
- Dirty chunks saved before unload (for persistence, if implemented)

#### Noise Configuration

```rust
// Height map (2D Perlin)
frequency: 0.01      // Controls "zoom" (lower = larger features)
octaves: 4           // Layers of detail
persistence: 0.5     // Amplitude decay per octave
lacunarity: 2.0      // Frequency increase per octave

// Cave noise (3D Simplex)
frequency: 0.05
threshold: 0.6       // Values above = air (cave)
```

**Terrain Height Calculation:**
```
base_height = 64 (sea level)
noise_value = perlin_2d(x * freq, z * freq)  // Range: -1 to 1
height = base_height + (noise_value * amplitude)
amplitude = 40  // ±40 blocks from sea level

Result range: Y=24 to Y=104
```

### 2. Rendering System (`src/renderer/`)

Handles all OpenGL rendering, including chunk meshes, entities, UI, and skybox.

#### Files
- `mod.rs` - Renderer initialization and frame coordination
- `shader.rs` - GLSL shader compilation and management
- `mesh.rs` - Mesh generation and optimization
- `camera.rs` - View/projection matrix calculations
- `texture.rs` - Texture atlas loading and binding

#### Rendering Pipeline

```
Frame Start
    │
    ▼
┌──────────────────┐
│  Clear Buffers   │ (color + depth)
└────────┬─────────┘
         │
         ▼
┌──────────────────────┐
│   Update Uniforms    │
│ • View matrix        │
│ • Projection matrix  │
│ • Time (for sky)     │
│ • Fog parameters     │
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│   Render Skybox      │ (depth test disabled)
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│   Frustum Culling    │
│ • Test chunk bounds  │
│ • Skip out-of-view   │
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│  Render Opaque       │
│  Chunks (front→back) │
│ • Bind texture atlas │
│ • Draw mesh batches  │
│ • Depth test ON      │
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│  Render Transparent  │
│  Blocks (back→front) │
│ • Water, glass       │
│ • Alpha blending ON  │
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│    Render Mobs       │ (simple geometry)
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│ Render Block Outline │ (targeted block)
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│     Render UI        │
│ • Crosshair          │
│ • Hotbar             │
│ • Health hearts      │
│ • Debug overlay (F3) │
│ • Inventory (if open)│
└────────┬─────────────┘
         │
         ▼
    Swap Buffers
```

#### Mesh Generation (Greedy Meshing)

**Naive Approach (not used):**
- 6 quads per block × 65,536 blocks/chunk = 393,216 quads
- Vertex count: ~1.5 million per chunk
- Unplayable performance

**Greedy Meshing Optimization:**

1. **Face Culling:** Don't generate faces between two solid blocks
2. **Greedy Expansion:** Merge adjacent identical faces into larger quads

```
Example: 4×1 row of stone blocks

Naive: 4 separate quads
  ┌───┐┌───┐┌───┐┌───┐
  │   ││   ││   ││   │
  └───┘└───┘└───┘└───┘
  Vertices: 16 (4 per quad)

Greedy: 1 merged quad
  ┌───────────────────┐
  │                   │
  └───────────────────┘
  Vertices: 4

Reduction: 75%
```

**Algorithm:**
```
For each axis (X, Y, Z):
  For each slice perpendicular to axis:
    Create visibility mask (true = face should render)
    For each visible face:
      Greedily expand right (same block type, same visibility)
      Greedily expand forward (matching row)
      Generate merged quad
      Mark used faces in mask
```

**Typical Results:**
- Chunk with varied terrain: 2,000-8,000 quads (vs 393K naive)
- 95-98% reduction in geometry
- Critical for 60 FPS performance

#### Vertex Data Layout

```rust
struct Vertex {
    position: [f32; 3],      // World-space XYZ
    tex_coords: [f32; 2],    // UV in texture atlas (0.0-1.0)
    normal: [f32; 3],        // Face normal for lighting
    ambient_occlusion: f32,  // AO factor (0.0-1.0)
}

// Packed into VBO, attributes:
// layout(location=0): position
// layout(location=1): tex_coords
// layout(location=2): normal
// layout(location=3): ao
```

#### Texture Atlas

```
16×16 grid = 256 block textures
Each texture: 16×16 pixels
Total atlas: 256×256 pixels (PNG)

Atlas layout (example):
┌────┬────┬────┬────┐
│Dirt│Grass│Stone│...│  Row 0
├────┼────┼────┼────┤
│Sand│Log│Leaf│... │  Row 1
├────┼────┼────┼────┤
│Coal│Iron│Gold│... │  Row 2
└────┴────┴────┴────┘

UV calculation for block ID:
atlas_x = (block_id % 16) / 16.0
atlas_y = (block_id / 16) / 16.0
uv_size = 1.0 / 16.0

Example: Stone (ID=2)
  atlas_x = 2/16 = 0.125
  atlas_y = 0/16 = 0.0
  UV range: (0.125, 0.0) to (0.1875, 0.0625)
```

#### Shader Programs

**Block Vertex Shader (`shaders/block.vert`):**
```glsl
Input:
  - position (world space)
  - tex_coords (atlas UV)
  - normal (face direction)
  - ao (ambient occlusion)

Uniforms:
  - view matrix (camera transform)
  - projection matrix (perspective)
  - model matrix (chunk offset, usually identity)

Output:
  - gl_Position (clip space)
  - Interpolated UVs, normals, world position for fragment shader
```

**Block Fragment Shader (`shaders/block.frag`):**
```glsl
Input:
  - Interpolated tex_coords
  - Interpolated normal
  - World position
  - AO value

Uniforms:
  - texture_atlas (sampler2D)
  - time (for day/night)
  - fog color
  - fog distance

Calculations:
  1. Sample texture atlas
  2. Calculate directional lighting (sun angle based on time)
  3. Apply ambient occlusion
  4. Apply distance fog
  5. Output final color
```

#### Frustum Culling

```
View frustum = 6 planes (near, far, left, right, top, bottom)

For each chunk:
  1. Compute AABB (axis-aligned bounding box)
     - Min: (chunk_x*16, 0, chunk_z*16)
     - Max: (chunk_x*16+16, 256, chunk_z*16+16)

  2. Test AABB against each frustum plane
     - If AABB is entirely outside any plane: CULL
     - Else: RENDER

Saves ~40-60% of chunk draw calls when looking in one direction
```

#### Fog Implementation

```glsl
// In fragment shader
float distance = length(camera_pos - fragment_pos);
float fog_factor = clamp((fog_end - distance) / (fog_end - fog_start), 0.0, 1.0);
final_color = mix(fog_color, texture_color, fog_factor);

// Parameters:
fog_start = render_distance * 0.7 * 16.0  // Chunks to blocks
fog_end = render_distance * 16.0
fog_color = sky_color (changes with time of day)
```

### 3. Player System (`src/player/`)

First-person controller with physics, collision detection, and interaction.

#### Files
- `mod.rs` - Player state and update loop
- `physics.rs` - Gravity, jumping, collision resolution
- `camera.rs` - First-person camera with mouse look
- `input.rs` - Input handling and state management
- `raycast.rs` - Block targeting for mining/placement

#### Player State

```rust
struct Player {
    // Transform
    position: Vec3,        // Eye position in world space
    velocity: Vec3,        // Current movement velocity
    yaw: f32,              // Horizontal rotation (radians)
    pitch: f32,            // Vertical rotation (radians, clamped)

    // Physics
    on_ground: bool,       // Touching ground (affects jump)
    is_sprinting: bool,    // Sprint state

    // Health
    health: f32,           // 0.0 to 20.0 (10 hearts)
    last_damage_time: f64, // For damage cooldown

    // Interaction
    hotbar_slot: usize,    // 0-8 selected slot
    reach_distance: f32,   // 5.0 blocks
}
```

#### Physics Simulation

```
Each frame (assuming 60 Hz):

1. Apply Input Forces
   ├─ WASD: Horizontal velocity (5.0 m/s walk, 8.0 m/s sprint)
   ├─ Sprint: Multiply by 1.6x
   └─ Jump: Vertical impulse (+10.0 m/s) if on_ground

2. Apply Gravity
   └─ velocity.y -= 32.0 * delta_time  (m/s²)

3. Apply Damping
   ├─ Horizontal: *= 0.85  (ground friction)
   └─ Vertical: *= 0.98    (air resistance)

4. Integrate Position
   └─ position += velocity * delta_time

5. Collision Detection & Response
   └─ (see below)

6. Update on_ground Flag
   └─ Test if block exists Y-0.01 below feet
```

#### Collision Detection (AABB)

**Player Bounding Box:**
```
Eye position: player.position
Feet position: player.position.y - 1.8

AABB:
  min: (pos.x - 0.3, pos.y - 1.8, pos.z - 0.3)
  max: (pos.x + 0.3, pos.y + 0.1, pos.z + 0.3)

Dimensions: 0.6×1.9×0.6 blocks (slightly thinner than 1 block)
```

**Collision Resolution (Sweep Test):**

```
For each axis (X, Y, Z) independently:
  1. Move player along axis by velocity[axis] * delta_time
  2. Get all block AABBs that might intersect
  3. For each block:
       If player AABB intersects block AABB:
         - Calculate penetration depth
         - Push player out along axis
         - Set velocity[axis] = 0
         - If axis == Y and moving down: on_ground = true

Separating axes prevents getting stuck in corners
```

**Block Query Optimization:**
```
Only test blocks near player:
  x_range = floor(player.x - 1) to ceil(player.x + 1)
  y_range = floor(player.y - 2) to ceil(player.y + 1)
  z_range = floor(player.z - 1) to ceil(player.z + 1)

Max 3×4×3 = 36 block tests per frame
```

#### Camera System

**View Matrix Construction:**
```
1. Compute forward vector from yaw/pitch:
   forward.x = cos(pitch) * sin(yaw)
   forward.y = sin(pitch)
   forward.z = cos(pitch) * cos(yaw)

2. Compute right vector:
   right = normalize(cross(forward, world_up))

3. Compute actual up vector:
   up = cross(right, forward)

4. Build view matrix:
   view = lookAt(player.position, player.position + forward, up)
```

**Projection Matrix:**
```
Perspective projection:
  fov = 70° (vertical)
  aspect = window_width / window_height
  near = 0.1
  far = render_distance * 16.0 * 1.5

projection = perspective(fov, aspect, near, far)
```

**Mouse Input:**
```
On mouse move (delta_x, delta_y):
  yaw += delta_x * sensitivity  // No clamping (360° rotation)
  pitch += delta_y * sensitivity
  pitch = clamp(pitch, -89°, +89°)  // Prevent flip

sensitivity = 0.002 (radians per pixel)
```

#### Raycasting (Block Selection)

**DDA Algorithm (Digital Differential Analyzer):**
```
Start: player.position
Direction: camera forward vector
Max distance: 5.0 blocks

Step through voxel grid:
  1. Initialize at player position
  2. Calculate step direction (+1 or -1 per axis)
  3. Calculate tMax (distance to next voxel boundary per axis)
  4. Calculate tDelta (distance between boundaries per axis)

  Loop:
    - Find axis with smallest tMax (next boundary)
    - Step to next voxel along that axis
    - Check if voxel contains solid block
    - If solid: RETURN (block position, face normal)
    - If distance > max: RETURN none
    - tMax[axis] += tDelta[axis]

Returns: Option<(BlockPos, Face)>
```

**Use Cases:**
- **Block Breaking:** Get targeted block position, remove it
- **Block Placement:** Get targeted block + face, place on adjacent position
- **Visual Highlight:** Render wireframe around targeted block

#### Fall Damage

```
On collision with ground (velocity.y < 0):
  fall_speed = abs(velocity.y)

  if fall_speed > 10.0:  // ~3 block fall
    damage = (fall_speed - 10.0) * 0.5
    player.health -= damage

    if player.health <= 0.0:
      trigger_death()
```

### 4. Inventory & Crafting (`src/inventory/`)

Item management and crafting system.

#### Files
- `mod.rs` - Inventory container and operations
- `crafting.rs` - Recipe matching and crafting logic
- `item.rs` - Item type definitions and properties

#### Inventory Structure

```
Hotbar: 9 slots (index 0-8)
Storage: 27 slots (index 9-35)
Crafting Grid: 9 slots (3×3, index 36-44)
Crafting Output: 1 slot (index 45)

Total: 46 slots

Slot struct:
  item_type: Option<ItemType>
  count: u32  (max stack size: 64 for most items, 1 for tools)
```

#### Crafting System

**Recipe Matching:**
```
Pattern-based recipes:
  Input: 3×3 grid of Option<ItemType>
  Output: ItemType + count

Example: Wooden Pickaxe
  Pattern:
    [Some(Plank), Some(Plank), Some(Plank)]
    [None,        Some(Stick), None       ]
    [None,        Some(Stick), None       ]

  Output: WoodenPickaxe × 1

Shapeless recipes (order doesn't matter):
  Input: List of ItemType
  Example: 4 planks (any position) → Crafting Table
```

**Recipe Database:**
```rust
Recipes (examples):
  - Log → 4 Planks (shapeless)
  - 2 Planks (vertical) → 4 Sticks (shaped)
  - 3 Planks + 2 Sticks → Wooden Pickaxe (shaped)
  - 3 Cobblestone + 2 Sticks → Stone Pickaxe (shaped)
  - 3 Iron + 2 Sticks → Iron Pickaxe (shaped)
  - etc.

Total recipes: ~20-30 core recipes
```

**Crafting Loop:**
```
On crafting grid change:
  1. Read 3×3 grid state
  2. Check all recipes for match
  3. If match found:
       - Display output in result slot
  4. If no match:
       - Clear result slot

On result slot click (with match):
  1. Remove ingredients from grid
  2. Give player the output item
  3. Re-run recipe check (for repeated crafting)
```

#### Tool Durability

```rust
Tool {
  item_type: ItemType,
  durability: u32,      // Current durability
  max_durability: u32,  // Max durability
}

Durability by tier:
  - Wood: 60 uses
  - Stone: 132 uses
  - Iron: 251 uses
  - Gold: 33 uses (fast but fragile)
  - Diamond: 1562 uses

On block break:
  if tool appropriate for block:
    tool.durability -= 1
    if tool.durability == 0:
      break_tool()  // Remove from inventory, play sound
```

#### Mining Speed

```rust
Block hardness (time to break by hand):
  - Dirt: 0.5s
  - Stone: 7.5s
  - Ore: 15.0s
  - Bedrock: ∞

Tool effectiveness multiplier:
  - Correct tool tier: 5× to 30× faster
  - Wrong tool: 1× (hand speed)

Example: Stone block
  - Hand: 7.5s
  - Wooden pickaxe: 1.5s (5× faster)
  - Stone pickaxe: 0.75s (10× faster)
  - Diamond pickaxe: 0.25s (30× faster)
```

### 5. Mob System (`src/mobs/`)

AI-driven entities that spawn, move, and interact with the player.

#### Files
- `mod.rs` - Mob manager and update loop
- `mob.rs` - Mob entity structure
- `ai.rs` - Behavior AI (pathfinding, targeting)
- `spawning.rs` - Spawn rules and logic

#### Mob Types

**Passive Mob (Pig):**
```rust
Pig {
  position: Vec3,
  velocity: Vec3,
  health: 10.0,

  // AI state
  wander_target: Option<Vec3>,
  wander_timer: f32,

  // Properties
  speed: 2.0 m/s,
  drop_on_death: Porkchop,
}
```

**Hostile Mob (Zombie):**
```rust
Zombie {
  position: Vec3,
  velocity: Vec3,
  health: 20.0,

  // AI state
  target: Option<EntityId>,  // Player ID when chasing
  attack_cooldown: f32,

  // Properties
  speed: 3.5 m/s,
  damage: 3.0,
  attack_range: 1.5 blocks,
  detection_range: 16 blocks,
  drop_on_death: RottenFlesh,
}
```

#### AI Behavior Trees

**Pig (Passive) Behavior:**
```
Every frame:
  ├─ If wander_timer expired:
  │   ├─ Pick random nearby position (±8 blocks XZ)
  │   ├─ Set as wander_target
  │   └─ Reset timer (5-10 seconds)
  │
  ├─ If wander_target exists:
  │   ├─ Move toward target
  │   └─ If reached: clear target
  │
  └─ Apply physics (gravity, collision)
```

**Zombie (Hostile) Behavior:**
```
Every frame:
  ├─ If no target:
  │   ├─ Search for player within detection_range
  │   └─ If found: set as target
  │
  ├─ If target exists:
  │   ├─ Check if target still valid (alive, in range)
  │   │
  │   ├─ If distance > detection_range * 1.5:
  │   │   └─ Clear target (lost interest)
  │   │
  │   ├─ If distance ≤ attack_range:
  │   │   ├─ If attack_cooldown ready:
  │   │   │   ├─ Deal damage to player
  │   │   │   └─ Reset cooldown (1 second)
  │   │   └─ Stop moving
  │   │
  │   └─ Else:
  │       └─ Pathfind toward target
  │
  └─ Apply physics
```

#### Pathfinding

**Simplified A\* (2D, ignores Y):**
```
Goal: Navigate from mob position to target position

Algorithm:
  1. Discretize to chunk grid (ignore sub-block precision)
  2. A* search on XZ plane:
     - Cost = distance + heuristic (Euclidean to target)
     - Passable = !is_solid_block(x, y, z) where y = ground level
  3. Return waypoint list
  4. Mob follows waypoints sequentially

Optimizations:
  - Only recalculate path every 0.5s (not every frame)
  - Cache path until target moves significantly
  - Limit search depth (max 64 nodes)
  - If path not found: move directly toward target (may get stuck)
```

**Obstacle Avoidance:**
```
If blocked by block:
  - Try jump (if block is 1 high)
  - If can't jump: recalculate path

If stuck for >3 seconds:
  - Teleport to last valid position
```

#### Spawning Rules

**Spawn Conditions:**
```
Zombie spawning:
  - Light level ≤ 7 (dark areas or night)
  - Y ≥ 50 (not deep underground for initial impl)
  - Distance from player: 24-64 blocks (spawn ring)
  - Max zombies in world: 20
  - Spawn rate: Try every 2 seconds

Pig spawning:
  - Light level ≥ 9 (daytime or well-lit)
  - On grass block
  - Distance from player: 24-64 blocks
  - Max pigs in world: 15
  - Spawn rate: Try every 5 seconds
```

**Light Level Calculation:**
```
Simplified lighting:
  - Sky light: 15 (day), 4 (night), interpolated during sunrise/sunset
  - Block light: Not implemented initially (would require light propagation)

  light_level = sky_light * (1.0 - block_depth_factor)

  For spawning purposes:
    If no block above: use sky_light
    Else: light_level = 0 (consider "dark")
```

#### Mob Rendering

**Simple Geometry:**
```
Pig:
  - Body: 1.0 × 0.8 × 1.0 box (textured cube)
  - Head: 0.6 × 0.6 × 0.6 box (offset forward)
  - Legs: 4 × (0.2 × 0.5 × 0.2) boxes

Zombie:
  - Body: 0.6 × 1.2 × 0.4 box (humanoid proportions)
  - Head: 0.5 × 0.5 × 0.5 box
  - Arms: 2 × (0.2 × 1.0 × 0.2) boxes
  - Legs: 2 × (0.2 × 1.0 × 0.2) boxes

Animation:
  - Simple rotation/offset based on movement
  - Walk cycle: sine wave applied to leg rotation
```

### 6. UI System (`src/ui/`)

Heads-up display, menus, and 2D rendering.

#### Files
- `mod.rs` - UI manager and layout
- `hud.rs` - Crosshair, hotbar, health display
- `inventory_screen.rs` - Inventory and crafting UI
- `debug.rs` - F3 debug overlay

#### UI Rendering Approach

**Orthographic Projection:**
```
Screen space (pixels):
  Origin (0,0) = top-left
  X increases right
  Y increases down

Projection matrix:
  orthographic(0, screen_width, screen_height, 0, -1, 1)

UI rendered last (after 3D world), depth test disabled
```

#### HUD Elements

**Crosshair:**
```
Position: screen center (width/2, height/2)
Size: 16×16 pixels
Texture: Simple '+' shape, white with black outline
Always visible
```

**Hotbar:**
```
Position: Bottom-center (width/2 - 90, height - 40)
Size: 9 slots × 20 pixels = 180×20 pixels
Each slot:
  - Background: Gray square
  - Item icon: 16×16 texture
  - Selected slot: Yellow border
  - Stack count: Number overlay (if > 1)
```

**Health:**
```
Position: Top-left (10, 10)
Layout: Row of heart icons
Full heart: Red texture
Half heart: Half-red texture
Empty heart: Black outline
Max: 10 hearts = 20 HP
```

**Debug Overlay (F3):**
```
Position: Top-left (10, 40)
Text info:
  FPS: 60
  Position: X=123.45, Y=64.00, Z=-98.76
  Chunk: (7, -7)
  Facing: North (yaw=0.0°, pitch=-15.0°)
  Looking at: Stone (123, 64, -99)
  Biome: Plains
  Light level: 15

Font: Monospace, 12pt
Background: Semi-transparent black
```

#### Inventory Screen

**Layout:**
```
┌──────────────────────────────────┐
│       Crafting Grid (3×3)        │
│  ┌───┬───┬───┐      ┌───┐       │
│  │   │   │   │  ==> │   │       │
│  ├───┼───┼───┤      └───┘       │
│  │   │   │   │     Output        │
│  ├───┼───┼───┤                   │
│  │   │   │   │                   │
│  └───┴───┴───┘                   │
│                                   │
│       Storage (3×9 grid)         │
│  ┌───┬───┬───┬───┬───┬───┬───┐  │
│  │   │   │   │   │   │   │   │  │
│  ├───┼───┼───┼───┼───┼───┼───┤  │
│  │   │   │   │   │   │   │   │  │
│  ├───┼───┼───┼───┼───┼───┼───┤  │
│  │   │   │   │   │   │   │   │  │
│  └───┴───┴───┴───┴───┴───┴───┘  │
│                                   │
│       Hotbar (1×9 row)           │
│  ┌───┬───┬───┬───┬───┬───┬───┐  │
│  │ 1 │ 2 │ 3 │ 4 │ 5 │ 6 │ 7 │  │
│  └───┴───┴───┴───┴───┴───┴───┘  │
└──────────────────────────────────┘

Interaction:
  - Left click: Pick up entire stack
  - Right click: Pick up half stack / place one item
  - Drag: Move stack
  - Shift+click: Quick transfer
```

### 7. Game Loop (`src/main.rs`)

Main entry point and frame orchestration.

```
Initialization:
  ├─ Create window (glutin/winit)
  ├─ Initialize OpenGL context
  ├─ Load shaders
  ├─ Load texture atlas
  ├─ Initialize world
  ├─ Spawn player
  └─ Enter main loop

Main Loop (target 60 FPS):
  ┌─ Frame Start
  │   └─ Calculate delta_time
  │
  ├─ Event Processing
  │   ├─ Window events (resize, close)
  │   ├─ Keyboard events (WASD, E, F3, ESC, etc.)
  │   └─ Mouse events (movement, clicks, scroll)
  │
  ├─ Update Phase (fixed timestep)
  │   ├─ Update player (physics, input)
  │   ├─ Update mobs (AI, physics)
  │   ├─ Update world (chunk loading/unloading)
  │   ├─ Update day/night cycle
  │   ├─ Process block breaking/placing
  │   └─ Check win/death conditions
  │
  ├─ Render Phase
  │   └─ (see Rendering Pipeline above)
  │
  ├─ Frame End
  │   └─ Swap buffers, cap to 60 FPS
  │
  └─ Loop until quit
```

## Data Flow Diagrams

### Block Breaking Flow

```
Player Input (Left Click Held)
           │
           ▼
    ┌─────────────┐
    │  Raycast    │ → No block hit → Nothing
    └──────┬──────┘
           │ Block hit
           ▼
    ┌─────────────┐
    │ Check Tool  │
    │ Efficiency  │
    └──────┬──────┘
           │
           ▼
    ┌─────────────────┐
    │ Accumulate Time │ (break_progress += dt / break_time)
    └──────┬──────────┘
           │
           ▼
    break_progress >= 1.0?
           │
           ├─ No → Render break animation
           │
           └─ Yes ↓
                ┌──────────────┐
                │ Remove Block │
                │ from Chunk   │
                └──────┬───────┘
                       │
                       ▼
                ┌──────────────┐
                │ Mark Chunk   │
                │ Dirty (remesh)│
                └──────┬───────┘
                       │
                       ▼
                ┌──────────────┐
                │ Spawn Item   │
                │ Drop Entity  │
                └──────┬───────┘
                       │
                       ▼
                ┌──────────────┐
                │ Damage Tool  │
                │ Durability   │
                └──────────────┘
```

### Chunk Loading Flow

```
Player Moves
     │
     ▼
┌──────────────────┐
│ Calculate New    │
│ Required Chunks  │
│ (spiral pattern) │
└────────┬─────────┘
         │
         ▼
  ┌─────────────┐     ┌──────────────┐
  │ Unload Far  │────▶│ Free Mesh    │
  │ Chunks      │     │ Data         │
  └─────────────┘     └──────────────┘
         │
         ▼
  ┌─────────────┐
  │ For Each    │
  │ New Chunk   │
  └──────┬──────┘
         │
         ├─ Spawn Generation Task
         │         │
         │         ▼
         │  ┌─────────────┐
         │  │ Generate    │
         │  │ Terrain     │
         │  │ (noise)     │
         │  └──────┬──────┘
         │         │
         │         ▼
         │  ┌─────────────┐
         │  │ Place       │
         │  │ Features    │
         │  │ (trees,ores)│
         │  └──────┬──────┘
         │         │
         │         ▼
         │  ┌─────────────┐
         │  │ Generate    │
         │  │ Mesh        │
         │  │ (greedy)    │
         │  └──────┬──────┘
         │         │
         └─────────┼────────▶ Chunk Ready
                   │
                   ▼
            ┌────────────┐
            │ Upload to  │
            │ GPU (VBO)  │
            └────────────┘
```

## Performance Considerations

### Critical Hot Paths

1. **Mesh Generation** (most expensive)
   - Runs when chunk generated or modified
   - Greedy meshing reduces vertices by 95%+
   - Amortize over multiple frames if needed
   - Cache generated meshes until chunk modified

2. **Chunk Rendering** (every frame)
   - Frustum culling reduces draw calls by 40-60%
   - VBO batching (one draw call per chunk)
   - Texture atlas eliminates texture binding overhead

3. **Collision Detection** (every frame, per entity)
   - Only test nearby blocks (3×4×3 = 36 max)
   - Early-out on non-solid blocks
   - Sweep test on each axis separately

4. **Mob AI** (every frame, per mob)
   - Limit pathfinding to 0.5s intervals
   - Cap max mobs (20 hostile, 15 passive)
   - Simple state machines, not complex trees

### Memory Budget

```
Chunk data:
  - Block storage: 65,536 bytes (16×256×16)
  - Mesh vertices: ~4KB average (2K-8K quads)
  - Total per chunk: ~70 KB

With 17×17 chunks loaded (8 render distance):
  - 289 chunks × 70 KB = ~20 MB
  - Acceptable for modern systems

GPU memory:
  - Texture atlas: 256×256 RGBA = 256 KB
  - Chunk VBOs: 289 × 4KB = ~1.2 MB
  - Total: <2 MB (negligible)
```

### Target Performance

| Hardware        | Render Dist | Expected FPS |
|-----------------|-------------|--------------|
| Integrated GPU  | 6 chunks    | 30-60 FPS    |
| Mid-range GPU   | 8 chunks    | 60-90 FPS    |
| High-end GPU    | 12 chunks   | 144+ FPS     |

### Bottleneck Mitigation

**CPU-Bound (mesh generation):**
- Async chunk generation on thread pool
- Limit remeshes per frame (max 4)
- Only remesh dirty chunks

**GPU-Bound (draw calls):**
- Frustum culling
- Occlusion culling (optional, advanced)
- Level-of-detail (optional, show distant chunks as low-poly)

**Memory-Bound:**
- Unload chunks beyond render distance + 2
- Stream chunk data (don't keep all in RAM)

## Key Design Decisions

### 1. Chunk Size: 16×256×16

**Rationale:**
- Matches Minecraft (familiar, proven)
- 16 = power of 2 (efficient bitwise ops)
- 256 = full height, no vertical chunking complexity
- Good balance: small enough for fast mesh gen, large enough to batch

**Alternative considered:** 32×128×32
- Fewer chunks to manage (simpler)
- But: 4× vertices per mesh (slower), less granular loading

### 2. Greedy Meshing

**Rationale:**
- Naive approach: 1.5M vertices per chunk (unplayable)
- Greedy meshing: 2K-8K vertices (95%+ reduction)
- Critical for 60 FPS target

**Alternative considered:** Marching cubes
- Smoother terrain (no blocky aesthetic)
- But: not Minecraft-style, more complex implementation

### 3. Single Texture Atlas

**Rationale:**
- One texture bind per chunk (vs 15+ for individual textures)
- Simpler shader (no texture array indexing)
- Easy to pack in 256×256 (supports 256 block types)

**Alternative considered:** Texture array
- More flexible (unlimited textures)
- But: Requires OpenGL 3.0+, more complex shader

### 4. Simplified Lighting

**Rationale:**
- Full light propagation is complex (flood fill, multiple passes)
- Day/night cycle + AO provides adequate visual quality
- Ambient occlusion handled per-vertex during meshing

**Alternative considered:** Full light engine
- Better visuals (realistic shadows)
- But: Significant complexity, performance cost

### 5. 2D Pathfinding (Ignoring Y)

**Rationale:**
- Mobs don't need to navigate complex 3D caves in initial version
- 2D A\* is fast and simple
- Good enough for surface navigation

**Alternative considered:** 3D A\*
- Handles caves, climbing
- But: Slower, more complex, diminishing returns for initial scope

### 6. Raw OpenGL Over wgpu/Vulkan

**Rationale:**
- OpenGL 3.3 widely supported (even integrated GPUs)
- Simpler API than Vulkan (fewer lines of code)
- Learning value (understanding graphics pipeline)

**Alternative considered:** wgpu
- Modern, safe API
- But: Adds abstraction layer (against "from scratch" requirement)

### 7. ECS vs Direct Entity Management

**Rationale:**
- Direct structs (Player, Mob) for simplicity
- Small entity count (player + ~35 mobs) doesn't need ECS
- Avoid overengineering for small scope

**Alternative considered:** ECS (e.g., hecs, bevy_ecs)
- More scalable for large entity counts
- But: Overkill for this project, adds dependency

## Testing Strategy

### Unit Testing
```rust
#[cfg(test)]
mod tests {
    // Coordinate conversion
    test_block_to_chunk_coords()
    test_chunk_to_block_coords()

    // Collision
    test_aabb_intersection()
    test_sweep_collision()

    // Raycasting
    test_raycast_hit_detection()
    test_raycast_face_normal()

    // Crafting
    test_recipe_matching()
    test_shapeless_recipes()

    // Inventory
    test_stack_splitting()
    test_item_transfer()
}
```

### Integration Testing
- Launch game, verify no panic
- Load 10+ chunks, check FPS
- Break/place 100 blocks, verify world state
- Craft items through chain (log→plank→stick→tool)
- Spawn mobs, verify AI behavior

### Performance Benchmarks
```bash
cargo bench
  - Chunk generation: <50ms target
  - Mesh generation: <20ms target
  - Collision detection: <1ms per entity
  - Raycast: <0.1ms
```

### Manual Playtest Checklist
- [ ] Spawn in world without crash
- [ ] Walk around, collision works (no clipping)
- [ ] Chunks load/unload smoothly
- [ ] Break blocks (all types)
- [ ] Place blocks (all types)
- [ ] Craft wooden pickaxe
- [ ] Craft stone pickaxe
- [ ] Mine stone with pickaxe
- [ ] Pigs spawn and wander
- [ ] Zombies spawn at night and chase player
- [ ] Take damage from zombie
- [ ] Take fall damage
- [ ] Die and respawn
- [ ] Day/night cycle completes (~10 min)
- [ ] FPS stays >30 throughout

## Future Enhancements (Out of Scope)

- World saving/loading (persistence)
- Multiplayer (networking)
- Advanced lighting (light propagation, colored lights)
- More biomes (desert, forest, snow)
- More mobs (creepers, skeletons, etc.)
- Enchanting and potions
- The Nether and End dimensions
- Redstone logic
- Flowing water and lava
- Sound and music
- Advanced graphics (PBR, shadows, reflections)
- Modding API

---

This architecture balances **performance**, **simplicity**, and **playability** to deliver a Minecraft-like experience from scratch in Rust.
