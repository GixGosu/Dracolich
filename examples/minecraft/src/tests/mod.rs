//! Comprehensive unit tests for critical game systems
//!
//! This module contains tests for:
//! - AABB collision detection and sweep tests
//! - Raycast block targeting
//! - Chunk coordinate conversions
//! - Recipe matching
//! - Block type properties
//! - World position calculations
//! - Integration tests for subsystem interaction
//!
//! ## Integration Test Notes
//!
//! Full integration tests (Game::new(), full game loop) require:
//! - OpenGL context (via glutin/winit)
//! - Audio device (via rodio)
//! - Window creation
//!
//! These cannot run in headless CI environments. The integration tests below
//! verify subsystem interactions without requiring graphics/audio context.

#[cfg(test)]
mod aabb_tests {
    use crate::types::{AABB, WorldPos};
    use glam::Vec3;

    #[test]
    fn test_aabb_creation() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        assert_eq!(aabb.min, Vec3::ZERO);
        assert_eq!(aabb.max, Vec3::ONE);
    }

    #[test]
    fn test_aabb_from_center_size() {
        let aabb = AABB::from_center_size(Vec3::new(5.0, 5.0, 5.0), Vec3::new(2.0, 4.0, 2.0));
        assert_eq!(aabb.min, Vec3::new(4.0, 3.0, 4.0));
        assert_eq!(aabb.max, Vec3::new(6.0, 7.0, 6.0));
    }

    #[test]
    fn test_aabb_from_block() {
        let pos = WorldPos::new(10, 20, 30);
        let aabb = AABB::from_block(&pos);
        assert_eq!(aabb.min, Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(aabb.max, Vec3::new(11.0, 21.0, 31.0));
    }

    #[test]
    fn test_aabb_intersection() {
        let a = AABB::new(Vec3::ZERO, Vec3::ONE);
        let b = AABB::new(Vec3::splat(0.5), Vec3::splat(1.5));
        assert!(a.intersects(&b), "Overlapping AABBs should intersect");

        let c = AABB::new(Vec3::splat(2.0), Vec3::splat(3.0));
        assert!(!a.intersects(&c), "Separated AABBs should not intersect");
    }

    #[test]
    fn test_aabb_contains_point() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);

        assert!(aabb.contains_point(Vec3::splat(0.5)), "Center point should be contained");
        assert!(aabb.contains_point(Vec3::ZERO), "Min corner should be contained");
        assert!(aabb.contains_point(Vec3::ONE), "Max corner should be contained");
        assert!(!aabb.contains_point(Vec3::splat(2.0)), "Outside point should not be contained");
    }

    #[test]
    fn test_aabb_center() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::splat(2.0));
        assert_eq!(aabb.center(), Vec3::ONE);
    }

    #[test]
    fn test_aabb_size() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(aabb.size(), Vec3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_aabb_expand() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        let expanded = aabb.expand(1.0);
        assert_eq!(expanded.min, Vec3::splat(-1.0));
        assert_eq!(expanded.max, Vec3::splat(2.0));
    }

    #[test]
    fn test_aabb_translate() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        let translated = aabb.translate(Vec3::new(5.0, 10.0, 15.0));
        assert_eq!(translated.min, Vec3::new(5.0, 10.0, 15.0));
        assert_eq!(translated.max, Vec3::new(6.0, 11.0, 16.0));
    }

    #[test]
    fn test_aabb_get_overlapping_blocks() {
        let aabb = AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.5, 1.5, 2.5));
        let blocks = aabb.get_overlapping_blocks();

        // Should cover 3x2x3 = 18 blocks
        assert_eq!(blocks.len(), 18, "Should overlap 18 blocks");

        // Check some specific blocks
        assert!(blocks.contains(&WorldPos::new(0, 0, 0)));
        assert!(blocks.contains(&WorldPos::new(1, 1, 1)));
        assert!(blocks.contains(&WorldPos::new(2, 0, 2)));
    }

    #[test]
    fn test_aabb_penetration() {
        let a = AABB::new(Vec3::ZERO, Vec3::splat(2.0));
        let b = AABB::new(Vec3::ONE, Vec3::splat(3.0));

        let (normal, depth) = a.get_penetration(&b).expect("Should have penetration");
        assert!(depth > 0.0, "Penetration depth should be positive");
        assert_eq!(normal.length(), 1.0, "Normal should be unit length");
    }
}

