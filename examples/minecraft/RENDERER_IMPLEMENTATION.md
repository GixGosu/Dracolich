# OpenGL Renderer Implementation

## Overview

Complete OpenGL 3.3+ rendering backend for the Minecraft-style voxel game. All components implemented from scratch using raw OpenGL via the `gl` crate.

## Files Created

### 1. **src/renderer/mod.rs** (238 lines)
Main renderer module that manages the complete rendering pipeline.

**Features:**
- OpenGL context initialization via glutin
- Window surface management
- Main `Renderer` struct coordinating all rendering systems
- Frame rendering methods (`begin_frame`, `end_frame`)
- Chunk rendering with fog and day/night cycle
- Skybox rendering
- Block highlight rendering
- Window resize handling

**Key Methods:**
- `new()` - Initialize OpenGL context, load shaders, create texture atlas
- `render_chunks()` - Render all visible chunk meshes with lighting and fog
- `render_skybox()` - Render sky dome with sun/moon
- `resize()` - Handle window resize events

### 2. **src/renderer/gl_wrapper.rs** (226 lines)
Safe RAII wrappers around raw OpenGL objects.

**Components:**
- `VAO` - Vertex Array Object wrapper with auto-cleanup
- `VBO` - Vertex Buffer Object wrapper
- `EBO` - Element Buffer Object (type alias to VBO)
- `Texture` - Texture object wrapper
- `enable_vertex_attrib()` - Helper for vertex attribute configuration
- `check_gl_error()` - Debug error checking (debug builds only)

**Safety:**
- All OpenGL objects automatically deleted on Drop
- Prevents resource leaks
- Type-safe bindings

### 3. **src/renderer/shader.rs** (184 lines)
Shader compilation and uniform management.

**Features:**
- Load shaders from files
- Compile vertex and fragment shaders
- Link shader programs
- Detailed error reporting with line numbers
- Type-safe uniform setters

**Uniform Types Supported:**
- `float`, `int`
- `Vec2`, `Vec3`, `Vec4`
- `Mat4`

**Methods:**
- `from_files()` - Load and compile shaders from disk
- `use_program()` - Activate shader
- `set_float/int/vec2/vec3/vec4/mat4()` - Set uniforms

### 4. **src/renderer/texture.rs** (115 lines)
Texture atlas loading and UV coordinate calculation.

**Features:**
- Load PNG texture atlas via `image` crate
- Calculate UV coordinates for block textures
- Tile-based atlas system (16x16 pixel tiles)
- Mipmap generation for LOD
- Nearest-neighbor filtering (pixel-perfect at close range)

**Methods:**
- `new()` - Load atlas from PNG file
- `get_uv_coords()` - Get UV rect for tile index
- `get_block_uvs()` - Get UVs for specific block face
- `bind/unbind()` - Texture binding

**Texture Setup:**
- Clamp to edge (no wrapping)
- Nearest filtering for crisp pixels
- Mipmaps for distant blocks

### 5. **src/renderer/mesh.rs** (315 lines)
Chunk mesh generation and vertex data structures.

**Key Structures:**

#### `Vertex` (repr(C))
Matches shader vertex layout exactly:
```rust
{
    position: [f32; 3],   // location 0
    tex_coord: [f32; 2],  // location 1
    normal: [f32; 3],     // location 2
    light: f32,           // location 3 (AO/light level)
}
```

#### `ChunkMesh`
Per-chunk GPU mesh with VAO/VBO.

**Methods:**
- `new()` - Create mesh with vertex attribute configuration
- `upload()` - Upload vertex data to GPU (STATIC_DRAW)
- `update()` - Update existing mesh data
- `draw()` - Render mesh using glDrawArrays
- `vertex_count()` - Get number of vertices

**Utilities:**
- `create_quad()` - Generate 6 vertices for a quad (2 triangles)
- `calculate_ao()` - Calculate ambient occlusion for vertex
- `create_cube_vertices()` - Generate debug cube mesh

### 6. **src/renderer/camera.rs** (234 lines)
First-person camera with frustum culling.

**Components:**

#### `Plane`
Frustum plane representation with signed distance tests.

#### `Frustum`
6-plane viewing frustum extracted from view-projection matrix.

**Culling Methods:**
- `is_sphere_visible()` - Sphere vs frustum test
- `is_aabb_visible()` - AABB vs frustum test
- `is_chunk_visible()` - Chunk-specific culling (16x256x16 bounds)

#### `Camera`
First-person camera with FPS-style controls.

**Properties:**
- `position` - World position
- `yaw` - Horizontal rotation (radians)
- `pitch` - Vertical rotation (radians, clamped to prevent gimbal lock)
- `fov` - Field of view (default 70°)
- `near/far` - Clip planes (0.1 to 1000.0)

**Methods:**
- `forward/right/up()` - Direction vectors
- `view_matrix()` - Generate view transform
- `projection_matrix()` - Generate perspective projection
- `frustum()` - Extract frustum for culling
- `move_local()` - Move in camera space
- `rotate()` - Apply rotation with pitch clamping

