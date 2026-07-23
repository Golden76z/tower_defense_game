use std::time::{Duration, Instant};

/// Fixed-timestep game clock following the "Fix Your Timestep!" accumulator
/// pattern (<https://gafferongames.com/post/fix_your_timestep/>).
///
/// Each frame, [`Time::begin_frame`] measures the real elapsed time and adds it
/// to an accumulator. The game loop then drains the accumulator in fixed-size
/// steps (default 60 Hz) via [`Time::next_fixed_step`], so logic advances at a
/// constant, deterministic rate regardless of how fast rendering runs.
///
/// Rendering stays variable (as fast as the host can draw); [`Time::alpha`]
/// exposes the leftover fraction of a step for optional render interpolation.
pub struct Time {
    last_frame: Instant,
    last_fps_log: Instant,
    frame_count: u32,

    /// Duration of one logic step (e.g. 1/60 s).
    fixed_delta: Duration,
    /// Unconsumed real time waiting to be stepped.
    accumulator: Duration,
    /// Largest real frame delta we accept; longer frames are clamped so a
    /// stall (debugger, window drag, GC pause) can't trigger a runaway catch-up
    /// loop ("spiral of death"). Equal to `fixed_delta * MAX_STEPS_PER_FRAME`.
    max_frame_time: Duration,
    /// Real (clamped) duration of the most recent frame.
    frame_delta: Duration,
    /// Fixed steps already run during the current frame (reset each frame).
    steps_this_frame: u32,
    /// Multiplier for the simulation speed (e.g. 0.5x, 1.0x, 2.0x).
    pub speed_multiplier: f32,
}

/// Hard cap on fixed steps per frame. Combined with the frame-time clamp this
/// guarantees the update loop terminates.
const MAX_STEPS_PER_FRAME: u32 = 8;

impl Time {
    /// Creates a clock running logic at 60 Hz.
    pub fn new() -> Self {
        Self::with_fixed_hz(60.0)
    }

    /// Creates a clock running logic at `hz` updates per second.
    pub fn with_fixed_hz(hz: f64) -> Self {
        let now = Instant::now();
        let fixed_delta = Duration::from_secs_f64(1.0 / hz);
        Self {
            last_frame: now,
            last_fps_log: now,
            frame_count: 0,
            fixed_delta,
            accumulator: Duration::ZERO,
            max_frame_time: fixed_delta * MAX_STEPS_PER_FRAME,
            frame_delta: Duration::ZERO,
            steps_this_frame: 0,
            speed_multiplier: 1.0,
        }
    }

    /// Call once at the start of each frame. Measures the real elapsed time,
    /// folds it into the accumulator, logs FPS once per second, and returns the
    /// (clamped) frame delta.
    pub fn begin_frame(&mut self) -> Duration {
        let now = Instant::now();
        let raw_delta = now.duration_since(self.last_frame);
        self.last_frame = now;

        self.frame_count += 1;
        let since_log = now.duration_since(self.last_fps_log);
        if since_log >= Duration::from_secs(1) {
            let fps = self.frame_count as f64 / since_log.as_secs_f64();
            println!("FPS: {:.2} (Frame Time: {:.2?})", fps, raw_delta);
            self.frame_count = 0;
            self.last_fps_log = now;
        }

        self.advance(raw_delta)
    }

    /// Folds a measured frame duration into the accumulator (clamping it to
    /// `max_frame_time`) and resets the per-frame step counter. Returns the
    /// clamped delta.
    ///
    /// Exposed separately from [`Time::begin_frame`] so the loop can be driven
    /// deterministically in tests without touching the real clock.
    pub fn advance(&mut self, raw_delta: Duration) -> Duration {
        let clamped = raw_delta.min(self.max_frame_time);
        self.frame_delta = clamped;
        self.accumulator += clamped.mul_f32(self.speed_multiplier);
        self.steps_this_frame = 0;
        clamped
    }

    /// Drains one fixed step from the accumulator if one is available and the
    /// per-frame cap hasn't been hit.
    ///
    /// Drive the logic loop with:
    /// ```ignore
    /// time.begin_frame();
    /// while time.next_fixed_step() {
    ///     world.update(time.fixed_delta_secs());
    /// }
    /// ```
    /// When the cap is reached the remaining backlog is dropped, so a long
    /// hitch makes the game momentarily run slow rather than spiral.
    pub fn next_fixed_step(&mut self) -> bool {
        if self.steps_this_frame >= MAX_STEPS_PER_FRAME {
            // Cap reached: discard the backlog to avoid the spiral of death.
            self.accumulator = Duration::ZERO;
            return false;
        }
        if self.accumulator >= self.fixed_delta {
            self.accumulator -= self.fixed_delta;
            self.steps_this_frame += 1;
            true
        } else {
            false
        }
    }