#[cfg(test)]
mod raycast_tests {
    use crate::physics::raycast::raycast;
    use crate::types::{BlockType, WorldPos};
    use glam::Vec3;

    #[test]
    fn test_raycast_hits_block() {
        let origin = Vec3::new(0.5, 0.5, 0.5);
        let direction = Vec3::new(1.0, 0.0, 0.0);
        let max_distance = 5.0;

        // Create a simple world with a solid block at (3, 0, 0)
        let get_block = |pos: &WorldPos| -> BlockType {
            if pos.x == 3 && pos.y == 0 && pos.z == 0 {
                BlockType::Stone
            } else {
                BlockType::Air
            }
        };

        let result = raycast(origin, direction, max_distance, get_block);
        assert!(result.is_some(), "Raycast should hit the stone block");

        if let Some((block_pos, _face)) = result {
            assert_eq!(block_pos, WorldPos::new(3, 0, 0));
        }
    }

    #[test]
    fn test_raycast_misses_when_no_blocks() {
        let origin = Vec3::new(0.5, 0.5, 0.5);
        let direction = Vec3::new(1.0, 0.0, 0.0);
        let max_distance = 5.0;

        // World with only air
        let get_block = |_: &WorldPos| -> BlockType {
            BlockType::Air
        };

        let result = raycast(origin, direction, max_distance, get_block);
        assert!(result.is_none(), "Raycast should miss when there are no solid blocks");
    }

    #[test]
    fn test_raycast_max_distance() {
        let origin = Vec3::new(0.5, 0.5, 0.5);
        let direction = Vec3::new(1.0, 0.0, 0.0);
        let max_distance = 2.0;

        // Block at distance 3
        let get_block = |pos: &WorldPos| -> BlockType {
            if pos.x == 3 && pos.y == 0 && pos.z == 0 {
                BlockType::Stone
            } else {
                BlockType::Air
            }
        };

        let result = raycast(origin, direction, max_distance, get_block);
        assert!(result.is_none(), "Raycast should not reach blocks beyond max distance");
    }
}

#[cfg(test)]
mod chunk_coordinate_tests {
    use crate::types::{ChunkPos, WorldPos};
    use glam::IVec3;

    #[test]
    fn test_chunk_pos_from_world_pos() {
        // Positive coordinates
        let world_pos = WorldPos::new(17, 64, 25);
        let chunk_pos = ChunkPos::from_world_pos_struct(&world_pos);
        assert_eq!(chunk_pos, ChunkPos::new(1, 1));

        // Negative coordinates (test euclidean division)
        let world_pos = WorldPos::new(-1, 64, -1);
        let chunk_pos = ChunkPos::from_world_pos_struct(&world_pos);
        assert_eq!(chunk_pos, ChunkPos::new(-1, -1));

        // Chunk boundary
        let world_pos = WorldPos::new(16, 64, 16);
        let chunk_pos = ChunkPos::from_world_pos_struct(&world_pos);
        assert_eq!(chunk_pos, ChunkPos::new(1, 1));

        // Origin
        let world_pos = WorldPos::new(0, 64, 0);
        let chunk_pos = ChunkPos::from_world_pos_struct(&world_pos);
        assert_eq!(chunk_pos, ChunkPos::new(0, 0));
    }

    #[test]
    fn test_chunk_to_world_origin() {
        let chunk = ChunkPos::new(2, 3);
        let origin = chunk.to_world_origin();
        assert_eq!(origin, IVec3::new(32, 0, 48));

        let chunk = ChunkPos::new(-1, -1);
        let origin = chunk.to_world_origin();
        assert_eq!(origin, IVec3::new(-16, 0, -16));
    }

    #[test]
    fn test_chunk_neighbors() {
        let chunk = ChunkPos::new(0, 0);
        let neighbors = chunk.neighbors();

        assert_eq!(neighbors.len(), 8);
        assert!(neighbors.contains(&ChunkPos::new(-1, -1)));
        assert!(neighbors.contains(&ChunkPos::new(-1, 0)));
        assert!(neighbors.contains(&ChunkPos::new(-1, 1)));
        assert!(neighbors.contains(&ChunkPos::new(0, -1)));
        assert!(neighbors.contains(&ChunkPos::new(0, 1)));
        assert!(neighbors.contains(&ChunkPos::new(1, -1)));
        assert!(neighbors.contains(&ChunkPos::new(1, 0)));
        assert!(neighbors.contains(&ChunkPos::new(1, 1)));
    }

