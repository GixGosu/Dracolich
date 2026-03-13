pub mod renderer;
pub mod world;
pub mod player;
pub mod physics;
pub mod ui;
pub mod mobs;
pub mod inventory;
pub mod audio;
pub mod types;
pub mod window;
pub mod input;
pub mod game_loop;
pub mod config;
pub mod state;
pub mod game;

#[cfg(test)]
pub mod tests;

// Re-export commonly used types
pub use types::{BlockType, ChunkPos, WorldPos, Direction, AABB};
pub use config::*;
pub use state::{GameState, StateManager};
pub use game::Game;
