/// Procedural texture atlas generator
/// Creates a 256x256 PNG texture atlas with 16x16 textures for all block types
/// Run this program to generate assets/atlas.png

use image::{ImageBuffer, Rgba, RgbaImage};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

const ATLAS_SIZE: u32 = 256;
const TILE_SIZE: u32 = 16;
const TILES_PER_ROW: u32 = ATLAS_SIZE / TILE_SIZE; // 16

type Color = Rgba<u8>;

// Color constants
const BLACK: Color = Rgba([0, 0, 0, 255]);
const GRASS_GREEN: Color = Rgba([92, 160, 55, 255]);
const DIRT_BROWN: Color = Rgba([134, 96, 67, 255]);
const STONE_GRAY: Color = Rgba([128, 128, 128, 255]);
const COBBLE_GRAY: Color = Rgba([110, 110, 110, 255]);
const SAND_TAN: Color = Rgba([218, 210, 158, 255]);
const GRAVEL_GRAY: Color = Rgba([136, 126, 126, 255]);
const BEDROCK_DARK: Color = Rgba([64, 64, 64, 255]);
const BARK_BROWN: Color = Rgba([102, 76, 51, 255]);
const WOOD_LIGHT: Color = Rgba([157, 128, 79, 255]);
const BIRCH_WHITE: Color = Rgba([216, 215, 201, 255]);
const BIRCH_MARK: Color = Rgba([60, 60, 60, 255]);
const LEAF_GREEN: Color = Rgba([76, 150, 24, 255]);
const LEAF_BIRCH: Color = Rgba([128, 167, 85, 255]);
const WATER_BLUE: Color = Rgba([32, 64, 255, 180]);
const GLASS_CYAN: Color = Rgba([200, 230, 255, 120]);
const COAL_BLACK: Color = Rgba([45, 45, 45, 255]);
const IRON_TAN: Color = Rgba([196, 181, 171, 255]);
const GOLD_YELLOW: Color = Rgba([255, 215, 0, 255]);
const DIAMOND_CYAN: Color = Rgba([93, 219, 213, 255]);
const PLANK_BROWN: Color = Rgba([162, 130, 78, 255]);
const FURNACE_GRAY: Color = Rgba([96, 96, 96, 255]);
const FURNACE_DARK: Color = Rgba([32, 32, 32, 255]);

fn main() {
    println!("Generating texture atlas...");

    let mut atlas = RgbaImage::new(ATLAS_SIZE, ATLAS_SIZE);
    let mut rng = StdRng::seed_from_u64(42); // Deterministic generation

    // Generate all tiles
    generate_tile(&mut atlas, 0, tile_air());
    generate_tile(&mut atlas, 1, tile_grass_top(&mut rng));
    generate_tile(&mut atlas, 2, tile_dirt(&mut rng));
    generate_tile(&mut atlas, 3, tile_grass_side(&mut rng));
    generate_tile(&mut atlas, 4, tile_stone(&mut rng));
    generate_tile(&mut atlas, 5, tile_cobblestone(&mut rng));
    generate_tile(&mut atlas, 6, tile_sand(&mut rng));
    generate_tile(&mut atlas, 7, tile_gravel(&mut rng));
    generate_tile(&mut atlas, 8, tile_bedrock(&mut rng));
    generate_tile(&mut atlas, 9, tile_oak_log_top(&mut rng));
    generate_tile(&mut atlas, 10, tile_oak_log_side(&mut rng));
    generate_tile(&mut atlas, 11, tile_birch_log_top(&mut rng));
    generate_tile(&mut atlas, 12, tile_birch_log_side(&mut rng));
    generate_tile(&mut atlas, 13, tile_leaves_oak(&mut rng));
    generate_tile(&mut atlas, 14, tile_leaves_birch(&mut rng));
    generate_tile(&mut atlas, 15, tile_water(&mut rng));
    generate_tile(&mut atlas, 16, tile_glass(&mut rng));
    generate_tile(&mut atlas, 17, tile_ore_coal(&mut rng));
    generate_tile(&mut atlas, 18, tile_ore_iron(&mut rng));
    generate_tile(&mut atlas, 19, tile_ore_gold(&mut rng));
    generate_tile(&mut atlas, 20, tile_ore_diamond(&mut rng));
    generate_tile(&mut atlas, 21, tile_planks(&mut rng));
    generate_tile(&mut atlas, 22, tile_crafting_table_top(&mut rng));
    generate_tile(&mut atlas, 24, tile_crafting_table_side(&mut rng));
    generate_tile(&mut atlas, 25, tile_furnace_front(&mut rng));
    generate_tile(&mut atlas, 26, tile_furnace_top(&mut rng));
    generate_tile(&mut atlas, 27, tile_furnace_side(&mut rng));

    // Save to file
    let output_path = "assets/atlas.png";
    std::fs::create_dir_all("assets").expect("Failed to create assets directory");
    atlas.save(output_path).expect("Failed to save atlas");
    println!("Atlas saved to {}", output_path);
}

