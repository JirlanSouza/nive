use std::{cell::Cell, rc::Rc};

use iced::{
    advanced::widget::{operation, Id},
    Rectangle, Vector,
};

#[derive(Debug, Clone)]
pub(crate) struct EnsureVisibleHandle {
    scrollable: Id,
    target: Rc<Cell<Option<Rectangle>>>,
}

impl EnsureVisibleHandle {
    pub(crate) fn new() -> Self {
        Self {
            scrollable: Id::unique(),
            target: Rc::new(Cell::new(None)),
        }
    }

    pub(crate) fn scrollable(&self) -> Id {
        self.scrollable.clone()
    }

    pub(crate) fn request(&self, target: Rectangle) {
        self.target.set(Some(target));
    }

    pub(crate) fn take(&self) -> Option<Rectangle> {
        self.target.take()
    }
}

impl Default for EnsureVisibleHandle {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn ensure_visible(scrollable: Id, target: Rectangle) -> impl operation::Operation {
    struct EnsureVisible {
        scrollable: Id,
        target: Rectangle,
    }

    impl operation::Operation for EnsureVisible {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
            operate(self);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            content_bounds: Rectangle,
            translation: Vector,
            state: &mut dyn operation::Scrollable,
        ) {
            if id != Some(&self.scrollable) {
                return;
            }

            let current_offset = finite_nonnegative(translation.y);
            let target = Rectangle {
                y: self.target.y - current_offset,
                ..self.target
            };
            let offset =
                ensure_visible_offset(bounds, target, current_offset, content_bounds.height);
            state.scroll_to(operation::scrollable::AbsoluteOffset {
                x: None,
                y: Some(offset),
            });
        }
    }

    EnsureVisible { scrollable, target }
}

pub(crate) fn ensure_visible_offset(
    viewport: Rectangle,
    target: Rectangle,
    current_offset: f32,
    content_height: f32,
) -> f32 {
    let viewport = super::geometry::sanitize_rectangle(viewport);
    let target = super::geometry::sanitize_rectangle(target);
    let current_offset = finite_nonnegative(current_offset);
    let content_height = finite_nonnegative(content_height);
    let maximum_offset = (content_height - viewport.height).max(0.0);
    let requested = if target.y < viewport.y {
        current_offset - (viewport.y - target.y)
    } else if target.y + target.height > viewport.y + viewport.height {
        current_offset + (target.y + target.height - viewport.y - viewport.height)
    } else {
        current_offset
    };

    requested.clamp(0.0, maximum_offset)
}

fn finite_nonnegative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use iced::{advanced::widget::Operation as _, Point, Size};

    use super::*;

    #[test]
    fn ensure_visible_scrolls_only_enough_to_reveal_the_target() {
        let viewport = Rectangle::new(Point::new(0.0, 40.0), Size::new(200.0, 100.0));

        assert_eq!(
            ensure_visible_offset(
                viewport,
                Rectangle::new(Point::new(0.0, 120.0), Size::new(200.0, 40.0)),
                40.0,
                300.0,
            ),
            60.0
        );
        assert_eq!(
            ensure_visible_offset(
                viewport,
                Rectangle::new(Point::new(0.0, 20.0), Size::new(200.0, 20.0)),
                40.0,
                300.0,
            ),
            20.0
        );
    }

    #[test]
    fn ensure_visible_sanitizes_and_clamps_offsets() {
        assert_eq!(
            ensure_visible_offset(
                Rectangle::with_size(Size::new(100.0, 50.0)),
                Rectangle::new(Point::new(0.0, 500.0), Size::new(100.0, 20.0)),
                f32::NAN,
                120.0,
            ),
            70.0
        );
    }

    #[test]
    fn ensure_visible_operation_targets_only_the_owned_scrollable() {
        #[derive(Default)]
        struct ScrollState(Option<operation::scrollable::AbsoluteOffset<Option<f32>>>);

        impl operation::Scrollable for ScrollState {
            fn snap_to(&mut self, _offset: operation::scrollable::RelativeOffset<Option<f32>>) {}

            fn scroll_to(&mut self, offset: operation::scrollable::AbsoluteOffset<Option<f32>>) {
                self.0 = Some(offset);
            }

            fn scroll_by(
                &mut self,
                _offset: operation::scrollable::AbsoluteOffset,
                _bounds: Rectangle,
                _content_bounds: Rectangle,
            ) {
            }
        }

        let id = Id::unique();
        let mut operation = ensure_visible(
            id.clone(),
            Rectangle::new(Point::new(0.0, 140.0), Size::new(100.0, 20.0)),
        );
        let mut state = ScrollState::default();
        operation.scrollable(
            Some(&id),
            Rectangle::with_size(Size::new(100.0, 100.0)),
            Rectangle::with_size(Size::new(100.0, 300.0)),
            Vector::new(0.0, 40.0),
            &mut state,
        );

        assert_eq!(state.0.and_then(|offset| offset.y), Some(60.0));
    }
}
