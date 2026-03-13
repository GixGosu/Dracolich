# Chunk System Implementation

## Overview

The chunk system provides efficient voxel world storage, management, and rendering for the Minecraft-style game. It consists of four main components:

1. **Chunk** - Block storage with palette compression
2. **World** - Chunk management and cross-boundary block access
3. **ChunkManager** - Dynamic loading/unloading based on player position
4. **Mesher** - Greedy meshing with ambient occlusion

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                       Game Loop                             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    ChunkManager                             │
│  - Tracks player position                                   │
│  - Queues chunks for generation                             │
│  - Queues chunks for meshing                                │
│  - Unloads distant chunks                                   │
└─────────────────────────────────────────────────────────────┘
                              │
                 ┌────────────┴────────────┐
                 ▼                         ▼
┌────────────────────────────┐  ┌──────────────────────────┐
│         World              │  │      Mesher              │
│  - HashMap<ChunkPos,Chunk> │  │  - mesh_chunk()          │
│  - get_block/set_block     │  │  - Greedy meshing        │
│  - Neighbor lookup         │  │  - Ambient occlusion     │
└────────────────────────────┘  └──────────────────────────┘
                 │                         │
                 ▼                         ▼
┌────────────────────────────┐  ┌──────────────────────────┐
│         Chunk              │  │      MeshData            │
│  - Palette compression     │  │  - Vec<Vertex>           │
│  - 16x256x16 blocks        │  │  - Ready for GPU upload  │
│  - Dirty flag              │  └──────────────────────────┘
└────────────────────────────┘
```

## Components

### 1. Chunk (`src/world/chunk.rs`)

**Storage:** Palette-compressed block storage

- Uses `Vec<BlockType>` palette + `Box<[u8]>` indices
- Automatically converts to raw storage if palette exceeds 256 types
- Memory efficient: ~68KB per diverse chunk vs 256KB raw storage

**Key Methods:**
```rust
let mut chunk = Chunk::new();                    // Create empty (all air)
let block = chunk.get_block(x, y, z);            // Get block at local coords
chunk.set_block(x, y, z, BlockType::Stone);      // Set block, marks dirty
chunk.is_dirty();                                 // Check if needs remeshing
chunk.fill_region(x1,y1,z1, x2,y2,z2, block);    // Fill rectangular region
```

**Coordinates:**
- Local chunk space: `x,z ∈ [0,15]`, `y ∈ [0,255]`
- Uses YZX ordering for cache locality during vertical iteration

### 2. World (`src/world/world.rs`)

**Storage:** `HashMap<ChunkPos, Chunk>`

Provides seamless cross-chunk block access and boundary handling.

**Key Methods:**
```rust
let mut world = World::new(seed);
world.load_chunk(ChunkPos::new(x, z));           // Load/create chunk
world.set_block(WorldPos::new(x,y,z), block);    // Set block (loads chunk if needed)
let block = world.get_block(&WorldPos::new(x,y,z)); // Get block (returns Air if not loaded)

// Chunk management
world.get_chunk(chunk_pos);                       // Get chunk reference
world.unload_chunk(chunk_pos);                    // Remove from memory
world.dirty_chunks();                             // Iterator over chunks needing remesh

// Statistics
let stats = world.stats();                        // WorldStats { loaded_chunks, total_solid_blocks }
```

**Boundary Handling:**
When `set_block()` modifies a block on a chunk boundary (x=0/15 or z=0/15), it automatically marks the neighboring chunk as dirty to ensure faces get remeshed correctly.

### 3. ChunkManager (`src/world/chunk_manager.rs`)

**Purpose:** Manages chunk lifecycle based on player position

**Responsibilities:**
- Track which chunks should be loaded (circular render distance)
- Queue new chunks for generation
- Queue dirty chunks for remeshing
- Unload distant chunks

**Usage:**
```rust
let mut manager = ChunkManager::new(render_distance);

// Every frame in game loop:
if manager.update_player_position(player.position) {
    // Player changed chunks
}
manager.update(&mut world); // Process loading/unloading

// Process work queues (can be threaded):
while let Some(chunk_pos) = manager.pop_generation_task() {
    let chunk = generate_terrain(chunk_pos); // From TerrainGenerator
    world.insert_chunk(chunk_pos, chunk);
}

