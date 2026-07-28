//! The hairline separating a Dialog's header and footer from its body, and
//! the scroll state that decides when each one is visible.

use iced::{
    advanced::{renderer, widget::operation, widget::Id},
    Background, Border, Rectangle, Shadow, Vector,
};

use crate::theme::BorderRole;
use crate::{Renderer, Theme};

pub(super) const SEAM_WIDTH: f32 = 1.0;
pub(super) const SEAM_VISIBILITY_EPSILON: f32 = 0.5;

/// Survives across `view()` rebuilds via `Tree::diff` (unlike `Dialog`
/// itself, rebuilt every frame), so `draw` can read the body's last known
/// scroll offset to decide seam visibility.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct DialogSeamState {
    pub(super) offset_y: f32,
}

/// Reads the body scrollable's current translation without downcasting its
/// private internal state.
#[derive(Default)]
pub(super) struct ScrollOffsetProbe {
    pub(super) offset_y: f32,
}

impl operation::Operation for ScrollOffsetProbe {
    fn traverse(&mut self, _operate: &mut dyn FnMut(&mut dyn operation::Operation)) {}

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn operation::Scrollable,
    ) {
        self.offset_y = translation.y;
    }
}

pub(super) fn header_seam_visible(scroll_offset_y: f32) -> bool {
    scroll_offset_y > SEAM_VISIBILITY_EPSILON
}

pub(super) fn footer_seam_visible(
    scroll_offset_y: f32,
    content_height: f32,
    viewport_height: f32,
) -> bool {
    content_height - viewport_height - scroll_offset_y > SEAM_VISIBILITY_EPSILON
}

pub(super) fn draw_seam(
    renderer: &mut Renderer,
    theme: &Theme,
    edge: Rectangle,
    frame: Rectangle,
    below: bool,
) {
    use iced::advanced::Renderer as _;

    let y = if below { edge.y + edge.height } else { edge.y };

    renderer.fill_quad(
        renderer::Quad {
            bounds: Rectangle {
                x: frame.x,
                y,
                width: frame.width,
                height: SEAM_WIDTH,
            },
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(theme.border(BorderRole::Subtle).color),
    );
}
