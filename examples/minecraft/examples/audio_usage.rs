// Example: How to integrate the audio system into the game
// This is a reference implementation showing best practices

use voxel_game::audio::{AudioManager, SoundEffect, SoundSettings, TimeOfDay};
use glam::Vec3;
use std::time::{Duration, Instant};

fn main() {
    println!("=== Audio System Integration Example ===\n");

    // 1. INITIALIZATION
    // Create audio manager at game startup
    let mut audio = AudioManager::new()
        .expect("Failed to initialize audio system");

    println!("✓ Audio system initialized\n");

    // 2. BASIC SOUND PLAYBACK
    println!("--- Playing 2D sounds (no position) ---");

    // UI sounds are typically 2D
    audio.play_sound_2d(SoundEffect::UiClick);
    std::thread::sleep(Duration::from_millis(200));

    audio.play_sound_2d(SoundEffect::Craft);
    std::thread::sleep(Duration::from_millis(300));

    // 3. 3D POSITIONAL SOUND
    println!("\n--- Playing 3D positioned sounds ---");

    let player_pos = Vec3::new(0.0, 64.0, 0.0);

    // Sound very close (full volume)
    let close_block = Vec3::new(1.0, 64.0, 0.0);
    println!("Block break 1 block away:");
    audio.play_sound_3d(SoundEffect::BlockBreak, close_block, player_pos);
    std::thread::sleep(Duration::from_millis(200));

    // Sound medium distance (attenuated)
    let medium_block = Vec3::new(10.0, 64.0, 0.0);
    println!("Block break 10 blocks away:");
    audio.play_sound_3d(SoundEffect::BlockBreak, medium_block, player_pos);
    std::thread::sleep(Duration::from_millis(200));

    // Sound far away (barely audible)
    let far_block = Vec3::new(40.0, 64.0, 0.0);
    println!("Block break 40 blocks away:");
    audio.play_sound_3d(SoundEffect::BlockBreak, far_block, player_pos);
    std::thread::sleep(Duration::from_millis(200));

    // Sound beyond max distance (silent)
    let too_far = Vec3::new(100.0, 64.0, 0.0);
    println!("Block break 100 blocks away (beyond max distance):");
    audio.play_sound_3d(SoundEffect::BlockBreak, too_far, player_pos);
    std::thread::sleep(Duration::from_millis(200));

    // 4. ADVANCED SOUND SETTINGS
    println!("\n--- Using SoundSettings for fine control ---");

    // Footstep with pitch variation (adds variety)
    for i in 0..5 {
        let pitch = 0.9 + (i as f32 * 0.05); // 0.9, 0.95, 1.0, 1.05, 1.1
        println!("Footstep with pitch {:.2}x", pitch);

        audio.play_sound(
            SoundEffect::Footstep,
            SoundSettings::new_3d(player_pos, player_pos)
                .with_pitch(pitch)
                .with_volume(0.7)
        );
        std::thread::sleep(Duration::from_millis(150));
    }

    // 5. MUSIC SYSTEM
    println!("\n--- Music system demonstration ---");

    // Simulate day/night cycle
    let times = [
        (TimeOfDay::Dawn, "Dawn"),
        (TimeOfDay::Day, "Day"),
        (TimeOfDay::Dusk, "Dusk"),
        (TimeOfDay::Night, "Night"),
    ];

    for (time, name) in &times {
        println!("Simulating {}...", name);
        audio.update_music(*time);
        std::thread::sleep(Duration::from_secs(2));
    }

    // 6. VOLUME CONTROLS
    println!("\n--- Volume control demonstration ---");

    println!("Master volume: 100%");
    audio.set_master_volume(1.0);
    audio.play_sound_2d(SoundEffect::Jump);
    std::thread::sleep(Duration::from_millis(300));

    println!("Master volume: 50%");
    audio.set_master_volume(0.5);
    audio.play_sound_2d(SoundEffect::Jump);
    std::thread::sleep(Duration::from_millis(300));

    println!("Master volume: 10%");
    audio.set_master_volume(0.1);
    audio.play_sound_2d(SoundEffect::Jump);
    std::thread::sleep(Duration::from_millis(300));

    // Reset to normal
    audio.set_master_volume(0.7);

    // 7. REALISTIC GAMEPLAY SCENARIOS
    println!("\n--- Realistic gameplay scenarios ---");

    // Mining sequence
    println!("\nMining 3 blocks:");
    for i in 0..3 {
        let block_pos = Vec3::new(i as f32, 64.0, 0.0);
        audio.play_sound_3d(SoundEffect::BlockBreak, block_pos, player_pos);
        std::thread::sleep(Duration::from_millis(500));
    }

    // Building sequence
    println!("\nPlacing 3 blocks:");
    for i in 0..3 {
        let block_pos = Vec3::new(i as f32, 64.0, 5.0);
        audio.play_sound_3d(SoundEffect::BlockPlace, block_pos, player_pos);
        std::thread::sleep(Duration::from_millis(300));
    }

    // Walking with varied footsteps
    println!("\nWalking (randomized footsteps):");
    let mut rng = rand::thread_rng();
    for _ in 0..10 {
        use rand::Rng;
        let pitch = 0.95 + rng.gen::<f32>() * 0.1; // 0.95 - 1.05
        audio.play_sound(
            SoundEffect::Footstep,
            SoundSettings::new_2d().with_pitch(pitch)
        );
        std::thread::sleep(Duration::from_millis(400));
    }

    // Combat scenario
    println!("\nCombat scenario:");
    audio.play_sound_2d(SoundEffect::MobHurt); // Hit mob
    std::thread::sleep(Duration::from_millis(300));
    audio.play_sound_2d(SoundEffect::Hurt); // Take damage
    std::thread::sleep(Duration::from_millis(500));
    audio.play_sound_2d(SoundEffect::MobHurt);
    std::thread::sleep(Duration::from_millis(300));
    audio.play_sound_2d(SoundEffect::MobDeath); // Kill mob
    std::thread::sleep(Duration::from_millis(300));
    audio.play_sound_2d(SoundEffect::ItemPickup); // Collect drop

    println!("\n✓ Audio demo complete!");
    println!("\n=== Integration Notes ===");
    println!("- All sounds are placeholder sine wave beeps");
    println!("- Frequencies/durations vary per sound type");
    println!("- 3D sounds attenuate with distance (inverse square law)");
    println!("- Music transitions automatically with time of day");
    println!("- Check console output for audio event logs");
}

