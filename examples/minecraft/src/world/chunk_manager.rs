/// Chunk manager - handles loading/unloading chunks based on player position
/// Queues chunks for generation and meshing

use std::collections::{HashSet, VecDeque};
use glam::Vec3;
use crate::types::{ChunkPos, WorldPos, RENDER_DISTANCE};
use super::world::World;

/// Manages chunk lifecycle: generation, loading, unloading
pub struct ChunkManager {
    /// Current player chunk position
    player_chunk: ChunkPos,
    /// Chunks that should be loaded (within render distance)
    desired_chunks: HashSet<ChunkPos>,
    /// Chunks queued for generation
    generation_queue: VecDeque<ChunkPos>,
    /// Chunks queued for meshing
    meshing_queue: VecDeque<ChunkPos>,
    /// Render distance in chunks
    render_distance: i32,
}

impl ChunkManager {
    /// Create a new chunk manager
    pub fn new(render_distance: i32) -> Self {
        let mut manager = Self {
            player_chunk: ChunkPos::new(0, 0),
            desired_chunks: HashSet::new(),
            generation_queue: VecDeque::new(),
            meshing_queue: VecDeque::new(),
            render_distance,
        };
        // Initialize desired_chunks for spawn position
        // Without this, if player spawns in chunk (0,0), no chunks would ever load
        // because update_player_position only recalculates when chunk changes
        manager.recalculate_desired_chunks();
        manager
    }

    /// Update player position and determine which chunks should be loaded
    /// Returns true if the player changed chunks
    pub fn update_player_position(&mut self, player_pos: Vec3) -> bool {
        let new_chunk = ChunkPos::from_world_pos(&WorldPos::from_vec3(player_pos));

        if new_chunk != self.player_chunk {
            self.player_chunk = new_chunk;
            self.recalculate_desired_chunks();
            true
        } else {
            false
        }
    }

    /// Recalculate which chunks should be loaded based on render distance
    fn recalculate_desired_chunks(&mut self) {
        self.desired_chunks.clear();

        let center_x = self.player_chunk.x;
        let center_z = self.player_chunk.z;

        // Circular render distance
        let radius_sq = (self.render_distance * self.render_distance) as f32;

        for dx in -self.render_distance..=self.render_distance {
            for dz in -self.render_distance..=self.render_distance {
                // Check if within circular distance
                let dist_sq = (dx * dx + dz * dz) as f32;
                if dist_sq <= radius_sq {
                    self.desired_chunks.insert(ChunkPos::new(center_x + dx, center_z + dz));
                }
            }
        }
    }

    /// Process chunk loading/unloading
    /// Should be called every frame
    pub fn update(&mut self, world: &mut World) {
        // Unload chunks that are too far away
        self.unload_distant_chunks(world);

        // Queue new chunks for generation
        self.queue_new_chunks(world);

        // Queue dirty chunks for remeshing
        self.queue_dirty_chunks(world);
    }

    /// Unload chunks that are beyond render distance
    fn unload_distant_chunks(&mut self, world: &mut World) {
        let chunks_to_unload: Vec<ChunkPos> = world
            .loaded_chunks()
            .filter(|pos| !self.desired_chunks.contains(pos))
            .collect();

        for pos in chunks_to_unload {
            world.unload_chunk(pos);
        }
    }

    /// Queue chunks for generation if they're not loaded
    fn queue_new_chunks(&mut self, world: &World) {
        for &chunk_pos in &self.desired_chunks {
            if !world.is_chunk_loaded(chunk_pos) && !self.generation_queue.contains(&chunk_pos) {
                self.generation_queue.push_back(chunk_pos);
            }
        }

        // Sort generation queue by distance from player (closest first)
        self.sort_generation_queue();
    }

    /// Sort generation queue by Manhattan distance from player
    fn sort_generation_queue(&mut self) {
        let player_chunk = self.player_chunk;

        // Convert to vec, sort, convert back
        let mut queue_vec: Vec<ChunkPos> = self.generation_queue.drain(..).collect();

        queue_vec.sort_by_key(|pos| {
            let dx = (pos.x - player_chunk.x).abs();
            let dz = (pos.z - player_chunk.z).abs();
            dx + dz
        });

        self.generation_queue = queue_vec.into();
    }

