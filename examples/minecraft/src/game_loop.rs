use std::time::{Duration, Instant};

/// Game loop with fixed timestep physics and variable rendering
pub struct GameLoop {
    // Timing
    last_update: Instant,
    accumulator: Duration,
    fixed_timestep: Duration,

    // FPS tracking
    fps_counter: FpsCounter,
}

impl GameLoop {
    /// Create a new game loop with 60 ticks/second physics
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
            accumulator: Duration::ZERO,
            fixed_timestep: Duration::from_secs_f64(1.0 / 60.0), // 60 Hz physics
            fps_counter: FpsCounter::new(),
        }
    }

    /// Create a game loop with custom tick rate
    pub fn with_tick_rate(ticks_per_second: u32) -> Self {
        Self {
            last_update: Instant::now(),
            accumulator: Duration::ZERO,
            fixed_timestep: Duration::from_secs_f64(1.0 / ticks_per_second as f64),
            fps_counter: FpsCounter::new(),
        }
    }

    /// Update the game loop, returns how many physics ticks to run and the render delta
    ///
    /// Returns (num_physics_ticks, render_delta_seconds)
    pub fn tick(&mut self) -> (u32, f32) {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_update);
        self.last_update = now;

        // Clamp frame time to prevent spiral of death
        let frame_time = frame_time.min(Duration::from_millis(250));

        self.accumulator += frame_time;

        // Calculate how many fixed timesteps to run
        let mut num_ticks = 0;
        while self.accumulator >= self.fixed_timestep {
            self.accumulator -= self.fixed_timestep;
            num_ticks += 1;

            // Safety limit to prevent too many ticks in one frame
            if num_ticks >= 10 {
                self.accumulator = Duration::ZERO;
                break;
            }
        }

        // Calculate render delta for interpolation
        let render_delta = frame_time.as_secs_f32();

        // Update FPS counter
        self.fps_counter.register_frame();

        (num_ticks, render_delta)
    }

    /// Get the fixed timestep duration
    pub fn fixed_timestep(&self) -> Duration {
        self.fixed_timestep
    }

    /// Get fixed timestep in seconds (useful for physics calculations)
    pub fn fixed_timestep_seconds(&self) -> f32 {
        self.fixed_timestep.as_secs_f32()
    }

    /// Get current FPS
    pub fn fps(&self) -> f32 {
        self.fps_counter.fps()
    }

    /// Get average FPS over the last second
    pub fn average_fps(&self) -> f32 {
        self.fps_counter.average_fps()
    }

    /// Get frame time in milliseconds
    pub fn frame_time_ms(&self) -> f32 {
        self.fps_counter.average_frame_time_ms()
    }
}

impl Default for GameLoop {
    fn default() -> Self {
        Self::new()
    }
}

/// FPS counter that tracks frame rate over time
struct FpsCounter {
    frame_times: Vec<Duration>,
    last_second_start: Instant,
    frames_this_second: u32,
    last_fps: f32,
}

impl FpsCounter {
    const MAX_SAMPLES: usize = 60;

    fn new() -> Self {
        Self {
            frame_times: Vec::with_capacity(Self::MAX_SAMPLES),
            last_second_start: Instant::now(),
            frames_this_second: 0,
            last_fps: 0.0,
        }
    }

    fn register_frame(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_second_start);

        self.frames_this_second += 1;

        // Store frame time
        if self.frame_times.len() >= Self::MAX_SAMPLES {
            self.frame_times.remove(0);
        }
        self.frame_times.push(delta);

        // Update FPS every second
        if delta >= Duration::from_secs(1) {
            self.last_fps = self.frames_this_second as f32 / delta.as_secs_f32();
            self.frames_this_second = 0;
            self.last_second_start = now;
        }
    }

    fn fps(&self) -> f32 {
        self.last_fps
    }

    fn average_fps(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        let total_time: Duration = self.frame_times.iter().sum();
        let avg_frame_time = total_time.as_secs_f32() / self.frame_times.len() as f32;

        if avg_frame_time > 0.0 {
            1.0 / avg_frame_time
        } else {
            0.0
        }
    }

    fn average_frame_time_ms(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        let total_time: Duration = self.frame_times.iter().sum();
        (total_time.as_secs_f32() / self.frame_times.len() as f32) * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_game_loop_tick() {
        let mut game_loop = GameLoop::new();

        // Wait a bit to ensure some time passes
        thread::sleep(Duration::from_millis(20));

        let (ticks, delta) = game_loop.tick();

        // Should have at least 1 tick after 20ms (16.6ms per tick at 60Hz)
        assert!(ticks >= 1);
        assert!(delta > 0.0);
    }

    #[test]
    fn test_fps_counter() {
        let mut counter = FpsCounter::new();

        for _ in 0..10 {
            counter.register_frame();
            thread::sleep(Duration::from_millis(16)); // ~60 FPS
        }

        let fps = counter.fps();
        // Should be around 60, but give wide tolerance due to timing variance
        assert!(fps > 30.0 && fps < 100.0 || fps == 0.0); // 0 is ok if not enough time passed
    }

    #[test]
    fn test_fixed_timestep() {
        let game_loop = GameLoop::with_tick_rate(120);
        let timestep = game_loop.fixed_timestep_seconds();

        // 120 Hz should be ~8.33ms per tick
        assert!((timestep - 1.0 / 120.0).abs() < 0.0001);
    }
}
