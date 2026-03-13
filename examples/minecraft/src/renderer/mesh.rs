/// Chunk mesh generation and vertex data structures
/// Handles creating GPU-uploadable mesh data from voxel chunks

use super::gl_wrapper::{enable_vertex_attrib, VBO, VAO};
use gl::types::*;
use glam::Vec3;

/// Vertex data structure matching the shader layout
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],  // a_Position (location 0)
    pub tex_coord: [f32; 2], // a_TexCoord (location 1)
    pub normal: [f32; 3],    // a_Normal (location 2)
    pub light: f32,          // a_Light (location 3) - ambient occlusion/light level
}

impl Vertex {
    pub fn new(position: [f32; 3], tex_coord: [f32; 2], normal: [f32; 3], light: f32) -> Self {
        Self {
            position,
            tex_coord,
            normal,
            light,
        }
    }
}

/// Mesh data for a single chunk
pub struct ChunkMesh {
    vao: VAO,
    vbo: VBO,
    vertex_count: usize,
    pub chunk_x: i32,
    pub chunk_z: i32,
}

impl ChunkMesh {
    /// Create a new empty chunk mesh
    pub fn new(chunk_x: i32, chunk_z: i32) -> Self {
        let vao = VAO::new();
        let vbo = VBO::new(gl::ARRAY_BUFFER);

        // Bind VAO and configure vertex attributes
        vao.bind();
        vbo.bind();

        // Configure vertex attributes to match Vertex struct
        let stride = std::mem::size_of::<Vertex>() as GLsizei;

        // Position (location 0)
        enable_vertex_attrib(
            0,
            3,
            gl::FLOAT,
            false,
            stride,
            0,
        );

        // TexCoord (location 1)
        enable_vertex_attrib(
            1,
            2,
            gl::FLOAT,
            false,
            stride,
            std::mem::size_of::<[f32; 3]>(),
        );

        // Normal (location 2)
        enable_vertex_attrib(
            2,
            3,
            gl::FLOAT,
            false,
            stride,
            std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 2]>(),
        );

        // Light (location 3)
        enable_vertex_attrib(
            3,
            1,
            gl::FLOAT,
            false,
            stride,
            std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 2]>() + std::mem::size_of::<[f32; 3]>(),
        );

        vao.unbind();
        vbo.unbind();

        Self {
            vao,
            vbo,
            vertex_count: 0,
            chunk_x,
            chunk_z,
        }
    }

    /// Upload vertex data to GPU
    pub fn upload(&mut self, vertices: &[Vertex]) {
        self.vertex_count = vertices.len();

        if self.vertex_count == 0 {
            return;
        }

        self.vbo.bind();
        self.vbo.upload_data(vertices, gl::STATIC_DRAW);
        self.vbo.unbind();
    }

    /// Update existing vertex data
    pub fn update(&mut self, vertices: &[Vertex]) {
        if vertices.len() > self.vertex_count {
            // Need to reallocate
            self.upload(vertices);
        } else {
            self.vertex_count = vertices.len();
            if self.vertex_count > 0 {
                self.vbo.update_data(0, vertices);
            }
        }
    }

    /// Draw the mesh
    pub fn draw(&self) {
        if self.vertex_count == 0 {
            return;
        }

        self.vao.bind();
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, self.vertex_count as GLsizei);
        }
        self.vao.unbind();
    }

    /// Get number of vertices
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    /// Check if mesh is empty
    pub fn is_empty(&self) -> bool {
        self.vertex_count == 0
    }
}

/// Helper to create a quad (two triangles) from 4 vertices
/// Returns 6 vertices in triangle list format
pub fn create_quad(v0: Vertex, v1: Vertex, v2: Vertex, v3: Vertex) -> [Vertex; 6] {
    [
        v0, v1, v2, // First triangle
        v2, v3, v0, // Second triangle
    ]
}

