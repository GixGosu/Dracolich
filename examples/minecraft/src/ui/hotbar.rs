// Hotbar rendering at bottom of screen
// Shows 9 inventory slots with item icons, counts, and selection highlight

use super::{HotbarState, ItemType, text::TextRenderer};
use glam::{Mat4, Vec4};
use std::mem;

const SLOT_SIZE: f32 = 40.0;
const SLOT_SPACING: f32 = 4.0;
const SELECTED_BORDER: f32 = 2.0;
const HOTBAR_BOTTOM_MARGIN: f32 = 20.0;

/// Hotbar renderer for inventory quick access
pub struct HotbarRenderer {
    vao: u32,
    vbo: u32,
    shader: u32,
}

impl HotbarRenderer {
    pub fn new() -> Result<Self, String> {
        let shader = create_ui_shader()?;

        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            // Allocate buffer for quads (dynamic)
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (6 * 4 * mem::size_of::<f32>()) as isize,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );

            // Position + Color attribute (xy + rgba)
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

    /// Render the hotbar at bottom of screen
    pub fn render(&self, projection: &Mat4, text: &TextRenderer, state: &HotbarState) {
        let screen_width = 800.0; // TODO: get from projection or pass as parameter

        // Calculate hotbar position (centered at bottom)
        let total_width = 9.0 * SLOT_SIZE + 8.0 * SLOT_SPACING;
        let start_x = (screen_width - total_width) / 2.0;
        let start_y = 600.0 - HOTBAR_BOTTOM_MARGIN - SLOT_SIZE; // TODO: get screen height

        unsafe {
            gl::UseProgram(self.shader);

            let proj_loc = gl::GetUniformLocation(self.shader, b"projection\0".as_ptr() as *const i8);
            gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.to_cols_array().as_ptr());

            gl::BindVertexArray(self.vao);
        }

        // Render each slot
        for i in 0..9 {
            let x = start_x + i as f32 * (SLOT_SIZE + SLOT_SPACING);
            let y = start_y;

            // Render slot background
            let slot_color = if i == state.selected_slot {
                Vec4::new(0.8, 0.8, 0.8, 0.9) // Highlighted
            } else {
                Vec4::new(0.3, 0.3, 0.3, 0.7) // Normal
            };
            self.render_quad(x, y, SLOT_SIZE, SLOT_SIZE, slot_color);

            // Render selected border
            if i == state.selected_slot {
                self.render_border(x, y, SLOT_SIZE, SLOT_SIZE, SELECTED_BORDER, Vec4::new(1.0, 1.0, 1.0, 1.0));
            }

            // Render item if present
            if let Some(ref item) = state.items[i] {
                // Render item icon (placeholder - use texture in full implementation)
                let icon_margin = 4.0;
                let icon_size = SLOT_SIZE - 2.0 * icon_margin;
                let icon_color = get_item_color(item.item_type);
                self.render_quad(x + icon_margin, y + icon_margin, icon_size, icon_size, icon_color);

                // Render item count if > 1
                if item.count > 1 {
                    let count_text = format!("{}", item.count);
                    let text_x = x + SLOT_SIZE - 12.0;
                    let text_y = y + SLOT_SIZE - 12.0;
                    text.render_text(
                        projection,
                        &count_text,
                        text_x,
                        text_y,
                        1.0,
                        Vec4::new(1.0, 1.0, 1.0, 1.0),
                    );
                }

                // Render durability bar for tools
                if let Some(durability) = item.durability {
                    let bar_width = SLOT_SIZE - 8.0;
                    let bar_height = 3.0;
                    let bar_x = x + 4.0;
                    let bar_y = y + SLOT_SIZE - 6.0;

                    // Background
                    self.render_quad(bar_x, bar_y, bar_width, bar_height, Vec4::new(0.2, 0.2, 0.2, 0.8));

                    // Durability fill
                    let fill_width = bar_width * durability;
                    let durability_color = if durability > 0.5 {
                        Vec4::new(0.0, 1.0, 0.0, 0.9) // Green
                    } else if durability > 0.25 {
                        Vec4::new(1.0, 1.0, 0.0, 0.9) // Yellow
                    } else {
                        Vec4::new(1.0, 0.0, 0.0, 0.9) // Red
                    };
                    self.render_quad(bar_x, bar_y, fill_width, bar_height, durability_color);
                }
            }
        }

        unsafe {
            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
    }

    /// Render a colored quad
    fn render_quad(&self, x: f32, y: f32, w: f32, h: f32, color: Vec4) {
        let vertices = [
            // Position (xy), Color (rgba)
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

    /// Render border around a quad
    fn render_border(&self, x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Vec4) {
        // Top
        self.render_quad(x - thickness, y - thickness, w + 2.0 * thickness, thickness, color);
        // Bottom
        self.render_quad(x - thickness, y + h, w + 2.0 * thickness, thickness, color);
        // Left
        self.render_quad(x - thickness, y, thickness, h, color);
        // Right
        self.render_quad(x + w, y, thickness, h, color);
    }
}

impl Drop for HotbarRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteProgram(self.shader);
        }
    }
}

/// Get placeholder color for item type (in full version, use texture atlas)
fn get_item_color(item_type: ItemType) -> Vec4 {
    match item_type {
        ItemType::Grass => Vec4::new(0.3, 0.8, 0.3, 1.0),
        ItemType::Dirt => Vec4::new(0.6, 0.4, 0.2, 1.0),
        ItemType::Stone => Vec4::new(0.5, 0.5, 0.5, 1.0),
        ItemType::Cobblestone => Vec4::new(0.4, 0.4, 0.4, 1.0),
        ItemType::Sand => Vec4::new(0.9, 0.8, 0.5, 1.0),
        ItemType::WoodOak => Vec4::new(0.5, 0.3, 0.1, 1.0),
        ItemType::WoodBirch => Vec4::new(0.7, 0.6, 0.4, 1.0),
        ItemType::Planks => Vec4::new(0.7, 0.5, 0.3, 1.0),
        ItemType::WoodenPickaxe => Vec4::new(0.6, 0.4, 0.2, 1.0),
        ItemType::StonePickaxe => Vec4::new(0.5, 0.5, 0.5, 1.0),
        ItemType::OreCoal => Vec4::new(0.3, 0.3, 0.3, 1.0),
        ItemType::OreIron => Vec4::new(0.7, 0.6, 0.5, 1.0),
        ItemType::Diamond => Vec4::new(0.2, 0.8, 0.9, 1.0),
        _ => Vec4::new(0.5, 0.5, 0.5, 1.0),
    }
}

/// Create UI shader for colored quads
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
