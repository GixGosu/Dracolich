// Biome system for terrain generation
// Defines different biome types and their generation parameters

use crate::types::BlockType;

/// Different biome types with unique generation rules
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    Plains,
    Hills,
    Mountains,
    Desert,
    Forest,
}

impl Biome {
    /// Select biome based on noise values
    /// temperature: -1.0 to 1.0 (cold to hot)
    /// moisture: -1.0 to 1.0 (dry to wet)
    pub fn from_noise(temperature: f64, moisture: f64) -> Self {
        // Hot and dry = desert
        if temperature > 0.3 && moisture < -0.2 {
            return Biome::Desert;
        }

        // Cold = mountains
        if temperature < -0.5 {
            return Biome::Mountains;
        }

        // Wet = forest
        if moisture > 0.4 {
            return Biome::Forest;
        }

        // Mid temperature = hills
        if temperature > -0.2 && temperature < 0.2 {
            return Biome::Hills;
        }

        // Default = plains
        Biome::Plains
    }

    /// Get base height range for this biome
    /// Returns (min_height, max_height) in blocks above bedrock
    pub fn height_range(&self) -> (i32, i32) {
        match self {
            Biome::Plains => (60, 68),
            Biome::Hills => (60, 85),
            Biome::Mountains => (70, 128),
            Biome::Desert => (62, 70),
            Biome::Forest => (60, 75),
        }
    }

    /// Get height variation multiplier for noise
    pub fn height_multiplier(&self) -> f64 {
        match self {
            Biome::Plains => 0.6,
            Biome::Hills => 1.2,
            Biome::Mountains => 2.5,
            Biome::Desert => 0.4,
            Biome::Forest => 0.8,
        }
    }

    /// Get surface block type
    pub fn surface_block(&self) -> BlockType {
        match self {
            Biome::Plains | Biome::Hills | Biome::Forest => BlockType::Grass,
            Biome::Mountains => BlockType::Stone,
            Biome::Desert => BlockType::Sand,
        }
    }

    /// Get subsurface block type (layer below surface)
    pub fn subsurface_block(&self) -> BlockType {
        match self {
            Biome::Plains | Biome::Hills | Biome::Forest => BlockType::Dirt,
            Biome::Mountains => BlockType::Stone,
            Biome::Desert => BlockType::Sand,
        }
    }

    /// Get tree spawn chance (0.0 - 1.0)
    pub fn tree_density(&self) -> f64 {
        match self {
            Biome::Plains => 0.002,
            Biome::Hills => 0.005,
            Biome::Mountains => 0.001,
            Biome::Desert => 0.0,
            Biome::Forest => 0.02,
        }
    }

    /// Get tree type to spawn
    pub fn tree_type(&self) -> TreeType {
        match self {
            Biome::Forest => {
                // Mix of oak and birch in forests
                if rand::random::<f32>() > 0.5 {
                    TreeType::Oak
                } else {
                    TreeType::Birch
                }
            }
            _ => TreeType::Oak,
        }
    }
}

/// Tree types for structure generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeType {
    Oak,
    Birch,
}

impl TreeType {
    /// Get the wood block type for this tree
    pub fn wood_block(&self) -> BlockType {
        match self {
            TreeType::Oak => BlockType::WoodOak,
            TreeType::Birch => BlockType::WoodBirch,
        }
    }

    /// Get the leaves block type for this tree
    pub fn leaves_block(&self) -> BlockType {
        match self {
            TreeType::Oak => BlockType::LeavesOak,
            TreeType::Birch => BlockType::LeavesBirch,
        }
    }

    /// Get tree height range (min, max)
    pub fn height_range(&self) -> (i32, i32) {
        match self {
            TreeType::Oak => (4, 6),
            TreeType::Birch => (5, 7),
        }
    }
}