    /// The fixed logic step length.
    pub fn fixed_delta(&self) -> Duration {
        self.fixed_delta
    }

    /// The fixed logic step length in seconds (pass this to logic updates).
    pub fn fixed_delta_secs(&self) -> f32 {
        self.fixed_delta.as_secs_f32()
    }

    /// The real (clamped) duration of the most recent frame.
    pub fn frame_delta(&self) -> Duration {
        self.frame_delta
    }

    /// The real (clamped) duration of the most recent frame, in seconds.
    pub fn frame_delta_secs(&self) -> f32 {
        self.frame_delta.as_secs_f32()
    }

    /// Fraction of a fixed step currently sitting unconsumed in the accumulator,
    /// in `[0, 1)`. Use it to interpolate rendered state between logic steps.
    pub fn alpha(&self) -> f32 {
        self.accumulator.as_secs_f32() / self.fixed_delta.as_secs_f32()
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs `frames` of `frame_delta` each through the accumulator and returns
    /// the total number of fixed steps taken.
    fn count_steps(time: &mut Time, frame_delta: Duration, frames: u32) -> u32 {
        let mut total = 0;
        for _ in 0..frames {
            time.advance(frame_delta);
            while time.next_fixed_step() {
                total += 1;
            }
        }
        total
    }

    #[test]
    fn runs_one_step_per_frame_at_matching_rate() {
        // 60 Hz logic, 60 fps render: exactly one step per frame.
        let mut time = Time::with_fixed_hz(60.0);
        let steps = count_steps(&mut time, Duration::from_secs_f64(1.0 / 60.0), 60);
        assert_eq!(steps, 60);
    }

    #[test]
    fn slow_render_runs_two_steps_per_frame() {
        // 60 Hz logic, 30 fps render: exactly two steps per frame.
        let mut time = Time::with_fixed_hz(60.0);
        let frame = time.fixed_delta() * 2;
        let steps = count_steps(&mut time, frame, 30);
        assert_eq!(steps, 60);
    }

    #[test]
    fn fast_render_averages_one_step_per_two_frames() {
        // 60 Hz logic, ~144 fps render over one simulated second. Sub-step
        // frames accumulate, so the step count tracks real time (~60), not the
        // frame count.
        let mut time = Time::with_fixed_hz(60.0);
        let frame = Duration::from_secs_f64(1.0 / 144.0);
        let steps = count_steps(&mut time, frame, 144);
        assert!(
            (58..=61).contains(&steps),
            "expected ~60 steps, got {steps}"
        );
    }

    #[test]
    fn total_step_count_is_framerate_independent() {
        // The same wall-clock time yields the same number of logic steps
        // (determinism) regardless of render rate: one second simulated as
        // 60 frames of one step or 30 frames of two steps both give 60.
        let mut at_60 = Time::with_fixed_hz(60.0);
        let one_step = at_60.fixed_delta();
        assert_eq!(count_steps(&mut at_60, one_step, 60), 60);

        let mut at_30 = Time::with_fixed_hz(60.0);
        let two_steps = at_30.fixed_delta() * 2;
        assert_eq!(count_steps(&mut at_30, two_steps, 30), 60);
    }

    #[test]
    fn spiral_of_death_is_capped() {
        // A single enormous frame (e.g. 5 s stall) must not run hundreds of
        // steps; the per-frame cap limits it.
        let mut time = Time::with_fixed_hz(60.0);
        time.advance(Duration::from_secs(5));
        let mut steps = 0;
        while time.next_fixed_step() {
            steps += 1;
        }
        assert_eq!(steps, MAX_STEPS_PER_FRAME);
        // Backlog dropped, so the next normal frame behaves normally again.
        let steps = count_steps(&mut time, Duration::from_secs_f64(1.0 / 60.0), 1);
        assert_eq!(steps, 1);
    }

    #[test]
    fn frame_time_is_clamped() {
        let mut time = Time::with_fixed_hz(60.0);
        let clamped = time.advance(Duration::from_secs(10));
        assert_eq!(clamped, time.fixed_delta() * MAX_STEPS_PER_FRAME);
    }

    #[test]
    fn alpha_stays_in_unit_range() {
        let mut time = Time::with_fixed_hz(60.0);
        time.advance(Duration::from_secs_f64(1.0 / 90.0)); // partial step
        while time.next_fixed_step() {}
        let a = time.alpha();
        assert!((0.0..1.0).contains(&a), "alpha out of range: {a}");
    }
}