    /// Queue dirty chunks for remeshing
    fn queue_dirty_chunks(&mut self, world: &World) {
        for (pos, _) in world.dirty_chunks() {
            if !self.meshing_queue.contains(&pos) {
                self.meshing_queue.push_back(pos);
            }
        }
    }

    /// Pop a chunk from generation queue (returns None if queue empty)
    pub fn pop_generation_task(&mut self) -> Option<ChunkPos> {
        self.generation_queue.pop_front()
    }

    /// Pop a chunk from meshing queue (returns None if queue empty)
    pub fn pop_meshing_task(&mut self) -> Option<ChunkPos> {
        self.meshing_queue.pop_front()
    }

    /// Check if there's generation work to do
    pub fn has_generation_work(&self) -> bool {
        !self.generation_queue.is_empty()
    }

    /// Check if there's meshing work to do
    pub fn has_meshing_work(&self) -> bool {
        !self.meshing_queue.is_empty()
    }

    /// Get number of chunks queued for generation
    pub fn generation_queue_len(&self) -> usize {
        self.generation_queue.len()
    }

    /// Get number of chunks queued for meshing
    pub fn meshing_queue_len(&self) -> usize {
        self.meshing_queue.len()
    }

    /// Get current player chunk position
    pub fn player_chunk(&self) -> ChunkPos {
        self.player_chunk
    }

    /// Get render distance
    pub fn render_distance(&self) -> i32 {
        self.render_distance
    }

    /// Set render distance (triggers recalculation)
    pub fn set_render_distance(&mut self, distance: i32) {
        if distance != self.render_distance {
            self.render_distance = distance.max(2); // Minimum 2 chunks
            self.recalculate_desired_chunks();
        }
    }

    /// Force queue a specific chunk for generation
    pub fn queue_chunk_generation(&mut self, pos: ChunkPos) {
        if !self.generation_queue.contains(&pos) {
            self.generation_queue.push_back(pos);
        }
    }

    /// Force queue a specific chunk for meshing
    pub fn queue_chunk_meshing(&mut self, pos: ChunkPos) {
        if !self.meshing_queue.contains(&pos) {
            self.meshing_queue.push_back(pos);
        }
    }

    /// Get statistics
    pub fn stats(&self) -> ChunkManagerStats {
        ChunkManagerStats {
            desired_chunks: self.desired_chunks.len(),
            generation_queue_len: self.generation_queue.len(),
            meshing_queue_len: self.meshing_queue.len(),
            player_chunk: self.player_chunk,
        }
    }

    /// Clear all queues
    pub fn clear_queues(&mut self) {
        self.generation_queue.clear();
        self.meshing_queue.clear();
    }

    /// Get all chunks that should be loaded
    pub fn desired_chunks(&self) -> &HashSet<ChunkPos> {
        &self.desired_chunks
    }

    /// Check if a chunk is within render distance
    pub fn is_chunk_desired(&self, pos: ChunkPos) -> bool {
        self.desired_chunks.contains(&pos)
    }

    /// Get chunks in a spiral pattern from player (for prioritized loading)
    /// Returns iterator of chunk positions sorted by distance
    pub fn chunks_in_spiral(&self) -> Vec<ChunkPos> {
        let mut chunks = Vec::new();

        for &pos in &self.desired_chunks {
            chunks.push(pos);
        }

        // Sort by distance from player
        let player_chunk = self.player_chunk;
        chunks.sort_by_key(|pos| {
            let dx = pos.x - player_chunk.x;
            let dz = pos.z - player_chunk.z;
            dx * dx + dz * dz
        });

        chunks
    }
}

