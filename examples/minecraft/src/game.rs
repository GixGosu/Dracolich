//! Main game struct that holds all subsystems
//!
//! This module orchestrates all game systems and provides a unified
//! interface for the main loop to update and render the game.

use crate::audio::{AudioManager, SoundEffect, SoundSettings, TimeOfDay};
use crate::config::*;
use crate::inventory::Inventory;
use crate::mobs::{MobInstance, MobSpawner, MobType, Mob, ZombieAction};
use crate::physics::collision::collide_and_slide;
use crate::physics::raycast::raycast;
use crate::player::{Player, update_camera, update_movement, update_interaction};
use crate::renderer::{Renderer, Camera, ChunkMesh};
use crate::state::{GameState, StateManager, StateInputRequirements};
use crate::types::{BlockType, ChunkPos, WorldPos};
use crate::ui::{UIRenderer, HotbarState, HotbarItem, ItemType, PauseMenuState, InventoryState, InventoryItem, DebugInfo};
use crate::world::{World, ChunkManager, mesh_chunk};
use crate::input::InputState;
use crate::window::Window;
use glam::{Vec2, Vec3, Vec3Swizzles};
use std::collections::HashMap;

/// Main game structure holding all subsystems
pub struct Game {
    // Core systems
    pub world: World,
    pub player: Player,
    pub renderer: Renderer,
    pub ui: UIRenderer,
    pub audio: AudioManager,
    pub camera: Camera,

    // Game state
    pub state: StateManager,

    // Chunk management
    pub chunk_manager: ChunkManager,
    chunk_meshes: HashMap<ChunkPos, ChunkMesh>,

    // Mob system
    pub mobs: Vec<MobInstance>,
    mob_spawner: MobSpawner,
    last_spawn_check: f32,

    // Player inventory (separate from hotbar)
    pub inventory: Inventory,

    // Timing
    game_time: f32,
    last_footstep: f32,

    // UI state
    pub show_debug: bool,

    // Input consumption tracking (prevents multi-tick processing)
    jump_consumed_this_frame: bool,
}

impl Game {
    /// Initialize the game with all subsystems
    pub fn new(window: &Window) -> Result<Self, String> {
        println!("Initializing game systems...");

        // Initialize renderer
        let renderer = Renderer::new(window)
            .map_err(|e| format!("Failed to create renderer: {}", e))?;
        println!("✓ Renderer initialized");

        // Initialize world and chunk manager
        let mut world = World::new(WORLD_SEED);
        let chunk_manager = ChunkManager::new(RENDER_DISTANCE as i32);
        println!("✓ World system initialized (seed: {})", WORLD_SEED);

        // Create player at spawn height
        let spawn_pos = Vec3::new(0.0, 80.0, 0.0);
        let player = Player::new(spawn_pos);
        println!("✓ Player spawned at {:?}", spawn_pos);

        // Initialize camera
        let aspect_ratio = window.aspect_ratio();
        let camera = Camera::new(
            spawn_pos + Vec3::new(0.0, PLAYER_EYE_HEIGHT, 0.0),
            0.0, // yaw
            0.0, // pitch
        );
        println!("✓ Camera initialized");

        // Initialize UI
        let (width, height) = window.dimensions();
        let ui = UIRenderer::new(width, height)
            .map_err(|e| format!("Failed to create UI: {}", e))?;
        println!("✓ UI system initialized");

        // Initialize audio
        let audio = AudioManager::new()
            .unwrap_or_else(|e| {
                eprintln!("Warning: Audio initialization failed: {}", e);
                eprintln!("Continuing without audio");
                // This will fail, but we handle it gracefully
                panic!("Audio required for full game experience");
            });
        println!("✓ Audio system initialized");

        // Initialize mob spawner
        let mob_spawner = MobSpawner::new(
            MAX_PASSIVE_MOBS,
            MAX_HOSTILE_MOBS,
        );
        println!("✓ Mob spawner initialized");

        // Create player inventory
        let mut inventory = Inventory::new();

        // Give starting items for testing
        use crate::inventory::{Item, ItemStack};
        inventory.add_item(ItemStack::new(Item::Block(BlockType::Dirt), 64));
        inventory.add_item(ItemStack::new(Item::Block(BlockType::Stone), 64));
        inventory.add_item(ItemStack::new(Item::Block(BlockType::WoodOak), 64));
        inventory.add_item(ItemStack::new(Item::Block(BlockType::Planks), 64));
        println!("✓ Inventory initialized with starting items");

        // Create state manager
        let state = StateManager::new();

        // Pre-load chunks around spawn point to prevent falling through world
        println!("Loading initial chunks around spawn...");
        let spawn_chunk = ChunkPos::from_world_coords(spawn_pos.x as i32, spawn_pos.z as i32);
        for dx in -2..=2 {
            for dz in -2..=2 {
                let chunk_pos = ChunkPos::new(spawn_chunk.x + dx, spawn_chunk.z + dz);
                world.load_chunk(chunk_pos);
            }
        }
        println!("✓ Loaded {} initial chunks", world.loaded_chunk_count());

        Ok(Self {
            world,
            player,
            renderer,
            ui,
            audio,
            camera,
            state,
            chunk_manager,
            chunk_meshes: HashMap::new(),
            mobs: Vec::new(),
            mob_spawner,
            last_spawn_check: 0.0,
            inventory,
            game_time: 0.0,
            last_footstep: 0.0,
            show_debug: false,
            jump_consumed_this_frame: false,
        })
    }

