# Texture System Documentation

## Overview

The texture system provides a complete texture atlas and block texture mapping for the Minecraft-style voxel game. It consists of three main components:

1. **Texture Atlas** (`assets/atlas.png`) - A 256×256 pixel PNG containing all block textures
2. **Block Texture Mappings** (`src/renderer/block_textures.rs`) - Rust code mapping block types to atlas coordinates
3. **Texture Atlas Generator** (`generate_atlas.py`) - Python script to regenerate the atlas

## Texture Atlas Layout

- **Atlas Size**: 256×256 pixels
- **Tile Size**: 16×16 pixels per block texture
- **Grid**: 16×16 tiles = 256 total tile slots
- **Used Tiles**: 28 tiles (indices 0-27)
- **Available Tiles**: 228 tiles for future expansion

### Tile Index Map

| Index | Texture | Description |
|-------|---------|-------------|
| 0 | Air | Black (empty) |
| 1 | Grass Top | Green with variation |
| 2 | Dirt | Brown with noise |
| 3 | Grass Side | Green top, brown bottom |
| 4 | Stone | Gray with dark speckles |
| 5 | Cobblestone | Gray with irregular pattern |
| 6 | Sand | Tan/beige |
| 7 | Gravel | Gray pebbles |
| 8 | Bedrock | Dark mottled pattern |
| 9 | Oak Log Top | Tree rings |
| 10 | Oak Log Side | Brown bark with lines |
| 11 | Birch Log Top | Light tree rings |
| 12 | Birch Log Side | White bark with dark marks |
| 13 | Oak Leaves | Green scattered pattern |
| 14 | Birch Leaves | Light green scattered |
| 15 | Water | Blue transparent with shimmer |
| 16 | Glass | Light cyan transparent |
| 17 | Coal Ore | Stone with black specks |
| 18 | Iron Ore | Stone with tan specks |
| 19 | Gold Ore | Stone with yellow specks |
| 20 | Diamond Ore | Stone with cyan specks |
| 21 | Planks | Wood planks with lines |
| 22 | Crafting Table Top | Planks with 2×2 grid |
| 24 | Crafting Table Side | Planks with tool symbols |
| 25 | Furnace Front | Gray with dark opening |
| 26 | Furnace Top | Gray stone texture |
| 27 | Furnace Side | Gray stone texture |

## Block Texture Mappings

Each block type can have different textures on different faces:

### Uniform Blocks (Same texture on all faces)
- Air, Dirt, Stone, Cobblestone, Sand, Gravel, Bedrock
- Oak Leaves, Birch Leaves, Water, Glass
- Coal Ore, Iron Ore, Gold Ore, Diamond Ore, Planks

### Multi-Face Blocks

**Grass Block:**
- Top: Grass top (green)
- Bottom: Dirt
- Sides: Grass side (green top transition)

**Wood Logs (Oak, Birch):**
- Top/Bottom: Tree rings
- Sides: Bark texture

**Crafting Table:**
- Top: Crafting grid pattern
- Bottom: Planks
- Sides: Tool symbols on planks

**Furnace:**
- Top/Bottom: Stone texture
- North (front): Opening
- South/East/West: Plain stone sides

## Usage in Code

### Getting Texture Info

```rust
use crate::types::BlockType;

// Get complete texture info for a block
let info = BlockType::Grass.texture_info();
println!("Top: {}, Bottom: {}, North: {}", info.top, info.bottom, info.north);

// Get texture for a specific face
let direction = Direction::Up;
let texture_index = info.get_face_texture(direction);

// Get texture by face index (0-5)
let texture_index = info.get_face_texture_by_index(4); // 4 = up face
```

### UV Coordinate Calculation

The `TextureAtlas` struct in `src/renderer/texture.rs` handles UV coordinate generation:

```rust
use crate::renderer::TextureAtlas;

// Load the atlas
let atlas = TextureAtlas::new("assets/atlas.png")?;

// Get UV coordinates for a specific tile
let (u_min, v_min, u_max, v_max) = atlas.get_uv_coords(tile_index);

// Get UV coordinates for a block face
let uvs = atlas.get_block_uvs(BlockType::Grass, 4); // 4 = up face
```

### Mesh Building Example

```rust
use crate::renderer::block_textures::BlockTextureInfo;
use crate::types::{BlockType, Direction};

fn add_face_to_mesh(block: BlockType, direction: Direction) {
    let info = block.texture_info();
    let texture_index = info.get_face_texture(direction);

    // Get UV coordinates
    let (u_min, v_min, u_max, v_max) = atlas.get_uv_coords(texture_index);

    // Create quad vertices with UVs...
}
```

## Regenerating the Atlas

If you need to modify textures or add new block types:

1. Edit `generate_atlas.py`:
   - Add new color constants
   - Create new tile generator functions
   - Add tile to atlas generation in `main()`

2. Run the generator:
   ```bash
   python3 generate_atlas.py
   ```

3. Update `src/renderer/block_textures.rs`:
   - Add new tile constants in `tiles` module
   - Update `BlockType::texture_info()` for new blocks

## Texture Properties

### Visual Characteristics

- **Noise Variation**: Most textures use random noise for organic look
- **Procedural Patterns**: All textures are generated procedurally
- **Transparency**: Water, glass, and leaves support alpha channel
- **Recognizable**: Each texture is distinct and recognizable at 16×16

### Performance Optimization

- **Single Atlas**: All textures in one file = minimal texture binding
- **Mipmaps**: Generated automatically for distant rendering
- **Nearest-Neighbor Filtering**: Preserves pixel art aesthetic
- **Power-of-Two Size**: 256×256 for optimal GPU performance

## Future Expansion

The atlas has 228 unused tile slots for future block types:

- Stone brick, sandstone, brick, nether blocks
- Additional wood types (spruce, jungle, acacia, dark oak)
- Crops and plants
- Ores and minerals
- Decorative blocks
- Redstone components

To add a new block:
1. Add tile generator to `generate_atlas.py`
2. Regenerate atlas
3. Add tile constant to `src/renderer/block_textures.rs`
4. Update `BlockType` enum in `src/types.rs`
5. Add texture mapping in `BlockType::texture_info()`

## Testing

The `block_textures.rs` module includes unit tests:

```bash
cargo test --lib renderer::block_textures
```

Tests verify:
- Grass has different top/side/bottom textures
- Uniform blocks use same texture on all faces
- Wood logs have bark and ring textures
- Furnace has unique front face

## Integration with Renderer

The texture system integrates with the existing renderer:

1. **Loading** (`src/renderer/texture.rs`):
   - `TextureAtlas::new("assets/atlas.png")` loads the atlas
   - Uploads to GPU with mipmaps and filtering

2. **Usage** (`src/world/chunk_mesher.rs`):
   - Query block texture info
   - Generate UV coordinates for each face
   - Build mesh with textured vertices

3. **Rendering** (`src/renderer/mod.rs`):
   - Bind atlas once per frame
   - Render all chunks with atlas
   - Shader samples atlas using vertex UVs

## File Locations

```
project/
├── assets/
│   └── atlas.png              # Generated 256×256 texture atlas
├── src/
│   ├── renderer/
│   │   ├── block_textures.rs  # Block→texture mappings
│   │   ├── texture.rs         # TextureAtlas loader
│   │   └── mod.rs             # Module exports
│   └── types.rs               # BlockType definitions
├── generate_atlas.py          # Atlas generator script
└── TEXTURE_SYSTEM.md          # This file
```

## Credits

- Texture generation: Procedural via Python PIL
- Color palette: Inspired by Minecraft Classic
- All textures: Programmatically generated, no external assets
