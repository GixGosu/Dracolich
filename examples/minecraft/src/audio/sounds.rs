// Sound effects system
// Defines all game sound effects and methods to play them with 3D positioning

use rodio::source::{SineWave, Source};
use glam::Vec3;
use std::time::Duration;

/// All sound effects in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundEffect {
    /// Block breaking sound (stone, wood, dirt variants would be separate in real impl)
    BlockBreak,

    /// Block placement sound
    BlockPlace,

    /// Footstep sound (would vary by block type walked on)
    Footstep,

    /// Player jump
    Jump,

    /// Player takes damage
    Hurt,

    /// Player takes damage (alias for Hurt)
    PlayerHurt,

    /// Mob takes damage
    MobHurt,

    /// Mob death sound
    MobDeath,

    /// Ambient cave sounds (drips, echoes)
    AmbientCave,

    /// Ambient surface sounds (birds, wind)
    AmbientSurface,

    /// Item pickup
    ItemPickup,

    /// Item drop
    ItemDrop,

    /// UI click
    UiClick,

    /// Crafting success
    Craft,

    /// Tool break (durability depleted)
    ToolBreak,
}

impl SoundEffect {
    /// Generate a placeholder sine wave beep for this sound effect
    ///
    /// Each sound has a unique frequency and duration to be distinguishable.
    /// In a real implementation, this would load from an audio file instead.
    ///
    /// # Arguments
    /// * `pitch` - Pitch multiplier (1.0 = normal pitch)
    pub fn generate_placeholder_source(self, pitch: f32) -> impl Source<Item = f32> {
        let (base_freq, duration_ms) = match self {
            SoundEffect::BlockBreak => (220.0, 150),     // A3, short
            SoundEffect::BlockPlace => (330.0, 100),     // E4, very short
            SoundEffect::Footstep => (165.0, 80),        // E3, very short
            SoundEffect::Jump => (440.0, 120),           // A4, short
            SoundEffect::Hurt => (180.0, 300),           // F#3, medium (pain sound lower)
            SoundEffect::PlayerHurt => (180.0, 300),     // Same as Hurt
            SoundEffect::MobHurt => (200.0, 250),        // Similar to player hurt
            SoundEffect::MobDeath => (150.0, 400),       // Low, longer
            SoundEffect::AmbientCave => (110.0, 500),    // Very low, longer
            SoundEffect::AmbientSurface => (660.0, 300), // High, airy
            SoundEffect::ItemPickup => (880.0, 80),      // High ping
            SoundEffect::ItemDrop => (550.0, 100),       // Mid-high
            SoundEffect::UiClick => (800.0, 50),         // Quick high click
            SoundEffect::Craft => (660.0, 200),          // Success chime
            SoundEffect::ToolBreak => (165.0, 350),      // Low crack
        };

        let freq = base_freq * pitch;

        SineWave::new(freq)
            .take_duration(Duration::from_millis(duration_ms))
            .amplify(0.2) // Keep volume low for placeholder beeps
    }

    /// Get a descriptive name for this sound (for logging)
    pub fn name(self) -> &'static str {
        match self {
            SoundEffect::BlockBreak => "block_break",
            SoundEffect::BlockPlace => "block_place",
            SoundEffect::Footstep => "footstep",
            SoundEffect::Jump => "jump",
            SoundEffect::Hurt => "hurt",
            SoundEffect::PlayerHurt => "player_hurt",
            SoundEffect::MobHurt => "mob_hurt",
            SoundEffect::MobDeath => "mob_death",
            SoundEffect::AmbientCave => "ambient_cave",
            SoundEffect::AmbientSurface => "ambient_surface",
            SoundEffect::ItemPickup => "item_pickup",
            SoundEffect::ItemDrop => "item_drop",
            SoundEffect::UiClick => "ui_click",
            SoundEffect::Craft => "craft",
            SoundEffect::ToolBreak => "tool_break",
        }
    }

    /// Get the audio file path for this sound (for real implementation)
    ///
    /// This is what you'd use when loading real audio files:
    /// ```ignore
    /// let path = sound_effect.file_path();
    /// let file = std::fs::File::open(path)?;
    /// let source = Decoder::new(BufReader::new(file))?;
    /// ```
    pub fn file_path(self) -> String {
        format!("assets/sounds/{}.ogg", self.name())
    }
}

/// Settings for playing a sound effect
#[derive(Debug, Clone, Copy)]
pub struct SoundSettings {
    /// Volume multiplier (0.0 to 1.0)
    pub volume: f32,

    /// Pitch multiplier (1.0 = normal pitch)
    pub pitch: f32,

    /// 3D position of the sound source (None = 2D sound)
    pub position: Option<Vec3>,

    /// Position of the listener (required if position is Some)
    pub listener_position: Option<Vec3>,

    /// Maximum distance at which sound can be heard
    pub max_distance: f32,
}

impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            volume: 1.0,
            pitch: 1.0,
            position: None,
            listener_position: None,
            max_distance: 64.0, // 4 chunks
        }
    }
}

impl SoundSettings {
    /// Create settings for a 2D sound (no positioning)
    pub fn new_2d() -> Self {
        Self::default()
    }

    /// Create settings for a 3D sound at a position
    pub fn new_3d(position: Vec3, listener_position: Vec3) -> Self {
        Self {
            position: Some(position),
            listener_position: Some(listener_position),
            ..Default::default()
        }
    }

    /// Set volume (builder pattern)
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set pitch (builder pattern)
    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch.max(0.1); // Prevent invalid pitch
        self
    }

    /// Set max distance (builder pattern)
    pub fn with_max_distance(mut self, distance: f32) -> Self {
        self.max_distance = distance.max(1.0);
        self
    }
}

// REAL IMPLEMENTATION GUIDE:
// ========================
//
// When implementing real audio files:
//
// 1. Record or source audio files for each SoundEffect variant
//    - Block break: stone/wood/dirt cracking sounds
//    - Footsteps: vary by surface (stone, dirt, wood, grass)
//    - Hurt sounds: various pain grunts
//    - Ambient: loops of nature sounds, cave ambience
//
// 2. Convert to .ogg format for best compression/quality ratio
//    - Use Audacity or ffmpeg to convert
//    - Keep file sizes reasonable (< 100KB for most SFX)
//
// 3. Organize in assets/sounds/ directory
//
// 4. Consider variations:
//    - Some sounds (like footsteps, block_break) should have multiple
//      variants to avoid repetition
//    - Randomly select from variants when playing
//
// 5. Balance levels:
//    - Normalize all sounds to same perceived loudness
//    - Use Audacity's "Normalize" effect
//
// 6. Add randomization:
//    - Slight pitch variation (0.9 - 1.1x) makes sounds less repetitive
//    - Already supported by SoundSettings::with_pitch()
//
// Example real usage:
// ```
// audio.play_sound(
//     SoundEffect::BlockBreak,
//     SoundSettings::new_3d(block_pos, player_pos)
//         .with_pitch(0.95 + rand::random::<f32>() * 0.1) // 0.95-1.05 range
// );
// ```