// ========================================
// INTEGRATION PATTERNS FOR GAME SYSTEMS
// ========================================

/// Example: Integrating with block breaking
#[allow(dead_code)]
fn example_block_breaking(
    audio: &AudioManager,
    block_pos: Vec3,
    player_pos: Vec3,
    block_hardness: f32,
) {
    // Pitch variation based on block hardness
    let pitch = 1.0 - (block_hardness * 0.1);

    audio.play_sound(
        SoundEffect::BlockBreak,
        SoundSettings::new_3d(block_pos, player_pos)
            .with_pitch(pitch)
    );
}

/// Example: Integrating with player movement
#[allow(dead_code)]
fn example_player_footsteps(
    audio: &AudioManager,
    player_pos: Vec3,
    velocity: Vec3,
    on_ground: bool,
    last_footstep: &mut Instant,
) {
    if !on_ground {
        return; // No footsteps in air
    }

    let speed = velocity.length();
    if speed < 0.1 {
        return; // Not moving
    }

    // Footstep interval based on speed
    let interval = Duration::from_millis(
        (400.0 / speed.max(1.0)) as u64
    );

    if last_footstep.elapsed() > interval {
        // Random pitch variation
        use rand::Rng;
        let pitch = 0.95 + rand::thread_rng().gen::<f32>() * 0.1;

        audio.play_sound(
            SoundEffect::Footstep,
            SoundSettings::new_2d().with_pitch(pitch)
        );

        *last_footstep = Instant::now();
    }
}

/// Example: Integrating with day/night cycle
#[allow(dead_code)]
fn example_time_of_day_update(
    audio: &mut AudioManager,
    elapsed_time: f32,
    day_duration: f32,
) {
    // Calculate normalized day time (0.0 - 1.0)
    let day_time = (elapsed_time % day_duration) / day_duration;

    // Convert to TimeOfDay enum
    let time_of_day = TimeOfDay::from_day_time(day_time);

    // Update music
    audio.update_music(time_of_day);
}

/// Example: Integrating with inventory/crafting
#[allow(dead_code)]
fn example_crafting_success(audio: &AudioManager) {
    audio.play_sound_2d(SoundEffect::Craft);
}

/// Example: Mob AI integration
#[allow(dead_code)]
fn example_mob_damage(
    audio: &AudioManager,
    mob_pos: Vec3,
    player_pos: Vec3,
    is_death: bool,
) {
    let sound = if is_death {
        SoundEffect::MobDeath
    } else {
        SoundEffect::MobHurt
    };

    audio.play_sound_3d(sound, mob_pos, player_pos);
}

/// Example: Tool durability
#[allow(dead_code)]
fn example_tool_break(audio: &AudioManager, durability: u32, max_durability: u32) {
    if durability == 0 {
        // Tool broke
        audio.play_sound_2d(SoundEffect::ToolBreak);
    } else if durability < max_durability / 10 {
        // Tool nearly broken - quiet warning sound
        audio.play_sound(
            SoundEffect::UiClick,
            SoundSettings::new_2d()
                .with_volume(0.3)
                .with_pitch(0.8)
        );
    }
}
