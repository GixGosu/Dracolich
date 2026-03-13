/// Skybox rendering with dynamic sky, sun, moon, and stars
/// Renders a sky dome with celestial bodies based on time of day

use super::camera::Camera;
use super::gl_wrapper::{enable_vertex_attrib, VBO, VAO};
use super::shader::ShaderProgram;
use gl::types::*;
use glam::{Mat4, Vec3};
use std::error::Error;
use std::f32::consts::PI;

pub struct Skybox {
    vao: VAO,
    vbo: VBO,
    vertex_count: usize,
    shader: ShaderProgram,
}

impl Skybox {
    /// Create a new skybox with sky dome geometry
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Load sky shaders
        let shader = ShaderProgram::from_files("shaders/sky.vert", "shaders/sky.frag")?;

        // Generate sky dome vertices
        let vertices = Self::generate_dome_vertices(50.0, 32, 16);
        let vertex_count = vertices.len();

        // Create VAO and VBO
        let vao = VAO::new();
        let vbo = VBO::new(gl::ARRAY_BUFFER);

        vao.bind();
        vbo.bind();

        // Upload vertex data
        vbo.upload_data(&vertices, gl::STATIC_DRAW);

        // Configure vertex attributes (only position for skybox)
        enable_vertex_attrib(0, 3, gl::FLOAT, false, 0, 0);

        vao.unbind();
        vbo.unbind();

        Ok(Self {
            vao,
            vbo,
            vertex_count,
            shader,
        })
    }

    /// Generate vertices for a hemisphere sky dome
    fn generate_dome_vertices(radius: f32, segments: usize, rings: usize) -> Vec<f32> {
        let mut vertices = Vec::new();

        // Generate dome vertices (hemisphere)
        for ring in 0..rings {
            let phi0 = (ring as f32 / rings as f32) * PI * 0.5;
            let phi1 = ((ring + 1) as f32 / rings as f32) * PI * 0.5;

            for segment in 0..segments {
                let theta0 = (segment as f32 / segments as f32) * 2.0 * PI;
                let theta1 = ((segment + 1) as f32 / segments as f32) * 2.0 * PI;

                // Calculate vertices for quad
                let v0 = Self::sphere_point(radius, theta0, phi0);
                let v1 = Self::sphere_point(radius, theta1, phi0);
                let v2 = Self::sphere_point(radius, theta1, phi1);
                let v3 = Self::sphere_point(radius, theta0, phi1);

                // First triangle
                vertices.extend_from_slice(&[v0.x, v0.y, v0.z]);
                vertices.extend_from_slice(&[v1.x, v1.y, v1.z]);
                vertices.extend_from_slice(&[v2.x, v2.y, v2.z]);

                // Second triangle
                vertices.extend_from_slice(&[v2.x, v2.y, v2.z]);
                vertices.extend_from_slice(&[v3.x, v3.y, v3.z]);
                vertices.extend_from_slice(&[v0.x, v0.y, v0.z]);
            }
        }

        vertices
    }

    /// Calculate point on sphere given angles
    fn sphere_point(radius: f32, theta: f32, phi: f32) -> Vec3 {
        Vec3::new(
            radius * phi.cos() * theta.cos(),
            radius * phi.sin(),
            radius * phi.cos() * theta.sin(),
        )
    }

    /// Calculate sun direction based on time of day
    /// Time of day: 0.0 = midnight, 0.5 = noon, 1.0 = midnight
    fn calculate_sun_direction(time_of_day: f32) -> Vec3 {
        // Sun rotates from east to west
        // At noon (0.5), sun is at zenith
        // At midnight (0.0 or 1.0), sun is below horizon
        let angle = (time_of_day - 0.5) * 2.0 * PI;

        Vec3::new(
            angle.cos(),
            angle.sin(),
            0.0,
        ).normalize()
    }

    /// Calculate moon direction (opposite to sun)
    fn calculate_moon_direction(time_of_day: f32) -> Vec3 {
        -Self::calculate_sun_direction(time_of_day)
    }

    /// Render the skybox
    pub fn render(&self, camera: &Camera, time_of_day: f32) {
        // Disable depth writing but keep depth test
        unsafe {
            gl::DepthMask(gl::FALSE);
        }

        self.shader.use_program();

        // Create view matrix without translation (keep sky centered on camera)
        let view = Mat4::look_at_rh(
            Vec3::ZERO,
            camera.forward(),
            Vec3::Y,
        );

        // Set uniforms
        self.shader.set_mat4("u_View", &view);
        self.shader.set_mat4("u_Projection", &camera.projection_matrix(16.0 / 9.0)); // Will be overridden by actual aspect
        self.shader.set_float("u_TimeOfDay", time_of_day);

        // Calculate and set celestial body directions
        let sun_dir = Self::calculate_sun_direction(time_of_day);
        let moon_dir = Self::calculate_moon_direction(time_of_day);
        self.shader.set_vec3("u_SunDirection", &sun_dir);
        self.shader.set_vec3("u_MoonDirection", &moon_dir);

        // Draw sky dome
        self.vao.bind();
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, self.vertex_count as GLsizei);
        }
        self.vao.unbind();

        // Re-enable depth writing
        unsafe {
            gl::DepthMask(gl::TRUE);
        }
    }

    /// Get sky color for fog based on time of day
    pub fn get_fog_color(time_of_day: f32) -> Vec3 {
        let day_night_cycle = 1.0 - (time_of_day - 0.5).abs() * 2.0;

        let sky_color_day = Vec3::new(0.53, 0.81, 0.92);
        let sky_color_night = Vec3::new(0.01, 0.01, 0.05);

        // Add sunset tint
        let mut color = sky_color_night.lerp(sky_color_day, day_night_cycle);

        // Sunset/sunrise (around 0.25 and 0.75)
        let sunset_color = Vec3::new(1.0, 0.5, 0.3);
        if (time_of_day - 0.25).abs() < 0.05 || (time_of_day - 0.75).abs() < 0.05 {
            let sunset_factor = if (time_of_day - 0.25).abs() < 0.05 {
                1.0 - (time_of_day - 0.25).abs() * 20.0
            } else {
                1.0 - (time_of_day - 0.75).abs() * 20.0
            };
            color = color.lerp(sunset_color, sunset_factor * 0.3);
        }

        color
    }
}