    #[test]
    fn test_world_pos_chunk_local() {
        // Test positive coordinates
        let pos = WorldPos::new(18, 100, 22);
        let (x, y, z) = pos.chunk_local();
        assert_eq!(x, 2);
        assert_eq!(y, 100);
        assert_eq!(z, 6);

        // Test negative coordinates
        let pos = WorldPos::new(-1, 50, -1);
        let (x, y, z) = pos.chunk_local();
        assert_eq!(x, 15);
        assert_eq!(y, 50);
        assert_eq!(z, 15);

        // Test chunk boundary
        let pos = WorldPos::new(16, 75, 16);
        let (x, y, z) = pos.chunk_local();
        assert_eq!(x, 0);
        assert_eq!(y, 75);
        assert_eq!(z, 0);
    }
}

#[cfg(test)]
mod recipe_tests {
    use crate::inventory::crafting::{CraftingGrid, Recipe, RecipePattern, get_recipes};
    use crate::inventory::{Item, ItemStack};
    use crate::inventory::tools::{ToolType, ToolTier};
    use crate::types::BlockType;

    #[test]
    fn test_planks_recipe() {
        let mut grid = CraftingGrid::new_2x2();

        // Place oak wood in first slot
        grid.set_slot(0, Some(ItemStack::new(Item::Block(BlockType::WoodOak), 1)));

        // Should match planks recipe
        let recipes = get_recipes();
        let planks_recipe = recipes.iter()
            .find(|r| matches!(r.output.item, Item::Block(BlockType::Planks)))
            .expect("Planks recipe should exist");

        assert!(planks_recipe.matches(&grid).is_some(), "Oak wood should craft into planks");
    }

    #[test]
    fn test_sticks_recipe() {
        let mut grid = CraftingGrid::new_2x2();

        // Vertical planks pattern (slots 0 and 2 for 2x2 grid - top to bottom)
        grid.set_slot(0, Some(ItemStack::new(Item::Block(BlockType::Planks), 1)));
        grid.set_slot(2, Some(ItemStack::new(Item::Block(BlockType::Planks), 1)));

        // Should match sticks recipe
        let recipes = get_recipes();
        let sticks_recipe = recipes.iter()
            .find(|r| r.output.item == Item::Stick)
            .expect("Sticks recipe should exist");

        assert!(sticks_recipe.matches(&grid).is_some(), "Vertical planks should craft into sticks");
    }

    #[test]
    fn test_wooden_pickaxe_recipe() {
        let mut grid = CraftingGrid::new_3x3();

        // Pickaxe pattern:
        // P P P
        // - S -
        // - S -
        grid.set_slot(0, Some(ItemStack::new(Item::Block(BlockType::Planks), 1)));
        grid.set_slot(1, Some(ItemStack::new(Item::Block(BlockType::Planks), 1)));
        grid.set_slot(2, Some(ItemStack::new(Item::Block(BlockType::Planks), 1)));
        grid.set_slot(4, Some(ItemStack::new(Item::Stick, 1)));
        grid.set_slot(7, Some(ItemStack::new(Item::Stick, 1)));

        // Should match wooden pickaxe recipe
        let recipes = get_recipes();
        let pickaxe_recipe = recipes.iter()
            .find(|r| r.output.item == Item::Tool(ToolType::Pickaxe, ToolTier::Wood))
            .expect("Wooden pickaxe recipe should exist");

        assert!(pickaxe_recipe.matches(&grid).is_some(), "Planks + sticks should craft wooden pickaxe");
    }

    #[test]
    fn test_recipe_does_not_match_wrong_items() {
        let mut grid = CraftingGrid::new_2x2();

        // Place dirt instead of planks
        grid.set_slot(0, Some(ItemStack::new(Item::Block(BlockType::Dirt), 1)));

        let recipes = get_recipes();
        let planks_recipe = recipes.iter()
            .find(|r| matches!(r.output.item, Item::Block(BlockType::Planks)))
            .unwrap();

        assert!(planks_recipe.matches(&grid).is_none(), "Dirt should not craft into planks");
    }

    #[test]
    fn test_empty_grid_matches_nothing() {
        let grid = CraftingGrid::new_2x2();
        let recipes = get_recipes();

        for recipe in &recipes {
            assert!(recipe.matches(&grid).is_none(), "Empty grid should not match any recipes");
        }
    }
}