/// Place a 16x16 tile at the given index in the atlas
fn generate_tile(atlas: &mut RgbaImage, index: usize, tile: RgbaImage) {
    let tile_x = (index as u32 % TILES_PER_ROW) * TILE_SIZE;
    let tile_y = (index as u32 / TILES_PER_ROW) * TILE_SIZE;

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            atlas.put_pixel(tile_x + x, tile_y + y, *tile.get_pixel(x, y));
        }
    }
}

/// Create a blank 16x16 tile
fn blank_tile() -> RgbaImage {
    RgbaImage::new(TILE_SIZE, TILE_SIZE)
}

/// Fill tile with color
fn fill_tile(color: Color) -> RgbaImage {
    ImageBuffer::from_fn(TILE_SIZE, TILE_SIZE, |_, _| color)
}

/// Add noise to a color
fn noise_color(base: Color, rng: &mut StdRng, variance: i16) -> Color {
    let r = (base[0] as i16 + rng.gen_range(-variance..=variance)).clamp(0, 255) as u8;
    let g = (base[1] as i16 + rng.gen_range(-variance..=variance)).clamp(0, 255) as u8;
    let b = (base[2] as i16 + rng.gen_range(-variance..=variance)).clamp(0, 255) as u8;
    Rgba([r, g, b, base[3]])
}

// === TILE GENERATORS ===

fn tile_air() -> RgbaImage {
    fill_tile(BLACK)
}

fn tile_grass_top(rng: &mut StdRng) -> RgbaImage {
    ImageBuffer::from_fn(TILE_SIZE, TILE_SIZE, |x, y| {
        if (x + y) % 3 == 0 {
            noise_color(GRASS_GREEN, rng, 15)
        } else {
            noise_color(GRASS_GREEN, rng, 8)
        }
    })
}

fn tile_dirt(rng: &mut StdRng) -> RgbaImage {
    ImageBuffer::from_fn(TILE_SIZE, TILE_SIZE, |_, _| {
        noise_color(DIRT_BROWN, rng, 12)
    })
}

fn tile_grass_side(rng: &mut StdRng) -> RgbaImage {
    ImageBuffer::from_fn(TILE_SIZE, TILE_SIZE, |_, y| {
        if y < 2 {
            noise_color(GRASS_GREEN, rng, 10)
        } else {
            noise_color(DIRT_BROWN, rng, 10)
        }
    })
}

fn tile_stone(rng: &mut StdRng) -> RgbaImage {
    ImageBuffer::from_fn(TILE_SIZE, TILE_SIZE, |x, y| {
        let base = noise_color(STONE_GRAY, rng, 15);
        // Add darker speckles
        if rng.gen_ratio(1, 6) {
            Rgba([base[0] - 30, base[1] - 30, base[2] - 30, base[3]])
        } else {
            base
        }
    })
}

fn tile_cobblestone(rng: &mut StdRng) -> RgbaImage {
    let mut tile = blank_tile();
    // Create irregular stone pattern
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let edge = (x % 4 == 0 || y % 4 == 0) && rng.gen_ratio(2, 3);
            let color = if edge {
                Rgba([60, 60, 60, 255])
            } else {
                noise_color(COBBLE_GRAY, rng, 20)
            };
            tile.put_pixel(x, y, color);
        }
    }
    tile
}

