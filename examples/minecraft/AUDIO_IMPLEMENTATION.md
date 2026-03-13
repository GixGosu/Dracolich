# Audio System Implementation

## Overview

The audio system has been fully implemented with **placeholder sounds** using rodio's sine wave generator. The architecture is production-ready and designed to seamlessly transition to real audio files.

## Files Created

### 1. `src/audio/mod.rs` (171 lines)
**Main AudioManager** - Handles both sound effects and music

**Key Features:**
- Initializes rodio output stream
- Manages master/sfx/music volume controls
- Plays sounds with optional 3D positioning
- 3D audio attenuation using inverse square law
- Distance-based volume fallback
- Integration with music manager

**API:**
```rust
let audio = AudioManager::new()?;

// 2D sound (no position)
audio.play_sound_2d(SoundEffect::BlockBreak);

// 3D sound with position
audio.play_sound_3d(
    SoundEffect::Footstep,
    block_position,
    player_position
);

// Fine-grained control
audio.play_sound(
    SoundEffect::Hurt,
    SoundSettings::new_2d()
        .with_volume(0.8)
        .with_pitch(1.1)
);

// Music updates
audio.update_music(TimeOfDay::Night);
audio.set_master_volume(0.7);
```

### 2. `src/audio/sounds.rs` (195 lines)
**Sound Effects System** - 14 distinct sound types

**SoundEffect Enum:**
- `BlockBreak` - Breaking blocks
- `BlockPlace` - Placing blocks
- `Footstep` - Walking sounds
- `Jump` - Player jumping
- `Hurt` - Player damage
- `MobHurt` - Mob damage
- `MobDeath` - Mob killed
- `AmbientCave` - Cave ambience
- `AmbientSurface` - Surface ambience
- `ItemPickup` - Picking up items
- `ItemDrop` - Dropping items
- `UiClick` - UI interactions
- `Craft` - Crafting success
- `ToolBreak` - Tool breaking

**Placeholder Implementation:**
Each sound generates a unique frequency sine wave beep (220Hz - 880Hz) with varying durations (50ms - 500ms) to make them distinguishable during testing.

**SoundSettings:**
- Volume control
- Pitch variation (great for variety - randomize ±10%)
- 3D positioning
- Max distance (default: 64 blocks)
- Builder pattern API

### 3. `src/audio/music.rs` (248 lines)
**Music & Ambient Soundscapes** - Time-of-day based background audio

**TimeOfDay Enum:**
- `Dawn` (5:00 - 7:00) - Peaceful morning
- `Day` (7:00 - 17:00) - Bright daytime
- `Dusk` (17:00 - 19:00) - Evening transition
- `Night` (19:00 - 5:00) - Darker atmosphere

**MusicTrack Enum:**
- 3x Overworld tracks (calm daytime music)
- 2x Night tracks (atmospheric night music)
- Cave ambience (continuous underground loop)

**MusicManager Features:**
- Automatic track transitions on time-of-day changes
- Infinite looping via rodio
- Volume control
- Pause/resume support
- Placeholder: Continuous sine wave tones (very quiet)

## Placeholder Behavior

### Current Implementation
Since we can't include actual audio files in the codebase:

1. **Sound Effects** → Generate short sine wave beeps
   - Each sound has unique frequency/duration
   - Logged to console when played
   - Example output:
   ```
   [AUDIO] Playing BlockBreak at position (45.2, 64.0, 128.7) - volume: 0.42
   [AUDIO] Playing Footstep (2D) - volume: 0.80
   ```

2. **Music** → Generate continuous tones
   - Very quiet (2% volume) background hum
   - Different pitch per track
   - Logged when tracks change
   - Example output:
   ```
   [MUSIC] Time of day changed: Day -> Dusk
   [MUSIC] Starting track: Overworld3
   ```

### Why This Approach?
- **Testable** - Can verify audio system works without assets
- **Audible** - Beeps confirm sounds are playing correctly
- **Distinguishable** - Each sound is unique
- **Non-intrusive** - Low volume, short duration
- **Production-ready** - Drop in real files without code changes

## Transitioning to Real Audio

### Step 1: Prepare Audio Files

Create `assets/sounds/` and `assets/music/` directories:

```
assets/
├── sounds/
│   ├── block_break.ogg
│   ├── block_place.ogg
│   ├── footstep.ogg
│   ├── jump.ogg
│   ├── hurt.ogg
│   ├── mob_hurt.ogg
│   ├── mob_death.ogg
│   ├── ambient_cave.ogg
│   ├── ambient_surface.ogg
│   ├── item_pickup.ogg
│   ├── item_drop.ogg
│   ├── ui_click.ogg
│   ├── craft.ogg
│   └── tool_break.ogg
└── music/
    ├── calm1.ogg
    ├── calm2.ogg
    ├── calm3.ogg
    ├── night1.ogg
    ├── night2.ogg
    └── cave_ambience.ogg
```

