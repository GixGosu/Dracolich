/// Chunk data structure with palette compression
/// Each chunk stores 16x256x16 blocks using a palette to reduce memory usage

use crate::types::{BlockType, CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH};

/// A single chunk of the world (16x256x16 blocks)
pub struct Chunk {
    /// Palette of unique block types in this chunk
    palette: Vec<BlockType>,
    /// Indices into the palette (one per block)
    /// Uses u8 for chunks with ≤256 unique block types (99% of cases)
    /// Falls back to raw BlockType storage if needed
    indices: ChunkData,
    /// Dirty flag - set to true when blocks change, requires remeshing
    dirty: bool,
}

/// Storage strategy for chunk blocks
enum ChunkData {
    /// Palette-compressed: indices into palette (memory efficient)
    Palette(Box<[u8; CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH]>),
    /// Raw block types (fallback for highly diverse chunks)
    Raw(Box<[BlockType; CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH]>),
}

impl Chunk {
    /// Create a new empty chunk (all air)
    pub fn new() -> Self {
        let palette = vec![BlockType::Air];
        let indices = Box::new([0u8; CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH]);

        Self {
            palette,
            indices: ChunkData::Palette(indices),
            dirty: true,
        }
    }

    /// Create a chunk filled with a specific block type
    pub fn filled(block: BlockType) -> Self {
        let palette = vec![block];
        let indices = Box::new([0u8; CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH]);

        Self {
            palette,
            indices: ChunkData::Palette(indices),
            dirty: true,
        }
    }

    /// Convert local chunk coordinates (x, y, z) to flat array index
    /// x, y, z must be in range [0, CHUNK_WIDTH), [0, CHUNK_HEIGHT), [0, CHUNK_DEPTH)
    #[inline]
    fn coords_to_index(x: usize, y: usize, z: usize) -> usize {
        debug_assert!(x < CHUNK_WIDTH);
        debug_assert!(y < CHUNK_HEIGHT);
        debug_assert!(z < CHUNK_DEPTH);

        // YZX ordering for better cache locality during vertical iteration
        y + (z * CHUNK_HEIGHT) + (x * CHUNK_HEIGHT * CHUNK_DEPTH)
    }

    /// Get block at local coordinates
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType {
        if x >= CHUNK_WIDTH || y >= CHUNK_HEIGHT || z >= CHUNK_DEPTH {
            return BlockType::Air;
        }

        let index = Self::coords_to_index(x, y, z);

        match &self.indices {
            ChunkData::Palette(indices) => {
                let palette_index = indices[index] as usize;
                self.palette.get(palette_index).copied().unwrap_or(BlockType::Air)
            }
            ChunkData::Raw(blocks) => blocks[index],
        }
    }

    /// Set block at local coordinates
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        if x >= CHUNK_WIDTH || y >= CHUNK_HEIGHT || z >= CHUNK_DEPTH {
            return;
        }

        let index = Self::coords_to_index(x, y, z);

        match &mut self.indices {
            ChunkData::Palette(indices) => {
                // Try to find block in existing palette
                if let Some(palette_idx) = self.palette.iter().position(|&b| b == block) {
                    indices[index] = palette_idx as u8;
                } else if self.palette.len() < 256 {
                    // Add to palette if there's room
                    self.palette.push(block);
                    indices[index] = (self.palette.len() - 1) as u8;
                } else {
                    // Palette full - convert to raw storage
                    self.convert_to_raw();
                    if let ChunkData::Raw(blocks) = &mut self.indices {
                        blocks[index] = block;
                    }
                }
            }
            ChunkData::Raw(blocks) => {
                blocks[index] = block;
            }
        }

