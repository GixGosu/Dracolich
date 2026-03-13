/// World management system
/// Stores chunks in a HashMap and handles cross-chunk boundary block access

use std::collections::HashMap;
use crate::types::{BlockType, ChunkPos, WorldPos, CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH};
use super::chunk::Chunk;
use super::generation::TerrainGenerator;

/// The world - infinite storage of chunks
pub struct World {
    /// Loaded chunks indexed by chunk position
    chunks: HashMap<ChunkPos, Chunk>,
    /// World seed for procedural generation
    seed: u32,
    /// Terrain generator
    generator: TerrainGenerator,
}

impl World {
    /// Create a new empty world
    pub fn new(seed: u32) -> Self {
        Self {
            chunks: HashMap::new(),
            seed,
            generator: TerrainGenerator::new(seed),
        }
    }

    /// Get the world seed
    pub fn seed(&self) -> u32 {
        self.seed
    }

    /// Check if a chunk is loaded
    pub fn is_chunk_loaded(&self, pos: ChunkPos) -> bool {
        self.chunks.contains_key(&pos)
    }

    /// Get a reference to a chunk (if loaded)
    pub fn get_chunk(&self, pos: &ChunkPos) -> Option<&Chunk> {
        self.chunks.get(pos)
    }

    /// Get a mutable reference to a chunk (if loaded)
    pub fn get_chunk_mut(&mut self, pos: &ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(pos)
    }

    /// Load or create a new chunk (generates terrain if new)
    pub fn load_chunk(&mut self, pos: ChunkPos) {
        // Check if chunk already exists
        if self.chunks.contains_key(&pos) {
            return;
        }

        // Generate terrain for new chunk
        let mut chunk = self.generator.generate_chunk(pos);

        // Generate and place structures
        let structures = self.generator.generate_structures(pos);
        for (block_pos, block_type) in structures {
            // Only place if within this chunk
            let block_chunk = ChunkPos::from_world_pos(&block_pos);
            if block_chunk == pos {
                let (x, y, z) = block_pos.chunk_local();
                chunk.set_block(x, y, z, block_type);
            }
        }

        // Insert the new chunk
        self.chunks.insert(pos, chunk);

        // Mark neighboring chunks as dirty so they re-mesh boundary faces
        let neighbors = [
            ChunkPos::new(pos.x - 1, pos.z),
            ChunkPos::new(pos.x + 1, pos.z),
            ChunkPos::new(pos.x, pos.z - 1),
            ChunkPos::new(pos.x, pos.z + 1),
        ];
        for neighbor_pos in neighbors {
            if let Some(neighbor) = self.chunks.get_mut(&neighbor_pos) {
                neighbor.mark_dirty();
            }
        }
    }

    /// Insert a pre-generated chunk
    pub fn insert_chunk(&mut self, pos: ChunkPos, chunk: Chunk) {
        self.chunks.insert(pos, chunk);
    }

    /// Unload a chunk, returning it if it exists
    pub fn unload_chunk(&mut self, pos: ChunkPos) -> Option<Chunk> {
        self.chunks.remove(&pos)
    }

