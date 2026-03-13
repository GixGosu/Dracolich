# Chunk System Quick Start

## 30-Second Integration

```rust
use crate::world::{World, ChunkManager, mesh_chunk};
use crate::renderer::mesh::ChunkMesh;
use std::collections::HashMap;

// Setup (once)
let mut world = World::new(12345);
let mut chunk_manager = ChunkManager::new(8);
let mut chunk_meshes = HashMap::new();

// Game Loop (every frame)
chunk_manager.update_player_position(player.position);
chunk_manager.update(&mut world);

// Generate terrain (4 per frame)
for _ in 0..4 {
    if let Some(pos) = chunk_manager.pop_generation_task() {
        world.load_chunk(pos); // Auto-generates via TerrainGenerator
    }
}

// Create meshes (2 per frame)
for _ in 0..2 {
    if let Some(pos) = chunk_manager.pop_meshing_task() {
        if let Some(mesh_data) = mesh_chunk(&world, pos) {
            chunk_meshes.entry(pos)
                .or_insert_with(|| ChunkMesh::new(pos.x, pos.z))
                .upload(&mesh_data.vertices);
            world.clear_chunk_dirty(pos);
        }
    }
}

// Render
for (pos, mesh) in &chunk_meshes {
    if camera.is_chunk_visible(*pos) {
        mesh.draw();
    }
}

// Player actions
world.set_block(WorldPos::new(x, y, z), BlockType::Air);  // Break
world.set_block(WorldPos::new(x, y, z), BlockType::Stone); // Place
```

## That's It!

See `CHUNK_SYSTEM.md` for details.

## Key Points

1. **World auto-generates terrain** when you call `load_chunk()`
2. **Chunks auto-mark dirty** when blocks change
3. **ChunkManager handles everything** - just call `update()`
4. **Limit work per frame** to maintain FPS (4 gen + 2 mesh recommended)
5. **Meshing needs neighbor chunks** - generate in spiral from player

## Common Gotchas

- Don't mesh chunks whose neighbors aren't loaded (faces will be wrong)
- Don't forget to `clear_chunk_dirty()` after uploading mesh
- Don't forget to remove meshes for unloaded chunks
- DO limit generation/meshing per frame (avoid lag spikes)

## File Locations

- `src/world/chunk.rs` - Storage
- `src/world/world.rs` - Management
- `src/world/chunk_manager.rs` - Loading
- `src/world/mesher.rs` - Meshing
- `CHUNK_SYSTEM.md` - Full docs
