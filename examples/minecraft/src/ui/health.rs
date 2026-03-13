// Health bar rendering
// Displays hearts representing player health

use super::text::TextRenderer;
use glam::{Mat4, Vec4};
use std::mem;

const HEART_SIZE: f32 = 16.0;
const HEART_SPACING: f32 = 2.0;
const HEALTH_MARGIN: f32 = 20.0;

/// Health renderer showing hearts
pub struct HealthRenderer {
    vao: u32,
    vbo: u32,
    shader: u32,
}

impl HealthRenderer {
    pub fn new() -> Result<Self, String> {
        let shader = create_ui_shader()?;

        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (6 * 6 * mem::size_of::<f32>()) as isize,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );

            // Position + Color
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                (6 * mem::size_of::<f32>()) as i32,
                std::ptr::null(),
            );

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                4,
                gl::FLOAT,
                gl::FALSE,
                (6 * mem::size_of::<f32>()) as i32,
                (2 * mem::size_of::<f32>()) as *const _,
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        Ok(Self { vao, vbo, shader })
    }

    /// Render health as hearts (above hotbar)
    pub fn render(&self, projection: &Mat4, text: &TextRenderer, health: u32, max_health: u32) {
        let screen_width = 800.0;  // TODO: get from projection
        let screen_height = 600.0; // TODO: get from projection

        // Position above hotbar
        let hearts = (max_health + 1) / 2; // 2 HP per heart
        let total_width = hearts as f32 * (HEART_SIZE + HEART_SPACING) - HEART_SPACING;
        let start_x = (screen_width - total_width) / 2.0;
        let start_y = screen_height - 80.0; // Above hotbar

        unsafe {
            gl::UseProgram(self.shader);

            let proj_loc = gl::GetUniformLocation(self.shader, b"projection\0".as_ptr() as *const i8);
            gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.to_cols_array().as_ptr());

            gl::BindVertexArray(self.vao);
        }

        // Render hearts
        for i in 0..hearts {
            let x = start_x + i as f32 * (HEART_SIZE + HEART_SPACING);
            let y = start_y;

            // Calculate heart state (full, half, empty)
            let heart_hp = (i * 2) as u32;
            let remaining = health.saturating_sub(heart_hp);

            let color = if remaining >= 2 {
                // Full heart
                Vec4::new(1.0, 0.0, 0.0, 1.0)
            } else if remaining == 1 {
                // Half heart (render as darker red)
                Vec4::new(0.7, 0.0, 0.0, 1.0)
            } else {
                // Empty heart (outline only)
                Vec4::new(0.3, 0.0, 0.0, 0.5)
            };

            // Render heart shape (simplified as square for now)
            // In full version, use heart-shaped sprite
            self.render_heart(x, y, HEART_SIZE, color);
        }

        unsafe {
            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }

        // Render numeric health display
        let health_text = format!("{}/{}", health, max_health);
        let text_x = start_x + total_width + 10.0;
        let text_y = start_y + 4.0;
        text.render_text(
            projection,
            &health_text,
            text_x,
            text_y,
            1.0,
            Vec4::new(1.0, 1.0, 1.0, 0.8),
        );
    }

    /// Render a heart (simplified as rounded square)
    fn render_heart(&self, x: f32, y: f32, size: f32, color: Vec4) {
        let vertices = [
            // Position (xy), Color (rgba)
            x, y + size, color.x, color.y, color.z, color.w,
            x, y, color.x, color.y, color.z, color.w,
            x + size, y, color.x, color.y, color.z, color.w,

            x, y + size, color.x, color.y, color.z, color.w,
            x + size, y, color.x, color.y, color.z, color.w,
            x + size, y + size, color.x, color.y, color.z, color.w,
        ];

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (vertices.len() * mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
            );
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }
}

impl Drop for HealthRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteProgram(self.shader);
        }
    }
}

fn create_ui_shader() -> Result<u32, String> {
    let vertex_src = r#"
        #version 330 core
        layout (location = 0) in vec2 position;
        layout (location = 1) in vec4 color;

        out vec4 Color;

        uniform mat4 projection;

        void main() {
            gl_Position = projection * vec4(position, 0.0, 1.0);
            Color = color;
        }
    "#;

    let fragment_src = r#"
        #version 330 core
        in vec4 Color;
        out vec4 FragColor;

        void main() {
            FragColor = Color;
        }
    "#;

    let vertex_shader = compile_shader(vertex_src, gl::VERTEX_SHADER)?;
    let fragment_shader = compile_shader(fragment_src, gl::FRAGMENT_SHADER)?;

    let program = unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);

        let mut success = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buffer = vec![0u8; len as usize];
            gl::GetProgramInfoLog(program, len, &mut len, buffer.as_mut_ptr() as *mut i8);
            return Err(String::from_utf8_lossy(&buffer).to_string());
        }

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        program
    };

    Ok(program)
}

fn compile_shader(src: &str, shader_type: u32) -> Result<u32, String> {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        let src_ptr = src.as_ptr() as *const i8;
        let src_len = src.len() as i32;
        gl::ShaderSource(shader, 1, &src_ptr, &src_len);
        gl::CompileShader(shader);

        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buffer = vec![0u8; len as usize];
            gl::GetShaderInfoLog(shader, len, &mut len, buffer.as_mut_ptr() as *mut i8);
            return Err(String::from_utf8_lossy(&buffer).to_string());
        }

        Ok(shader)
    }
}
