// Pause menu UI (ESC key)
// Shows Resume and Quit buttons

use super::{PauseMenuState, PauseButton, UIAction, text::TextRenderer};
use glam::{Mat4, Vec4};
use std::mem;

const BUTTON_WIDTH: f32 = 200.0;
const BUTTON_HEIGHT: f32 = 50.0;
const BUTTON_SPACING: f32 = 20.0;

/// Pause menu renderer
pub struct PauseMenuRenderer {
    vao: u32,
    vbo: u32,
    shader: u32,
}

impl PauseMenuRenderer {
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

    /// Render pause menu centered on screen
    pub fn render(&self, projection: &Mat4, text: &TextRenderer, state: &PauseMenuState) {
        let screen_width = 800.0;
        let screen_height = 600.0;

        unsafe {
            gl::UseProgram(self.shader);

            let proj_loc = gl::GetUniformLocation(self.shader, b"projection\0".as_ptr() as *const i8);
            gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.to_cols_array().as_ptr());

            gl::BindVertexArray(self.vao);
        }

        // Render semi-transparent overlay
        self.render_quad(
            0.0,
            0.0,
            screen_width,
            screen_height,
            Vec4::new(0.0, 0.0, 0.0, 0.7),
        );

        // Calculate button positions (centered)
        let center_x = screen_width / 2.0;
        let center_y = screen_height / 2.0;

        // Title
        let title = "Game Paused";
        let title_width = text.measure_text(title, 2.0);
        let title_x = center_x - title_width / 2.0;
        let title_y = center_y - 100.0;
        text.render_text(
            projection,
            title,
            title_x,
            title_y,
            2.0,
            Vec4::new(1.0, 1.0, 1.0, 1.0),
        );

        // Resume button
        let resume_y = center_y - BUTTON_HEIGHT / 2.0 - BUTTON_SPACING / 2.0;
        let resume_hovered = state.hovered_button == Some(PauseButton::Resume);
        self.render_button(
            projection,
            text,
            center_x,
            resume_y,
            "Resume",
            resume_hovered,
        );

        // Quit button
        let quit_y = center_y + BUTTON_HEIGHT / 2.0 + BUTTON_SPACING / 2.0;
        let quit_hovered = state.hovered_button == Some(PauseButton::Quit);
        self.render_button(
            projection,
            text,
            center_x,
            quit_y,
            "Quit to Desktop",
            quit_hovered,
        );

        unsafe {
            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
    }

    /// Render a single button
    fn render_button(
        &self,
        projection: &Mat4,
        text: &TextRenderer,
        center_x: f32,
        center_y: f32,
        label: &str,
        hovered: bool,
    ) {
        let x = center_x - BUTTON_WIDTH / 2.0;
        let y = center_y - BUTTON_HEIGHT / 2.0;

        // Button background
        let bg_color = if hovered {
            Vec4::new(0.4, 0.4, 0.4, 0.9)
        } else {
            Vec4::new(0.25, 0.25, 0.25, 0.9)
        };
        self.render_quad(x, y, BUTTON_WIDTH, BUTTON_HEIGHT, bg_color);

        // Button border
        let border_color = if hovered {
            Vec4::new(1.0, 1.0, 1.0, 1.0)
        } else {
            Vec4::new(0.6, 0.6, 0.6, 1.0)
        };
        self.render_border(x, y, BUTTON_WIDTH, BUTTON_HEIGHT, 2.0, border_color);

        // Button text (centered)
        let text_width = text.measure_text(label, 1.5);
        let (_, text_height) = text.char_size(1.5);
        let text_x = center_x - text_width / 2.0;
        let text_y = center_y - text_height / 2.0;

        let text_color = if hovered {
            Vec4::new(1.0, 1.0, 1.0, 1.0)
        } else {
            Vec4::new(0.9, 0.9, 0.9, 1.0)
        };
        text.render_text(projection, label, text_x, text_y, 1.5, text_color);
    }

    /// Handle click on pause menu
    pub fn handle_click(&self, state: &PauseMenuState, x: f32, y: f32) -> UIAction {
        let screen_width = 800.0;
        let screen_height = 600.0;

        let center_x = screen_width / 2.0;
        let center_y = screen_height / 2.0;

        // Check Resume button
        let resume_y = center_y - BUTTON_HEIGHT / 2.0 - BUTTON_SPACING / 2.0;
        if self.is_point_in_button(x, y, center_x, resume_y) {
            return UIAction::PauseButton(PauseButton::Resume);
        }

        // Check Quit button
        let quit_y = center_y + BUTTON_HEIGHT / 2.0 + BUTTON_SPACING / 2.0;
        if self.is_point_in_button(x, y, center_x, quit_y) {
            return UIAction::PauseButton(PauseButton::Quit);
        }

        UIAction::None
    }

    /// Check if point is inside button bounds
    fn is_point_in_button(&self, x: f32, y: f32, center_x: f32, center_y: f32) -> bool {
        let button_x = center_x - BUTTON_WIDTH / 2.0;
        let button_y = center_y - BUTTON_HEIGHT / 2.0;

        x >= button_x
            && x <= button_x + BUTTON_WIDTH
            && y >= button_y
            && y <= button_y + BUTTON_HEIGHT
    }

    /// Update hover state based on mouse position
    pub fn update_hover(&self, x: f32, y: f32) -> Option<PauseButton> {
        let screen_width = 800.0;
        let screen_height = 600.0;

        let center_x = screen_width / 2.0;
        let center_y = screen_height / 2.0;

        // Check Resume button
        let resume_y = center_y - BUTTON_HEIGHT / 2.0 - BUTTON_SPACING / 2.0;
        if self.is_point_in_button(x, y, center_x, resume_y) {
            return Some(PauseButton::Resume);
        }

        // Check Quit button
        let quit_y = center_y + BUTTON_HEIGHT / 2.0 + BUTTON_SPACING / 2.0;
        if self.is_point_in_button(x, y, center_x, quit_y) {
            return Some(PauseButton::Quit);
        }

        None
    }

    /// Render a colored quad
    fn render_quad(&self, x: f32, y: f32, w: f32, h: f32, color: Vec4) {
        let vertices = [
            x, y + h, color.x, color.y, color.z, color.w,
            x, y, color.x, color.y, color.z, color.w,
            x + w, y, color.x, color.y, color.z, color.w,

            x, y + h, color.x, color.y, color.z, color.w,
            x + w, y, color.x, color.y, color.z, color.w,
            x + w, y + h, color.x, color.y, color.z, color.w,
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

    /// Render border
    fn render_border(&self, x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Vec4) {
        self.render_quad(x, y, w, thickness, color); // Top
        self.render_quad(x, y + h - thickness, w, thickness, color); // Bottom
        self.render_quad(x, y, thickness, h, color); // Left
        self.render_quad(x + w - thickness, y, thickness, h, color); // Right
    }
}

impl Drop for PauseMenuRenderer {
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