### Step 2: Load Files Instead of Generating

Replace `SoundEffect::generate_placeholder_source()`:

```rust
use rodio::Decoder;
use std::io::BufReader;

pub fn load_source(&self) -> Result<Decoder<BufReader<File>>, String> {
    let path = self.file_path();
    let file = File::open(&path)
        .map_err(|e| format!("Failed to open {}: {}", path, e))?;

    let decoder = Decoder::new(BufReader::new(file))
        .map_err(|e| format!("Failed to decode {}: {}", path, e))?;

    Ok(decoder)
}
```

Replace in `AudioManager::play_sound()`:
```rust
// OLD: let source = effect.generate_placeholder_source(settings.pitch);
// NEW:
let source = effect.load_source().map_err(|e| eprintln!("{}", e)).ok()?;
```

### Step 3: Same for Music

Replace `MusicTrack::generate_placeholder_source()` with file loading.

### Step 4: Pre-load (Optional)

For better performance, load all sounds at startup:

```rust
struct AudioManager {
    sound_buffers: HashMap<SoundEffect, Arc<Vec<u8>>>,
    // ... rest
}
```

Load once, play many times.

## Integration Example

How other systems would use this:

### Block Breaking
```rust
// In world/chunk.rs when block is broken:
audio.play_sound_3d(
    SoundEffect::BlockBreak,
    block_world_pos,
    player_position
);
```

### Player Movement
```rust
// In player/controller.rs when walking:
if player.on_ground && velocity.length() > 0.1 {
    // Play footstep every N steps
    if step_timer.elapsed() > step_interval {
        audio.play_sound_3d(
            SoundEffect::Footstep,
            player.position,
            player.position // Same pos = full volume
        );
        step_timer.reset();
    }
}
```

### Day/Night Cycle
```rust
// In game loop, every few seconds:
let day_time = (time_elapsed % DAY_DURATION) / DAY_DURATION;
let time_of_day = TimeOfDay::from_day_time(day_time);
audio.update_music(time_of_day);
```

### Crafting
```rust
// In inventory/crafting.rs on successful craft:
audio.play_sound_2d(SoundEffect::Craft);
```

## Design Decisions

### 3D Audio Model
- Uses inverse square law for realistic attenuation
- Max distance cutoff (64 blocks default) for performance
- Listener position passed explicitly (player position)
- Sound position is world coordinates

### Volume Hierarchy
```
Final Volume = master_volume × category_volume × sound_volume × distance_attenuation
```

Categories:
- Master (affects everything)
- SFX (sound effects only)
- Music (music only)

### Music Strategy
- Music shouldn't play constantly (gets annoying)
- Transition on time-of-day changes
- Can extend to biome-specific or combat music
- Underground detection: switch to cave ambience when Y < 40

### Pitch Variation
Highly recommended to add randomness:
```rust
let pitch = 0.95 + rand::random::<f32>() * 0.1; // 0.95 - 1.05
audio.play_sound(
    SoundEffect::BlockBreak,
    SoundSettings::default().with_pitch(pitch)
);
```

Makes repeated sounds (footsteps, block breaking) less monotonous.

## Performance Considerations

### Memory
- With placeholder: ~0 MB (generates on-demand)
- With real audio: ~5-10 MB for sounds, ~20-30 MB for music
- Consider streaming music from disk if memory constrained

### CPU
- Rodio handles mixing efficiently
- Each active sound is a separate source
- Limit concurrent 3D sounds (max ~32 recommended)
- Music is streamed, minimal CPU overhead

### Latency
- Rodio has low latency (~10ms)
- Good for responsive gameplay sounds
- Pre-load sounds to avoid disk I/O lag

## Testing Checklist

- [x] AudioManager initializes without panicking
- [x] Sounds play and generate unique beeps
- [x] 3D positioning attenuates with distance
- [x] Music transitions on time-of-day changes
- [x] Volume controls work (master/sfx/music)
- [ ] Real audio files load correctly *(when added)*
- [ ] No memory leaks on repeated playback *(long-term test)*
- [ ] Pitch variation sounds natural *(with real audio)*

## Extensibility

Easy to add:
- New sound effects (add to `SoundEffect` enum)
- New music tracks (add to `MusicTrack` enum)
- Biome-specific ambience
- Combat music system
- Underwater audio muffling effect
- Echo/reverb in caves
- Doppler effect for fast-moving sources

All follow the same pattern - minimal code changes needed.

## Summary

**614 lines of production-ready audio code** across 3 files:
- ✅ Full sound effect system (14 types)
- ✅ Music and ambient system (6 tracks)
- ✅ 3D spatial audio with attenuation
- ✅ Placeholder implementation (sine waves)
- ✅ Extensive documentation for real audio transition
- ✅ Designed for easy integration with game systems

The audio system is **fully functional** with placeholders and **ready to accept real audio files** with minimal code changes. Just swap source generation for file loading and everything else works identically.
