use glam::{Mat4, Vec3};
use std::error::Error;
use crate::window::Window;

pub mod block_textures;
pub mod camera;
pub mod gl_wrapper;
pub mod mesh;
pub mod shader;
pub mod skybox;
pub mod texture;

pub use block_textures::BlockTextureInfo;
pub use camera::Camera;
pub use mesh::{ChunkMesh, Vertex};
pub use shader::ShaderProgram;
pub use skybox::Skybox;
pub use texture::{TextureAtlas, calculate_uv};

/// Main renderer that manages OpenGL state and rendering
/// Note: GL context is assumed to be already initialized by Window
pub struct Renderer {
    shader_program: ShaderProgram,
    highlight_shader: ShaderProgram,
    ui_shader: ShaderProgram,
    texture_atlas: TextureAtlas,
    skybox: Skybox,
    window_width: u32,
    window_height: u32,
}

impl Renderer {
    /// Initialize the renderer (GL context must be current)
    pub fn new(window: &Window) -> Result<Self, Box<dyn Error>> {
        // GL context already initialized by Window, just load resources
        let (width, height) = window.dimensions();

        // Load shaders
        let shader_program = ShaderProgram::from_files("shaders/block.vert", "shaders/block.frag")?;
        let highlight_shader = ShaderProgram::from_files("shaders/highlight.vert", "shaders/highlight.frag")?;
        let ui_shader = ShaderProgram::from_files("shaders/ui.vert", "shaders/ui.frag")?;

        // Load texture atlas
        let texture_atlas = TextureAtlas::new("assets/atlas.png")?;

        // Create skybox
        let skybox = Skybox::new()?;

        Ok(Self {
            shader_program,
            highlight_shader,
            ui_shader,
            texture_atlas,
            skybox,
            window_width: width,
            window_height: height,
        })
    }

    /// Resize the viewport when window is resized
    pub fn resize(&mut self, width: u32, height: u32) {
        self.window_width = width;
        self.window_height = height;

        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
    }

    /// Begin a new frame
    pub fn begin_frame(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    /// Render the skybox
    pub fn render_skybox(&self, camera: &Camera, time_of_day: f32) {
        self.skybox.render(camera, time_of_day);
    }

    /// Render chunk meshes
    pub fn render_chunks(
        &self,
        meshes: &[&ChunkMesh],
        camera: &Camera,
        time_of_day: f32,
        fog_color: Vec3,
        render_distance: f32,
    ) {
        self.shader_program.use_program();

        // Activate texture unit 0 and bind texture atlas
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
        }
        self.texture_atlas.bind();
        self.shader_program.set_int("u_TextureAtlas", 0);

        // Set uniforms
        self.shader_program.set_mat4("u_View", &camera.view_matrix());
        self.shader_program.set_mat4("u_Projection", &camera.projection_matrix(
            self.window_width as f32 / self.window_height as f32
        ));
        self.shader_program.set_vec3("u_CameraPos", &camera.position);
        self.shader_program.set_float("u_TimeOfDay", time_of_day);
        self.shader_program.set_vec3("u_FogColor", &fog_color);
        self.shader_program.set_float("u_FogStart", render_distance * 0.6);
        self.shader_program.set_float("u_FogEnd", render_distance);
        self.shader_program.set_float("u_RenderDistance", render_distance);

        // Render each chunk
        for mesh in meshes {
            if mesh.vertex_count() == 0 {
                continue;
            }

            // Set model matrix (chunk offset)
            let model = Mat4::from_translation(Vec3::new(
                mesh.chunk_x as f32 * 16.0,
                0.0,
                mesh.chunk_z as f32 * 16.0,
            ));
            self.shader_program.set_mat4("u_Model", &model);

            mesh.draw();
        }
    }

    /// Render block highlight wireframe
    pub fn render_highlight(&self, position: Vec3, camera: &Camera, time: f32) {
        self.highlight_shader.use_program();

        // Set uniforms
        let model = Mat4::from_translation(position);
        self.highlight_shader.set_mat4("u_Model", &model);
        self.highlight_shader.set_mat4("u_View", &camera.view_matrix());
        self.highlight_shader.set_mat4("u_Projection", &camera.projection_matrix(
            self.window_width as f32 / self.window_height as f32
        ));
        self.highlight_shader.set_float("u_Time", time);

        // Draw wireframe cube
        unsafe {
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            gl::Disable(gl::CULL_FACE);
            // TODO: Draw cube mesh
            gl::Enable(gl::CULL_FACE);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        }
    }

    /// End frame (swap buffers handled by Window)
    pub fn end_frame(&self) {
        // Swap buffers is handled by Window, this is just a marker
    }

    /// Get aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        self.window_width as f32 / self.window_height as f32
    }

    /// Render a single chunk mesh at a world position
    pub fn render_chunk(&self, mesh: &ChunkMesh, world_pos: Vec3, camera: &Camera) {
        if mesh.vertex_count() == 0 {
            return;
        }

        self.shader_program.use_program();

        // Activate texture unit 0 and bind texture atlas
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
        }
        self.texture_atlas.bind();
        self.shader_program.set_int("u_TextureAtlas", 0);

        // Set uniforms
        let model = Mat4::from_translation(world_pos);
        self.shader_program.set_mat4("u_Model", &model);
        self.shader_program.set_mat4("u_View", &camera.view_matrix());
        self.shader_program.set_mat4("u_Projection", &camera.projection_matrix(self.aspect_ratio()));
        self.shader_program.set_vec3("u_CameraPos", &camera.position);

        // Set fog and lighting uniforms (defaults - should be passed from game state)
        self.shader_program.set_float("u_TimeOfDay", 0.5); // Noon
        self.shader_program.set_vec3("u_FogColor", &Vec3::new(0.6, 0.8, 1.0)); // Sky blue
        self.shader_program.set_float("u_FogStart", 80.0);
        self.shader_program.set_float("u_FogEnd", 120.0);
        self.shader_program.set_float("u_RenderDistance", 120.0);

        mesh.draw();
    }

    /// Render block highlight
    pub fn render_block_highlight(&self, block_pos: &crate::types::WorldPos, camera: &Camera) {
        let pos = Vec3::new(block_pos.x as f32, block_pos.y as f32, block_pos.z as f32);
        self.render_highlight(pos, camera, 0.0);
    }

    /// Render a mob (simple box rendering)
    pub fn render_mob(&self, vertices: &[[f32; 3]], color: [f32; 3], camera: &Camera) {
        // TODO: Implement mob rendering with a simple shader
        // For now, this is a placeholder
    }
}