### 7. **src/renderer/skybox.rs** (175 lines)
Sky dome rendering with day/night cycle.

**Features:**
- Hemisphere sky dome (50-unit radius)
- Procedurally generated dome mesh (32 segments × 16 rings)
- Dynamic sun/moon positioning
- Procedural stars at night
- Sunrise/sunset color transitions

**Rendering:**
- Depth mask disabled (sky always behind)
- View matrix without translation (sky centered on camera)
- z/w = 1.0 trick for maximum depth

**Methods:**
- `new()` - Generate dome geometry and load shaders
- `render()` - Render sky with time-of-day uniforms
- `calculate_sun_direction()` - Sun position from time
- `calculate_moon_direction()` - Moon position (opposite sun)
- `get_fog_color()` - Get fog color for current time

**Day/Night Cycle:**
- 0.0 = midnight
- 0.25 = sunrise
- 0.5 = noon
- 0.75 = sunset
- 1.0 = midnight

## Integration with Shaders

The renderer works with these shader pairs:

1. **block.vert/frag** - Block/chunk rendering
   - Uniforms: MVP matrices, camera position, time of day, fog parameters
   - Per-vertex lighting and AO
   - Distance fog
   - Day/night color tinting

2. **sky.vert/frag** - Skybox rendering
   - Uniforms: View/projection (no translation), time of day, sun/moon directions
   - Gradient sky
   - Celestial bodies
   - Procedural stars

3. **highlight.vert/frag** - Block selection highlight
   - Wireframe rendering
   - Pulsing alpha animation

4. **ui.vert/frag** - UI overlay
   - Orthographic projection
   - Textured/colored elements
   - Alpha blending

## OpenGL State Management

**Enabled Features:**
- `GL_DEPTH_TEST` - Depth testing for 3D rendering
- `GL_CULL_FACE` - Backface culling (CCW winding)
- `GL_BLEND` - Alpha blending (SRC_ALPHA, ONE_MINUS_SRC_ALPHA)

**Clear Color:**
- Sky blue (0.53, 0.81, 0.92, 1.0)

**Viewport:**
- Automatically updated on window resize

## Performance Features

1. **Frustum Culling**
   - Extract frustum planes from view-projection matrix
   - Test chunks against frustum before rendering
   - Skip invisible chunks entirely

2. **Static Mesh Upload**
   - Chunks uploaded once with `GL_STATIC_DRAW`
   - Only re-upload when chunk data changes
   - GPU-side storage minimizes bandwidth

3. **Texture Atlas**
   - Single texture bind for all blocks
   - Reduces state changes
   - Mipmaps for LOD

4. **Backface Culling**
   - Don't render block faces between solid blocks
   - Implemented at mesh generation time (not shown here, for world module)

5. **Distance Fog**
   - Hides chunk pop-in at render distance
   - Blends with sky color

## Usage Example

```rust
use renderer::{Renderer, Camera, ChunkMesh};

// Initialize
let renderer = Renderer::new(&window)?;
let mut camera = Camera::new(Vec3::new(0.0, 100.0, 0.0), 0.0, 0.0);

// Game loop
loop {
    renderer.begin_frame();

    // Render sky
    renderer.render_skybox(&camera, time_of_day);

    // Frustum cull chunks
    let frustum = camera.frustum(renderer.aspect_ratio());
    let visible_chunks: Vec<&ChunkMesh> = chunks.iter()
        .filter(|chunk| frustum.is_chunk_visible(chunk.chunk_x, chunk.chunk_z))
        .collect();

    // Render visible chunks
    let fog_color = Skybox::get_fog_color(time_of_day);
    renderer.render_chunks(&visible_chunks, &camera, time_of_day, fog_color, 128.0);

    renderer.end_frame();
}
```

## Dependencies

- **gl** (0.14) - OpenGL bindings
- **glam** (0.24) - Math library (Vec3, Mat4, etc.)
- **image** (0.24) - PNG loading for texture atlas
- **glutin** (0.31) - OpenGL context creation
- **winit** (0.29) - Windowing

## Next Steps for Integration

The world module will need to:
1. Generate `ChunkMesh` objects from voxel data
2. Implement greedy meshing or naive meshing
3. Handle face culling (don't render hidden faces)
4. Calculate per-vertex ambient occlusion
5. Assign correct texture UVs based on block type and face

The player module will need to:
1. Update camera position/rotation from input
2. Handle mouse look (yaw/pitch)
3. Calculate movement in camera space

## Statistics

- **Total Lines:** ~1,487 lines of Rust
- **7 modules** fully implemented
- **Zero external rendering libraries** (raw OpenGL only)
- **Complete feature set** for Minecraft-style rendering
- **Production-ready** RAII resource management
- **Frustum culling** for optimal performance
- **Day/night cycle** with dynamic lighting
