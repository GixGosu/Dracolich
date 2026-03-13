// Full inventory screen UI
// Shows 36 inventory slots, crafting grid, and drag-drop functionality

use super::{InventoryState, ItemType, UIAction, DragAction, text::TextRenderer};
use glam::{Mat4, Vec4};
use std::mem;

const SLOT_SIZE: f32 = 36.0;
const SLOT_SPACING: f32 = 4.0;
const GRID_MARGIN: f32 = 20.0;
const BACKGROUND_ALPHA: f32 = 0.8;

/// Inventory screen renderer with crafting grid
pub struct InventoryScreenRenderer {
    vao: u32,
    vbo: u32,
    shader: u32,
}

impl InventoryScreenRenderer {
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

    /// Render full inventory screen
    pub fn render(&self, projection: &Mat4, text: &TextRenderer, state: &InventoryState) {
        let screen_width = 800.0;
        let screen_height = 600.0;

        unsafe {
            gl::UseProgram(self.shader);

            let proj_loc = gl::GetUniformLocation(self.shader, b"projection\0".as_ptr() as *const i8);
            gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.to_cols_array().as_ptr());

            gl::BindVertexArray(self.vao);
        }

        // Render semi-transparent background
        self.render_quad(
            0.0,
            0.0,
            screen_width,
            screen_height,
            Vec4::new(0.0, 0.0, 0.0, 0.6),
        );

        // Calculate inventory panel position (centered)
        let panel_width = 9.0 * (SLOT_SIZE + SLOT_SPACING) + 2.0 * GRID_MARGIN;
        let panel_height = 6.0 * (SLOT_SIZE + SLOT_SPACING) + 2.0 * GRID_MARGIN + 60.0; // Extra for crafting
        let panel_x = (screen_width - panel_width) / 2.0;
        let panel_y = (screen_height - panel_height) / 2.0;

        // Render panel background
        self.render_quad(
            panel_x,
            panel_y,
            panel_width,
            panel_height,
            Vec4::new(0.2, 0.2, 0.2, BACKGROUND_ALPHA),
        );

        // Title
        text.render_text(
            projection,
            "Inventory",
            panel_x + GRID_MARGIN,
            panel_y + 10.0,
            1.5,
            Vec4::new(1.0, 1.0, 1.0, 1.0),
        );

        // Render inventory grid (4 rows of 9 slots)
        let inv_start_x = panel_x + GRID_MARGIN;
        let inv_start_y = panel_y + 80.0;

        for row in 0..4 {
            for col in 0..9 {
                let slot = row * 9 + col;
                if slot >= 36 {
                    break;
                }

                let x = inv_start_x + col as f32 * (SLOT_SIZE + SLOT_SPACING);
                let y = inv_start_y + row as f32 * (SLOT_SIZE + SLOT_SPACING);

                self.render_inventory_slot(projection, text, x, y, state.inventory_slots[slot].as_ref());
            }
        }

        // Render crafting grid (3x3) in top-right
        let craft_start_x = panel_x + panel_width - GRID_MARGIN - 3.0 * (SLOT_SIZE + SLOT_SPACING);
        let craft_start_y = panel_y + 40.0;

        text.render_text(
            projection,
            "Crafting",
            craft_start_x,
            craft_start_y - 20.0,
            1.2,
            Vec4::new(1.0, 1.0, 1.0, 1.0),
        );

        for row in 0..3 {
            for col in 0..3 {
                let slot = row * 3 + col;
                let x = craft_start_x + col as f32 * (SLOT_SIZE + SLOT_SPACING);
                let y = craft_start_y + row as f32 * (SLOT_SIZE + SLOT_SPACING);

                self.render_inventory_slot(projection, text, x, y, state.crafting_slots[slot].as_ref());
            }
        }

        // Render crafting result slot (with arrow)
        let result_x = craft_start_x + 3.5 * (SLOT_SIZE + SLOT_SPACING);
        let result_y = craft_start_y + (SLOT_SIZE + SLOT_SPACING);

        // Arrow
        text.render_text(
            projection,
            "->",
            result_x - 20.0,
            result_y + 10.0,
            1.5,
            Vec4::new(1.0, 1.0, 1.0, 1.0),
        );

