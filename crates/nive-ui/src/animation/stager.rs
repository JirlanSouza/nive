use super::AnimationFrame;

/// A reusable "staggered pulse" curve: several items that brighten and dim in
/// sequence, with optional overlap and a rest gap before the cycle restarts.
///
/// It shapes a single [`AnimationFrame`] into a per-item activity value, so a
/// row of dots can share one animation. See the bootstrap loading dots.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StaggeredPulse {
    count: usize,
    overlap: f32,
    rest: f32,
}

impl StaggeredPulse {
    pub const fn new(count: usize) -> Self {
        Self {
            count,
            overlap: 0.0,
            rest: 0.0,
        }
    }

    pub const fn overlap(mut self, overlap: f32) -> Self {
        self.overlap = overlap;
        self
    }

    pub const fn rest(mut self, rest: f32) -> Self {
        self.rest = rest;
        self
    }

    pub fn activity(self, frame: AnimationFrame, index: usize) -> f32 {
        if self.count == 0 || index >= self.count {
            return 0.0;
        }

        let progress = frame.progress().clamp(0.0, 1.0);
        let active_span = 1.0 - self.rest.clamp(0.0, 1.0);

        if active_span <= f32::EPSILON || progress >= active_span {
            return 0.0;
        }

        let overlap = self.overlap.max(0.0);
        let step = active_span / (self.count as f32 + overlap);
        let span = step * (1.0 + overlap);
        let start = index as f32 * step;
        let end = start + span;

        if progress < start || progress > end {
            return 0.0;
        }

        let local = (progress - start) / span;

        if local <= 0.5 {
            local * 2.0
        } else {
            (1.0 - local) * 2.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOT_PULSE: StaggeredPulse = StaggeredPulse::new(3).overlap(0.65).rest(0.28);

    fn frame_at(progress: f32) -> AnimationFrame {
        AnimationFrame::from_progress(progress)
    }

    #[test]
    fn staggered_pulse_stays_in_unit_range() {
        for step in 0..=100 {
            let frame = frame_at(step as f32 / 100.0);

            for index in 0..3 {
                let value = DOT_PULSE.activity(frame, index);

                assert!((0.0..=1.0).contains(&value));
            }
        }
    }

    #[test]
    fn staggered_pulse_peaks_are_evenly_spaced() {
        let active_span = 1.0 - 0.28;
        let step = active_span / (3.0 + 0.65);
        let span = step * (1.0 + 0.65);

        assert_nearly_eq(DOT_PULSE.activity(frame_at(span * 0.5), 0), 1.0);
        assert_nearly_eq(DOT_PULSE.activity(frame_at(step + span * 0.5), 1), 1.0);
        assert_nearly_eq(
            DOT_PULSE.activity(frame_at(step * 2.0 + span * 0.5), 2),
            1.0,
        );
    }

    #[test]
    fn staggered_pulse_overlaps_adjacent_items() {
        let active_span = 1.0 - 0.28;
        let step = active_span / (3.0 + 0.65);
        let span = step * (1.0 + 0.65);
        let overlap_sample = step + (span - step) / 2.0;

        assert!(DOT_PULSE.activity(frame_at(overlap_sample), 0) > 0.0);
        assert!(DOT_PULSE.activity(frame_at(overlap_sample), 1) > 0.0);
    }

    #[test]
    fn staggered_pulse_rests_after_last_item() {
        let active_span = 1.0 - 0.28;
        let rest_sample = (active_span + 1.0) / 2.0;

        assert_eq!(DOT_PULSE.activity(frame_at(active_span), 2), 0.0);
        assert_eq!(DOT_PULSE.activity(frame_at(rest_sample), 0), 0.0);
        assert_eq!(DOT_PULSE.activity(frame_at(rest_sample), 1), 0.0);
        assert_eq!(DOT_PULSE.activity(frame_at(rest_sample), 2), 0.0);
    }

    #[test]
    fn staggered_pulse_ignores_invalid_items() {
        assert_eq!(DOT_PULSE.activity(frame_at(0.25), 3), 0.0);
        assert_eq!(StaggeredPulse::new(0).activity(frame_at(0.25), 0), 0.0);
    }

    fn assert_nearly_eq(left: f32, right: f32) {
        assert!((left - right).abs() < 0.000_001);
    }
}
