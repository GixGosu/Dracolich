# VoxelCraft - A Minecraft-Style Voxel Game in Rust

A fully playable voxel-based sandbox game built from scratch in Rust using raw OpenGL. Explore infinite procedurally-generated terrain, mine resources, craft tools, build structures, and survive against hostile mobs.

## Features

### World Generation
- Infinite procedurally-generated terrain using Perlin/Simplex noise
- Varied biomes with plains, hills, and mountains
- Natural cave systems carved into terrain
- Procedurally-placed trees with logs and leaves
- Water bodies at sea level
- Bedrock layer (indestructible bottom boundary)
- Dynamic chunk loading and unloading based on player position

### Block Types
15+ distinct block types with unique textures:
- Natural: Grass, Dirt, Stone, Sand, Gravel, Bedrock
- Organic: Oak Log, Oak Leaves, Oak Planks
- Ores: Coal Ore, Iron Ore, Gold Ore, Diamond Ore
- Liquids: Water
- Crafted: Wooden Planks, Sticks

### Rendering Engine
- Custom OpenGL 3.3+ renderer
- Texture atlas system for efficient block rendering
- Greedy meshing for hidden face culling
- Frustum culling for chunk optimization
- Distance fog for atmosphere
- Day/night cycle (~10 minutes real-time)
- Dynamic sky rendering with sun and moon
- Smooth lighting transitions
- Targeted block highlighting

### Player Mechanics
- First-person camera with mouse look
- WASD movement with sprint capability
- Jump mechanics with gravity simulation
- Proper AABB collision detection (no clipping through blocks)
- Fall damage system
- Health and death mechanics
- Respawn system

### Block Interaction
- Raycasting-based block selection
- Block breaking with visual progress
- Mining speed affected by tool type and block hardness
- Block placement with proper face detection
- Visual highlight on targeted block
- 9-slot hotbar for quick access

### Inventory & Crafting
- Grid-based inventory system (9 hotbar slots + 27 storage slots)
- Toggle inventory screen (E key)
- 3x3 crafting grid
- Crafting recipes:
  - Oak Log → Oak Planks (4 planks per log)
  - Oak Planks → Sticks (4 sticks per 2 planks)
  - Sticks + Planks → Wooden Pickaxe
  - Sticks + Cobblestone → Stone Pickaxe
  - Additional recipes for tools and items
- Tool durability system
- Tool tier progression (wood → stone → iron → gold → diamond)

### Mobs
- **Passive Mob (Pig):** Wanders randomly, drops porkchop on death
- **Hostile Mob (Zombie):** Spawns in darkness, pathfinds to player, deals contact damage
- Light-level based spawning rules
- Day/night spawning mechanics
- Simple but functional AI behavior

### User Interface
- Centered crosshair
- Hotbar with item/block display and selection indicator
- Heart-based health display
- Debug overlay (F3) showing:
  - Player position (X, Y, Z)
  - Chunk coordinates
  - Current FPS
  - Facing direction
  - Currently targeted block
- Pause menu (ESC)

## System Requirements

### Minimum Requirements
- **OS:** Windows 10/11, Linux (Ubuntu 20.04+), macOS 10.15+
- **CPU:** Dual-core 2.0 GHz
- **RAM:** 4 GB
- **GPU:** OpenGL 3.3+ compatible graphics card (integrated graphics acceptable)
- **Storage:** 200 MB

### Recommended Requirements
- **OS:** Windows 11, Linux (Ubuntu 22.04+), macOS 12+
- **CPU:** Quad-core 3.0 GHz
- **RAM:** 8 GB
- **GPU:** Dedicated graphics card with OpenGL 4.5+ support
- **Storage:** 500 MB

## Building from Source

### Prerequisites
- Rust toolchain (1.75.0 or later)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### Linux-Specific Dependencies
```bash
# Ubuntu/Debian
sudo apt-get install libgl1-mesa-dev libxcursor-dev libxi-dev libxrandr-dev

# Fedora
sudo dnf install mesa-libGL-devel libXcursor-devel libXi-devel libXrandr-devel
```

