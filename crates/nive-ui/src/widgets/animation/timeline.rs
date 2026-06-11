use iced::time::Duration;

pub use iced::animation::Easing;

/// The timing specification of an animation: how long it runs, how its progress
/// is shaped ([`Easing`]), whether it repeats, and an optional start [`delay`].
///
/// An [`Animation`] is pure data — it knows how to project a [`AnimationFrame`]
/// for a given elapsed time via [`Animation::frame`]. The widgets
/// (`AnimatedVisual`, `AnimatedLayout`) drive it against the window clock.
///
/// [`delay`]: Animation::delay
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Animation {
    duration: Duration,
    easing: Easing,
    repeat: Repeat,
    auto_reverse: bool,
    delay: Duration,
}

/// How many times an [`Animation`] plays its `0.0 -> 1.0` cycle.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Repeat {
    /// Plays the cycle once and rests at the end.
    #[default]
    Never,
    /// Plays the cycle forever.
    Forever,
    /// Plays the cycle a fixed number of times, then rests.
    Times(u32),
}

/// A snapshot of an [`Animation`] at a point in time.
///
/// `progress` is already shaped by the animation's [`Easing`] and is always in
/// `0.0..=1.0`; use [`AnimationFrame::lerp`] to map it onto a value range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationFrame {
    elapsed: Duration,
    progress: f32,
    cycle: u64,
}

impl Animation {
    const DEFAULT_DURATION: Duration = Duration::from_millis(900);

    /// A linearly-timed animation that plays once.
    pub fn linear(duration: Duration) -> Self {
        Self {
            duration,
            easing: Easing::Linear,
            repeat: Repeat::Never,
            auto_reverse: false,
            delay: Duration::ZERO,
        }
    }

    /// Shapes the animation's progress with the given [`Easing`].
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Repeats the cycle forever.
    pub fn repeat(mut self) -> Self {
        self.repeat = Repeat::Forever;
        self
    }

    /// Repeats the cycle a fixed number of `times`, then rests.
    pub fn repeat_times(mut self, times: u32) -> Self {
        self.repeat = Repeat::Times(times);
        self
    }

    /// Reverses every other cycle, producing a ping-pong motion.
    pub fn auto_reverse(mut self) -> Self {
        self.auto_reverse = true;
        self
    }

    /// Delays the start of the animation by `delay`.
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    pub(super) fn frame(self, elapsed: Duration) -> AnimationFrame {
        let duration_secs = self.duration.as_secs_f32();

        if duration_secs <= f32::EPSILON {
            return AnimationFrame {
                elapsed,
                progress: self.easing.value(1.0),
                cycle: 0,
            };
        }

        let active = (elapsed.as_secs_f32() - self.delay.as_secs_f32()).max(0.0);
        let total = active / duration_secs;
        let cycle = total.floor().max(0.0) as u64;
        let finished = self.finished_total(total);

        let mut t = if finished {
            self.rest_t()
        } else {
            total.fract()
        };

        if self.auto_reverse && !finished && cycle % 2 == 1 {
            t = 1.0 - t;
        }

        AnimationFrame {
            elapsed,
            progress: self.easing.value(t.clamp(0.0, 1.0)),
            cycle,
        }
    }

    // Whether the animation has come to rest at `elapsed`; the driving widgets
    // use it to stop scheduling redraws.
    pub(super) fn finished(self, elapsed: Duration) -> bool {
        let duration_secs = self.duration.as_secs_f32();

        if duration_secs <= f32::EPSILON {
            return !matches!(self.repeat, Repeat::Forever);
        }

        let active = (elapsed.as_secs_f32() - self.delay.as_secs_f32()).max(0.0);

        self.finished_total(active / duration_secs)
    }

    fn finished_total(self, total: f32) -> bool {
        match self.repeat {
            Repeat::Never => total >= 1.0,
            Repeat::Forever => false,
            Repeat::Times(times) => total >= times as f32,
        }
    }

    fn rest_t(self) -> f32 {
        match self.repeat {
            // A finished ping-pong rests where its last cycle left it: an even
            // number of cycles ends back at the start.
            Repeat::Times(times) if self.auto_reverse => {
                if times % 2 == 1 {
                    1.0
                } else {
                    0.0
                }
            }
            _ => 1.0,
        }
    }
}

