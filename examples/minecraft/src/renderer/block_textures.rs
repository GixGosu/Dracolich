/// Block texture information and UV coordinate mapping
/// Maps each BlockType to texture atlas coordinates, handling blocks with
/// different textures on different faces (grass, wood logs, crafting table, etc.)

use crate::types::{BlockType, Direction};

/// Information about a block's texture mapping
#[derive(Debug, Clone, Copy)]
pub struct BlockTextureInfo {
    /// Texture index for top face (+Y)
    pub top: usize,
    /// Texture index for bottom face (-Y)
    pub bottom: usize,
    /// Texture index for north face (+Z)
    pub north: usize,
    /// Texture index for south face (-Z)
    pub south: usize,
    /// Texture index for east face (+X)
    pub east: usize,
    /// Texture index for west face (-X)
    pub west: usize,
}

impl BlockTextureInfo {
    /// Create texture info where all faces use the same texture
    pub const fn uniform(index: usize) -> Self {
        Self {
            top: index,
            bottom: index,
            north: index,
            south: index,
            east: index,
            west: index,
        }
    }

    /// Create texture info with different top/bottom and uniform sides
    pub const fn top_bottom_sides(top: usize, bottom: usize, sides: usize) -> Self {
        Self {
            top,
            bottom,
            north: sides,
            south: sides,
            east: sides,
            west: sides,
        }
    }

    /// Create texture info with custom textures for all 6 faces
    pub const fn all_faces(
        top: usize,
        bottom: usize,
        north: usize,
        south: usize,
        east: usize,
        west: usize,
    ) -> Self {
        Self {
            top,
            bottom,
            north,
            south,
            east,
            west,
        }
    }

    /// Get texture index for a specific face direction
    pub fn get_face_texture(&self, direction: Direction) -> usize {
        match direction {
            Direction::Up => self.top,
            Direction::Down => self.bottom,
            Direction::North => self.north,
            Direction::South => self.south,
            Direction::East => self.east,
            Direction::West => self.west,
        }
    }

    /// Get texture index for a face by index (0-5)
    /// 0=north, 1=south, 2=east, 3=west, 4=up, 5=down
    pub fn get_face_texture_by_index(&self, face_index: usize) -> usize {
        match face_index {
            0 => self.north,
            1 => self.south,
            2 => self.east,
            3 => self.west,
            4 => self.top,
            5 => self.bottom,
            _ => 0, // Fallback
        }
    }
}

impl BlockType {
    /// Get complete texture information for this block type
    pub fn texture_info(&self) -> BlockTextureInfo {
        match self {
            // Air - no texture
            BlockType::Air => BlockTextureInfo::uniform(0),

            // Grass - green top, brown sides, dirt bottom
            BlockType::Grass => BlockTextureInfo::top_bottom_sides(1, 2, 3),

            // Dirt - uniform brown
            BlockType::Dirt => BlockTextureInfo::uniform(2),

            // Stone - uniform gray speckled
            BlockType::Stone => BlockTextureInfo::uniform(4),

            // Cobblestone - uniform gray irregular
            BlockType::Cobblestone => BlockTextureInfo::uniform(5),

            // Sand - uniform tan/beige
            BlockType::Sand => BlockTextureInfo::uniform(6),

            // Gravel - uniform gray pebbles
            BlockType::Gravel => BlockTextureInfo::uniform(7),

            // Bedrock - uniform dark mottled
            BlockType::Bedrock => BlockTextureInfo::uniform(8),

            // Oak Wood - rings on top/bottom, bark on sides
            BlockType::WoodOak => BlockTextureInfo::top_bottom_sides(9, 9, 10),

            // Birch Wood - rings on top/bottom, white bark on sides
            BlockType::WoodBirch => BlockTextureInfo::top_bottom_sides(11, 11, 12),

            // Oak Leaves - uniform green scattered
            BlockType::LeavesOak => BlockTextureInfo::uniform(13),

            // Birch Leaves - uniform light green scattered
            BlockType::LeavesBirch => BlockTextureInfo::uniform(14),

            // Water - uniform blue transparent
            BlockType::Water => BlockTextureInfo::uniform(15),

            // Glass - uniform light cyan transparent
            BlockType::Glass => BlockTextureInfo::uniform(16),

            // Coal Ore - gray stone with black specks
            BlockType::OreCoal => BlockTextureInfo::uniform(17),

            // Iron Ore - gray stone with tan specks
            BlockType::OreIron => BlockTextureInfo::uniform(18),

            // Gold Ore - gray stone with yellow specks
            BlockType::OreGold => BlockTextureInfo::uniform(19),

            // Diamond Ore - gray stone with cyan specks
            BlockType::OreDiamond => BlockTextureInfo::uniform(20),

            // Planks - uniform wood planks pattern
            BlockType::Planks => BlockTextureInfo::uniform(21),

            // Crafting Table - crafting grid on top, planks on bottom, tools on sides
            BlockType::CraftingTable => BlockTextureInfo::top_bottom_sides(22, 21, 24),

            // Furnace - furnace front with opening on one side
            BlockType::Furnace => BlockTextureInfo::all_faces(
                26, // top (stone)
                26, // bottom (stone)
                25, // north (furnace opening)
                27, // south (plain stone)
                27, // east (plain stone)
                27, // west (plain stone)
            ),
        }
    }
}

