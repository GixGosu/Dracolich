# Build Minecraft From Scratch

Build a fully playable Minecraft-style voxel game from scratch in Rust. No game engine. No voxel libraries. The player must be able to walk around an infinite procedurally-generated world, place blocks, break blocks, and survive.

## Technical Constraints

- **Language:** Rust (latest stable)
- **Graphics:** Raw OpenGL 3.3+ (via `gl` crate + windowing crate of your choice)
- **No game engines.** No Bevy, no Macroquad, no Piston, no wgpu abstractions, no voxel crates.
- **Utility crates are allowed:** math (`glam`), noise (`noise`), image loading (`image`), windowing (`winit`, `glfw-rs`, `glutin`), audio (`rodio`). Standard Rust ecosystem tools only.
- **Must compile with `cargo build --release` and run.**

## What the Game Must Do

### World
- Infinite procedurally-generated terrain using noise functions
- Varied terrain: plains, hills, mountains — not flat
- Chunk-based world architecture
- Dynamic chunk loading/unloading as the player moves
- Caves carved into terrain
- Trees
- Water at a fixed sea level (non-flowing is fine)
- At least 15 distinct block types with unique textures (grass, dirt, stone, sand, wood, leaves, ores, etc.)
- Bedrock floor (indestructible)

### Rendering
- Textured blocks with a texture atlas
- Don't render hidden faces between adjacent solid blocks
- Frustum culling
- Distance fog
- Sky rendering with sun/moon
- Day/night cycle (~10 minutes real time) with lighting changes
- 30+ FPS with 8+ chunk render distance

### Player
- First-person camera (mouse look + WASD)
- Collision detection against the voxel grid (no clipping through walls or floors)
- Gravity and jumping
- Fall damage
- Health system (player can die)
- Respawn on death

### Block Interaction
- Raycasting to determine which block the player is looking at
- Block breaking (left click, time based on block hardness and equipped tool)
- Block placement (right click, place on face of targeted block)
- Visual feedback on targeted block (highlight/wireframe)
- Hotbar with block/item selection

### Inventory & Crafting
- Player inventory (hotbar + grid storage)
- Inventory screen toggle
- Crafting system with grid-based recipes
- At minimum: logs→planks, planks→sticks, sticks+planks→wooden pickaxe, sticks+cobblestone→stone pickaxe
- Tool durability
- Tool tiers affect mining speed

### Mobs
- At least one passive mob (wanders, drops item on death)
- At least one hostile mob (spawns in dark/at night, pathfinds toward player, deals damage on contact)
- Mob spawning rules based on light level and time of day
- Mob rendering (simple geometry is acceptable — boxes, capsules)

### UI
- Crosshair
- Hotbar display
- Health bar
- Debug overlay (F3): position, chunk coordinates, FPS, facing direction
- Pause menu (ESC)

## Deliverables

```
output/[project]/
├── src/                        # All source code
├── shaders/                    # GLSL shaders
├── Cargo.toml                  # Build configuration
├── README.md                   # Build instructions, controls, gameplay description
├── ARCHITECTURE.md             # How the systems are organized and why
└── BUILD_LOG.md                # What worked, what didn't, what was hardest
```

## Success Criteria

The game is successful if a player can:
1. Launch it with `cargo run --release`
2. Spawn into a procedurally-generated 3D voxel world
3. Walk around with proper collision (no clipping)
4. See new terrain generate as they explore
5. Break blocks and place blocks
6. Craft basic tools from harvested materials
7. Experience a day/night cycle
8. Encounter and fight (or flee from) hostile mobs
9. Die and respawn
10. Build a shelter

## Constraints

- No game engines or voxel libraries.
- Must compile and run. Code that doesn't build is worth nothing.
- Performance matters. Unplayable framerates are a failure.
- The game must be playable, not just a tech demo. A player should be able to spend 30 minutes in this world doing Minecraft-like things.