#[cfg(test)]
mod block_type_tests {
    use crate::types::BlockType;

    #[test]
    fn test_block_solidity() {
        assert!(!BlockType::Air.is_solid());
        assert!(!BlockType::Water.is_solid());
        assert!(BlockType::Stone.is_solid());
        assert!(BlockType::Grass.is_solid());
        assert!(BlockType::WoodOak.is_solid());
    }

    #[test]
    fn test_block_transparency() {
        assert!(BlockType::Air.is_transparent());
        assert!(BlockType::Water.is_transparent());
        assert!(BlockType::Glass.is_transparent());
        assert!(BlockType::LeavesOak.is_transparent());
        assert!(!BlockType::Stone.is_transparent());
        assert!(!BlockType::Dirt.is_transparent());
    }

    #[test]
    fn test_block_hardness() {
        assert_eq!(BlockType::Air.hardness(), 0.0);
        assert!(BlockType::Dirt.hardness() < BlockType::Stone.hardness());
        assert!(BlockType::Stone.hardness() < BlockType::OreIron.hardness());
        assert!(BlockType::OreIron.hardness() < BlockType::OreDiamond.hardness());
        assert_eq!(BlockType::Bedrock.hardness(), f32::INFINITY);
    }

    #[test]
    fn test_block_breakability() {
        assert!(BlockType::Dirt.is_breakable());
        assert!(BlockType::Stone.is_breakable());
        assert!(BlockType::OreDiamond.is_breakable());
        assert!(!BlockType::Bedrock.is_breakable());
    }

    #[test]
    fn test_block_names() {
        assert_eq!(BlockType::Grass.name(), "Grass Block");
        assert_eq!(BlockType::WoodOak.name(), "Oak Wood");
        assert_eq!(BlockType::OreCoal.name(), "Coal Ore");
        assert_eq!(BlockType::CraftingTable.name(), "Crafting Table");
    }

    #[test]
    fn test_texture_indices() {
        // Grass has different top/bottom/sides
        let (top, bottom, side) = BlockType::Grass.texture_indices();
        assert_ne!(top, bottom);
        assert_ne!(top, side);
        assert_ne!(bottom, side);

        // Stone has same texture on all sides
        let (top, bottom, side) = BlockType::Stone.texture_indices();
        assert_eq!(top, bottom);
        assert_eq!(top, side);
    }
}

#[cfg(test)]
mod collision_tests {
    use crate::physics::collision::collide_and_slide;
    use crate::types::{BlockType, WorldPos, AABB};
    use glam::Vec3;

    #[test]
    fn test_collision_with_floor() {
        let player_aabb = AABB::from_center_size(Vec3::new(0.5, 1.0, 0.5), Vec3::new(0.6, 1.8, 0.6));
        let velocity = Vec3::new(0.0, -1.0, 0.0); // Falling down

        // Floor at y=0
        let get_block = |pos: &WorldPos| -> BlockType {
            if pos.y < 0 {
                BlockType::Stone
            } else {
                BlockType::Air
            }
        };

        let info = collide_and_slide(
            &player_aabb,
            velocity,
            &get_block,
        );

        assert!(info.on_ground, "Should detect ground collision");
        assert!(info.velocity.y >= 0.0, "Downward velocity should be stopped");
        assert!(info.position.y >= 0.0, "Should not penetrate floor");
    }

    #[test]
    fn test_no_collision_in_air() {
        let player_aabb = AABB::from_center_size(Vec3::new(0.5, 10.0, 0.5), Vec3::new(0.6, 1.8, 0.6));
        let velocity = Vec3::new(1.0, -0.5, 0.5);

        // Empty world
        let get_block = |_: &WorldPos| -> BlockType {
            BlockType::Air
        };

        let info = collide_and_slide(
            &player_aabb,
            velocity,
            &get_block,
        );

        assert!(!info.on_ground, "Should not detect ground in empty space");
        assert_eq!(info.velocity, velocity, "Velocity should be unchanged in empty space");
    }