fn tile_sand(rng: &mut StdRng) -> RgbaImage {
    ImageBuffer::from_fn(TILE_SIZE, TILE_SIZE, |_, _| {
        noise_color(SAND_TAN, rng, 10)
    })
}

fn tile_gravel(rng: &mut StdRng) -> RgbaImage {
    ImageBuffer::from_fn(TILE_SIZE, TILE_SIZE, |x, y| {
        if (x + y) % 2 == 0 {
            noise_color(GRAVEL_GRAY, rng, 20)
        } else {
            noise_color(GRAVEL_GRAY, rng, 15)
        }
    })
}

fn tile_bedrock(rng: &mut StdRng) -> RgbaImage {
    ImageBuffer::from_fn(TILE_SIZE, TILE_SIZE, |x, y| {
        let pattern = ((x / 2 + y / 2) % 3) as i16;
        let base = BEDROCK_DARK;
        let offset = (pattern - 1) * 15;
        Rgba([
            (base[0] as i16 + offset).clamp(0, 255) as u8,
            (base[1] as i16 + offset).clamp(0, 255) as u8,
            (base[2] as i16 + offset).clamp(0, 255) as u8,
            base[3],
        ])
    })
}

fn tile_oak_log_top(rng: &mut StdRng) -> RgbaImage {
    let mut tile = fill_tile(WOOD_LIGHT);
    // Draw concentric rings
    for r in 1..6 {
        let angle_step = std::f32::consts::PI * 2.0 / 32.0;
        for i in 0..32 {
            let angle = i as f32 * angle_step;
            let x = (8.0 + r as f32 * angle.cos()) as u32;
            let y = (8.0 + r as f32 * angle.sin()) as u32;
            if x < TILE_SIZE && y < TILE_SIZE {
                tile.put_pixel(x, y, noise_color(BARK_BROWN, rng, 10));
            }
        }
    }
    tile
}

fn tile_oak_log_side(rng: &mut StdRng) -> RgbaImage {
    ImageBuffer::from_fn(TILE_SIZE, TILE_SIZE, |_, y| {
        if y % 3 == 0 {
            noise_color(BARK_BROWN, rng, 15)
        } else {
            noise_color(BARK_BROWN, rng, 20)
        }
    })
}

fn tile_birch_log_top(rng: &mut StdRng) -> RgbaImage {
    let mut tile = fill_tile(WOOD_LIGHT);
    // Similar to oak but lighter
    for r in 1..6 {
        let angle_step = std::f32::consts::PI * 2.0 / 32.0;
        for i in 0..32 {
            let angle = i as f32 * angle_step;
            let x = (8.0 + r as f32 * angle.cos()) as u32;
            let y = (8.0 + r as f32 * angle.sin()) as u32;
            if x < TILE_SIZE && y < TILE_SIZE {
                tile.put_pixel(x, y, noise_color(BIRCH_WHITE, rng, 8));
            }
        }
    }
    tile
}

fn tile_birch_log_side(rng: &mut StdRng) -> RgbaImage {
    let mut tile = fill_tile(BIRCH_WHITE);
    // Add dark marks
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            if rng.gen_ratio(1, 8) {
                tile.put_pixel(x, y, BIRCH_MARK);
            }
        }
    }
    tile
}

fn tile_leaves_oak(rng: &mut StdRng) -> RgbaImage {
    ImageBuffer::from_fn(TILE_SIZE, TILE_SIZE, |x, y| {
        if (x + y) % 2 == 0 && rng.gen_ratio(1, 3) {
            Rgba([0, 0, 0, 0]) // Transparent for scattered look
        } else {
            noise_color(LEAF_GREEN, rng, 20)
        }
    })
}

fn tile_leaves_birch(rng: &mut StdRng) -> RgbaImage {
    ImageBuffer::from_fn(TILE_SIZE, TILE_SIZE, |x, y| {
        if (x + y) % 2 == 0 && rng.gen_ratio(1, 3) {
            Rgba([0, 0, 0, 0])
        } else {
            noise_color(LEAF_BIRCH, rng, 20)
        }
    })
}

