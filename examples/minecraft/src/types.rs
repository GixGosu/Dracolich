use glam::{IVec3, Vec3};

/// All block types in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BlockType {
    Air = 0,
    Grass,
    Dirt,
    Stone,
    Cobblestone,
    Sand,
    Gravel,
    Bedrock,
    WoodOak,
    WoodBirch,
    LeavesOak,
    LeavesBirch,
    Water,
    Glass,
    OreCoal,
    OreIron,
    OreGold,
    OreDiamond,
    Planks,
    CraftingTable,
    Furnace,
}

impl BlockType {
    /// Returns true if this block is solid (has collision)
    pub fn is_solid(&self) -> bool {
        !matches!(self, BlockType::Air | BlockType::Water)
    }

    /// Returns true if this block is transparent (should render faces behind it)
    pub fn is_transparent(&self) -> bool {
        matches!(self, BlockType::Air | BlockType::Water | BlockType::Glass | BlockType::LeavesOak | BlockType::LeavesBirch)
    }

    /// Returns true if this block is opaque (blocks light completely)
    pub fn is_opaque(&self) -> bool {
        !self.is_transparent()
    }

    /// Returns the mining hardness of this block (time to break in seconds with bare hands)
    pub fn hardness(&self) -> f32 {
        match self {
            BlockType::Air => 0.0,
            BlockType::Grass | BlockType::Dirt => 0.5,
            BlockType::Sand | BlockType::Gravel => 0.4,
            BlockType::WoodOak | BlockType::WoodBirch => 2.0,
            BlockType::Planks => 1.5,
            BlockType::LeavesOak | BlockType::LeavesBirch => 0.2,
            BlockType::Stone | BlockType::Cobblestone => 4.0,
            BlockType::OreCoal => 3.0,
            BlockType::OreIron => 5.0,
            BlockType::OreGold => 6.0,
            BlockType::OreDiamond => 8.0,
            BlockType::Glass => 0.3,
            BlockType::Bedrock => f32::INFINITY,
            BlockType::Water => 0.0,
            BlockType::CraftingTable => 2.0,
            BlockType::Furnace => 4.0,
        }
    }

    /// Returns true if this block can be broken
    pub fn is_breakable(&self) -> bool {
        !matches!(self, BlockType::Bedrock)
    }

    /// Returns the display name for this block type
    pub fn name(&self) -> &'static str {
        match self {
            BlockType::Air => "Air",
            BlockType::Grass => "Grass Block",
            BlockType::Dirt => "Dirt",
            BlockType::Stone => "Stone",
            BlockType::Cobblestone => "Cobblestone",
            BlockType::Sand => "Sand",
            BlockType::Gravel => "Gravel",
            BlockType::Bedrock => "Bedrock",
            BlockType::WoodOak => "Oak Wood",
            BlockType::WoodBirch => "Birch Wood",
            BlockType::LeavesOak => "Oak Leaves",
            BlockType::LeavesBirch => "Birch Leaves",
            BlockType::Water => "Water",
            BlockType::Glass => "Glass",
            BlockType::OreCoal => "Coal Ore",
            BlockType::OreIron => "Iron Ore",
            BlockType::OreGold => "Gold Ore",
            BlockType::OreDiamond => "Diamond Ore",
            BlockType::Planks => "Planks",
            BlockType::CraftingTable => "Crafting Table",
            BlockType::Furnace => "Furnace",
        }
    }

    /// Get texture indices for this block (top, bottom, sides)
    /// Returns (top_index, bottom_index, side_index) into texture atlas
    pub fn texture_indices(&self) -> (usize, usize, usize) {
        match self {
            BlockType::Air => (0, 0, 0),
            BlockType::Grass => (1, 2, 3),
            BlockType::Dirt => (2, 2, 2),
            BlockType::Stone => (4, 4, 4),
            BlockType::Cobblestone => (5, 5, 5),
            BlockType::Sand => (6, 6, 6),
            BlockType::Gravel => (7, 7, 7),
            BlockType::Bedrock => (8, 8, 8),
            BlockType::WoodOak => (9, 9, 10),
            BlockType::WoodBirch => (11, 11, 12),
            BlockType::LeavesOak => (13, 13, 13),
            BlockType::LeavesBirch => (14, 14, 14),
            BlockType::Water => (15, 15, 15),
            BlockType::Glass => (16, 16, 16),
            BlockType::OreCoal => (17, 17, 17),
            BlockType::OreIron => (18, 18, 18),
            BlockType::OreGold => (19, 19, 19),
            BlockType::OreDiamond => (20, 20, 20),
            BlockType::Planks => (21, 21, 21),
            BlockType::CraftingTable => (22, 23, 24),
            BlockType::Furnace => (25, 26, 27),
        }
    }
}