/// Statistics about chunk manager state
#[derive(Debug, Clone)]
pub struct ChunkManagerStats {
    pub desired_chunks: usize,
    pub generation_queue_len: usize,
    pub meshing_queue_len: usize,
    pub player_chunk: ChunkPos,
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self::new(RENDER_DISTANCE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_manager_creation() {
        let manager = ChunkManager::new(8);
        assert_eq!(manager.render_distance(), 8);
        assert_eq!(manager.player_chunk(), ChunkPos::new(0, 0));
    }

    #[test]
    fn test_player_position_update() {
        let mut manager = ChunkManager::new(8);

        // Move player to new chunk
        let changed = manager.update_player_position(Vec3::new(20.0, 64.0, 20.0));
        assert!(changed);
        assert_eq!(manager.player_chunk(), ChunkPos::new(1, 1));

        // Move within same chunk
        let changed = manager.update_player_position(Vec3::new(21.0, 64.0, 21.0));
        assert!(!changed);
    }

    #[test]
    fn test_desired_chunks_calculation() {
        let mut manager = ChunkManager::new(2);
        manager.update_player_position(Vec3::new(0.0, 64.0, 0.0));

        let desired = manager.desired_chunks();

        // Should include center and nearby chunks
        assert!(desired.contains(&ChunkPos::new(0, 0)));
        assert!(desired.contains(&ChunkPos::new(1, 0)));
        assert!(desired.contains(&ChunkPos::new(0, 1)));
        assert!(desired.contains(&ChunkPos::new(-1, 0)));

        // Should NOT include very distant chunks
        assert!(!desired.contains(&ChunkPos::new(10, 10)));
    }

    #[test]
    fn test_generation_queue() {
        let mut manager = ChunkManager::new(2);
        let mut world = World::new(0);

        manager.update_player_position(Vec3::new(0.0, 64.0, 0.0));
        manager.update(&mut world);

        // Should have chunks queued for generation
        assert!(manager.has_generation_work());
        assert!(manager.generation_queue_len() > 0);

        // Pop a task
        let task = manager.pop_generation_task();
        assert!(task.is_some());
    }

    #[test]
    fn test_chunk_unloading() {
        let mut manager = ChunkManager::new(2);
        let mut world = World::new(0);

        // Load some chunks
        world.load_chunk(ChunkPos::new(0, 0));
        world.load_chunk(ChunkPos::new(10, 10)); // Far away

        // Update manager at origin
        manager.update_player_position(Vec3::new(0.0, 64.0, 0.0));
        manager.update(&mut world);

        // Far chunk should be unloaded
        assert!(!world.is_chunk_loaded(ChunkPos::new(10, 10)));
        // Close chunk should remain
        assert!(world.is_chunk_loaded(ChunkPos::new(0, 0)));
    }

    #[test]
    fn test_render_distance_change() {
        let mut manager = ChunkManager::new(4);

        manager.update_player_position(Vec3::new(0.0, 64.0, 0.0));
        let chunks_at_4 = manager.desired_chunks().len();

        manager.set_render_distance(8);
        let chunks_at_8 = manager.desired_chunks().len();

        // More chunks should be desired with higher render distance
        assert!(chunks_at_8 > chunks_at_4);
    }

    #[test]
    fn test_meshing_queue() {
        let mut manager = ChunkManager::new(4);
        let mut world = World::new(0);

        // Create a dirty chunk
        world.load_chunk(ChunkPos::new(0, 0));
        world.set_block(WorldPos::new(0, 64, 0), crate::types::BlockType::Stone);

        manager.update(&mut world);

        // Should be queued for meshing
        assert!(manager.has_meshing_work());
        assert_eq!(manager.meshing_queue_len(), 1);
    }

    #[test]
    fn test_spiral_ordering() {
        let mut manager = ChunkManager::new(3);
        manager.update_player_position(Vec3::new(0.0, 64.0, 0.0));

        let spiral = manager.chunks_in_spiral();

        // First chunk should be player's chunk or very close
        let player_chunk = manager.player_chunk();
        let first_dist = (spiral[0].x - player_chunk.x).abs() + (spiral[0].z - player_chunk.z).abs();
        assert!(first_dist <= 1);
    }
}