### Build Commands
```bash
# Clone or navigate to the project directory
cd voxelcraft

# Build in release mode (optimized)
cargo build --release

# Run the game
cargo run --release

# Build and run in one command
cargo run --release
```

**Note:** Release mode is critical for playable performance. Debug builds will run at <10 FPS.

### Build Time
- First build: 5-10 minutes (depending on system)
- Subsequent builds: 30-60 seconds

## Controls

### Movement
- **W** - Move forward
- **A** - Move left
- **S** - Move backward
- **D** - Move right
- **Space** - Jump
- **Left Shift** - Sprint (hold while moving)
- **Mouse** - Look around (first-person camera)

### Block Interaction
- **Left Click** - Break block (hold to mine)
- **Right Click** - Place block
- **Scroll Wheel** - Cycle through hotbar slots
- **1-9 Number Keys** - Select hotbar slot directly

### Interface
- **E** - Open/close inventory and crafting menu
- **F3** - Toggle debug overlay
- **ESC** - Pause menu
- **Left Click (in UI)** - Select/move items
- **Right Click (in UI)** - Split stack / place single item

### Camera
- **Mouse Movement** - Rotate camera (uncapped sensitivity)
- Mouse sensitivity can be adjusted in code (default: 0.002)

## Gameplay Guide

### Getting Started

1. **Spawn:** You'll spawn in a randomly generated world on the surface
2. **Look Around:** Use the mouse to orient yourself and find trees
3. **Gather Wood:**
   - Look at a tree trunk (Oak Log)
   - Hold left click to break it
   - Walk over the dropped item to collect it
4. **Open Inventory:** Press E to access your inventory and crafting grid

### Basic Crafting Chain

1. **Logs → Planks:**
   - Place Oak Log in any crafting grid slot
   - Take 4 Oak Planks from output

2. **Planks → Sticks:**
   - Stack 2 Oak Planks vertically in crafting grid
   - Take 4 Sticks from output

3. **Craft Wooden Pickaxe:**
   ```
   [Plank] [Plank] [Plank]
   [     ] [Stick] [     ]
   [     ] [Stick] [     ]
   ```

4. **Mine Stone:**
   - Use wooden pickaxe to mine stone blocks
   - Collect Cobblestone

5. **Craft Stone Pickaxe:**
   ```
   [Cobble] [Cobble] [Cobble]
   [      ] [Stick ] [      ]
   [      ] [Stick ] [      ]
   ```

### Survival Tips

- **Find Shelter Before Night:** Hostile mobs spawn in darkness
- **Light Sources:** Torches prevent mob spawning (if implemented)
- **Watch Your Health:** Hearts in the top-left corner show your health
- **Fall Damage:** Falls > 3 blocks cause damage; > 20 blocks can be fatal
- **Mining Strategy:** Mine downward to find ores (coal, iron, gold, diamond)
- **Tool Efficiency:** Better tools mine faster and last longer
- **Respawn:** If you die, you'll respawn at the world spawn point

### Building

1. Select a block type in your hotbar
2. Look at the face of an existing block where you want to place
3. Right-click to place the block
4. Build walls, floors, roofs to create shelter

### Combat

- Hostile mobs (zombies) spawn at night or in dark areas
- They will pathfind toward you and deal damage on contact
- Create distance or build barriers to protect yourself
- Killing mobs drops items

## Performance Optimization

### Render Distance
The default render distance is 8 chunks. If experiencing low FPS:
- Reduce render distance in `src/config.rs`
- Lower values: 6 chunks (better performance), 4 chunks (minimum viable)
- Higher values: 12 chunks (requires strong GPU), 16 chunks (high-end systems)