    /// Get all loaded chunk positions
    pub fn loaded_chunks(&self) -> impl Iterator<Item = ChunkPos> + '_ {
        self.chunks.keys().copied()
    }

    /// Count of loaded chunks
    pub fn loaded_chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Get block at world position
    /// Returns Air if chunk is not loaded or position is out of bounds
    pub fn get_block(&self, pos: &WorldPos) -> BlockType {
        // Convert world position to chunk position
        let chunk_pos = ChunkPos::from_world_pos(pos);

        // Get chunk-local coordinates
        let (local_x, local_y, local_z) = pos.chunk_local();

        // Look up chunk and get block
        self.chunks
            .get(&chunk_pos)
            .map(|chunk| chunk.get_block(local_x, local_y, local_z))
            .unwrap_or(BlockType::Air)
    }

    /// Set block at world position
    /// Loads chunk if not already loaded
    /// Marks neighboring chunks as dirty if on a boundary
    pub fn set_block(&mut self, pos: WorldPos, block: BlockType) {
        // Clamp Y to valid range
        if pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 {
            return;
        }

        // Convert to chunk position and local coordinates
        let chunk_pos = ChunkPos::from_world_pos(&pos);
        let (local_x, local_y, local_z) = pos.chunk_local();

        // Load chunk if needed
        self.load_chunk(chunk_pos);

        // Set block in the chunk
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.set_block(local_x, local_y, local_z, block);
        }

        // Mark neighboring chunks dirty if we're on a boundary
        self.mark_neighbors_dirty_if_boundary(pos, local_x, local_y, local_z);
    }

    /// Mark neighboring chunks dirty if block is on chunk boundary
    /// This ensures faces between chunks get remeshed correctly
    fn mark_neighbors_dirty_if_boundary(&mut self, pos: WorldPos, local_x: usize, local_y: usize, local_z: usize) {
        let chunk_pos = ChunkPos::from_world_pos(&pos);

        // Check -X boundary
        if local_x == 0 {
            if let Some(chunk) = self.chunks.get_mut(&ChunkPos::new(chunk_pos.x - 1, chunk_pos.z)) {
                chunk.mark_dirty();
            }
        }

        // Check +X boundary
        if local_x == CHUNK_WIDTH - 1 {
            if let Some(chunk) = self.chunks.get_mut(&ChunkPos::new(chunk_pos.x + 1, chunk_pos.z)) {
                chunk.mark_dirty();
            }
        }

        // Check -Z boundary
        if local_z == 0 {
            if let Some(chunk) = self.chunks.get_mut(&ChunkPos::new(chunk_pos.x, chunk_pos.z - 1)) {
                chunk.mark_dirty();
            }
        }

        // Check +Z boundary
        if local_z == CHUNK_DEPTH - 1 {
            if let Some(chunk) = self.chunks.get_mut(&ChunkPos::new(chunk_pos.x, chunk_pos.z + 1)) {
                chunk.mark_dirty();
            }
        }

        // Y boundaries don't cross chunks (chunks are 256 blocks tall)
    }

    /// Get block at world position with neighbor chunk lookup
    /// Used by mesher to check blocks in adjacent chunks
    pub fn get_block_with_neighbors(&self, pos: WorldPos) -> BlockType {
        self.get_block(&pos)
    }

    /// Mark a chunk as dirty (needs remeshing)
    pub fn mark_chunk_dirty(&mut self, pos: ChunkPos) {
        if let Some(chunk) = self.chunks.get_mut(&pos) {
            chunk.mark_dirty();
        }
    }

    /// Get all dirty chunks that need remeshing
    pub fn dirty_chunks(&self) -> impl Iterator<Item = (ChunkPos, &Chunk)> + '_ {
        self.chunks
            .iter()
            .filter(|(_, chunk)| chunk.is_dirty())
            .map(|(pos, chunk)| (*pos, chunk))
    }

    /// Clear dirty flag on a chunk
    pub fn clear_chunk_dirty(&mut self, pos: ChunkPos) {
        if let Some(chunk) = self.chunks.get_mut(&pos) {
            chunk.clear_dirty();
        }
    }

    /// Get memory usage statistics
    pub fn stats(&self) -> WorldStats {
        let chunk_count = self.chunks.len();
        let total_blocks: usize = self.chunks.values().map(|c| c.count_solid_blocks()).sum();

        WorldStats {
            loaded_chunks: chunk_count,
            total_solid_blocks: total_blocks,
        }
    }

    /// Remove all chunks (for world reset)
    pub fn clear(&mut self) {
        self.chunks.clear();
    }

    /// Iterator over all chunks (position, chunk reference)
    pub fn iter_chunks(&self) -> impl Iterator<Item = (ChunkPos, &Chunk)> + '_ {
        self.chunks.iter().map(|(pos, chunk)| (*pos, chunk))
    }

    /// Mutable iterator over all chunks
    pub fn iter_chunks_mut(&mut self) -> impl Iterator<Item = (ChunkPos, &mut Chunk)> + '_ {
        self.chunks.iter_mut().map(|(pos, chunk)| (*pos, chunk))
    }

    /// Alias for iter_chunks that returns references
    pub fn chunks(&self) -> impl Iterator<Item = (&ChunkPos, &Chunk)> + '_ {
        self.chunks.iter()
    }
}

