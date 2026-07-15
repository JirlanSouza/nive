use iced::mouse;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OverflowAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OverflowDirection {
    Backward,
    Forward,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Overflow {
    pub(super) offset: f32,
    pub(super) max_offset: f32,
    pub(super) content_extent: f32,
    pub(super) viewport_extent: f32,
    pub(super) has_overflow: bool,
}

impl Default for Overflow {
    fn default() -> Self {
        Self {
            offset: 0.0,
            max_offset: 0.0,
            content_extent: 0.0,
            viewport_extent: 0.0,
            has_overflow: false,
        }
    }
}

impl Overflow {
    pub(super) fn update_extents(&mut self, content_extent: f32, viewport_extent: f32) {
        self.content_extent = content_extent;
        self.viewport_extent = viewport_extent;
        self.max_offset = (content_extent - viewport_extent).max(0.0);
        self.has_overflow = content_extent > viewport_extent + 0.5;
        self.clamp_offset();
    }

    pub(super) fn clamp_offset(&mut self) {
        self.offset = self.offset.clamp(0.0, self.max_offset);
    }

    pub(super) fn scroll_by(&mut self, delta: f32) {
        self.offset = (self.offset - delta).clamp(0.0, self.max_offset);
    }

    pub(super) fn page_step(&mut self, direction: OverflowDirection, factor: f32) {
        let step = factor * self.viewport_extent;
        match direction {
            OverflowDirection::Backward => {
                self.offset = (self.offset - step).max(0.0);
            }
            OverflowDirection::Forward => {
                self.offset = (self.offset + step).min(self.max_offset);
            }
        }
    }

    pub(super) fn show_start_chevron(&self) -> bool {
        self.has_overflow && self.offset > 0.0
    }

    pub(super) fn show_end_chevron(&self) -> bool {
        self.has_overflow && self.offset < self.max_offset
    }
}

pub(super) fn wheel_delta(axis: OverflowAxis, delta: mouse::ScrollDelta) -> f32 {
    match (axis, delta) {
        (OverflowAxis::Horizontal, mouse::ScrollDelta::Lines { x, y }) => {
            (if x.abs() > f32::EPSILON { x } else { y }) * 24.0
        }
        (OverflowAxis::Horizontal, mouse::ScrollDelta::Pixels { x, y }) => {
            if x.abs() > f32::EPSILON {
                x
            } else {
                y
            }
        }
        (OverflowAxis::Vertical, mouse::ScrollDelta::Lines { y, .. }) => y * 24.0,
        (OverflowAxis::Vertical, mouse::ScrollDelta::Pixels { y, .. }) => y,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_offset_after_extent_updates() {
        let mut overflow = Overflow {
            offset: 120.0,
            ..Overflow::default()
        };

        overflow.update_extents(180.0, 100.0);

        assert_eq!(overflow.offset, 80.0);
        assert_eq!(overflow.max_offset, 80.0);
        assert!(overflow.has_overflow);

        overflow.update_extents(80.0, 100.0);

        assert_eq!(overflow.offset, 0.0);
        assert_eq!(overflow.max_offset, 0.0);
        assert!(!overflow.has_overflow);
    }

    #[test]
    fn chevrons_reflect_start_and_end_reachability() {
        let mut overflow = Overflow::default();
        overflow.update_extents(180.0, 100.0);

        assert!(!overflow.show_start_chevron());
        assert!(overflow.show_end_chevron());

        overflow.offset = 80.0;

        assert!(overflow.show_start_chevron());
        assert!(!overflow.show_end_chevron());
    }

    #[test]
    fn extracts_wheel_delta_by_axis() {
        assert_eq!(
            wheel_delta(
                OverflowAxis::Horizontal,
                mouse::ScrollDelta::Lines { x: 2.0, y: 5.0 }
            ),
            48.0
        );
        assert_eq!(
            wheel_delta(
                OverflowAxis::Horizontal,
                mouse::ScrollDelta::Lines { x: 0.0, y: 5.0 }
            ),
            120.0
        );
        assert_eq!(
            wheel_delta(
                OverflowAxis::Vertical,
                mouse::ScrollDelta::Pixels { x: 2.0, y: 5.0 }
            ),
            5.0
        );
    }

    #[test]
    fn page_step_scrolls_and_clamps() {
        let mut overflow = Overflow::default();
        overflow.update_extents(180.0, 100.0);
        overflow.offset = 5.0;

        overflow.page_step(OverflowDirection::Backward, 0.8);
        assert_eq!(overflow.offset, 0.0);

        overflow.page_step(OverflowDirection::Forward, 0.8);
        assert_eq!(overflow.offset, 80.0);
    }
}
