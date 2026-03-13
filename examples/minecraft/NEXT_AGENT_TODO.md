# Next Agent Integration Checklist

## Chunk System Status: ✅ COMPLETE

The chunk system is fully implemented and ready for integration.

## Your Mission

Integrate the chunk system into the main game loop and connect it with player controls.

## Prerequisites (Already Done ✅)

- ✅ OpenGL renderer (OPENGL_RENDERER agent)
- ✅ Type definitions (PROJECT_ARCHITECT agent)
- ✅ Chunk system (CHUNK_SYSTEM agent - this deliverable)
- ✅ Terrain generation (previous agent)

## Integration Steps

### 1. Update `src/main.rs` Game Loop

Add these fields to your game state:
```rust
use crate::world::{World, ChunkManager, mesh_chunk};
use std::collections::HashMap;

struct Game {
    world: World,
    chunk_manager: ChunkManager,
    chunk_meshes: HashMap<ChunkPos, ChunkMesh>,
    // ... existing fields (renderer, camera, player, etc.)
}
```

### 2. In `Game::new()`

```rust
impl Game {
    fn new() -> Self {
        Self {
            world: World::new(rand::random()), // Or fixed seed
            chunk_manager: ChunkManager::new(RENDER_DISTANCE),
            chunk_meshes: HashMap::new(),
            // ... other initialization
        }
    }
}
```

### 3. In `Game::update()`

```rust
fn update(&mut self, delta_time: f32) {
    // Update player physics (existing code)
    self.player.update(&self.world, delta_time);

    // Update chunk system
    self.chunk_manager.update_player_position(self.player.position);
    self.chunk_manager.update(&mut self.world);

    // Process generation queue (limit to avoid lag)
    for _ in 0..4 {
        if let Some(chunk_pos) = self.chunk_manager.pop_generation_task() {
            self.world.load_chunk(chunk_pos); // Auto-generates terrain
        } else {
            break;
        }
    }

    // Process meshing queue (limit to avoid lag)
    for _ in 0..2 {
        if let Some(chunk_pos) = self.chunk_manager.pop_meshing_task() {
            if let Some(mesh_data) = mesh_chunk(&self.world, chunk_pos) {
                self.chunk_meshes
                    .entry(chunk_pos)
                    .or_insert_with(|| ChunkMesh::new(chunk_pos.x, chunk_pos.z))
                    .upload(&mesh_data.vertices);

                self.world.clear_chunk_dirty(chunk_pos);
            }
        } else {
            break;
        }
    }

    // Clean up meshes for unloaded chunks
    self.chunk_meshes.retain(|pos, _| self.world.is_chunk_loaded(*pos));
}
```

### 4. In `Game::render()`

```rust
fn render(&self) {
    // Setup rendering (existing code)
    self.renderer.begin_frame();

    // Render chunks
    for (chunk_pos, mesh) in &self.chunk_meshes {
        if !mesh.is_empty() && self.camera.is_chunk_visible(*chunk_pos) {
            mesh.draw();
        }
    }

    // Render other stuff (skybox, UI, etc.)
    self.renderer.end_frame();
}
```

### 5. Connect Player Actions

**Block Breaking:**
```rust
fn break_block(&mut self, hit_pos: WorldPos) {
    if self.world.get_block(&hit_pos) != BlockType::Air {
        self.world.set_block(hit_pos, BlockType::Air);
        // Chunk automatically marked dirty, will remesh next frame
    }
}
```

**Block Placing:**
```rust
fn place_block(&mut self, place_pos: WorldPos, block_type: BlockType) {
    if self.world.get_block(&place_pos) == BlockType::Air {
        self.world.set_block(place_pos, block_type);
        // Chunk automatically marked dirty, will remesh next frame
    }
}
```

### 6. Player Collision (in `src/player/mod.rs`)

```rust
impl Player {
    pub fn update(&mut self, world: &World, delta_time: f32) {
        // Apply movement...
        let new_pos = self.position + self.velocity * delta_time;

        // Collision detection
        let player_aabb = AABB::from_center_size(
            new_pos,
            Vec3::new(0.6, 1.8, 0.6)
        );

        // Check blocks in AABB vicinity
        let min = WorldPos::from_vec3(player_aabb.min);
        let max = WorldPos::from_vec3(player_aabb.max);

        for x in min.x..=max.x {
            for y in min.y..=max.y {
                for z in min.z..=max.z {
                    let pos = WorldPos::new(x, y, z);
                    if world.get_block(&pos).is_solid() {
                        let block_aabb = AABB::from_block(&pos);
                        if player_aabb.intersects(&block_aabb) {
                            // Resolve collision (see physics module)
                        }
                    }
                }
            }
        }
    }
}
```

## Testing Checklist

After integration, verify:

- [ ] Chunks generate around player spawn
- [ ] New chunks load as player moves
- [ ] Distant chunks unload (check memory)
- [ ] Blocks can be broken (left click)
- [ ] Blocks can be placed (right click)
- [ ] Changed blocks remesh correctly
- [ ] Chunk boundaries look correct (no gaps)
- [ ] Player collision works (can't walk through blocks)
- [ ] Player can stand on blocks (gravity + collision)
- [ ] FPS stays above 30 (at render distance 8)
- [ ] No memory leaks (use Task Manager / htop)

## Performance Tuning

If FPS drops:
- Reduce `MAX_GEN_PER_FRAME` (currently 4)
- Reduce `MAX_MESH_PER_FRAME` (currently 2)
- Reduce `RENDER_DISTANCE` (currently 8)
- Add more aggressive frustum culling
- Profile with `cargo flamegraph`

## Common Issues

**Issue:** Chunks don't load
- **Fix:** Ensure `chunk_manager.update()` is called every frame

**Issue:** Holes at chunk boundaries
- **Fix:** Ensure neighbor chunks are loaded before meshing

**Issue:** Lag spikes when moving
- **Fix:** Reduce generation/meshing limits per frame

**Issue:** Blocks don't break/place
- **Fix:** Check raycasting returns valid WorldPos, ensure `set_block()` is called

**Issue:** Player falls through floor
- **Fix:** Collision detection must run AFTER world is loaded

## Documentation

- `CHUNK_QUICKSTART.md` - Copy-paste integration code
- `CHUNK_SYSTEM.md` - Full technical documentation (400+ lines)
- `CHUNK_SYSTEM_SUMMARY.md` - Implementation overview

## File Locations

All chunk system code is in `src/world/`:
- `chunk.rs` - Block storage
- `world.rs` - World management
- `chunk_manager.rs` - Loading/unloading
- `mesher.rs` - Mesh generation

## Success Criteria

Integration is successful when:

1. ✅ Player spawns in generated terrain
2. ✅ Can walk around (WASD + mouse)
3. ✅ New chunks load as player explores
4. ✅ Can break blocks (left click)
5. ✅ Can place blocks (right click)
6. ✅ Changed chunks remesh correctly
7. ✅ No visible gaps at chunk boundaries
8. ✅ Collision works (can't clip through blocks)
9. ✅ 30+ FPS at 8 chunk render distance
10. ✅ Memory usage stable (no leaks)

## Next Steps After Chunk Integration

1. Physics system (gravity, jumping, fall damage)
2. UI system (crosshair, hotbar, health bar)
3. Inventory system (block selection, crafting)
4. Mob system (spawning, AI, rendering)
5. Audio system (block sounds, ambient)

---

**Good luck! The chunk system is rock-solid and ready to use.**

See `CHUNK_QUICKSTART.md` for the fastest integration path.
