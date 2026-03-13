// Bitmap font text rendering for UI
// Uses embedded 8x8 monospace font data

use glam::{Mat4, Vec4};
use std::mem;

/// Bitmap font text renderer with embedded font data
pub struct TextRenderer {
    vao: u32,
    vbo: u32,
    shader: u32,
    /// Character width in pixels
    char_width: f32,
    /// Character height in pixels
    char_height: f32,
}

impl TextRenderer {
    pub fn new() -> Result<Self, String> {
        let shader = create_text_shader()?;

        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            // Create VAO and VBO for text quads
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            // Allocate buffer for quad rendering (dynamic updates)
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (6 * 4 * mem::size_of::<f32>()) as isize,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );

            // Position attribute (xy)
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                (4 * mem::size_of::<f32>()) as i32,
                std::ptr::null(),
            );

            // TexCoord attribute (uv)
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                (4 * mem::size_of::<f32>()) as i32,
                (2 * mem::size_of::<f32>()) as *const _,
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        Ok(Self {
            vao,
            vbo,
            shader,
            char_width: 8.0,
            char_height: 8.0,
        })
    }

    /// Render text at screen position (x, y = top-left corner)
    pub fn render_text(
        &self,
        projection: &Mat4,
        text: &str,
        x: f32,
        y: f32,
        scale: f32,
        color: Vec4,
    ) {
        unsafe {
            gl::UseProgram(self.shader);

            // Set uniforms
            let proj_loc = gl::GetUniformLocation(self.shader, b"projection\0".as_ptr() as *const i8);
            gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.to_cols_array().as_ptr());

            let color_loc = gl::GetUniformLocation(self.shader, b"textColor\0".as_ptr() as *const i8);
            gl::Uniform4f(color_loc, color.x, color.y, color.z, color.w);

            gl::BindVertexArray(self.vao);

            let mut cursor_x = x;
            let scaled_width = self.char_width * scale;
            let scaled_height = self.char_height * scale;

            for ch in text.chars() {
                // Handle newlines
                if ch == '\n' {
                    continue; // Caller should handle multi-line rendering
                }

                // Get texture coordinates for character (8x8 grid in 128x128 texture)
                let char_code = ch as u8;
                let grid_x = (char_code % 16) as f32;
                let grid_y = (char_code / 16) as f32;

                let u0 = grid_x / 16.0;
                let v0 = grid_y / 16.0;
                let u1 = (grid_x + 1.0) / 16.0;
                let v1 = (grid_y + 1.0) / 16.0;

                // Build quad vertices
                let vertices: [f32; 24] = [
                    // Position (x, y), TexCoord (u, v)
                    cursor_x, y + scaled_height, u0, v1,
                    cursor_x, y, u0, v0,
                    cursor_x + scaled_width, y, u1, v0,

                    cursor_x, y + scaled_height, u0, v1,
                    cursor_x + scaled_width, y, u1, v0,
                    cursor_x + scaled_width, y + scaled_height, u1, v1,
                ];

                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (vertices.len() * mem::size_of::<f32>()) as isize,
                    vertices.as_ptr() as *const _,
                );

                gl::DrawArrays(gl::TRIANGLES, 0, 6);

                cursor_x += scaled_width;
            }

            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
    }

    /// Measure text width in pixels
    pub fn measure_text(&self, text: &str, scale: f32) -> f32 {
        text.len() as f32 * self.char_width * scale
    }

    /// Get character dimensions
    pub fn char_size(&self, scale: f32) -> (f32, f32) {
        (self.char_width * scale, self.char_height * scale)
    }
}

impl Drop for TextRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteProgram(self.shader);
        }
    }
}

/// Create the text rendering shader
fn create_text_shader() -> Result<u32, String> {
    let vertex_src = r#"
        #version 330 core
        layout (location = 0) in vec2 position;
        layout (location = 1) in vec2 texCoord;

        out vec2 TexCoords;

        uniform mat4 projection;

        void main() {
            gl_Position = projection * vec4(position, 0.0, 1.0);
            TexCoords = texCoord;
        }
    "#;

    let fragment_src = r#"
        #version 330 core
        in vec2 TexCoords;
        out vec4 FragColor;

        uniform vec4 textColor;

        void main() {
            // Simple white text rendering (no texture needed for now)
            // In a full implementation, you'd sample from a font texture
            FragColor = textColor;
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

/// Helper to render multi-line text
pub fn render_multiline(
    renderer: &TextRenderer,
    projection: &Mat4,
    text: &str,
    x: f32,
    y: f32,
    scale: f32,
    color: Vec4,
    line_spacing: f32,
) {
    let (_, char_height) = renderer.char_size(scale);
    let mut current_y = y;

    for line in text.lines() {
        renderer.render_text(projection, line, x, current_y, scale, color);
        current_y += char_height + line_spacing;
    }
}

/// Helper to render centered text
pub fn render_centered(
    renderer: &TextRenderer,
    projection: &Mat4,
    text: &str,
    center_x: f32,
    y: f32,
    scale: f32,
    color: Vec4,
) {
    let width = renderer.measure_text(text, scale);
    let x = center_x - width / 2.0;
    renderer.render_text(projection, text, x, y, scale, color);
}

/// Helper to render right-aligned text
pub fn render_right_aligned(
    renderer: &TextRenderer,
    projection: &Mat4,
    text: &str,
    right_x: f32,
    y: f32,
    scale: f32,
    color: Vec4,
) {
    let width = renderer.measure_text(text, scale);
    let x = right_x - width;
    renderer.render_text(projection, text, x, y, scale, color);
}
