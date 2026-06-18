use std::time::{Duration, Instant};

pub struct Time {
    last_frame: Instant,
    last_fps_log: Instant,
    frame_count: u32,
    accumulated_time: Duration,
    delta_time: Duration,
}

impl Time {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            last_frame: now,
            last_fps_log: now,
            frame_count: 0,
            accumulated_time: Duration::ZERO,
            delta_time: Duration::ZERO,
        }
    }

    /// Updates the timer at the start of a frame.
    /// Calculates delta time and logs average FPS to console every second.
    pub fn update(&mut self) -> Duration {
        let now = Instant::now();
        self.delta_time = now.duration_since(self.last_frame);
        self.last_frame = now;

        self.frame_count += 1;
        self.accumulated_time += self.delta_time;

        // Log FPS every second
        let time_since_last_log = now.duration_since(self.last_fps_log);
        if time_since_last_log >= Duration::from_secs(1) {
            let fps = self.frame_count as f64 / time_since_last_log.as_secs_f64();
            println!("FPS: {:.2} (Frame Time: {:.2?})", fps, self.delta_time);
            self.frame_count = 0;
            self.last_fps_log = now;
        }

        self.delta_time
    }

    pub fn delta_time(&self) -> Duration {
        self.delta_time
    }

    pub fn delta_time_secs(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}