fn tile_water(rng: &mut StdRng) -> RgbaImage {
    ImageBuffer::from_fn(TILE_SIZE, TILE_SIZE, |x, y| {
        if (x + y) % 4 == 0 {
            Rgba([40, 80, 255, 180])
        } else {
            noise_color(WATER_BLUE, rng, 10)
        }
    })
}

fn tile_glass(_rng: &mut StdRng) -> RgbaImage {
    fill_tile(GLASS_CYAN)
}

fn tile_ore(base_stone: Color, ore_color: Color, rng: &mut StdRng) -> RgbaImage {
    ImageBuffer::from_fn(TILE_SIZE, TILE_SIZE, |x, y| {
        let is_ore = ((x / 2 + y / 3) % 5 == 0) || ((x / 3 + y / 2) % 7 == 1);
        if is_ore {
            noise_color(ore_color, rng, 15)
        } else {
            noise_color(base_stone, rng, 12)
        }
    })
}

fn tile_ore_coal(rng: &mut StdRng) -> RgbaImage {
    tile_ore(STONE_GRAY, COAL_BLACK, rng)
}

fn tile_ore_iron(rng: &mut StdRng) -> RgbaImage {
    tile_ore(STONE_GRAY, IRON_TAN, rng)
}

fn tile_ore_gold(rng: &mut StdRng) -> RgbaImage {
    tile_ore(STONE_GRAY, GOLD_YELLOW, rng)
}

fn tile_ore_diamond(rng: &mut StdRng) -> RgbaImage {
    tile_ore(STONE_GRAY, DIAMOND_CYAN, rng)
}

fn tile_planks(rng: &mut StdRng) -> RgbaImage {
    ImageBuffer::from_fn(TILE_SIZE, TILE_SIZE, |_, y| {
        if y % 4 == 0 {
            Rgba([PLANK_BROWN[0] - 20, PLANK_BROWN[1] - 20, PLANK_BROWN[2] - 20, 255])
        } else {
            noise_color(PLANK_BROWN, rng, 10)
        }
    })
}

fn tile_crafting_table_top(rng: &mut StdRng) -> RgbaImage {
    let mut tile = tile_planks(rng);
    // Draw 2x2 grid
    for i in 0..TILE_SIZE {
        tile.put_pixel(7, i, Rgba([80, 60, 40, 255]));
        tile.put_pixel(8, i, Rgba([80, 60, 40, 255]));
        tile.put_pixel(i, 7, Rgba([80, 60, 40, 255]));
        tile.put_pixel(i, 8, Rgba([80, 60, 40, 255]));
    }
    tile
}

fn tile_crafting_table_side(rng: &mut StdRng) -> RgbaImage {
    let mut tile = tile_planks(rng);
    // Draw simple tools (pickaxe, axe shapes)
    for y in 4..7 {
        for x in 2..6 {
            tile.put_pixel(x, y, Rgba([160, 160, 160, 255])); // Tool head
        }
    }
    for y in 7..12 {
        tile.put_pixel(4, y, Rgba([120, 90, 60, 255])); // Handle
    }
    tile
}

fn tile_furnace_front(_rng: &mut StdRng) -> RgbaImage {
    let mut tile = fill_tile(FURNACE_GRAY);
    // Draw opening
    for y in 5..11 {
        for x in 5..11 {
            tile.put_pixel(x, y, FURNACE_DARK);
        }
    }
    tile
}

fn tile_furnace_top(rng: &mut StdRng) -> RgbaImage {
    ImageBuffer::from_fn(TILE_SIZE, TILE_SIZE, |_, _| {
        noise_color(FURNACE_GRAY, rng, 10)
    })
}

fn tile_furnace_side(rng: &mut StdRng) -> RgbaImage {
    ImageBuffer::from_fn(TILE_SIZE, TILE_SIZE, |x, y| {
        if (x + y) % 4 == 0 {
            noise_color(FURNACE_GRAY, rng, 15)
        } else {
            noise_color(FURNACE_GRAY, rng, 8)
        }
    })
}