### Graphics Settings (Code-Level)
Located in `src/renderer/mod.rs`:
- Fog density: Controls visibility fade-out distance
- Vsync: Can be disabled for higher FPS (may cause tearing)
- Chunk mesh update rate: Affects smoothness of terrain generation

### Expected Performance
- **High-end GPU:** 144+ FPS at 12-16 chunk distance
- **Mid-range GPU:** 60-90 FPS at 8 chunk distance
- **Integrated graphics:** 30-60 FPS at 6 chunk distance
- **Low-end systems:** 30 FPS at 4 chunk distance

## Known Limitations

### Scope Limitations
- **Water Physics:** Water is static (no flowing water simulation)
- **Redstone/Logic:** No electrical or logic systems
- **Advanced Mobs:** Limited mob variety (pig and zombie only)
- **Biomes:** Single biome type with terrain variation
- **Multiplayer:** Single-player only
- **Sound:** No audio or music (can be added with `rodio` crate)
- **Advanced Lighting:** Simplified lighting model (no smooth shadows)
- **Weather:** No rain, snow, or weather effects
- **World Saving:** No persistence (world regenerates on restart)
- **Villages/Structures:** No generated structures beyond trees

### Technical Limitations
- **Chunk Borders:** Rare visual seams between chunks during loading
- **Mob AI:** Basic pathfinding (mobs can get stuck on obstacles)
- **Inventory Stacking:** Fixed stack size, no customization
- **Floating Point Precision:** Minor jitter at extreme distances (>1,000,000 blocks)
- **Crafting Recipes:** Limited recipe set (15-20 core recipes)

### Platform-Specific Notes
- **macOS:** May require manual OpenGL context setup on newer versions
- **Linux Wayland:** Use X11 backend for better compatibility
- **Windows:** Optimal platform, no known issues

## Troubleshooting

### Game Won't Launch
- Verify OpenGL 3.3+ support: `glxinfo | grep "OpenGL version"` (Linux)
- Update graphics drivers
- Check console output for specific error messages

### Low FPS
- Run with `--release` flag (debug mode is 10-20x slower)
- Reduce render distance in configuration
- Close background applications
- Verify GPU is being used (not integrated graphics)

### Black Screen
- Shader compilation may have failed
- Check `shaders/` directory exists with `.vert` and `.frag` files
- Review console for OpenGL errors

### Crashes on Startup
- Ensure all dependencies are installed (Linux)
- Check Rust version: `rustc --version` (should be 1.75.0+)
- Try: `cargo clean && cargo build --release`

### Mouse Not Working
- Game captures mouse on launch
- Press ESC to release mouse cursor
- Some window managers may interfere with mouse capture

## Project Structure

```
voxelcraft/
├── src/                    # Source code
│   ├── main.rs            # Entry point
│   ├── world/             # World generation and management
│   ├── renderer/          # OpenGL rendering engine
│   ├── player/            # Player controller and physics
│   ├── inventory/         # Inventory and crafting systems
│   ├── mobs/              # Mob AI and behavior
│   └── ui/                # User interface rendering
├── shaders/               # GLSL shader programs
│   ├── block.vert         # Block vertex shader
│   ├── block.frag         # Block fragment shader
│   └── ...
├── assets/                # Texture atlas and resources
├── Cargo.toml             # Rust dependencies
├── README.md              # This file
├── ARCHITECTURE.md        # Technical architecture documentation
└── BUILD_LOG.md           # Development log and decisions
```

## Credits

Built entirely from scratch in Rust without game engines or voxel libraries.

### Core Dependencies
- `gl` - OpenGL bindings
- `glam` - Mathematics library
- `noise` - Procedural noise generation
- `winit` - Cross-platform windowing
- `image` - Texture loading

### Inspiration
- Minecraft (Mojang Studios)
- Minetest (open-source voxel engine)
- Various voxel rendering techniques from community research

## License

This project is provided as-is for educational and entertainment purposes.

---

**Have fun building and exploring!** 🎮⛏️🌍
