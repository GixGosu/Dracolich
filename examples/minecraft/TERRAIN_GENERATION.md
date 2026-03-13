# Terrain Generation System

## Overview

The terrain generation system creates infinite, procedurally-generated voxel worlds using multi-octave Perlin noise. The system is deterministic - the same seed will always generate the same terrain at any given coordinates.

## Architecture

The system consists of three main modules:

### 1. `world/generation.rs` - TerrainGenerator

The core terrain generator using the `noise` crate. Implements:

#### **Height Map Generation**
- Multi-octave Perlin noise at 3 scales:
  - **Large features** (scale 0.003): Continental shapes, mountain ranges
  - **Medium features** (scale 0.01): Hills and valleys
  - **Small features** (scale 0.05): Surface detail and bumps
- Height range: 60-128 blocks above bedrock
- Biome-specific height multipliers for variation

#### **Biome Selection**
- Two independent noise layers:
  - **Temperature noise**: Controls hot/cold biomes
  - **Moisture noise**: Controls wet/dry biomes
- Biomes blend smoothly using large-scale noise (0.003)
- 5 biome types: Plains, Hills, Mountains, Desert, Forest

#### **Cave Generation**
- 3D Perlin noise for natural cave systems
- Scale: 0.05 for interconnected tunnels
- Threshold: 0.6 (higher = fewer caves)
- Carves through stone, dirt, and gravel
- Active between y=5 and y=120
- Respects bedrock layer (y=0)

#### **Ore Placement**
- Separate noise layer for each ore type
- Depth-based probability:
  - **Coal**: Common, any depth below y=120
  - **Iron**: Common, below y=64
  - **Gold**: Uncommon, below y=32
  - **Diamond**: Rare, below y=16
- Only replaces stone blocks

#### **Structure Generation**
- Deterministic per-chunk seeding
- Trees placed based on biome density
- Returns blocks that may extend into neighboring chunks

### 2. `world/biome.rs` - Biome System

Defines 5 distinct biomes with unique generation parameters:

| Biome | Height Range | Surface | Subsurface | Tree Density |
|-------|-------------|---------|------------|--------------|
| Plains | 60-68 | Grass | Dirt | 0.2% |
| Hills | 60-85 | Grass | Dirt | 0.5% |
| Mountains | 70-128 | Stone | Stone | 0.1% |
| Desert | 62-70 | Sand | Sand | 0% |
| Forest | 60-75 | Grass | Dirt | 2% |

**Biome Selection Logic:**
```
Hot + Dry (temp > 0.3, moisture < -0.2) → Desert
Cold (temp < -0.5) → Mountains
Wet (moisture > 0.4) → Forest
Mid-temp (temp -0.2 to 0.2) → Hills
Default → Plains
```

### 3. `world/structure.rs` - Structure Generation

#### **Tree Generation**
- Two tree types: Oak and Birch
- **Trunk**: 4-6 blocks (Oak) or 5-7 blocks (Birch)
- **Crown**: 3-layer leaf sphere starting 2 blocks below trunk top
  - Top layer: radius 1
  - Middle layers: radius 2
  - Bottom layer: radius 2
- Corners omitted for natural shape
- Requires solid ground and 7 blocks of air above

#### **Boulder Generation**
- Small cobblestone clusters (2-3 blocks)
- Potential for mountain biome decoration

## World Manager (`world/mod.rs`)

The `World` struct manages chunk lifecycle and block access:

### **Chunk Storage**
- HashMap of loaded chunks indexed by ChunkPos
- Chunks are 16×256×16 blocks
- Lazy loading - chunks generate on first access

### **Block Access**
- `get_block(pos)`: Returns block at world position, generates chunk if needed
- `set_block(pos, block)`: Sets block and marks chunk as dirty

### **Chunk Management**
- `ensure_chunk_loaded(pos)`: Generate chunk if not in memory
- `load_chunks_around(center, radius)`: Pre-load chunks in radius
- `unload_distant_chunks(center, max_distance)`: Memory cleanup
- `unload_chunk(pos)`: Remove chunk from memory