/// Statistics about the world
#[derive(Debug, Clone)]
pub struct WorldStats {
    pub loaded_chunks: usize,
    pub total_solid_blocks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let world = World::new(12345);
        assert_eq!(world.seed(), 12345);
        assert_eq!(world.loaded_chunk_count(), 0);
    }

    #[test]
    fn test_chunk_loading() {
        let mut world = World::new(0);

        let pos = ChunkPos::new(0, 0);
        assert!(!world.is_chunk_loaded(pos));

        world.load_chunk(pos);
        assert!(world.is_chunk_loaded(pos));
        assert_eq!(world.loaded_chunk_count(), 1);
    }

    #[test]
    fn test_chunk_unloading() {
        let mut world = World::new(0);

        let pos = ChunkPos::new(5, -3);
        world.load_chunk(pos);
        assert!(world.is_chunk_loaded(pos));

        let chunk = world.unload_chunk(pos);
        assert!(chunk.is_some());
        assert!(!world.is_chunk_loaded(pos));
    }

    #[test]
    fn test_block_get_set() {
        let mut world = World::new(0);

        let pos = WorldPos::new(10, 64, 20);

        // Initially air (chunk auto-loads)
        world.set_block(pos, BlockType::Stone);
        assert_eq!(world.get_block(&pos), BlockType::Stone);

        // Different position should be air
        let pos2 = WorldPos::new(11, 64, 20);
        assert_eq!(world.get_block(&pos2), BlockType::Air);
    }

    #[test]
    fn test_cross_chunk_boundary() {
        let mut world = World::new(0);

        // Place blocks in two adjacent chunks
        let pos1 = WorldPos::new(15, 64, 0);  // Chunk (0, 0), local (15, 64, 0)
        let pos2 = WorldPos::new(16, 64, 0);  // Chunk (1, 0), local (0, 64, 0)

        world.set_block(pos1, BlockType::Stone);
        world.set_block(pos2, BlockType::Dirt);

        assert_eq!(world.get_block(&pos1), BlockType::Stone);
        assert_eq!(world.get_block(&pos2), BlockType::Dirt);

        // Two chunks should be loaded
        assert_eq!(world.loaded_chunk_count(), 2);
    }

    #[test]
    fn test_negative_coordinates() {
        let mut world = World::new(0);

        let pos = WorldPos::new(-10, 64, -20);
        world.set_block(pos, BlockType::Cobblestone);

        assert_eq!(world.get_block(&pos), BlockType::Cobblestone);
    }

    #[test]
    fn test_dirty_chunks() {
        let mut world = World::new(0);

        // Load a chunk and set a block
        let pos = WorldPos::new(0, 64, 0);
        world.set_block(pos, BlockType::Stone);

        // Chunk should be dirty
        let dirty: Vec<_> = world.dirty_chunks().collect();
        assert_eq!(dirty.len(), 1);

        // Clear dirty flag
        world.clear_chunk_dirty(ChunkPos::new(0, 0));

        // Should have no dirty chunks
        let dirty: Vec<_> = world.dirty_chunks().collect();
        assert_eq!(dirty.len(), 0);
    }

    #[test]
    fn test_boundary_dirtying() {
        let mut world = World::new(0);

        // Pre-load two adjacent chunks
        world.load_chunk(ChunkPos::new(0, 0));
        world.load_chunk(ChunkPos::new(1, 0));

        // Clear dirty flags
        world.clear_chunk_dirty(ChunkPos::new(0, 0));
        world.clear_chunk_dirty(ChunkPos::new(1, 0));

        // Place block on boundary
        let boundary_pos = WorldPos::new(15, 64, 0); // local_x = 15 (boundary)
        world.set_block(boundary_pos, BlockType::Stone);

        // Both chunks should be dirty
        let dirty: Vec<_> = world.dirty_chunks().map(|(pos, _)| pos).collect();
        assert!(dirty.contains(&ChunkPos::new(0, 0)));
        assert!(dirty.contains(&ChunkPos::new(1, 0)));
    }

    #[test]
    fn test_stats() {
        let mut world = World::new(0);

        world.set_block(WorldPos::new(0, 64, 0), BlockType::Stone);
        world.set_block(WorldPos::new(1, 64, 0), BlockType::Dirt);

        let stats = world.stats();
        assert_eq!(stats.loaded_chunks, 1);
        assert_eq!(stats.total_solid_blocks, 2);
    }
}