while let Some(chunk_pos) = manager.pop_meshing_task() {
    if let Some(mesh_data) = mesh_chunk(&world, chunk_pos) {
        upload_to_gpu(mesh_data); // Create/update ChunkMesh
        world.clear_chunk_dirty(chunk_pos);
    }
}
```

**Configuration:**
```rust
manager.set_render_distance(12);              // Change render distance
let stats = manager.stats();                  // Get queue lengths, player chunk
```

### 4. Mesher (`src/world/mesher.rs`)

**Algorithm:** Greedy meshing with ambient occlusion

**Features:**
- Face culling: Only renders faces adjacent to transparent blocks
- Greedy merging: Combines adjacent identical faces into larger quads
- Ambient occlusion: Vertex lighting based on corner occlusion
- Cross-chunk neighbor lookup: Correctly handles faces at chunk boundaries
- AO-aware triangle orientation: Prevents lighting artifacts

**Usage:**
```rust
if let Some(mesh_data) = mesh_chunk(&world, chunk_pos) {
    // mesh_data.vertices: Vec<Vertex>
    // Each vertex has: position, tex_coord, normal, light (AO)

    let mut chunk_mesh = ChunkMesh::new(chunk_pos.x, chunk_pos.z);
    chunk_mesh.upload(&mesh_data.vertices);

    // Later, during rendering:
    chunk_mesh.draw();
}
```

**Performance:**
- Empty chunks: 0 vertices (instant)
- Sparse chunks: ~100-500 vertices
- Dense chunks: ~2000-8000 vertices
- Greedy meshing reduces vertex count by 60-80% vs naive approach

## Integration Example

### Main Game Loop

```rust
use crate::world::{World, ChunkManager, mesh_chunk};
use crate::renderer::mesh::ChunkMesh;
use crate::types::{ChunkPos};
use std::collections::HashMap;

pub struct Game {
    world: World,
    chunk_manager: ChunkManager,
    chunk_meshes: HashMap<ChunkPos, ChunkMesh>,
    terrain_gen: TerrainGenerator,
}

impl Game {
    pub fn new() -> Self {
        Self {
            world: World::new(12345),
            chunk_manager: ChunkManager::new(8),
            chunk_meshes: HashMap::new(),
            terrain_gen: TerrainGenerator::new(12345),
        }
    }

    pub fn update(&mut self, player_pos: Vec3) {
        // Update chunk manager
        self.chunk_manager.update_player_position(player_pos);
        self.chunk_manager.update(&mut self.world);

        // Process generation queue (limit per frame to avoid lag)
        for _ in 0..4 {
            if let Some(chunk_pos) = self.chunk_manager.pop_generation_task() {
                let chunk = self.terrain_gen.generate_chunk(chunk_pos);
                self.world.insert_chunk(chunk_pos, chunk);
            } else {
                break;
            }
        }

        // Process meshing queue (limit per frame)
        for _ in 0..2 {
            if let Some(chunk_pos) = self.chunk_manager.pop_meshing_task() {
                if let Some(mesh_data) = mesh_chunk(&self.world, chunk_pos) {
                    // Create or update GPU mesh
                    let chunk_mesh = self.chunk_meshes
                        .entry(chunk_pos)
                        .or_insert_with(|| ChunkMesh::new(chunk_pos.x, chunk_pos.z));

                    chunk_mesh.upload(&mesh_data.vertices);
                    self.world.clear_chunk_dirty(chunk_pos);
                }
            } else {
                break;
            }
        }

        // Clean up meshes for unloaded chunks
        self.chunk_meshes.retain(|pos, _| self.world.is_chunk_loaded(*pos));
    }

    pub fn render(&self, camera: &Camera) {
        for (chunk_pos, mesh) in &self.chunk_meshes {
            if !mesh.is_empty() && camera.is_chunk_visible(*chunk_pos) {
                mesh.draw();
            }
        }
    }

    pub fn break_block(&mut self, pos: WorldPos) {
        self.world.set_block(pos, BlockType::Air);
        // Chunk automatically marked dirty, will be remeshed next frame
    }