impl Default for Animation {
    fn default() -> Self {
        Self::linear(Self::DEFAULT_DURATION).repeat()
    }
}

impl AnimationFrame {
    pub(super) fn initial() -> Self {
        Self {
            elapsed: Duration::ZERO,
            progress: 0.0,
            cycle: 0,
        }
    }

    #[cfg(test)]
    pub(super) fn from_progress(progress: f32) -> Self {
        Self {
            elapsed: Duration::ZERO,
            progress,
            cycle: 0,
        }
    }

    /// Wall-clock time elapsed since the animation started.
    pub fn elapsed(self) -> Duration {
        self.elapsed
    }

    /// Eased progress through the current cycle, in `0.0..=1.0`.
    pub fn progress(self) -> f32 {
        self.progress
    }

    /// How many full cycles have elapsed.
    pub fn cycle(self) -> u64 {
        self.cycle
    }

    /// Progress expressed as turns — an alias of [`progress`] for rotations.
    ///
    /// [`progress`]: AnimationFrame::progress
    pub fn turns(self) -> f32 {
        self.progress
    }

    /// Interpolates between `from` and `to` using the eased [`progress`].
    ///
    /// [`progress`]: AnimationFrame::progress
    pub fn lerp(self, from: f32, to: f32) -> f32 {
        from + (to - from) * self.progress
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_animation_wraps_progress_and_counts_cycles() {
        let animation = Animation::linear(Duration::from_millis(100)).repeat();
        let frame = animation.frame(Duration::from_millis(250));

        assert_eq!(frame.cycle(), 2);
        assert!((frame.progress() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn one_shot_animation_clamps_progress() {
        let animation = Animation::linear(Duration::from_millis(100));
        let frame = animation.frame(Duration::from_millis(250));

        assert_eq!(frame.progress(), 1.0);
        assert!(animation.finished(Duration::from_millis(250)));
        assert!(!animation.finished(Duration::from_millis(50)));
    }

    #[test]
    fn easing_shapes_progress() {
        let eased = Animation::linear(Duration::from_millis(100)).easing(Easing::EaseInOut);
        let frame = eased.frame(Duration::from_millis(50));

        // EaseInOut is symmetric: the midpoint stays at 0.5.
        assert!((frame.progress() - 0.5).abs() < 1e-6);

        let frame = eased.frame(Duration::from_millis(25));
        // ...but a quarter through it lags behind linear (which would be 0.25).
        assert!(frame.progress() < 0.25);
    }

    #[test]
    fn lerp_maps_progress_onto_a_range() {
        let frame = Animation::linear(Duration::from_millis(100)).frame(Duration::from_millis(25));

        assert!((frame.lerp(0.0, 200.0) - 50.0).abs() < 1e-4);
        assert!((frame.lerp(10.0, 20.0) - 12.5).abs() < 1e-4);
    }

    #[test]
    fn repeat_times_rests_after_the_last_cycle() {
        let animation = Animation::linear(Duration::from_millis(100)).repeat_times(2);

        assert!(!animation.finished(Duration::from_millis(150)));
        assert_eq!(animation.frame(Duration::from_millis(250)).progress(), 1.0);
        assert!(animation.finished(Duration::from_millis(200)));
    }

    #[test]
    fn auto_reverse_flips_odd_cycles() {
        let animation = Animation::linear(Duration::from_millis(100))
            .repeat()
            .auto_reverse();

        // Cycle 0 runs forward: 0.25 of the way through is 0.25.
        assert!((animation.frame(Duration::from_millis(25)).progress() - 0.25).abs() < 1e-6);
        // Cycle 1 runs backward: 0.25 of the way through is 0.75.
        assert!((animation.frame(Duration::from_millis(125)).progress() - 0.75).abs() < 1e-6);
    }

    #[test]
    fn delay_holds_at_the_start() {
        let animation =
            Animation::linear(Duration::from_millis(100)).delay(Duration::from_millis(50));

        assert_eq!(animation.frame(Duration::from_millis(25)).progress(), 0.0);
        assert!((animation.frame(Duration::from_millis(100)).progress() - 0.5).abs() < 1e-6);
    }
}