/// Texture atlas layout
/// 256x256 atlas with 16x16 tiles = 16x16 grid = 256 tiles
/// Current usage: 28 tiles (indices 0-27)
pub const ATLAS_SIZE: u32 = 256;
pub const TILE_SIZE: u32 = 16;
pub const TILES_PER_ROW: u32 = ATLAS_SIZE / TILE_SIZE; // 16

/// Texture atlas tile assignments
pub mod tiles {
    /// Air / Empty (black)
    pub const AIR: usize = 0;

    /// Grass top (green)
    pub const GRASS_TOP: usize = 1;

    /// Dirt (brown)
    pub const DIRT: usize = 2;

    /// Grass side (green top, brown bottom)
    pub const GRASS_SIDE: usize = 3;

    /// Stone (gray speckled)
    pub const STONE: usize = 4;

    /// Cobblestone (gray irregular)
    pub const COBBLESTONE: usize = 5;

    /// Sand (tan/beige)
    pub const SAND: usize = 6;

    /// Gravel (gray pebbles)
    pub const GRAVEL: usize = 7;

    /// Bedrock (dark mottled)
    pub const BEDROCK: usize = 8;

    /// Oak log top (tree rings)
    pub const OAK_LOG_TOP: usize = 9;

    /// Oak log side (brown bark)
    pub const OAK_LOG_SIDE: usize = 10;

    /// Birch log top (tree rings)
    pub const BIRCH_LOG_TOP: usize = 11;

    /// Birch log side (white bark with dark marks)
    pub const BIRCH_LOG_SIDE: usize = 12;

    /// Oak leaves (green scattered)
    pub const LEAVES_OAK: usize = 13;

    /// Birch leaves (light green scattered)
    pub const LEAVES_BIRCH: usize = 14;

    /// Water (blue transparent)
    pub const WATER: usize = 15;

    /// Glass (light cyan transparent)
    pub const GLASS: usize = 16;

    /// Coal ore (stone with black specks)
    pub const ORE_COAL: usize = 17;

    /// Iron ore (stone with tan specks)
    pub const ORE_IRON: usize = 18;

    /// Gold ore (stone with yellow specks)
    pub const ORE_GOLD: usize = 19;

    /// Diamond ore (stone with cyan specks)
    pub const ORE_DIAMOND: usize = 20;

    /// Wood planks
    pub const PLANKS: usize = 21;

    /// Crafting table top (crafting grid)
    pub const CRAFTING_TABLE_TOP: usize = 22;

    /// Crafting table bottom (planks - same as PLANKS)
    pub const CRAFTING_TABLE_BOTTOM: usize = 21;

    /// Crafting table side (tools on planks)
    pub const CRAFTING_TABLE_SIDE: usize = 24;

    /// Furnace front (opening)
    pub const FURNACE_FRONT: usize = 25;

    /// Furnace top/bottom (stone)
    pub const FURNACE_TOP: usize = 26;

    /// Furnace side (plain stone)
    pub const FURNACE_SIDE: usize = 27;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grass_has_different_faces() {
        let info = BlockType::Grass.texture_info();
        assert_eq!(info.top, 1); // Grass top
        assert_eq!(info.bottom, 2); // Dirt
        assert_eq!(info.north, 3); // Grass side
        assert_eq!(info.south, 3);
        assert_eq!(info.east, 3);
        assert_eq!(info.west, 3);
    }

    #[test]
    fn test_stone_is_uniform() {
        let info = BlockType::Stone.texture_info();
        assert_eq!(info.top, 4);
        assert_eq!(info.bottom, 4);
        assert_eq!(info.north, 4);
        assert_eq!(info.south, 4);
    }

    #[test]
    fn test_wood_has_bark_and_rings() {
        let oak = BlockType::WoodOak.texture_info();
        assert_eq!(oak.top, 9); // Rings
        assert_eq!(oak.bottom, 9); // Rings
        assert_eq!(oak.north, 10); // Bark
    }

    #[test]
    fn test_furnace_has_unique_front() {
        let furnace = BlockType::Furnace.texture_info();
        assert_eq!(furnace.north, 25); // Front with opening
        assert_eq!(furnace.south, 27); // Plain side
        assert_eq!(furnace.top, 26); // Stone top
    }
}