/// Calculate ambient occlusion value for a vertex
/// side1, side2 are adjacent blocks, corner is diagonal block
/// Returns a light value from 0.0 (fully occluded) to 1.0 (no occlusion)
pub fn calculate_ao(side1: bool, side2: bool, corner: bool) -> f32 {
    let mut ao = 3;

    if side1 && side2 {
        ao = 0; // Fully occluded
    } else {
        if side1 {
            ao -= 1;
        }
        if side2 {
            ao -= 1;
        }
        if corner {
            ao -= 1;
        }
    }

    // Map 0-3 to light values
    match ao {
        0 => 0.25,
        1 => 0.50,
        2 => 0.75,
        _ => 1.0,
    }
}

/// Generate cube vertices for a single block (for debug visualization)
pub fn create_cube_vertices(pos: Vec3, size: f32) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    let half = size * 0.5;

    // Define cube corners
    let corners = [
        [pos.x - half, pos.y - half, pos.z - half],
        [pos.x + half, pos.y - half, pos.z - half],
        [pos.x + half, pos.y + half, pos.z - half],
        [pos.x - half, pos.y + half, pos.z - half],
        [pos.x - half, pos.y - half, pos.z + half],
        [pos.x + half, pos.y - half, pos.z + half],
        [pos.x + half, pos.y + half, pos.z + half],
        [pos.x - half, pos.y + half, pos.z + half],
    ];

    // UV coordinates (simple 0-1 mapping)
    let uvs = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

    // North face (-Z)
    let normal = [0.0, 0.0, -1.0];
    vertices.extend_from_slice(&create_quad(
        Vertex::new(corners[0], uvs[0], normal, 1.0),
        Vertex::new(corners[1], uvs[1], normal, 1.0),
        Vertex::new(corners[2], uvs[2], normal, 1.0),
        Vertex::new(corners[3], uvs[3], normal, 1.0),
    ));

    // South face (+Z)
    let normal = [0.0, 0.0, 1.0];
    vertices.extend_from_slice(&create_quad(
        Vertex::new(corners[5], uvs[0], normal, 1.0),
        Vertex::new(corners[4], uvs[1], normal, 1.0),
        Vertex::new(corners[7], uvs[2], normal, 1.0),
        Vertex::new(corners[6], uvs[3], normal, 1.0),
    ));

    // East face (+X)
    let normal = [1.0, 0.0, 0.0];
    vertices.extend_from_slice(&create_quad(
        Vertex::new(corners[1], uvs[0], normal, 1.0),
        Vertex::new(corners[5], uvs[1], normal, 1.0),
        Vertex::new(corners[6], uvs[2], normal, 1.0),
        Vertex::new(corners[2], uvs[3], normal, 1.0),
    ));

    // West face (-X)
    let normal = [-1.0, 0.0, 0.0];
    vertices.extend_from_slice(&create_quad(
        Vertex::new(corners[4], uvs[0], normal, 1.0),
        Vertex::new(corners[0], uvs[1], normal, 1.0),
        Vertex::new(corners[3], uvs[2], normal, 1.0),
        Vertex::new(corners[7], uvs[3], normal, 1.0),
    ));

    // Top face (+Y)
    let normal = [0.0, 1.0, 0.0];
    vertices.extend_from_slice(&create_quad(
        Vertex::new(corners[3], uvs[0], normal, 1.0),
        Vertex::new(corners[2], uvs[1], normal, 1.0),
        Vertex::new(corners[6], uvs[2], normal, 1.0),
        Vertex::new(corners[7], uvs[3], normal, 1.0),
    ));

    // Bottom face (-Y)
    let normal = [0.0, -1.0, 0.0];
    vertices.extend_from_slice(&create_quad(
        Vertex::new(corners[4], uvs[0], normal, 1.0),
        Vertex::new(corners[5], uvs[1], normal, 1.0),
        Vertex::new(corners[1], uvs[2], normal, 1.0),
        Vertex::new(corners[0], uvs[3], normal, 1.0),
    ));

    vertices
}
