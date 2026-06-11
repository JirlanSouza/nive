use iced::{
    advanced::Shell,
    time::{Duration, Instant},
};

use super::timeline::{Animation, AnimationFrame};

/// The per-widget animation clock, stored in the widget tree state.
///
/// Shared by `AnimatedVisual` and `AnimatedLayout`: it records when the
/// animation started, advances against the window's redraw clock, and schedules
/// the next frame while the animation is still running.
#[derive(Debug, Default)]
pub(super) struct AnimationState {
    started_at: Option<Instant>,
    now: Option<Instant>,
}

impl AnimationState {
    fn tick(&mut self, now: Instant) {
        let started_at = *self.started_at.get_or_insert(now);
        self.now = Some(now.max(started_at));
    }

    pub(super) fn frame(&self, animation: Animation) -> AnimationFrame {
        match (self.started_at, self.now) {
            (Some(started_at), Some(now)) => animation.frame(now.duration_since(started_at)),
            _ => animation.frame(Duration::ZERO),
        }
    }

    /// Advances the clock to `now` and, while the animation is still running,
    /// schedules the next redraw — at the native refresh rate by default, or at
    /// a fixed cadence when `frame_interval` is set.
    pub(super) fn advance<Message>(
        &mut self,
        now: Instant,
        animation: Animation,
        frame_interval: Option<Duration>,
        shell: &mut Shell<'_, Message>,
    ) -> AnimationFrame {
        self.tick(now);

        let frame = self.frame(animation);

        if !animation.finished(frame.elapsed()) {
            match frame_interval {
                Some(interval) => shell.request_redraw_at(now + interval),
                None => shell.request_redraw(),
            }
        }

        frame
    }
}
