/// Chunk meshing with greedy algorithm and ambient occlusion
/// Converts voxel data into optimized triangle meshes for rendering

use crate::types::{BlockType, ChunkPos, Direction, WorldPos, CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH};
use crate::renderer::mesh::{Vertex, calculate_ao};
use crate::renderer::calculate_uv;
use super::chunk::Chunk;
use super::world::World;

/// Result of meshing a chunk
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub chunk_pos: ChunkPos,
}

impl MeshData {
    pub fn new(chunk_pos: ChunkPos) -> Self {
        Self {
            vertices: Vec::new(),
            chunk_pos,
        }
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

/// Mesh a chunk using greedy meshing algorithm
/// Requires access to neighbor chunks for boundary face culling
pub fn mesh_chunk(world: &World, chunk_pos: ChunkPos) -> Option<MeshData> {
    let chunk = world.get_chunk(&chunk_pos)?;

    // Skip meshing for empty chunks
    if chunk.is_empty() {
        return Some(MeshData::new(chunk_pos));
    }

    let mut mesh_data = MeshData::new(chunk_pos);

    // Mesh each direction separately for greedy meshing
    for direction in Direction::all() {
        mesh_direction(world, chunk_pos, chunk, direction, &mut mesh_data);
    }

    Some(mesh_data)
}

/// Mesh all faces in a specific direction using greedy algorithm
fn mesh_direction(
    world: &World,
    chunk_pos: ChunkPos,
    chunk: &Chunk,
    direction: Direction,
    mesh_data: &mut MeshData,
) {
    let (width, height, depth) = get_dimensions_for_direction(direction);

    // Scan through each layer perpendicular to the direction
    for layer in 0..depth {
        // 2D mask for this layer (true = needs a face)
        let mut mask = vec![None; width * height];

        // Build mask for this layer
        for h in 0..height {
            for w in 0..width {
                let (x, y, z) = map_to_coords(direction, w, h, layer);

                if x >= CHUNK_WIDTH || y >= CHUNK_HEIGHT || z >= CHUNK_DEPTH {
                    continue;
                }

                let block = chunk.get_block(x, y, z);

                // Skip air blocks
                if block == BlockType::Air {
                    continue;
                }

                // Check if face should be rendered (adjacent block is transparent)
                if should_render_face(world, chunk_pos, x, y, z, direction) {
                    mask[h * width + w] = Some(block);
                }
            }
        }

        // Greedy mesh the mask
        greedy_mesh_layer(world, chunk_pos, direction, layer, width, height, &mask, mesh_data);
    }
}

/// Greedy meshing algorithm - combines adjacent faces into larger quads
fn greedy_mesh_layer(
    world: &World,
    chunk_pos: ChunkPos,
    direction: Direction,
    layer: usize,
    width: usize,
    height: usize,
    mask: &[Option<BlockType>],
    mesh_data: &mut MeshData,
) {
    let mut visited = vec![false; width * height];

    for h in 0..height {
        for w in 0..width {
            let index = h * width + w;

            if visited[index] {
                continue;
            }

            let Some(block_type) = mask[index] else {
                continue;
            };

            // Find the maximum width of this quad
            let mut quad_width = 1;
            while w + quad_width < width {
                let next_index = h * width + (w + quad_width);
                if visited[next_index] || mask[next_index] != Some(block_type) {
                    break;
                }
                quad_width += 1;
            }

            // Find the maximum height of this quad (must match width)
            let mut quad_height = 1;
            'height_loop: while h + quad_height < height {
                for dw in 0..quad_width {
                    let test_index = (h + quad_height) * width + (w + dw);
                    if visited[test_index] || mask[test_index] != Some(block_type) {
                        break 'height_loop;
                    }
                }
                quad_height += 1;
            }

            // Mark quad area as visited
            for dh in 0..quad_height {
                for dw in 0..quad_width {
                    visited[(h + dh) * width + (w + dw)] = true;
                }
            }

            // Generate quad vertices
            let (x, y, z) = map_to_coords(direction, w, h, layer);
            generate_quad(
                world,
                chunk_pos,
                x,
                y,
                z,
                quad_width,
                quad_height,
                direction,
                block_type,
                mesh_data,
            );
        }
    }
}

/// Generate a single quad with proper UVs and ambient occlusion
fn generate_quad(
    world: &World,
    chunk_pos: ChunkPos,
    x: usize,
    y: usize,
    z: usize,
    width: usize,
    height: usize,
    direction: Direction,
    block_type: BlockType,
    mesh_data: &mut MeshData,
) {
    // Get texture coordinates
    let (top_tex, bottom_tex, side_tex) = block_type.texture_indices();
    let tex_index = match direction {
        Direction::Up => top_tex,
        Direction::Down => bottom_tex,
        _ => side_tex,
    };

    // Calculate quad corners in world space
    let chunk_origin = chunk_pos.to_world_origin();
    let world_x = chunk_origin.x + x as i32;
    let world_y = y as i32;
    let world_z = chunk_origin.z + z as i32;

    let (v0, v1, v2, v3, uvs) = get_quad_vertices(
        world_x as f32,
        world_y as f32,
        world_z as f32,
        width,
        height,
        direction,
        tex_index,
    );

    // Calculate ambient occlusion for each vertex
    let ao0 = calculate_vertex_ao(world, world_x, world_y, world_z, direction, 0, 0, width, height);
    let ao1 = calculate_vertex_ao(world, world_x, world_y, world_z, direction, width, 0, width, height);
    let ao2 = calculate_vertex_ao(world, world_x, world_y, world_z, direction, width, height, width, height);
    let ao3 = calculate_vertex_ao(world, world_x, world_y, world_z, direction, 0, height, width, height);

    let normal = direction.normal();
    let normal_arr = [normal.x, normal.y, normal.z];

    // Create vertices
    let vertex0 = Vertex::new(v0, uvs[0], normal_arr, ao0);
    let vertex1 = Vertex::new(v1, uvs[1], normal_arr, ao1);
    let vertex2 = Vertex::new(v2, uvs[2], normal_arr, ao2);
    let vertex3 = Vertex::new(v3, uvs[3], normal_arr, ao3);

    // Generate two triangles
    // Quad vertices are CCW: v0 → v1 → v2 → v3
    // Normal diagonal (v0-v2): triangles (v0,v1,v2) and (v0,v2,v3)
    // Alt diagonal (v1-v3): triangles (v0,v1,v3) and (v1,v2,v3)
    // Flip diagonal when AO values suggest it to prevent lighting artifacts
    if ao0 + ao2 > ao1 + ao3 {
        // Use alternate diagonal v1-v3
        mesh_data.vertices.push(vertex0);
        mesh_data.vertices.push(vertex1);
        mesh_data.vertices.push(vertex3);

        mesh_data.vertices.push(vertex1);
        mesh_data.vertices.push(vertex2);
        mesh_data.vertices.push(vertex3);
    } else {
        // Use normal diagonal v0-v2
        mesh_data.vertices.push(vertex0);
        mesh_data.vertices.push(vertex1);
        mesh_data.vertices.push(vertex2);

        mesh_data.vertices.push(vertex0);
        mesh_data.vertices.push(vertex2);
        mesh_data.vertices.push(vertex3);
    }
}

/// Get quad corner positions and UVs for a specific face direction
fn get_quad_vertices(
    x: f32,
    y: f32,
    z: f32,
    width: usize,
    height: usize,
    direction: Direction,
    tex_index: usize,
) -> ([f32; 3], [f32; 3], [f32; 3], [f32; 3], [[f32; 2]; 4]) {
    let w = width as f32;
    let h = height as f32;

    // Calculate UV coordinates for the texture atlas
    // NOTE: For greedy-meshed quads larger than 1x1, we map the ENTIRE quad
    // to a single texture tile. This means large quads will stretch the texture
    // rather than tiling it. This is necessary because:
    // 1. Atlas uses CLAMP_TO_EDGE wrapping (can't tile)
    // 2. Tiles are adjacent in atlas space (can't extend UVs without bleeding)
    // 3. This is the standard approach for voxel engines with texture atlases
    //
    // The visual result is acceptable for most block textures (grass, stone, etc.)
    // Use small inset (0.5px) to avoid sampling adjacent tiles at edges
    //
    // UV coordinate system (no V-flip, image top = GL texture bottom):
    // - calculate_uv(tex_index, 0.5, 0.5) returns [u_low, v_low] (top-left in image = bottom-left in GL)
    // - calculate_uv(tex_index, 15.5, 15.5) returns [u_high, v_high] (bottom-right in image = top-right in GL)
    let uv_min = calculate_uv(tex_index, 0.5, 0.5);
    let uv_max = calculate_uv(tex_index, 15.5, 15.5);

    // Extract the actual min/max U and V values
    let u_min = uv_min[0];
    let u_max = uv_max[0];
    let v_min = uv_min[1];
    let v_max = uv_max[1];

    // Base UV corners for OpenGL's bottom-left origin
    // In GL texture space: v_min is bottom, v_max is top
    // But our textures are oriented with image-top at v_min, so:
    // - "top" of texture content is at v_min (GL bottom)
    // - "bottom" of texture content is at v_max (GL top)
    // For correct orientation, we flip the V mapping in the quad corners
    let bl = [u_min, v_max];  // bottom-left of quad -> bottom of image content (v_max in GL)
    let br = [u_max, v_max];  // bottom-right
    let tr = [u_max, v_min];  // top-right of quad -> top of image content (v_min in GL)
    let tl = [u_min, v_min];  // top-left

    match direction {
        Direction::Up => (
            // Counterclockwise when viewed from above (+Y looking down)
            // Vertices go: back-left, back-right, front-right, front-left
            // UVs map: back=top, front=bottom, left=left, right=right
            [x, y + 1.0, z + h],         // v0: back-left  -> top-left
            [x + w, y + 1.0, z + h],     // v1: back-right -> top-right
            [x + w, y + 1.0, z],         // v2: front-right -> bottom-right
            [x, y + 1.0, z],             // v3: front-left -> bottom-left
            [tl, tr, br, bl],
        ),
        Direction::Down => (
            // Counterclockwise when viewed from below (-Y looking up)
            // Vertices go: front-left, front-right, back-right, back-left
            // UVs map: front=bottom, back=top (but inverted because looking up)
            [x, y, z],                   // v0: front-left  -> bottom-left
            [x + w, y, z],               // v1: front-right -> bottom-right
            [x + w, y, z + h],           // v2: back-right  -> top-right
            [x, y, z + h],               // v3: back-left   -> top-left
            [bl, br, tr, tl],
        ),
        Direction::North => (
            // Looking from -Z toward +Z, vertices counterclockwise
            [x + w, y, z],      // v0: bottom-right (in world: +x, low y)
            [x, y, z],          // v1: bottom-left
            [x, y + h, z],      // v2: top-left
            [x + w, y + h, z],  // v3: top-right
            [br, bl, tl, tr],
        ),
        Direction::South => (
            // Looking from +Z toward -Z, vertices counterclockwise
            [x, y, z + 1.0],           // v0: bottom-left
            [x + w, y, z + 1.0],       // v1: bottom-right
            [x + w, y + h, z + 1.0],   // v2: top-right
            [x, y + h, z + 1.0],       // v3: top-left
            [bl, br, tr, tl],
        ),
        Direction::East => (
            // +X face, normal points +X. Viewed from +X: -Z is left, +Z is right
            // CCW winding for +X normal: v0→v1 is +Y, v1→v2 is +Z
            [x + 1.0, y, z],           // v0: bottom-left (small z)
            [x + 1.0, y + h, z],       // v1: top-left
            [x + 1.0, y + h, z + w],   // v2: top-right (large z)
            [x + 1.0, y, z + w],       // v3: bottom-right
            [bl, tl, tr, br],
        ),
        Direction::West => (
            // -X face, normal points -X. Viewed from -X: +Z is left, -Z is right
            // CCW winding for -X normal: v0→v1 is +Y, v1→v2 is -Z
            [x, y, z + w],             // v0: bottom-left (large z)
            [x, y + h, z + w],         // v1: top-left
            [x, y + h, z],             // v2: top-right (small z)
            [x, y, z],                 // v3: bottom-right
            [bl, tl, tr, br],
        ),
    }
}

/// Calculate ambient occlusion for a vertex
fn calculate_vertex_ao(
    world: &World,
    x: i32,
    y: i32,
    z: i32,
    direction: Direction,
    corner_u: usize,
    corner_v: usize,
    _width: usize,
    _height: usize,
) -> f32 {
    // Get the three blocks that influence AO for this vertex
    let (side1_offset, side2_offset, corner_offset) = get_ao_offsets(direction, corner_u, corner_v);

    let dir_offset = direction.offset();

    let side1_pos = WorldPos::new(
        x + dir_offset.x + side1_offset.0,
        y + dir_offset.y + side1_offset.1,
        z + dir_offset.z + side1_offset.2,
    );

    let side2_pos = WorldPos::new(
        x + dir_offset.x + side2_offset.0,
        y + dir_offset.y + side2_offset.1,
        z + dir_offset.z + side2_offset.2,
    );

    let corner_pos = WorldPos::new(
        x + dir_offset.x + corner_offset.0,
        y + dir_offset.y + corner_offset.1,
        z + dir_offset.z + corner_offset.2,
    );

    let side1 = world.get_block(&side1_pos).is_opaque();
    let side2 = world.get_block(&side2_pos).is_opaque();
    let corner = world.get_block(&corner_pos).is_opaque();

    calculate_ao(side1, side2, corner)
}

/// Get AO neighbor offsets for a specific vertex corner
fn get_ao_offsets(direction: Direction, u: usize, v: usize) -> ((i32, i32, i32), (i32, i32, i32), (i32, i32, i32)) {
    let u_sign = if u > 0 { 1 } else { -1 };
    let v_sign = if v > 0 { 1 } else { -1 };

    match direction {
        Direction::Up | Direction::Down => {
            let side1 = (u_sign, 0, 0);
            let side2 = (0, 0, v_sign);
            let corner = (u_sign, 0, v_sign);
            (side1, side2, corner)
        }
        Direction::North | Direction::South => {
            let side1 = (u_sign, 0, 0);
            let side2 = (0, v_sign, 0);
            let corner = (u_sign, v_sign, 0);
            (side1, side2, corner)
        }
        Direction::East | Direction::West => {
            let side1 = (0, 0, u_sign);
            let side2 = (0, v_sign, 0);
            let corner = (0, v_sign, u_sign);
            (side1, side2, corner)
        }
    }
}

/// Check if a face should be rendered (neighbor is transparent or out of bounds)
fn should_render_face(
    world: &World,
    chunk_pos: ChunkPos,
    x: usize,
    y: usize,
    z: usize,
    direction: Direction,
) -> bool {
    let offset = direction.offset();
    let chunk_origin = chunk_pos.to_world_origin();

    let neighbor_pos = WorldPos::new(
        chunk_origin.x + x as i32 + offset.x,
        y as i32 + offset.y,
        chunk_origin.z + z as i32 + offset.z,
    );

    // Out of bounds below/above world
    if neighbor_pos.y < 0 || neighbor_pos.y >= CHUNK_HEIGHT as i32 {
        return direction == Direction::Up; // Only render top face at max height
    }

    let neighbor_block = world.get_block(&neighbor_pos);

    // Render if neighbor is transparent
    neighbor_block.is_transparent()
}

/// Get layer dimensions for a specific direction (used for greedy meshing)
fn get_dimensions_for_direction(direction: Direction) -> (usize, usize, usize) {
    match direction {
        Direction::Up | Direction::Down => (CHUNK_WIDTH, CHUNK_DEPTH, CHUNK_HEIGHT),
        Direction::North | Direction::South => (CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH),
        Direction::East | Direction::West => (CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH),
    }
}

/// Map 2D layer coordinates to 3D chunk coordinates
fn map_to_coords(direction: Direction, w: usize, h: usize, layer: usize) -> (usize, usize, usize) {
    match direction {
        Direction::Up | Direction::Down => (w, layer, h),
        Direction::North | Direction::South => (w, h, layer),
        Direction::East | Direction::West => (layer, h, w),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_chunk_mesh() {
        let mut world = World::new(0);
        let chunk = Chunk::new();
        let pos = ChunkPos::new(0, 0);

        world.insert_chunk(pos, chunk);

        let mesh = mesh_chunk(&world, pos).unwrap();
        assert!(mesh.is_empty());
    }

    #[test]
    fn test_single_block_mesh() {
        let mut world = World::new(0);
        let pos = ChunkPos::new(0, 0);

        world.load_chunk(pos);
        world.set_block(WorldPos::new(0, 64, 0), BlockType::Stone);

        let mesh = mesh_chunk(&world, pos).unwrap();

        // A single block in air should have 6 faces * 6 vertices = 36 vertices
        assert_eq!(mesh.vertex_count(), 36);
    }

    #[test]
    fn test_face_culling() {
        let mut world = World::new(0);
        let pos = ChunkPos::new(0, 0);

        world.load_chunk(pos);

        // Create a 2x2x2 cube of blocks
        for x in 0..2 {
            for y in 64..66 {
                for z in 0..2 {
                    world.set_block(WorldPos::new(x, y, z), BlockType::Stone);
                }
            }
        }

        let mesh = mesh_chunk(&world, pos).unwrap();

        // Interior faces should be culled, only exterior faces rendered
        // 2x2x2 cube has 24 exterior faces
        // Each face = 6 vertices
        assert_eq!(mesh.vertex_count(), 24 * 6);
    }

    #[test]
    fn test_greedy_meshing() {
        let mut world = World::new(0);
        let pos = ChunkPos::new(0, 0);

        world.load_chunk(pos);

        // Create a flat 4x4 layer of blocks
        for x in 0..4 {
            for z in 0..4 {
                world.set_block(WorldPos::new(x, 64, z), BlockType::Stone);
            }
        }

        let mesh = mesh_chunk(&world, pos).unwrap();

        // Greedy meshing should combine into larger quads
        // Top face: 1 large quad = 6 vertices
        // Bottom face: 1 large quad = 6 vertices
        // Sides: 4 edges * 4 blocks wide = perimeter faces
        assert!(mesh.vertex_count() < 4 * 4 * 6 * 6); // Less than individual block faces
    }
}
