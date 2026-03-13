// World management system
// Handles chunk generation, storage, loading/unloading,
// block get/set operations, and terrain generation

mod biome;
mod chunk;
mod chunk_manager;
mod generation;
mod mesher;
mod structure;
mod world;

pub use biome::{Biome, TreeType};
pub use chunk::{Chunk, ChunkBlockIterator};
pub use chunk_manager::{ChunkManager, ChunkManagerStats};
pub use generation::TerrainGenerator;
pub use mesher::{mesh_chunk, MeshData};
pub use structure::{TreeGenerator, BoulderGenerator};
pub use world::{World, WorldStats};