        self.render_inventory_slot(projection, text, result_x, result_y, state.crafting_result.as_ref());

        // Render held item (if any) at cursor position
        // TODO: Would need mouse position from game state
        // For now, just show it in corner if present
        if let Some(ref held) = state.held_item {
            let held_x = screen_width - 60.0;
            let held_y = 20.0;
            self.render_inventory_slot(projection, text, held_x, held_y, Some(held));
            text.render_text(
                projection,
                "Held",
                held_x,
                held_y - 15.0,
                1.0,
                Vec4::new(1.0, 1.0, 0.0, 1.0),
            );
        }

        unsafe {
            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
    }

    /// Render a single inventory slot with item
    fn render_inventory_slot(
        &self,
        projection: &Mat4,
        text: &TextRenderer,
        x: f32,
        y: f32,
        item: Option<&super::InventoryItem>,
    ) {
        // Slot background
        self.render_quad(
            x,
            y,
            SLOT_SIZE,
            SLOT_SIZE,
            Vec4::new(0.15, 0.15, 0.15, 0.9),
        );

        // Slot border
        self.render_border(x, y, SLOT_SIZE, SLOT_SIZE, 1.0, Vec4::new(0.4, 0.4, 0.4, 0.9));

        // Render item if present
        if let Some(item) = item {
            // Item icon (colored square placeholder)
            let icon_margin = 4.0;
            let icon_size = SLOT_SIZE - 2.0 * icon_margin;
            let icon_color = get_item_color(item.item_type);
            self.render_quad(x + icon_margin, y + icon_margin, icon_size, icon_size, icon_color);

            // Item count
            if item.count > 1 {
                let count_text = format!("{}", item.count);
                let text_x = x + SLOT_SIZE - 16.0;
                let text_y = y + SLOT_SIZE - 12.0;
                text.render_text(
                    projection,
                    &count_text,
                    text_x,
                    text_y,
                    0.9,
                    Vec4::new(1.0, 1.0, 1.0, 1.0),
                );
            }

            // Durability bar for tools
            if let Some(durability) = item.durability {
                let bar_width = SLOT_SIZE - 8.0;
                let bar_height = 2.0;
                let bar_x = x + 4.0;
                let bar_y = y + SLOT_SIZE - 5.0;

                // Background
                self.render_quad(bar_x, bar_y, bar_width, bar_height, Vec4::new(0.2, 0.2, 0.2, 0.9));

                // Fill
                let fill_width = bar_width * durability;
                let color = if durability > 0.5 {
                    Vec4::new(0.0, 1.0, 0.0, 0.9)
                } else if durability > 0.25 {
                    Vec4::new(1.0, 1.0, 0.0, 0.9)
                } else {
                    Vec4::new(1.0, 0.0, 0.0, 0.9)
                };
                self.render_quad(bar_x, bar_y, fill_width, bar_height, color);
            }
        }
    }

    /// Handle click on inventory screen
    pub fn handle_click(&self, state: &InventoryState, x: f32, y: f32) -> UIAction {
        // TODO: Calculate which slot was clicked based on x, y
        // For now, return placeholder
        let slot = self.get_slot_at(x, y);
        if let Some(slot_index) = slot {
            if slot_index < 36 {
                UIAction::SlotClick { slot: slot_index }
            } else if slot_index < 45 {
                UIAction::CraftingClick { slot: slot_index - 36 }
            } else if slot_index == 45 {
                UIAction::TakeResult
            } else {
                UIAction::None
            }
        } else {
            UIAction::None
        }
    }

    /// Handle drag action
    pub fn handle_drag(&self, state: &InventoryState, x: f32, y: f32) -> Option<DragAction> {
        // TODO: Implement drag logic
        None
    }

    /// Get slot index at screen position
    fn get_slot_at(&self, x: f32, y: f32) -> Option<usize> {
        // TODO: Implement proper hit testing
        // This is a placeholder
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

impl Drop for InventoryScreenRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteProgram(self.shader);
        }
    }
}

/// Get color for item type (placeholder)
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
        ItemType::Diamond => Vec4::new(0.2, 0.8, 0.9, 1.0),
        _ => Vec4::new(0.5, 0.5, 0.5, 1.0),
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
