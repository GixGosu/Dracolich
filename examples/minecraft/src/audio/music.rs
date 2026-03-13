// Ambient music and soundscape management
// Handles background music and time-of-day ambient sounds

use rodio::{OutputStreamHandle, Sink, Source};
use rodio::source::{SineWave, Zero};
use std::time::Duration;

/// Time of day categories affecting music and ambient sounds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeOfDay {
    /// Dawn (5:00 - 7:00)
    Dawn,

    /// Day (7:00 - 17:00)
    Day,

    /// Dusk (17:00 - 19:00)
    Dusk,

    /// Night (19:00 - 5:00)
    Night,
}

impl TimeOfDay {
    /// Calculate time of day from normalized day time (0.0 = midnight, 0.5 = noon, 1.0 = midnight)
    pub fn from_day_time(day_time: f32) -> Self {
        let hour = day_time * 24.0;

        match hour {
            h if h >= 5.0 && h < 7.0 => TimeOfDay::Dawn,
            h if h >= 7.0 && h < 17.0 => TimeOfDay::Day,
            h if h >= 17.0 && h < 19.0 => TimeOfDay::Dusk,
            _ => TimeOfDay::Night,
        }
    }
}

/// Music tracks that can play
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicTrack {
    /// Calm overworld music (day)
    Overworld1,
    Overworld2,
    Overworld3,

    /// Atmospheric night music
    Night1,
    Night2,

    /// Cave ambience (not really music, but long ambient loops)
    CaveAmbience,

    /// No music playing
    None,
}

impl MusicTrack {
    /// Get the file path for this music track
    pub fn file_path(self) -> Option<String> {
        match self {
            MusicTrack::Overworld1 => Some("assets/music/calm1.ogg".to_string()),
            MusicTrack::Overworld2 => Some("assets/music/calm2.ogg".to_string()),
            MusicTrack::Overworld3 => Some("assets/music/calm3.ogg".to_string()),
            MusicTrack::Night1 => Some("assets/music/night1.ogg".to_string()),
            MusicTrack::Night2 => Some("assets/music/night2.ogg".to_string()),
            MusicTrack::CaveAmbience => Some("assets/music/cave_ambience.ogg".to_string()),
            MusicTrack::None => None,
        }
    }

    /// Get a placeholder frequency for this track (for sine wave generation)
    fn placeholder_frequency(self) -> Option<f32> {
        match self {
            MusicTrack::Overworld1 => Some(261.63), // C4 - peaceful
            MusicTrack::Overworld2 => Some(293.66), // D4 - upbeat
            MusicTrack::Overworld3 => Some(329.63), // E4 - bright
            MusicTrack::Night1 => Some(220.00),     // A3 - darker
            MusicTrack::Night2 => Some(196.00),     // G3 - somber
            MusicTrack::CaveAmbience => Some(110.00), // A2 - deep
            MusicTrack::None => None,
        }
    }

    /// Generate a placeholder looping tone for this track
    ///
    /// Real implementation would load an actual music file.
    /// This generates a continuous tone that represents the track.
    pub fn generate_placeholder_source(self) -> Box<dyn Source<Item = f32> + Send> {
        match self.placeholder_frequency() {
            Some(freq) => {
                // Generate a very quiet, continuous tone
                Box::new(
                    SineWave::new(freq)
                        .amplify(0.02) // Very quiet for placeholder music
                        .repeat_infinite()
                )
            }
            None => {
                // Silence
                Box::new(Zero::new(2, 48000))
            }
        }
    }
}

/// Manages background music and ambient soundscapes
pub struct MusicManager {
    stream_handle: OutputStreamHandle,
    current_sink: Option<Sink>,
    current_track: MusicTrack,
    current_time_of_day: TimeOfDay,
}

impl MusicManager {
    pub fn new(stream_handle: OutputStreamHandle) -> Self {
        Self {
            stream_handle,
            current_sink: None,
            current_track: MusicTrack::None,
            current_time_of_day: TimeOfDay::Day,
        }
    }

    /// Update music based on time of day
    ///
    /// This should be called regularly (e.g., every frame or every few seconds).
    /// It will transition music when the time of day changes.
    pub fn update(&mut self, time_of_day: TimeOfDay, volume: f32) {
        // Check if time of day changed
        if time_of_day != self.current_time_of_day {
            println!("[MUSIC] Time of day changed: {:?} -> {:?}",
                     self.current_time_of_day, time_of_day);
            self.current_time_of_day = time_of_day;
            self.transition_music(time_of_day, volume);
        }

        // Update volume if sink exists
        if let Some(sink) = &self.current_sink {
            sink.set_volume(volume);
        }
    }