        self.dirty = true;
    }

    /// Convert from palette to raw storage (expensive, only when palette exhausted)
    fn convert_to_raw(&mut self) {
        if let ChunkData::Palette(indices) = &self.indices {
            let mut raw = Box::new([BlockType::Air; CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH]);

            for i in 0..(CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH) {
                let palette_idx = indices[i] as usize;
                raw[i] = self.palette.get(palette_idx).copied().unwrap_or(BlockType::Air);
            }

            self.indices = ChunkData::Raw(raw);
            self.palette.clear(); // Free palette memory
        }
    }

    /// Check if chunk needs remeshing
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark chunk as clean (meshing complete)
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Mark chunk as dirty (needs remeshing)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Get iterator over all blocks in the chunk
    /// Returns (x, y, z, block_type) for each block
    pub fn iter_blocks(&self) -> ChunkBlockIterator {
        ChunkBlockIterator {
            chunk: self,
            index: 0,
        }
    }

    /// Check if chunk is entirely air (optimization for empty chunks)
    pub fn is_empty(&self) -> bool {
        match &self.indices {
            ChunkData::Palette(indices) => {
                // If palette has only one entry and it's Air, chunk is empty
                self.palette.len() == 1 && self.palette[0] == BlockType::Air
            }
            ChunkData::Raw(_) => false, // Raw chunks are never empty (palette exhaustion means diversity)
        }
    }

    /// Count non-air blocks in chunk (for statistics)
    pub fn count_solid_blocks(&self) -> usize {
        let mut count = 0;
        for (_, _, _, block) in self.iter_blocks() {
            if block != BlockType::Air {
                count += 1;
            }
        }
        count
    }

    /// Fill a rectangular region with a block type
    /// Useful for terrain generation
    pub fn fill_region(
        &mut self,
        min_x: usize,
        min_y: usize,
        min_z: usize,
        max_x: usize,
        max_y: usize,
        max_z: usize,
        block: BlockType,
    ) {
        for x in min_x..=max_x.min(CHUNK_WIDTH - 1) {
            for y in min_y..=max_y.min(CHUNK_HEIGHT - 1) {
                for z in min_z..=max_z.min(CHUNK_DEPTH - 1) {
                    self.set_block(x, y, z, block);
                }
            }
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over all blocks in a chunk
pub struct ChunkBlockIterator<'a> {
    chunk: &'a Chunk,
    index: usize,
}

impl<'a> Iterator for ChunkBlockIterator<'a> {
    type Item = (usize, usize, usize, BlockType);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH {
            return None;
        }

        // Convert flat index back to coordinates (inverse of coords_to_index)
        let x = self.index / (CHUNK_HEIGHT * CHUNK_DEPTH);
        let rem = self.index % (CHUNK_HEIGHT * CHUNK_DEPTH);
        let z = rem / CHUNK_HEIGHT;
        let y = rem % CHUNK_HEIGHT;

        let block = self.chunk.get_block(x, y, z);
        self.index += 1;

        Some((x, y, z, block))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_creation() {
        let chunk = Chunk::new();
        assert!(chunk.is_empty());
        assert!(chunk.is_dirty());
    }

    #[test]
    fn test_block_get_set() {
        let mut chunk = Chunk::new();

        // Set a block
        chunk.set_block(5, 10, 7, BlockType::Stone);
        assert_eq!(chunk.get_block(5, 10, 7), BlockType::Stone);

        // Other blocks should still be air
        assert_eq!(chunk.get_block(0, 0, 0), BlockType::Air);
        assert_eq!(chunk.get_block(15, 255, 15), BlockType::Air);
    }

    #[test]
    fn test_bounds_checking() {
        let mut chunk = Chunk::new();

        // Out of bounds access should return Air
        assert_eq!(chunk.get_block(16, 0, 0), BlockType::Air);
        assert_eq!(chunk.get_block(0, 256, 0), BlockType::Air);
        assert_eq!(chunk.get_block(0, 0, 16), BlockType::Air);

        // Out of bounds set should not panic
        chunk.set_block(20, 300, 20, BlockType::Stone);
    }

    #[test]
    fn test_palette_compression() {
        let mut chunk = Chunk::new();

        // Add several different blocks
        chunk.set_block(0, 0, 0, BlockType::Stone);
        chunk.set_block(1, 0, 0, BlockType::Dirt);
        chunk.set_block(2, 0, 0, BlockType::Grass);

        // Palette should have 4 entries: Air, Stone, Dirt, Grass
        assert_eq!(chunk.palette.len(), 4);

        // Adding more of the same type shouldn't grow palette
        chunk.set_block(3, 0, 0, BlockType::Stone);
        assert_eq!(chunk.palette.len(), 4);
    }

    #[test]
    fn test_iterator() {
        let mut chunk = Chunk::new();
        chunk.set_block(0, 0, 0, BlockType::Stone);
        chunk.set_block(15, 255, 15, BlockType::Bedrock);

        let stones: Vec<_> = chunk
            .iter_blocks()
            .filter(|(_, _, _, b)| *b == BlockType::Stone)
            .collect();

        assert_eq!(stones.len(), 1);
        assert_eq!(stones[0], (0, 0, 0, BlockType::Stone));
    }

    #[test]
    fn test_fill_region() {
        let mut chunk = Chunk::new();
        chunk.fill_region(0, 0, 0, 15, 3, 15, BlockType::Stone);

        // First 4 layers should be stone
        assert_eq!(chunk.get_block(0, 0, 0), BlockType::Stone);
        assert_eq!(chunk.get_block(15, 3, 15), BlockType::Stone);

        // Above should be air
        assert_eq!(chunk.get_block(0, 4, 0), BlockType::Air);
    }
}
