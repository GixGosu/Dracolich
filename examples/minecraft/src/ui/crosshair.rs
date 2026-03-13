// Crosshair rendering at screen center
// Simple crosshair with customizable size and color

use glam::{Mat4, Vec4};
use std::mem;

/// Crosshair renderer for targeting blocks
pub struct CrosshairRenderer {
    vao: u32,
    vbo: u32,
    shader: u32,
    size: f32,
    thickness: f32,
    gap: f32,
}

impl CrosshairRenderer {
    pub fn new() -> Result<Self, String> {
        let shader = create_crosshair_shader()?;

        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            // Allocate buffer for lines (dynamic)
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (16 * 2 * mem::size_of::<f32>()) as isize,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );

            // Position attribute only
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                (2 * mem::size_of::<f32>()) as i32,
                std::ptr::null(),
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        Ok(Self {
            vao,
            vbo,
            shader,
            size: 10.0,      // Length of each line
            thickness: 2.0,  // Line thickness
            gap: 4.0,        // Gap from center
        })
    }

    /// Render crosshair at screen center
    pub fn render(&self, projection: &Mat4, screen_width: u32, screen_height: u32) {
        let center_x = screen_width as f32 / 2.0;
        let center_y = screen_height as f32 / 2.0;

        self.render_at(projection, center_x, center_y, Vec4::new(1.0, 1.0, 1.0, 0.8));
    }

    /// Render crosshair at specific position with custom color
    pub fn render_at(&self, projection: &Mat4, x: f32, y: f32, color: Vec4) {
        unsafe {
            gl::UseProgram(self.shader);

            // Set uniforms
            let proj_loc = gl::GetUniformLocation(self.shader, b"projection\0".as_ptr() as *const i8);
            gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.to_cols_array().as_ptr());

            let color_loc = gl::GetUniformLocation(self.shader, b"color\0".as_ptr() as *const i8);
            gl::Uniform4f(color_loc, color.x, color.y, color.z, color.w);

            // Build crosshair lines (4 segments forming a plus)
            let vertices = [
                // Horizontal line (left segment)
                x - self.size - self.gap, y,
                x - self.gap, y,

                // Horizontal line (right segment)
                x + self.gap, y,
                x + self.size + self.gap, y,

                // Vertical line (top segment)
                x, y - self.size - self.gap,
                x, y - self.gap,

                // Vertical line (bottom segment)
                x, y + self.gap,
                x, y + self.size + self.gap,
            ];

            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (vertices.len() * mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
            );

            // Set line width
            gl::LineWidth(self.thickness);

            // Draw 4 line segments
            gl::DrawArrays(gl::LINES, 0, 8);

            gl::LineWidth(1.0);
            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
    }

    /// Update crosshair appearance
    pub fn set_style(&mut self, size: f32, thickness: f32, gap: f32) {
        self.size = size;
        self.thickness = thickness;
        self.gap = gap;
    }
}

impl Drop for CrosshairRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteProgram(self.shader);
        }
    }
}

/// Create the crosshair shader
fn create_crosshair_shader() -> Result<u32, String> {
    let vertex_src = r#"
        #version 330 core
        layout (location = 0) in vec2 position;

        uniform mat4 projection;

        void main() {
            gl_Position = projection * vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_src = r#"
        #version 330 core
        out vec4 FragColor;

        uniform vec4 color;

        void main() {
            FragColor = color;
        }
    "#;

    let vertex_shader = compile_shader(vertex_src, gl::VERTEX_SHADER)?;
    let fragment_shader = compile_shader(fragment_src, gl::FRAGMENT_SHADER)?;

    let program = unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);

        // Check for link errors
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

        // Check for compile errors
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
