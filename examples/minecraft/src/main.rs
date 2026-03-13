mod renderer;
mod world;
mod player;
mod physics;
mod ui;
mod mobs;
mod inventory;
mod audio;
mod types;
mod window;
mod input;
mod game_loop;
mod config;
mod state;
mod game;

use window::Window;
use input::InputState;
use game_loop::GameLoop;
use game::Game;
use state::GameState;
use winit::event::{Event, WindowEvent, DeviceEvent};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    println!("===========================================");
    println!("   VoxelCraft - Minecraft from Scratch");
    println!("===========================================");
    println!();

    // Create event loop
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    // Create window with OpenGL context
    let mut window = Window::new(&event_loop, "VoxelCraft - Minecraft Clone", 1280, 720);
    println!("✓ Window created: {}x{}", window.dimensions().0, window.dimensions().1);

    // Create input state tracker
    let mut input = InputState::new();

    // Create game loop with fixed timestep
    let mut game_loop = GameLoop::new();
    println!("✓ Game loop initialized (fixed timestep: {:.2}ms, {} ticks/sec)",
        game_loop.fixed_timestep_seconds() * 1000.0,
        1.0 / game_loop.fixed_timestep_seconds()
    );

    // Initialize game (this loads all subsystems)
    let mut game = match Game::new(&window) {
        Ok(game) => game,
        Err(e) => {
            eprintln!("Failed to initialize game: {}", e);
            eprintln!("This may be due to:");
            eprintln!("  - Missing OpenGL 3.3+ support");
            eprintln!("  - Missing audio device");
            eprintln!("  - Missing shader files");
            return;
        }
    };

    println!();
    println!("===========================================");
    println!("   Game Initialized Successfully!");
    println!("===========================================");
    println!();
    println!("Controls:");
    println!("  WASD - Move");
    println!("  Space - Jump");
    println!("  Shift - Sprint");
    println!("  Mouse - Look around");
    println!("  Left Click - Break block");
    println!("  Right Click - Place block");
    println!("  E - Inventory");
    println!("  ESC - Pause menu");
    println!("  F3 - Debug overlay");
    println!("  1-9 - Select hotbar slot");
    println!();
    println!("Click in the window to grab mouse cursor.");
    println!();

    let mut running = true;

    // Main game loop
    event_loop.run(move |event, window_target| {
        window_target.set_control_flow(ControlFlow::Poll);

        match event {
            // === Window Events ===
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    println!("Window close requested - shutting down...");
                    running = false;
                    window_target.exit();
                }

                WindowEvent::Resized(physical_size) => {
                    println!("Window resized to {}x{}", physical_size.width, physical_size.height);
                    window.resize(physical_size.width, physical_size.height);
                    game.resize(physical_size.width, physical_size.height);
                }

                WindowEvent::KeyboardInput { event, .. } => {
                    input.handle_keyboard(event);
                }

                WindowEvent::MouseInput { state, button, .. } => {
                    input.handle_mouse_button(button, state);
                }

                WindowEvent::CursorMoved { position, .. } => {
                    input.handle_cursor_moved((position.x, position.y));
                }

                WindowEvent::Focused(focused) => {
                    if !focused {
                        // Release cursor when window loses focus
                        window.set_cursor_grab(false);
                        input.set_cursor_grabbed(false);
                    }
                }

                WindowEvent::RedrawRequested => {
                    // Render the game
                    game.render();
                    // Present the frame
                    window.swap_buffers();
                }

                _ => {}
            },

            // === Device Events ===
            Event::DeviceEvent { event, .. } => {
                // Handle raw mouse motion for FPS camera
                // This is more reliable than position-based delta on Windows
                if let DeviceEvent::MouseMotion { delta } = event {
                    input.handle_raw_mouse_motion(delta);
                }
            }

            // === Main Update Loop ===
            Event::AboutToWait => {
                if !running {
                    return;
                }

                // Update game loop timing
                let (num_physics_ticks, render_delta) = game_loop.tick();

                // Run physics ticks at fixed timestep
                for _ in 0..num_physics_ticks {
                    game.update_physics(&input, game_loop.fixed_timestep_seconds());
                }

                // Update game state at variable frame rate
                game.update(&mut input, render_delta, &window);

                // Clear input state for next frame (after game has processed it)
                input.begin_frame();

                // Request next frame
                window.request_redraw();
            }

            Event::LoopExiting => {
                println!();
                println!("===========================================");
                println!("   Shutting down VoxelCraft");
                println!("===========================================");
                println!("Thanks for playing!");
            }

            _ => {}
        }
    }).expect("Event loop error");
}