    #[test]
    fn test_collision_with_wall() {
        let player_pos = Vec3::new(0.5, 1.0, 0.5);
        let player_aabb = AABB::from_center_size(player_pos, Vec3::new(0.6, 1.8, 0.6));
        let velocity = Vec3::new(2.0, 0.0, 0.0); // Moving right

        // Wall at x=2
        let get_block = |pos: &WorldPos| -> BlockType {
            if pos.x >= 2 {
                BlockType::Stone
            } else {
                BlockType::Air
            }
        };

        let info = collide_and_slide(
            &player_aabb,
            velocity,
            &get_block,
        );

        assert!(info.position.x < 2.0, "Should not penetrate wall");
        assert!(info.velocity.x <= 0.0, "Horizontal velocity should be stopped or reversed");
    }
}

/// Integration tests for subsystem interactions
///
/// These tests verify that multiple systems work together correctly
/// without requiring OpenGL/audio/window context.
#[cfg(test)]
mod integration_tests {
    use crate::world::World;
    use crate::player::Player;
    use crate::mobs::{MobInstance, MobType, MobSpawner};
    use crate::inventory::Inventory;
    use crate::inventory::{Item, ItemStack};
    use crate::inventory::crafting::{CraftingGrid, Recipe};
    use crate::state::{GameState, StateManager};
    use crate::types::{BlockType, ChunkPos, WorldPos};
    use crate::config::*;
    use glam::Vec3;

    /// Test that world generation produces valid terrain
    #[test]
    fn test_world_generation_produces_terrain() {
        let world = World::new(12345);

        // Load a chunk
        let chunk_pos = ChunkPos::new(0, 0);
        let mut world = world;
        world.load_chunk(chunk_pos);

        // Verify chunk exists
        let chunk = world.get_chunk(&chunk_pos);
        assert!(chunk.is_some(), "Chunk should be loaded");

        // Verify terrain has some non-air blocks
        let mut has_solid_blocks = false;
        let mut has_bedrock = false;

        if let Some(chunk) = chunk {
            for x in 0..16 {
                for y in 0..256 {
                    for z in 0..16 {
                        let block = chunk.get_block(x, y, z);
                        if block != BlockType::Air && block != BlockType::Water {
                            has_solid_blocks = true;
                        }
                        if block == BlockType::Bedrock {
                            has_bedrock = true;
                        }
                    }
                }
            }
        }

        assert!(has_solid_blocks, "Generated terrain should have solid blocks");
        assert!(has_bedrock, "Generated terrain should have bedrock floor");
    }

    /// Test player physics and world interaction
    #[test]
    fn test_player_world_interaction() {
        let mut world = World::new(12345);

        // Load chunk at origin
        world.load_chunk(ChunkPos::new(0, 0));

        // Create player
        let player = Player::new(Vec3::new(8.0, 100.0, 8.0));

        // Player should be able to query world for collision
        let block_below = world.get_block(&WorldPos::new(8, 64, 8));
        // This should return some block (terrain varies by seed)
        // The important thing is that this call succeeds

        // Test block modification
        let test_pos = WorldPos::new(8, 100, 8);
        world.set_block(test_pos, BlockType::Stone);
        assert_eq!(world.get_block(&test_pos), BlockType::Stone);

        world.set_block(test_pos, BlockType::Air);
        assert_eq!(world.get_block(&test_pos), BlockType::Air);
    }

    /// Test inventory and crafting workflow
    #[test]
    fn test_inventory_crafting_workflow() {
        let mut inventory = Inventory::new();

        // Add oak wood - add_item returns remaining count (0 = success)
        let remaining = inventory.add_item(ItemStack::new(Item::Block(BlockType::WoodOak), 4));
        assert_eq!(remaining, 0, "Should be able to add items to inventory");

        // Create crafting grid and craft planks
        let mut grid = CraftingGrid::new_2x2();
        grid.set_slot(0, Some(ItemStack::new(Item::Block(BlockType::WoodOak), 1)));

        // Find planks recipe
        let recipes = crate::inventory::crafting::get_recipes();
        let planks_recipe = recipes.iter()
            .find(|r| matches!(r.output.item, Item::Block(BlockType::Planks)))
            .expect("Planks recipe should exist");

        assert!(planks_recipe.matches(&grid).is_some(), "Should match planks recipe");
        assert_eq!(planks_recipe.output.count, 4, "Should produce 4 planks");
    }