    pub fn place_block(&mut self, pos: WorldPos, block: BlockType) {
        self.world.set_block(pos, block);
        // Chunk automatically marked dirty, will be remeshed next frame
    }
}
```

### Player Movement

```rust
impl Player {
    pub fn update(&mut self, world: &World, delta_time: f32) {
        // Apply movement...
        let new_pos = self.position + self.velocity * delta_time;

        // Collision detection (see physics module for full implementation)
        let player_aabb = AABB::from_center_size(new_pos, Vec3::new(0.6, 1.8, 0.6));

        // Check blocks in AABB vicinity
        let min = WorldPos::from_vec3(player_aabb.min);
        let max = WorldPos::from_vec3(player_aabb.max);

        for x in min.x..=max.x {
            for y in min.y..=max.y {
                for z in min.z..=max.z {
                    let block_pos = WorldPos::new(x, y, z);
                    let block = world.get_block(&block_pos);

                    if block.is_solid() {
                        let block_aabb = AABB::from_block(&block_pos);
                        if player_aabb.intersects(&block_aabb) {
                            // Handle collision...
                        }
                    }
                }
            }
        }
    }
}
```

## Performance Characteristics

### Memory Usage

**Per Chunk:**
- Palette compressed: ~68KB (typical)
- Raw storage: 256KB (only if >256 unique block types)
- Average compression ratio: 70-80% savings

**World with 8 chunk render distance:**
- ~200 chunks loaded
- ~13-50 MB total chunk data
- Plus GPU meshes (~100KB per chunk average)

### CPU Performance

**Generation:**
- ~5-20ms per chunk (noise + structures)
- Can be threaded easily

**Meshing:**
- Empty chunk: <0.1ms
- Sparse chunk: 1-3ms
- Dense chunk: 5-15ms
- Can be threaded (read-only world access)

**Updates:**
- Single block change: <0.1ms (just mark dirty)
- Remeshing affected chunks: 1-15ms (1-2 chunks typically)

### Optimization Tips

1. **Limit work per frame:**
   ```rust
   const MAX_GEN_PER_FRAME: usize = 4;
   const MAX_MESH_PER_FRAME: usize = 2;
   ```

2. **Use threading (future enhancement):**
   - Generation can run on worker threads
   - Meshing can run on worker threads (read-only world access)
   - Main thread only handles World updates and GPU uploads

3. **Frustum culling:**
   - Don't render chunks outside camera view
   - See `Camera::is_chunk_visible()` in renderer

4. **Skip empty chunks:**
   ```rust
   if chunk.is_empty() {
       continue; // No meshing needed
   }
   ```

## Testing

Each module has comprehensive unit tests:

```bash
cargo test --lib world::chunk
cargo test --lib world::world
cargo test --lib world::chunk_manager
cargo test --lib world::mesher
```

**Test coverage:**
- Chunk: palette compression, bounds checking, iteration
- World: cross-chunk boundaries, dirty tracking, loading/unloading
- ChunkManager: render distance, queue management, player movement
- Mesher: face culling, greedy merging, AO calculation

## Known Limitations

1. **No threading yet** - Generation and meshing run on main thread
2. **No chunk persistence** - Chunks are regenerated on load (no saving)
3. **Fixed palette limit** - Falls back to raw storage at 256 types (rare but not ideal)
4. **Greedy meshing doesn't cross block types** - Could be more aggressive
5. **No LOD** - All chunks use full detail

## Future Enhancements

1. **Multi-threaded generation/meshing**
   - Use rayon or custom thread pool
   - Queue system already designed for this

2. **Chunk saving/loading**
   - Serialize modified chunks to disk
   - Load from disk before regenerating

3. **Better palette compression**
   - Use variable-width indices (1, 2, 4 bits per block)
   - Run-length encoding for uniform regions

4. **Level of detail (LOD)**
   - Lower detail meshes for distant chunks
   - Reduce vertex count by 4x+ for far chunks

5. **Chunk streaming**
   - Background thread for I/O
   - Priority queue based on player direction

## Troubleshooting

### Problem: Chunks not loading
**Solution:** Check `ChunkManager::update()` is called every frame and generation queue is being processed.

### Problem: Holes at chunk boundaries
**Solution:** Ensure neighbor chunks are loaded before meshing. The mesher requires neighbor chunks for boundary face culling.

### Problem: Performance drops when moving
**Solution:** Limit generation/meshing tasks per frame. Increase limits if FPS is consistently high.

### Problem: Lighting artifacts on quads
**Solution:** The mesher automatically flips triangle winding to prevent AO artifacts. Ensure AO calculation is using correct corner blocks.

### Problem: Memory usage too high
**Solution:** Reduce render distance or implement chunk unloading more aggressively.

## File Locations

- `src/world/chunk.rs` - Chunk storage (392 lines)
- `src/world/world.rs` - World management (312 lines)
- `src/world/chunk_manager.rs` - Loading/unloading (356 lines)
- `src/world/mesher.rs` - Mesh generation (449 lines)
- `src/world/mod.rs` - Module exports

**Total:** ~1,500 lines of production code + comprehensive tests
