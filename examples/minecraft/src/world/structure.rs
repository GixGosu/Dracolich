// Structure generation system
// Handles tree generation and other world structures

use crate::types::{BlockType, WorldPos};
use super::biome::TreeType;
use rand::Rng;

/// Represents a block to be placed as part of a structure
#[derive(Debug, Clone)]
pub struct StructureBlock {
    pub offset: (i32, i32, i32),
    pub block_type: BlockType,
}

/// Tree structure generator
pub struct TreeGenerator;

impl TreeGenerator {
    /// Generate a tree at the given position
    /// Returns list of blocks to place (position + block type)
    pub fn generate_tree(base_pos: WorldPos, tree_type: TreeType, rng: &mut impl Rng) -> Vec<(WorldPos, BlockType)> {
        let mut blocks = Vec::new();

        let (min_height, max_height) = tree_type.height_range();
        let trunk_height = rng.gen_range(min_height..=max_height);

        let wood = tree_type.wood_block();
        let leaves = tree_type.leaves_block();

        // Generate trunk
        for y in 0..trunk_height {
            blocks.push((
                WorldPos::new(base_pos.x, base_pos.y + y, base_pos.z),
                wood,
            ));
        }

        // Generate leaves crown
        let crown_base = base_pos.y + trunk_height - 2;
        let crown_top = base_pos.y + trunk_height + 1;

        for y in crown_base..=crown_top {
            let radius = if y == crown_top {
                1 // Top layer is smaller
            } else if y == crown_base {
                2 // Bottom layer
            } else {
                2 // Middle layers
            };

            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    let dx: i32 = dx;
                    let dz: i32 = dz;
                    // Skip corners for more natural shape
                    if dx.abs() == radius && dz.abs() == radius {
                        continue;
                    }

                    // Don't replace trunk
                    if dx == 0 && dz == 0 && y < base_pos.y + trunk_height {
                        continue;
                    }

                    blocks.push((
                        WorldPos::new(base_pos.x + dx, y, base_pos.z + dz),
                        leaves,
                    ));
                }
            }
        }

        blocks
    }

    /// Check if a tree can be placed at the given position
    /// Requires solid ground and enough space
    pub fn can_place_tree(base_pos: &WorldPos, check_block: impl Fn(&WorldPos) -> BlockType) -> bool {
        // Check block below is solid
        let ground = WorldPos::new(base_pos.x, base_pos.y - 1, base_pos.z);
        let ground_block = check_block(&ground);
        if !ground_block.is_solid() {
            return false;
        }

        // Check there's air above for trunk
        for y in 0..7 {
            let pos = WorldPos::new(base_pos.x, base_pos.y + y, base_pos.z);
            let block = check_block(&pos);
            if block != BlockType::Air {
                return false;
            }
        }

        true
    }
}

/// Boulder/rock structure for mountains
pub struct BoulderGenerator;

impl BoulderGenerator {
    /// Generate a small boulder (3-5 blocks)
    pub fn generate_boulder(base_pos: WorldPos, rng: &mut impl Rng) -> Vec<(WorldPos, BlockType)> {
        let mut blocks = Vec::new();
        let size = rng.gen_range(2..=3);

        // Center stone
        blocks.push((base_pos, BlockType::Cobblestone));

        // Add surrounding stones
        for _ in 0..size {
            let dx = rng.gen_range(-1..=1);
            let dy = rng.gen_range(0..=1);
            let dz = rng.gen_range(-1..=1);

            blocks.push((
                WorldPos::new(base_pos.x + dx, base_pos.y + dy, base_pos.z + dz),
                BlockType::Cobblestone,
            ));
        }

        blocks
    }
}