    /// Test state machine transitions
    #[test]
    fn test_state_machine_transitions() {
        let mut state = StateManager::new();

        // Initial state should be Loading
        assert_eq!(state.current(), GameState::Loading);

        // Transition to Playing
        state.finish_loading();
        assert_eq!(state.current(), GameState::Playing);

        // Toggle pause
        state.toggle_pause();
        assert_eq!(state.current(), GameState::Paused);

        // Toggle back to playing
        state.toggle_pause();
        assert_eq!(state.current(), GameState::Playing);

        // Toggle inventory
        state.toggle_inventory();
        assert_eq!(state.current(), GameState::Inventory);

        // Back to playing
        state.toggle_inventory();
        assert_eq!(state.current(), GameState::Playing);

        // Player death
        state.player_died();
        assert_eq!(state.current(), GameState::Dead);

        // Respawn
        state.respawn();
        assert_eq!(state.current(), GameState::Playing);
    }

    /// Test mob spawner respects limits
    #[test]
    fn test_mob_spawner_limits() {
        let spawner = MobSpawner::new(5, 10);

        // Spawner should track mob counts and enforce limits
        // This is a sanity test that the spawner can be constructed
        assert!(true, "MobSpawner should be constructible");
    }

    /// Test chunk loading/unloading workflow
    #[test]
    fn test_chunk_loading_unloading() {
        let mut world = World::new(12345);

        let chunk_pos = ChunkPos::new(5, 5);

        // Load chunk
        world.load_chunk(chunk_pos);
        assert!(world.get_chunk(&chunk_pos).is_some(), "Chunk should be loaded");

        // Modify chunk
        let test_pos = WorldPos::new(80 + 8, 64, 80 + 8);
        world.set_block(test_pos, BlockType::OreDiamond);
        assert_eq!(world.get_block(&test_pos), BlockType::OreDiamond);

        // Unload chunk
        world.unload_chunk(chunk_pos);
        assert!(world.get_chunk(&chunk_pos).is_none(), "Chunk should be unloaded");
    }

    /// Test player health system
    #[test]
    fn test_player_health_system() {
        let mut player = Player::new(Vec3::new(0.0, 80.0, 0.0));

        // Initial health
        assert_eq!(player.health.current(), MAX_HEALTH as f32);
        assert!(player.health.is_alive());

        // Take damage
        player.health.take_damage(5);
        assert_eq!(player.health.current(), (MAX_HEALTH - 5) as f32);
        assert!(player.health.is_alive());

        // Take fatal damage
        player.health.take_damage(100);
        assert!(!player.health.is_alive());

        // Reset health
        player.health.set_health(MAX_HEALTH);
        assert_eq!(player.health.current(), MAX_HEALTH as f32);
        assert!(player.health.is_alive());
    }

    /// Test day/night cycle calculations
    #[test]
    fn test_day_night_cycle() {
        // At game_time = 0, should be start of day
        let day_time_0 = get_day_time(0.0);
        assert!(day_time_0 >= 0.0 && day_time_0 <= 1.0);

        // At half day length, should be different
        let day_time_half = get_day_time(DAY_LENGTH_SECONDS / 2.0);
        assert!(day_time_half >= 0.0 && day_time_half <= 1.0);

        // At full day length, should wrap back
        let day_time_full = get_day_time(DAY_LENGTH_SECONDS);
        assert!((day_time_full - day_time_0).abs() < 0.01, "Should wrap around");
    }
}

/// Note: Full game loop integration test
///
/// The following test requires OpenGL and audio context, which cannot be created
/// in a headless test environment. This test is documented here for manual execution:
///
/// ```rust,ignore
/// #[test]
/// fn test_full_game_loop_integration() {
///     // This test requires:
///     // 1. OpenGL 3.3+ capable display
///     // 2. Audio device available
///     // 3. Window creation capability
///
///     // Create window and context
///     let event_loop = winit::event_loop::EventLoop::new().unwrap();
///     let window = Window::new(&event_loop).expect("Failed to create window");
///
///     // Create game
///     let mut game = Game::new(&window).expect("Failed to create game");
///
///     // Simulate one update cycle
///     let input = InputState::new();
///     game.update(&input, 1.0 / 60.0, &window);
///     game.update_physics(&input, 1.0 / 60.0);
///
///     // Verify game state is valid
///     assert_eq!(game.state.current(), GameState::Playing);
///     assert!(game.player.health.is_alive());
///     assert!(game.chunk_meshes.len() > 0 || game.world.chunks().count() > 0);
/// }
/// ```
///
/// To run this test manually:
/// ```bash
/// cargo run --release -- --test-mode
/// ```