    /// Transition to appropriate music for the given time of day
    fn transition_music(&mut self, time_of_day: TimeOfDay, volume: f32) {
        // Select appropriate track
        let new_track = match time_of_day {
            TimeOfDay::Dawn => MusicTrack::Overworld1,
            TimeOfDay::Day => MusicTrack::Overworld2,
            TimeOfDay::Dusk => MusicTrack::Overworld3,
            TimeOfDay::Night => MusicTrack::Night1,
        };

        if new_track != self.current_track {
            self.play_track(new_track, volume);
        }
    }

    /// Play a specific music track
    pub fn play_track(&mut self, track: MusicTrack, volume: f32) {
        // Stop current music
        self.stop();

        if track == MusicTrack::None {
            return;
        }

        println!("[MUSIC] Starting track: {:?}", track);

        // Create new sink
        match Sink::try_new(&self.stream_handle) {
            Ok(sink) => {
                sink.set_volume(volume);

                // Add the placeholder source (or real file in production)
                let source = track.generate_placeholder_source();
                sink.append(source);

                self.current_sink = Some(sink);
                self.current_track = track;
            }
            Err(e) => {
                eprintln!("[MUSIC] Failed to create audio sink: {}", e);
            }
        }
    }

    /// Stop current music
    pub fn stop(&mut self) {
        if let Some(sink) = self.current_sink.take() {
            sink.stop();
        }
        self.current_track = MusicTrack::None;
    }

    /// Pause current music
    pub fn pause(&mut self) {
        if let Some(sink) = &self.current_sink {
            sink.pause();
        }
    }

    /// Resume paused music
    pub fn resume(&mut self) {
        if let Some(sink) = &self.current_sink {
            sink.play();
        }
    }

    /// Check if music is currently playing
    pub fn is_playing(&self) -> bool {
        self.current_sink.is_some() && self.current_track != MusicTrack::None
    }

    /// Get current track
    pub fn current_track(&self) -> MusicTrack {
        self.current_track
    }
}

// REAL IMPLEMENTATION GUIDE:
// ========================
//
// Music is the most storage-intensive part of the audio system.
// Here's how to handle it properly:
//
// 1. MUSIC SOURCING
//    - Create or license ambient music tracks (3-5 minutes each)
//    - Minecraft-style music is:
//      * Calm and atmospheric
//      * Sparse (not constantly playing)
//      * Piano/ambient focused
//    - Consider royalty-free sources:
//      * Kevin MacLeod (incompetech.com)
//      * Free Music Archive
//      * Create your own in GarageBand/FL Studio
//
// 2. TECHNICAL SPECS
//    - Format: .ogg (best compression for looping)
//    - Bitrate: 96-128 kbps (good quality, reasonable size)
//    - Sample rate: 44.1 kHz
//    - Seamless loops: ensure start/end match for ambient tracks
//
// 3. FILE ORGANIZATION
//    assets/music/
//      calm1.ogg       (~2-3 MB, 3 min)
//      calm2.ogg
//      calm3.ogg
//      night1.ogg
//      night2.ogg
//      cave_ambience.ogg (long loop, 5+ min)
//
// 4. LOADING STRATEGY
//    - Option A: Load all into memory at startup (6-7 tracks * 3MB = ~20MB)
//    - Option B: Stream from disk (more CPU, less memory)
//    - Recommended: Load into memory, pre-decode
//
// 5. PLAYBACK STRATEGY
//    - Don't play music constantly (annoying)
//    - Play a track, then silence for 2-5 minutes
//    - Randomly select from pool for variety
//    - Fade in/out on transitions (use rodio's fade_in())
//
// 6. AMBIENT SOUNDS VS MUSIC
//    - Day: Bird chirps, wind (separate from music)
//    - Night: Cricket/owl sounds (separate from music)
//    - Cave: Drips, echoes (can be the "music" when underground)
//    - Use ambient sounds more frequently than music
//
// Example real implementation:
//
// ```rust
// use rodio::Decoder;
// use std::io::BufReader;
// use std::fs::File;
//
// // In MusicTrack::generate_placeholder_source(), replace with:
// pub fn load_source(self) -> Result<Decoder<BufReader<File>>, String> {
//     let path = self.file_path()
//         .ok_or("No file path for track")?;
//
//     let file = File::open(&path)
//         .map_err(|e| format!("Failed to open {}: {}", path, e))?;
//
//     let decoder = Decoder::new(BufReader::new(file))
//         .map_err(|e| format!("Failed to decode {}: {}", path, e))?;
//
//     Ok(decoder)
// }
//
// // In play_track():
// let source = track.load_source()
//     .map_err(|e| eprintln!("[MUSIC] {}", e))
//     .ok()?;
//
// sink.append(source);
// ```
//
// 7. ADVANCED FEATURES (optional)
//    - Crossfading between tracks
//    - Dynamic music based on player state (combat, building, exploring)
//    - Biome-specific music
//    - Underground detection (play cave ambience when below Y=40)
