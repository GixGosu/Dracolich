# Renderer API Quick Reference

## For World Module Integration

### Creating Chunk Meshes

```rust
use crate::renderer::mesh::{ChunkMesh, Vertex, create_quad, calculate_ao};
use crate::renderer::texture::TextureAtlas;

// Create mesh for chunk at (chunk_x, chunk_z)
let mut mesh = ChunkMesh::new(chunk_x, chunk_z);

// Build vertex list
let mut vertices = Vec::new();

// For each block face that should be rendered:
let position = [x as f32, y as f32, z as f32];
let (u_min, v_min, u_max, v_max) = texture_atlas.get_block_uvs(block_type, face_index);
let normal = face_normal; // [0.0, 1.0, 0.0] for top face, etc.
let ao = calculate_ao(side1_solid, side2_solid, corner_solid);

let v0 = Vertex::new(pos0, [u_min, v_min], normal, ao);
let v1 = Vertex::new(pos1, [u_max, v_min], normal, ao);
let v2 = Vertex::new(pos2, [u_max, v_max], normal, ao);
let v3 = Vertex::new(pos3, [u_min, v_max], normal, ao);

let quad = create_quad(v0, v1, v2, v3);
vertices.extend_from_slice(&quad);

// Upload to GPU
mesh.upload(&vertices);
```

### Face Indices
- 0 = North (-Z)
- 1 = South (+Z)
- 2 = East (+X)
- 3 = West (-X)
- 4 = Up (+Y)
- 5 = Down (-Y)

### Face Culling Logic

```rust
// Only render a face if the adjacent block is transparent
fn should_render_face(current_block: BlockType, neighbor_block: BlockType, direction: Direction) -> bool {
    if current_block == BlockType::Air {
        return false; // Never render air
    }

    // Check if neighbor is transparent (air, water, glass, leaves)
    neighbor_block.is_transparent()
}
```

## For Player Module Integration

### Camera Control

```rust
use crate::renderer::Camera;

// Create camera
let mut camera = Camera::new(
    Vec3::new(0.0, 100.0, 0.0), // position
    0.0, // yaw
    0.0  // pitch
);

// Mouse look (sensitivity = 0.002 recommended)
camera.rotate(delta_x * sensitivity, -delta_y * sensitivity);

// WASD movement (dt = delta time)
let speed = 10.0; // blocks per second
camera.move_local(
    (w_pressed - s_pressed) as f32 * speed * dt, // forward
    (d_pressed - a_pressed) as f32 * speed * dt, // right
    (space_pressed - shift_pressed) as f32 * speed * dt // up
);

// Get camera vectors for physics
let forward = camera.forward();
let right = camera.right();
let up = camera.up();
```

## Main Render Loop

```rust
use crate::renderer::{Renderer, Camera};
use crate::renderer::skybox::Skybox;

let mut renderer = Renderer::new(&window)?;
let mut camera = Camera::default();
let mut time_of_day = 0.5; // Start at noon

loop {
    // Update time of day (10 minute cycle)
    time_of_day += delta_time / 600.0;
    if time_of_day >= 1.0 {
        time_of_day -= 1.0;
    }

    // Clear frame
    renderer.begin_frame();

    // Render sky first
    renderer.render_skybox(&camera, time_of_day);

    // Frustum culling
    let frustum = camera.frustum(renderer.aspect_ratio());
    let visible_chunks: Vec<&ChunkMesh> = world.chunks()
        .filter(|chunk| frustum.is_chunk_visible(chunk.chunk_x, chunk.chunk_z))
        .collect();

    // Render chunks
    let fog_color = Skybox::get_fog_color(time_of_day);
    let render_distance = 128.0; // 8 chunks * 16 blocks
    renderer.render_chunks(
        &visible_chunks,
        &camera,
        time_of_day,
        fog_color,
        render_distance
    );

    // Render block highlight (if targeting a block)
    if let Some(target_pos) = targeted_block {
        renderer.render_highlight(target_pos.to_vec3(), &camera, elapsed_time);
    }

    // Present frame
    renderer.end_frame();
}
```

## Texture Atlas Format

Expected file: `assets/textures/atlas.png`

Layout:
- Each tile is 16x16 pixels
- Tiles are arranged in a grid (e.g., 16x16 grid = 256 tiles)
- Tile indices match `BlockType::texture_indices()` in `types.rs`

Example tile mapping (from types.rs):
```
Index 0:  (unused)
Index 1:  Grass top
Index 2:  Dirt
Index 3:  Grass side
Index 4:  Stone
Index 5:  Cobblestone
... (see types.rs for full mapping)
```

## OpenGL Requirements

- OpenGL 3.3 Core or higher
- Extensions: None required
- Features used: VAO, VBO, depth testing, face culling, blending

## Performance Tips

1. **Batch by chunk** - Upload one mesh per chunk, not per block
2. **Frustum cull** - Use `frustum.is_chunk_visible()` before rendering
3. **Static upload** - Use `mesh.upload()` once, only `update()` on change
4. **Face culling** - Don't generate vertices for hidden faces
5. **Sort by distance** - Render near chunks first (may help early-z)
6. **Greedy meshing** - Merge adjacent faces with same texture (advanced)

## Debugging

```rust
use crate::renderer::gl_wrapper::check_gl_error;

// In debug builds, check for GL errors
check_gl_error("after chunk render");
```

## Common Issues

**Black screen:**
- Check shader compilation errors
- Verify texture atlas loads correctly
- Ensure camera is positioned above world (y > 0)

**Z-fighting:**
- Check face culling is working
- Ensure no duplicate faces
- Verify depth test is enabled

**Missing textures:**
- Verify UV coordinates are in [0, 1] range
- Check texture atlas loads (look for errors)
- Ensure texture is bound before rendering

**Low FPS:**
- Enable frustum culling
- Implement face culling (don't render hidden faces)
- Check vertex count isn't excessive
- Consider greedy meshing

## Shader Uniform Reference

### block.vert/frag
```glsl
uniform mat4 u_Model;           // Chunk offset transform
uniform mat4 u_View;            // Camera view matrix
uniform mat4 u_Projection;      // Perspective projection
uniform vec3 u_CameraPos;       // Camera position (for fog)
uniform sampler2D u_TextureAtlas; // Block textures
uniform float u_TimeOfDay;      // 0.0 to 1.0
uniform vec3 u_FogColor;        // Sky color
uniform float u_FogStart;       // Fog start distance
uniform float u_FogEnd;         // Fog end distance
uniform float u_RenderDistance; // Max render distance
```

### sky.vert/frag
```glsl
uniform mat4 u_View;            // View matrix (no translation)
uniform mat4 u_Projection;      // Perspective projection
uniform float u_TimeOfDay;      // 0.0 to 1.0
uniform vec3 u_SunDirection;    // Sun direction vector
uniform vec3 u_MoonDirection;   // Moon direction vector
```

## Next Steps

1. **World module** should generate ChunkMesh objects from voxel data
2. **Player module** should update Camera from input
3. **Physics module** can use Camera vectors for movement
4. **UI module** can use UI shader for crosshair/HUD