### **Generation Pipeline**
1. Generate base terrain (height map + biomes)
2. Carve caves (3D noise)
3. Place ores (depth + noise)
4. Generate structures (trees, etc.)

## Deterministic Generation

The system guarantees identical terrain for the same seed:

- **Chunk coordinates** deterministically derive RNG seeds
- **Structure placement** uses per-chunk seeded RNG
- **Noise functions** use consistent seeds derived from world seed
- **Same world position** always generates same blocks regardless of load order

Formula for chunk seed:
```rust
seed = world_seed
seed = seed * 31 + chunk_x
seed = seed * 31 + chunk_z
```

## Performance Characteristics

### **Memory**
- Each chunk: ~1MB (16×256×16 bytes for block IDs)
- Render distance 8: ~400 chunks loaded = ~400MB
- Old chunks unloaded when player moves away

### **Generation Speed**
- Single chunk generation: ~5-15ms (depends on structure density)
- Parallelizable (chunks are independent)
- Cave carving is the slowest step (3D noise sampling)

### **Optimization Opportunities**
1. **Parallel chunk generation**: Use thread pool for multi-chunk batches
2. **Chunk caching**: Save generated chunks to disk
3. **Structure batching**: Generate structures for multiple chunks at once
4. **Noise pre-computation**: Cache noise values for frequently accessed areas

## Integration Points

### **Renderer**
- Access chunks via `World::get_chunk(pos)`
- Check `chunk.dirty` flag to know when to rebuild mesh
- Iterate through `world.loaded_chunks()` for visible chunks

### **Physics/Collision**
- Use `World::get_block(pos)` for collision detection
- Block is solid if `block.is_solid()` returns true
- Water and air have no collision

### **Player Interaction**
- Breaking blocks: `world.set_block(pos, BlockType::Air)`
- Placing blocks: `world.set_block(pos, block_type)`
- Chunk auto-loads when player accesses block

### **Chunk Loading Strategy**
```rust
// In game loop:
let player_chunk = ChunkPos::from_world_pos(&player.position);

// Load chunks in render distance
world.load_chunks_around(player_chunk, RENDER_DISTANCE);

// Unload far chunks to save memory
world.unload_distant_chunks(player_chunk, RENDER_DISTANCE + 2);
```

## Testing

Two unit tests verify correctness:

1. **`test_terrain_generation`**: Verifies bedrock at y=0
2. **`test_deterministic_generation`**: Confirms same seed = same terrain

Run tests:
```bash
cargo test --lib world::generation
```

## Future Enhancements

Potential improvements:

1. **More biomes**: Tundra, jungle, ocean, swamp
2. **Better cave systems**: Ravines, underground lakes, abandoned mineshafts
3. **Villages**: Structure generation with building templates
4. **Strongholds**: Rare underground structures
5. **Nether/End dimensions**: Alternative generation rules
6. **Biome blending**: Smooth transitions between biomes
7. **Climate zones**: Latitude-based temperature variation
8. **Rivers**: Connected water systems flowing to ocean

## Constants Reference

```rust
// Chunk dimensions
CHUNK_WIDTH = 16
CHUNK_HEIGHT = 256
CHUNK_DEPTH = 16

// World constants
SEA_LEVEL = 64        // Water fills to this height
BEDROCK_LEVEL = 0     // Indestructible floor

// Noise scales
HEIGHT_LARGE = 0.003   // Continental features
HEIGHT_MEDIUM = 0.01   // Hills and valleys
HEIGHT_SMALL = 0.05    // Surface detail
BIOME_SCALE = 0.003    // Biome transitions
CAVE_SCALE = 0.05      // Cave tunnels
ORE_SCALE = 0.1        // Ore veins

// Thresholds
CAVE_THRESHOLD = 0.6   // Cave carving cutoff
COAL_THRESHOLD = 0.7   // Coal ore spawn
IRON_THRESHOLD = 0.75  // Iron ore spawn
GOLD_THRESHOLD = 0.82  // Gold ore spawn
DIAMOND_THRESHOLD = 0.88 // Diamond ore spawn
```
