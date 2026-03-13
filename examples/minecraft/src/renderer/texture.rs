/// Texture atlas loading and UV coordinate calculation
/// Manages the block texture atlas and provides UV coordinates for each block face

use super::gl_wrapper::Texture;
use crate::types::BlockType;
use gl::types::*;
use image::GenericImageView;
use std::error::Error;

/// Texture atlas that contains all block textures in a grid
pub struct TextureAtlas {
    texture: Texture,
    atlas_width: u32,
    atlas_height: u32,
    tile_size: u32,
    tiles_per_row: u32,
}

impl TextureAtlas {
    /// Load texture atlas from a PNG file
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        // Load image
        let img = image::open(path)?;
        let (width, height) = img.dimensions();
        let rgba = img.to_rgba8();

        // Assume square tiles - calculate tile size
        // For a 16x16 texture atlas with 256x256px total, tile_size = 16
        let tile_size = 16; // Each block texture is 16x16 pixels
        let tiles_per_row = width / tile_size;

        // Create and configure texture
        let texture = Texture::new();
        texture.bind(gl::TEXTURE_2D);

        // Upload texture data
        texture.upload_2d(
            width as i32,
            height as i32,
            gl::RGBA as GLint,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            rgba.as_ptr() as *const _,
        );

        // Set texture parameters
        texture.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
        texture.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
        texture.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_LINEAR as GLint);
        texture.set_parameter(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);

        // Generate mipmaps for distant blocks
        texture.generate_mipmap(gl::TEXTURE_2D);

        texture.unbind(gl::TEXTURE_2D);

        Ok(Self {
            texture,
            atlas_width: width,
            atlas_height: height,
            tile_size,
            tiles_per_row,
        })
    }

    /// Bind the texture atlas
    pub fn bind(&self) {
        self.texture.bind(gl::TEXTURE_2D);
    }

    /// Unbind the texture atlas
    pub fn unbind(&self) {
        self.texture.unbind(gl::TEXTURE_2D);
    }

    /// Get UV coordinates for a specific tile index
    /// Returns (u_min, v_min, u_max, v_max)
    /// Note: No V-flip needed - glTexImage2D preserves row order
    pub fn get_uv_coords(&self, tile_index: usize) -> (f32, f32, f32, f32) {
        let tile_x = (tile_index as u32) % self.tiles_per_row;
        let tile_y = (tile_index as u32) / self.tiles_per_row;

        let u_min = (tile_x * self.tile_size) as f32 / self.atlas_width as f32;
        let u_max = ((tile_x + 1) * self.tile_size) as f32 / self.atlas_width as f32;
        // No V-flip: tile_y=0 maps to v≈0 (bottom of GL texture = top of image)
        let v_min = (tile_y * self.tile_size) as f32 / self.atlas_height as f32;
        let v_max = ((tile_y + 1) * self.tile_size) as f32 / self.atlas_height as f32;

        (u_min, v_min, u_max, v_max)
    }

    /// Get UV coordinates for a block face
    /// face: 0=north, 1=south, 2=east, 3=west, 4=up, 5=down
    pub fn get_block_uvs(&self, block: BlockType, face: usize) -> (f32, f32, f32, f32) {
        let (top, bottom, side) = block.texture_indices();

        let tile_index = match face {
            4 => top,    // Up face
            5 => bottom, // Down face
            _ => side,   // North, South, East, West faces
        };

        self.get_uv_coords(tile_index)
    }
}

/// Calculate UV coordinates for a texture atlas tile
/// Returns (u, v) for a given tile index and offset within the tile
/// offset_x and offset_y are in pixels (0.0 to 16.0 for a full tile)
/// Note: glTexImage2D uploads image data with row 0 at v=0 (bottom of GL texture).
/// Since our atlas has textures starting at row 0 (top of image file),
/// tile_y=0 should map to v≈0 (bottom of GL texture).
/// No V-flip needed - the image loader preserves row order.
pub fn calculate_uv(tile_index: usize, offset_x: f32, offset_y: f32) -> [f32; 2] {
    const TILE_SIZE: u32 = 16;
    const ATLAS_WIDTH: u32 = 256;  // Assuming 16x16 grid = 256px
    const ATLAS_HEIGHT: u32 = 256; // Atlas height
    const TILES_PER_ROW: u32 = ATLAS_WIDTH / TILE_SIZE;

    // Small padding in pixels to prevent texture bleeding at tile edges
    // This prevents sampling from adjacent tiles due to filtering/mipmapping
    const PADDING_PX: f32 = 0.5;

    let tile_x = (tile_index as u32) % TILES_PER_ROW;
    let tile_y = (tile_index as u32) / TILES_PER_ROW;

    // Clamp offset to tile bounds with padding
    let clamped_x = offset_x.clamp(PADDING_PX, TILE_SIZE as f32 - PADDING_PX);
    let clamped_y = offset_y.clamp(PADDING_PX, TILE_SIZE as f32 - PADDING_PX);

    let u = ((tile_x * TILE_SIZE) as f32 + clamped_x) / ATLAS_WIDTH as f32;
    // No V-flip: tile_y=0 maps to v≈0 (bottom of GL texture = top of image = where textures are)
    let v = ((tile_y * TILE_SIZE) as f32 + clamped_y) / ATLAS_HEIGHT as f32;

    [u, v]
}

/// Generate UV coordinates for a quad face
/// Returns array of 4 UV coordinates (bottom-left, bottom-right, top-right, top-left)
pub fn quad_uvs(uv_min: (f32, f32), uv_max: (f32, f32)) -> [[f32; 2]; 4] {
    [
        [uv_min.0, uv_max.1], // Bottom-left
        [uv_max.0, uv_max.1], // Bottom-right
        [uv_max.0, uv_min.1], // Top-right
        [uv_min.0, uv_min.1], // Top-left
    ]
}