/// Chunk coordinates (each chunk is 16x256x16 blocks)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Convert world position to chunk position (reference version)
    pub fn from_world_pos(pos: &WorldPos) -> Self {
        Self {
            x: pos.x.div_euclid(16),
            z: pos.z.div_euclid(16),
        }
    }

    /// Convert world position to chunk position (struct version - alias)
    pub fn from_world_pos_struct(pos: &WorldPos) -> Self {
        Self::from_world_pos(pos)
    }

    /// Convert world position (x, z integers) to chunk position
    pub fn from_world_coords(x: i32, z: i32) -> Self {
        Self {
            x: x.div_euclid(16),
            z: z.div_euclid(16),
        }
    }

    /// Get the world position of the chunk's origin (0,0,0) corner
    pub fn to_world_origin(&self) -> IVec3 {
        IVec3::new(self.x * 16, 0, self.z * 16)
    }

    /// Get neighboring chunk positions
    pub fn neighbors(&self) -> [ChunkPos; 8] {
        [
            ChunkPos::new(self.x - 1, self.z - 1),
            ChunkPos::new(self.x - 1, self.z),
            ChunkPos::new(self.x - 1, self.z + 1),
            ChunkPos::new(self.x, self.z - 1),
            ChunkPos::new(self.x, self.z + 1),
            ChunkPos::new(self.x + 1, self.z - 1),
            ChunkPos::new(self.x + 1, self.z),
            ChunkPos::new(self.x + 1, self.z + 1),
        ]
    }
}

/// World position (block coordinates)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl WorldPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn from_vec3(v: Vec3) -> Self {
        Self {
            x: v.x.floor() as i32,
            y: v.y.floor() as i32,
            z: v.z.floor() as i32,
        }
    }

    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }

    pub fn to_ivec3(&self) -> IVec3 {
        IVec3::new(self.x, self.y, self.z)
    }

    /// Get position within chunk (0-15 for x and z, 0-255 for y)
    pub fn chunk_local(&self) -> (usize, usize, usize) {
        let x = self.x.rem_euclid(16) as usize;
        let y = self.y.clamp(0, 255) as usize;
        let z = self.z.rem_euclid(16) as usize;
        (x, y, z)
    }
}

/// Cardinal and ordinal directions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,  // -Z
    South,  // +Z
    East,   // +X
    West,   // -X
    Up,     // +Y
    Down,   // -Y
}

impl Direction {
    /// Get all six directions
    pub fn all() -> [Direction; 6] {
        [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
            Direction::Up,
            Direction::Down,
        ]
    }

    /// Get the offset vector for this direction
    pub fn offset(&self) -> IVec3 {
        match self {
            Direction::North => IVec3::new(0, 0, -1),
            Direction::South => IVec3::new(0, 0, 1),
            Direction::East => IVec3::new(1, 0, 0),
            Direction::West => IVec3::new(-1, 0, 0),
            Direction::Up => IVec3::new(0, 1, 0),
            Direction::Down => IVec3::new(0, -1, 0),
        }
    }

    /// Get the opposite direction
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }

    /// Get normal vector for rendering
    pub fn normal(&self) -> Vec3 {
        match self {
            Direction::North => Vec3::new(0.0, 0.0, -1.0),
            Direction::South => Vec3::new(0.0, 0.0, 1.0),
            Direction::East => Vec3::new(1.0, 0.0, 0.0),
            Direction::West => Vec3::new(-1.0, 0.0, 0.0),
            Direction::Up => Vec3::new(0.0, 1.0, 0.0),
            Direction::Down => Vec3::new(0.0, -1.0, 0.0),
        }
    }
}

/// Axis-Aligned Bounding Box for collision detection
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create AABB from center point and size
    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        let half_size = size * 0.5;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    /// Create AABB for a block at the given position
    pub fn from_block(pos: &WorldPos) -> Self {
        let min = Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32);
        let max = min + Vec3::ONE;
        Self { min, max }
    }

    /// Check if this AABB intersects with another
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Get the center point of the AABB
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get the size of the AABB
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Expand the AABB by a certain amount in all directions
    pub fn expand(&self, amount: f32) -> AABB {
        AABB {
            min: self.min - Vec3::splat(amount),
            max: self.max + Vec3::splat(amount),
        }
    }

    /// Move the AABB by an offset
    pub fn translate(&self, offset: Vec3) -> AABB {
        AABB {
            min: self.min + offset,
            max: self.max + offset,
        }
    }
}

/// Chunk dimensions
pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;
pub const CHUNK_DEPTH: usize = 16;

/// World constants
pub const SEA_LEVEL: i32 = 64;
pub const BEDROCK_LEVEL: i32 = 0;

/// Render distance in chunks
pub const RENDER_DISTANCE: i32 = 8;
