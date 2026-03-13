// Terrain generation system using multi-octave noise
// Generates infinite procedural terrain with biomes, caves, ores, and structures

use noise::{NoiseFn, Perlin, Seedable};
use crate::types::{BlockType, ChunkPos, WorldPos, CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH, SEA_LEVEL};
use super::biome::Biome;
use super::structure::TreeGenerator;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Terrain generator using multiple noise layers
pub struct TerrainGenerator {
    seed: u32,

    // Height map noise (multi-octave)
    height_noise: Perlin,

    // Biome selection noise
    temperature_noise: Perlin,
    moisture_noise: Perlin,

    // Cave generation noise (3D)
    cave_noise: Perlin,

    // Ore placement noise
    coal_noise: Perlin,
    iron_noise: Perlin,
    gold_noise: Perlin,
    diamond_noise: Perlin,
}

impl TerrainGenerator {
    /// Create a new terrain generator with the given seed
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            height_noise: Perlin::new(seed),
            temperature_noise: Perlin::new(seed.wrapping_add(1)),
            moisture_noise: Perlin::new(seed.wrapping_add(2)),
            cave_noise: Perlin::new(seed.wrapping_add(3)),
            coal_noise: Perlin::new(seed.wrapping_add(4)),
            iron_noise: Perlin::new(seed.wrapping_add(5)),
            gold_noise: Perlin::new(seed.wrapping_add(6)),
            diamond_noise: Perlin::new(seed.wrapping_add(7)),
        }
    }

    /// Generate a single chunk's terrain data
    /// Returns a Chunk with all blocks filled
    pub fn generate_chunk(&self, chunk_pos: ChunkPos) -> super::Chunk {
        use super::Chunk;

        let mut chunk = Chunk::new();

        // First pass: Generate base terrain
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                let world_x = chunk_pos.x * CHUNK_WIDTH as i32 + x as i32;
                let world_z = chunk_pos.z * CHUNK_DEPTH as i32 + z as i32;

                // Get biome for this column
                let biome = self.get_biome(world_x, world_z);

                // Get height for this column
                let height = self.get_height(world_x, world_z, &biome);

                // Fill column with blocks
                self.generate_column(&mut chunk, x, z, height, &biome);
            }
        }

        // Second pass: Carve caves (3D noise)
        self.carve_caves(&mut chunk, chunk_pos);

        // Third pass: Place ores
        self.place_ores(&mut chunk, chunk_pos);

        chunk
    }

    /// Generate structures (trees, etc.) for a chunk
    /// Returns list of blocks to place (may extend into neighboring chunks)
    pub fn generate_structures(&self, chunk_pos: ChunkPos) -> Vec<(WorldPos, BlockType)> {
        let mut structures = Vec::new();
        let mut rng = StdRng::seed_from_u64(self.chunk_seed(chunk_pos));

        // Try to place trees
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                let world_x = chunk_pos.x * CHUNK_WIDTH as i32 + x as i32;
                let world_z = chunk_pos.z * CHUNK_DEPTH as i32 + z as i32;

                let biome = self.get_biome(world_x, world_z);
                let tree_chance = biome.tree_density();

                if rng.gen::<f64>() < tree_chance {
                    // Find surface height
                    let height = self.get_height(world_x, world_z, &biome);
                    let tree_pos = WorldPos::new(world_x, height, world_z);

                    // Generate tree
                    let tree_type = biome.tree_type();
                    let tree_blocks = TreeGenerator::generate_tree(tree_pos, tree_type, &mut rng);
                    structures.extend(tree_blocks);
                }
            }
        }

        structures
    }

    /// Get the biome at the given world coordinates
    fn get_biome(&self, x: i32, z: i32) -> Biome {
        let scale = 0.003; // Large scale for smooth biome transitions

        let temperature = self.temperature_noise.get([x as f64 * scale, z as f64 * scale]);
        let moisture = self.moisture_noise.get([x as f64 * scale + 1000.0, z as f64 * scale + 1000.0]);

        Biome::from_noise(temperature, moisture)
    }

    /// Get terrain height at the given world coordinates
    /// Uses multi-octave noise for natural-looking terrain
    fn get_height(&self, x: i32, z: i32, biome: &Biome) -> i32 {
        let (base_min, base_max) = biome.height_range();
        let base_height = (base_min + base_max) / 2;
        let multiplier = biome.height_multiplier();

        // Multi-octave noise sampling
        let mut height = 0.0;

        // Large features (continents, mountain ranges)
        height += self.height_noise.get([x as f64 * 0.003, z as f64 * 0.003]) * 20.0;

        // Medium features (hills, valleys)
        height += self.height_noise.get([x as f64 * 0.01, z as f64 * 0.01]) * 10.0;

        // Small features (bumps, detail)
        height += self.height_noise.get([x as f64 * 0.05, z as f64 * 0.05]) * 3.0;

        // Apply biome multiplier
        height *= multiplier;

        // Add to base height
        let final_height = base_height + height as i32;

        // Clamp to valid range
        final_height.clamp(1, CHUNK_HEIGHT as i32 - 10)
    }

    /// Fill a single column with terrain blocks
    fn generate_column(
        &self,
        chunk: &mut super::Chunk,
        x: usize,
        z: usize,
        height: i32,
        biome: &Biome,
    ) {
        let height = height as usize;

        for y in 0..CHUNK_HEIGHT {
            let block = if y == 0 {
                // Bedrock floor
                BlockType::Bedrock
            } else if y < height - 4 {
                // Deep underground = stone
                BlockType::Stone
            } else if y < height {
                // Subsurface layer
                biome.subsurface_block()
            } else if y == height {
                // Surface block
                if y < SEA_LEVEL as usize {
                    // Below sea level, surface is underwater
                    biome.subsurface_block()
                } else {
                    biome.surface_block()
                }
            } else if y <= SEA_LEVEL as usize {
                // Fill with water up to sea level
                BlockType::Water
            } else {
                // Air above surface
                BlockType::Air
            };

            chunk.set_block(x, y, z, block);
        }
    }

    /// Carve caves using 3D noise
    fn carve_caves(
        &self,
        chunk: &mut super::Chunk,
        chunk_pos: ChunkPos,
    ) {
        let scale = 0.05; // Cave scale
        let threshold = 0.6; // Higher = less caves

        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                for y in 1..CHUNK_HEIGHT {
                    // Don't carve caves at bedrock level or near surface
                    if y < 5 || y > 120 {
                        continue;
                    }

                    let world_x = chunk_pos.x * CHUNK_WIDTH as i32 + x as i32;
                    let world_z = chunk_pos.z * CHUNK_DEPTH as i32 + z as i32;

                    // Sample 3D noise
                    let cave_value = self.cave_noise.get([
                        world_x as f64 * scale,
                        y as f64 * scale,
                        world_z as f64 * scale,
                    ]);

                    // If noise is above threshold, carve out the block
                    if cave_value.abs() < (1.0 - threshold) {
                        let current_block = chunk.get_block(x, y, z);

                        // Only carve through stone/dirt/etc, not air or water
                        if current_block.is_solid() && current_block != BlockType::Bedrock {
                            chunk.set_block(x, y, z, BlockType::Air);
                        }
                    }
                }
            }
        }
    }

    /// Place ore veins using noise and depth-based probability
    fn place_ores(
        &self,
        chunk: &mut super::Chunk,
        chunk_pos: ChunkPos,
    ) {
        let scale = 0.1;

        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                for y in 1..CHUNK_HEIGHT {
                    let current_block = chunk.get_block(x, y, z);

                    // Only place ores in stone
                    if current_block != BlockType::Stone {
                        continue;
                    }

                    let world_x = chunk_pos.x * CHUNK_WIDTH as i32 + x as i32;
                    let world_z = chunk_pos.z * CHUNK_DEPTH as i32 + z as i32;

                    // Coal: common, any depth
                    if y < 120 {
                        let coal_value = self.coal_noise.get([
                            world_x as f64 * scale,
                            y as f64 * scale,
                            world_z as f64 * scale,
                        ]);
                        if coal_value > 0.7 {
                            chunk.set_block(x, y, z, BlockType::OreCoal);
                            continue;
                        }
                    }

                    // Iron: common, below y=64
                    if y < 64 {
                        let iron_value = self.iron_noise.get([
                            world_x as f64 * scale,
                            y as f64 * scale,
                            world_z as f64 * scale,
                        ]);
                        if iron_value > 0.75 {
                            chunk.set_block(x, y, z, BlockType::OreIron);
                            continue;
                        }
                    }

                    // Gold: uncommon, below y=32
                    if y < 32 {
                        let gold_value = self.gold_noise.get([
                            world_x as f64 * scale,
                            y as f64 * scale,
                            world_z as f64 * scale,
                        ]);
                        if gold_value > 0.82 {
                            chunk.set_block(x, y, z, BlockType::OreGold);
                            continue;
                        }
                    }

                    // Diamond: rare, below y=16
                    if y < 16 {
                        let diamond_value = self.diamond_noise.get([
                            world_x as f64 * scale,
                            y as f64 * scale,
                            world_z as f64 * scale,
                        ]);
                        if diamond_value > 0.88 {
                            chunk.set_block(x, y, z, BlockType::OreDiamond);
                        }
                    }
                }
            }
        }
    }

    /// Get a deterministic seed for a chunk (for structure generation)
    fn chunk_seed(&self, chunk_pos: ChunkPos) -> u64 {
        let mut seed = self.seed as u64;
        seed = seed.wrapping_mul(31).wrapping_add(chunk_pos.x as u64);
        seed = seed.wrapping_mul(31).wrapping_add(chunk_pos.z as u64);
        seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_generation() {
        let gen = TerrainGenerator::new(12345);
        let chunk = gen.generate_chunk(ChunkPos::new(0, 0));

        // Verify bedrock at bottom
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                assert_eq!(chunk.get_block(x, 0, z), BlockType::Bedrock);
            }
        }
    }

    #[test]
    fn test_deterministic_generation() {
        let gen1 = TerrainGenerator::new(12345);
        let gen2 = TerrainGenerator::new(12345);

        let chunk1 = gen1.generate_chunk(ChunkPos::new(0, 0));
        let chunk2 = gen2.generate_chunk(ChunkPos::new(0, 0));

        // Same seed should produce identical terrain
        for x in 0..CHUNK_WIDTH {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_DEPTH {
                    assert_eq!(
                        chunk1.get_block(x, y, z),
                        chunk2.get_block(x, y, z)
                    );
                }
            }
        }
    }
}
