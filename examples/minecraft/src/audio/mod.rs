// Audio system for voxel game
// Uses rodio for audio playback with placeholder sounds (sine wave beeps)
// Ready to drop in real audio files by replacing the source generation

mod sounds;
mod music;

pub use sounds::{SoundEffect, SoundSettings};
pub use music::{MusicManager, MusicTrack, TimeOfDay};

use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::sync::Arc;
use glam::Vec3;

/// Main audio manager handling both sound effects and music
pub struct AudioManager {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    music_manager: MusicManager,
    master_volume: f32,
    sfx_volume: f32,
    music_volume: f32,
}

impl AudioManager {
    /// Create a new audio manager
    ///
    /// This initializes the rodio output stream and music system.
    /// In a real implementation, this would also load audio files into memory.
    pub fn new() -> Result<Self, String> {
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| format!("Failed to initialize audio output: {}", e))?;

        println!("[AUDIO] Audio system initialized (placeholder mode)");
        println!("[AUDIO] Using sine wave beeps as placeholder sounds");
        println!("[AUDIO] Replace source generation with file loading for real audio");

        let music_manager = MusicManager::new(stream_handle.clone());

        Ok(Self {
            _stream,
            stream_handle,
            music_manager,
            master_volume: 0.5,
            sfx_volume: 0.8,
            music_volume: 0.6,
        })
    }

    /// Play a sound effect with optional 3D positioning
    ///
    /// If position is Some, the sound will be attenuated based on distance from listener.
    /// Listener is assumed to be at the player's position (passed separately).
    pub fn play_sound(&self, effect: SoundEffect, settings: SoundSettings) {
        // Calculate volume based on distance if 3D positioning is used
        let volume = self.calculate_volume(settings);

        // Log the sound that would play (placeholder behavior)
        match settings.position {
            Some(pos) => {
                println!(
                    "[AUDIO] Playing {:?} at position ({:.1}, {:.1}, {:.1}) - volume: {:.2}",
                    effect, pos.x, pos.y, pos.z, volume
                );
            }
            None => {
                println!("[AUDIO] Playing {:?} (2D) - volume: {:.2}", effect, volume);
            }
        }

        // Generate and play placeholder sound
        if volume > 0.01 {
            let source = effect.generate_placeholder_source(settings.pitch);
            let sink = Sink::try_new(&self.stream_handle).unwrap();
            sink.set_volume(volume);
            sink.append(source);
            sink.detach(); // Let it play and clean up automatically
        }
    }

    /// Quick helper to play a sound without position (2D sound)
    pub fn play_sound_2d(&self, effect: SoundEffect) {
        self.play_sound(effect, SoundSettings::default());
    }

    /// Quick helper to play a sound at a 3D position
    pub fn play_sound_3d(&self, effect: SoundEffect, position: Vec3, listener_position: Vec3) {
        let settings = SoundSettings {
            position: Some(position),
            listener_position: Some(listener_position),
            ..Default::default()
        };
        self.play_sound(effect, settings);
    }

    /// Update music based on time of day
    pub fn update_music(&mut self, time_of_day: TimeOfDay) {
        self.music_manager.update(time_of_day, self.music_volume * self.master_volume);
    }

    /// Set master volume (0.0 to 1.0)
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Set sound effects volume (0.0 to 1.0)
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
    }

    /// Set music volume (0.0 to 1.0)
    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
    }

    /// Calculate final volume based on position and settings
    fn calculate_volume(&self, settings: SoundSettings) -> f32 {
        let mut volume = self.master_volume * self.sfx_volume * settings.volume;

        // Apply 3D attenuation if position is specified
        if let (Some(sound_pos), Some(listener_pos)) =
            (settings.position, settings.listener_position)
        {
            let distance = sound_pos.distance(listener_pos);

            // Inverse square law with minimum distance to avoid extreme volumes
            let min_distance = 1.0;
            let attenuation = 1.0 / (1.0 + (distance / min_distance).powi(2));

            volume *= attenuation;

            // Apply max distance cutoff
            if distance > settings.max_distance {
                volume = 0.0;
            }
        }

        volume.clamp(0.0, 1.0)
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            eprintln!("[AUDIO] Failed to create audio manager: {}", e);
            eprintln!("[AUDIO] Audio will be disabled");
            panic!("Audio initialization failed");
        })
    }
}

// REAL IMPLEMENTATION NOTES:
// ========================
//
// To use real audio files instead of placeholder beeps:
//
// 1. Create an assets/sounds/ directory with .ogg or .wav files:
//    assets/sounds/block_break.ogg
//    assets/sounds/block_place.ogg
//    assets/sounds/footstep.ogg
//    etc.
//
// 2. In AudioManager::new(), load all sound files into memory:
//    use rodio::Decoder;
//    use std::io::BufReader;
//
//    let file = std::fs::File::open("assets/sounds/block_break.ogg")?;
//    let source = Decoder::new(BufReader::new(file))?;
//
// 3. Store decoded sources in a HashMap<SoundEffect, Arc<Vec<u8>>>
//
// 4. In play_sound(), use the loaded source instead of generate_placeholder_source()
//
// 5. For music, use the same approach but with longer tracks
//
// The architecture is designed to make this transition seamless - just swap
// the source generation for file loading and everything else stays the same.