    /// Update game state for one frame
    pub fn update(&mut self, input: &mut InputState, delta_time: f32, window: &Window) {
        // Reset jump consumption at start of new frame
        self.jump_consumed_this_frame = false;

        // Update game time
        self.game_time += delta_time;

        // Handle state transitions from input (also syncs cursor grab state)
        self.handle_state_transitions(input, window);

        // Get current state requirements
        let requirements = StateInputRequirements::for_state(self.state.current());

        // Only update gameplay systems if in Playing state
        if self.state.current() == GameState::Playing {
            // Update player camera from mouse
            if requirements.process_mouse_look {
                self.update_player_camera(input);
            }

            // Update chunks around player
            self.update_chunks();

            // Update mobs
            self.update_mobs(delta_time);

            // Update music based on time of day
            self.update_audio();
        }
    }

    /// Update physics at fixed timestep
    pub fn update_physics(&mut self, input: &InputState, dt: f32) {
        if self.state.current() != GameState::Playing {
            return;
        }

        // Update player movement and physics
        let requirements = StateInputRequirements::for_state(self.state.current());

        if requirements.process_movement {
            // Get movement input
            let movement_input = input.movement_input();
            let is_sprinting = input.is_sprint();

            // Only allow jump on first physics tick of frame to prevent
            // multi-tick consumption when multiple physics updates occur
            let jump_pressed = input.is_jump_just_pressed() && !self.jump_consumed_this_frame;
            if jump_pressed {
                self.jump_consumed_this_frame = true;
            }

            // Update player movement with collision detection
            update_movement(
                &mut self.player,
                movement_input,
                is_sprinting,
                jump_pressed,
                dt,
                |pos| self.world.get_block(pos),
            );

            // Play footstep sounds
            if self.player.on_ground && self.player.velocity.xz().length() > 0.1 {
                let time_since_footstep = self.game_time - self.last_footstep;
                let footstep_interval = if is_sprinting { 0.25 } else { 0.35 };

                if time_since_footstep > footstep_interval {
                    self.audio.play_sound(
                        SoundEffect::Footstep,
                        SoundSettings::new_2d().with_volume(0.3),
                    );
                    self.last_footstep = self.game_time;
                }
            }
        }

        // Check for fall damage when landing
        if self.player.on_ground && self.player.fall_distance > FALL_DAMAGE_THRESHOLD {
            let damage = ((self.player.fall_distance - FALL_DAMAGE_THRESHOLD) * FALL_DAMAGE_MULTIPLIER) as i32;
            if damage > 0 {
                self.player.health.take_damage(damage);
                self.audio.play_sound(SoundEffect::PlayerHurt, SoundSettings::new_2d());

                if !self.player.health.is_alive() {
                    self.handle_player_death();
                }
            }
        }

        // Update block interaction (breaking/placing)
        if requirements.process_actions {
            let is_attacking = input.is_attack();
            let is_placing = input.is_use_just_pressed();

            // Collect block changes to apply after update
            let mut block_changes: Vec<(crate::types::WorldPos, BlockType)> = Vec::new();
            let eye_position = self.player.eye_position();

            {
                let world = &self.world;
                update_interaction(
                    &mut self.player,
                    is_attacking,
                    is_placing,
                    dt,
                    |pos| world.get_block(pos),
                    |pos, block| {
                        block_changes.push((*pos, block));
                    },
                );
            }

            // Apply block changes and play sounds
            for (pos, block) in block_changes {
                self.world.set_block(pos, block);

                // Play appropriate sound
                if block == BlockType::Air {
                    self.audio.play_sound_3d(
                        SoundEffect::BlockBreak,
                        Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32),
                        eye_position,
                    );
                } else {
                    self.audio.play_sound_3d(
                        SoundEffect::BlockPlace,
                        Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32),
                        eye_position,
                    );
                }
            }
        }

        // Handle hotbar selection
        if let Some(slot) = input.hotbar_selection() {
            self.player.selected_slot = slot;
        }
    }

    /// Render the current frame
    pub fn render(&mut self) {
        // Update camera position to follow player
        self.camera.set_position(self.player.eye_position());
        self.camera.set_rotation(self.player.yaw, self.player.pitch);

        // Begin rendering
        self.renderer.begin_frame();

        // Render world chunks
        self.render_chunks();

        // Render mobs
        self.render_mobs();

        // Render targeted block highlight
        if let Some(target) = &self.player.interaction.targeted_block {
            self.renderer.render_block_highlight(&target.block_pos, &self.camera);
        }

        // End 3D rendering
        self.renderer.end_frame();

        // Render UI
        self.render_ui();
    }

    /// Handle window resize
    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
        self.ui.resize(width, height);
        self.camera.set_aspect_ratio(width as f32 / height as f32);
    }

    // === Private helper methods ===

    fn handle_state_transitions(&mut self, input: &mut InputState, window: &Window) {
        // Toggle pause
        if input.is_pause_just_pressed() {
            self.state.toggle_pause();
        }

        // Toggle inventory
        if input.is_inventory_just_pressed() {
            self.state.toggle_inventory();
        }

        // Toggle debug overlay
        if input.is_debug_just_pressed() {
            self.show_debug = !self.show_debug;
        }

        // Handle respawn when dead
        if self.state.current() == GameState::Dead {
            // TODO: Show death screen with respawn button
            // For now, respawn immediately
            self.respawn_player();
        }

        // Update cursor grab based on state - sync both window and input state
        let requirements = StateInputRequirements::for_state(self.state.current());
        window.set_cursor_grab(requirements.grab_cursor);
        input.set_cursor_grabbed(requirements.grab_cursor);

        // Finish loading if we're in loading state
        if self.state.current() == GameState::Loading {
            self.state.finish_loading();
        }
    }

    fn update_player_camera(&mut self, input: &InputState) {
        let mouse_delta = input.mouse_look_delta(MOUSE_SENSITIVITY);
        update_camera(&mut self.player, mouse_delta);
    }

    fn update_chunks(&mut self) {
        // Update player position in chunk manager
        self.chunk_manager.update_player_position(self.player.position);

        // Update chunk manager (queues chunks for loading/unloading)
        self.chunk_manager.update(&mut self.world);

        // Process generation queue - load chunks
        for _ in 0..MAX_CHUNKS_GENERATED_PER_FRAME {
            if let Some(chunk_pos) = self.chunk_manager.pop_generation_task() {
                self.world.load_chunk(chunk_pos);
            } else {
                break;
            }
        }

        // Mesh dirty chunks
        let dirty_chunks: Vec<ChunkPos> = self.world.chunks()
            .filter(|(_, chunk)| chunk.is_dirty())
            .map(|(pos, _)| *pos)
            .take(MAX_CHUNKS_MESHED_PER_FRAME)
            .collect();

        for chunk_pos in dirty_chunks {
            if let Some(_chunk) = self.world.get_chunk(&chunk_pos) {
                // Generate mesh
                if let Some(mesh_data) = mesh_chunk(&self.world, chunk_pos) {
                    // Upload to GPU
                    let mut chunk_mesh = ChunkMesh::new(chunk_pos.x, chunk_pos.z);
                    chunk_mesh.upload(&mesh_data.vertices);

                    self.chunk_meshes.insert(chunk_pos, chunk_mesh);

                    // Mark chunk as clean
                    if let Some(chunk) = self.world.get_chunk_mut(&chunk_pos) {
                        chunk.clear_dirty();
                    }
                }
            }
        }
    }

    fn update_mobs(&mut self, delta_time: f32) {
        // Collect data needed for mob updates
        let player_position = self.player.position;

        // Track damage to apply after iteration
        let mut damage_taken = 0;

        // Update existing mobs
        let world = &self.world;
        self.mobs.retain_mut(|mob| {
            let action = mob.update(delta_time, player_position, |pos| world.get_block(pos));

            // Handle zombie attacks
            if let Some(ZombieAction::Attack) = action {
                if let MobInstance::Zombie(_zombie) = mob {
                    damage_taken += 3;
                }
            }

            // Keep alive mobs
            mob.is_alive()
        });

        // Apply damage after iteration
        if damage_taken > 0 {
            self.player.health.take_damage(damage_taken);
            self.audio.play_sound(SoundEffect::PlayerHurt, SoundSettings::new_2d());

            if !self.player.health.is_alive() {
                self.handle_player_death();
            }
        }

        // Spawn new mobs periodically
        self.last_spawn_check += delta_time;
        if self.last_spawn_check > MOB_SPAWN_INTERVAL {
            self.last_spawn_check = 0.0;

            let is_night = crate::config::is_night(get_day_time(self.game_time));

            // Try to spawn mobs
            if let Some(spawn) = self.mob_spawner.try_spawn(
                self.player.position,
                is_night,
                |pos| {
                    // Simple light level check (just check if it's night for now)
                    if is_night { 0 } else { 15 }
                },
            ) {
                let mob = MobInstance::new(spawn.mob_type, spawn.position);
                self.mobs.push(mob);
            }
        }
    }

    fn update_audio(&mut self) {
        let day_time = get_day_time(self.game_time);
        let time_of_day = TimeOfDay::from_day_time(day_time);
        self.audio.update_music(time_of_day);
    }

    fn handle_player_death(&mut self) {
        self.state.player_died();
        self.audio.play_sound(SoundEffect::PlayerHurt, SoundSettings::new_2d());
    }

    fn respawn_player(&mut self) {
        // Reset player to spawn
        self.player.position = Vec3::new(0.0, 80.0, 0.0);
        self.player.velocity = Vec3::ZERO;
        self.player.health.set_health(MAX_HEALTH);
        self.player.fall_distance = 0.0;

        // Return to playing state
        self.state.respawn();
    }

    fn render_chunks(&mut self) {
        for (chunk_pos, chunk_mesh) in &self.chunk_meshes {
            // Frustum cull
            if self.camera.is_chunk_visible(chunk_pos) {
                // Calculate world position
                let world_x = chunk_pos.x * CHUNK_SIZE;
                let world_z = chunk_pos.z * CHUNK_SIZE;

                // Render chunk
                self.renderer.render_chunk(
                    chunk_mesh,
                    Vec3::new(world_x as f32, 0.0, world_z as f32),
                    &self.camera,
                );
            }
        }
    }

    fn render_mobs(&self) {
        for mob in &self.mobs {
            self.renderer.render_mob(
                &mob.get_render_vertices(),
                mob.get_color(),
                &self.camera,
            );
        }
    }

    fn render_ui(&mut self) {
        match self.state.current() {
            GameState::Playing => {
                // Render HUD
                let (screen_width, screen_height) = self.ui.screen_size();
                self.ui.crosshair.render(self.ui.projection(), screen_width, screen_height);

                let hotbar_state = self.build_hotbar_state();
                self.ui.hotbar.render(
                    self.ui.projection(),
                    &self.ui.text,
                    &hotbar_state,
                );

                self.ui.health.render(
                    self.ui.projection(),
                    &self.ui.text,
                    self.player.health.current() as u32,
                    self.player.health.max() as u32,
                );

                if self.show_debug {
                    self.render_debug_overlay();
                }
            }
            GameState::Paused => {
                // Still render HUD underneath
                let (screen_width, screen_height) = self.ui.screen_size();
                self.ui.crosshair.render(self.ui.projection(), screen_width, screen_height);

                let hotbar_state = self.build_hotbar_state();
                self.ui.hotbar.render(
                    self.ui.projection(),
                    &self.ui.text,
                    &hotbar_state,
                );

                // Render pause menu on top
                let pause_state = PauseMenuState { hovered_button: None };
                self.ui.pause_menu.render(self.ui.projection(), &self.ui.text, &pause_state);
            }
            GameState::Inventory => {
                // Render inventory screen
                let inventory_state = self.build_inventory_state();
                self.ui.inventory_screen.render(
                    self.ui.projection(),
                    &self.ui.text,
                    &inventory_state,
                );
            }
            GameState::Dead => {
                // TODO: Render death screen
            }
            GameState::Loading => {
                // TODO: Render loading screen
            }
        }
    }

    fn render_debug_overlay(&mut self) {
        let chunk_pos = ChunkPos::from_world_coords(
            self.player.position.x as i32,
            self.player.position.z as i32,
        );

        // Calculate facing direction from yaw and pitch
        let facing = Vec3::new(
            self.player.yaw.cos() * self.player.pitch.cos(),
            self.player.pitch.sin(),
            self.player.yaw.sin() * self.player.pitch.cos(),
        ).normalize();

        let debug_info = DebugInfo {
            position: self.player.position,
            chunk_coords: (chunk_pos.x, chunk_pos.z),
            facing,
            fps: 60, // TODO: Get actual FPS from game loop
            loaded_chunks: self.chunk_meshes.len(),
        };

        self.ui.debug.render(
            self.ui.projection(),
            &self.ui.text,
            &debug_info,
        );
    }

    /// Build HotbarState from player's hotbar for rendering
    fn build_hotbar_state(&self) -> HotbarState {
        let mut items: [Option<HotbarItem>; 9] = Default::default();

        for i in 0..9 {
            if let Some(block_type) = self.player.hotbar.get_block(i) {
                let count = self.player.hotbar.get_count(i);
                if count > 0 {
                    items[i] = Some(HotbarItem {
                        item_type: block_type.into(),
                        count,
                        durability: None, // Blocks don't have durability
                    });
                }
            }
        }

        HotbarState {
            selected_slot: self.player.selected_slot,
            items,
        }
    }

    /// Build InventoryState from player's inventory for rendering
    fn build_inventory_state(&self) -> InventoryState {
        let mut inventory_slots: [Option<InventoryItem>; 36] = std::array::from_fn(|_| None);

        for i in 0..36 {
            if let Some(item_stack) = &self.inventory.slots[i] {
                inventory_slots[i] = Some(InventoryItem {
                    item_type: item_stack.item.into(),
                    count: item_stack.count,
                    durability: item_stack.durability.map(|d| d as f32 / 100.0), // Convert to 0.0-1.0
                });
            }
        }

        InventoryState {
            inventory_slots,
            crafting_slots: Default::default(), // TODO: Implement crafting slots
            crafting_result: None,
            held_item: None,
        }
    }
}
