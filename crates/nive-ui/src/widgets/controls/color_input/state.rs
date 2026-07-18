use iced::{
    advanced::widget::{operation, Id},
    Color, Rectangle,
};

use crate::advanced::focus::FocusState;

#[derive(Debug, Default)]
pub(super) struct ColorInputState {
    focus: FocusState,
    session: Option<ColorInputSession>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ColorInputSession {
    initial: Color,
    current: Color,
}

impl ColorInputState {
    pub(super) fn is_active(&self) -> bool {
        self.focus.is_active()
    }

    pub(super) fn is_focus_visible(&self) -> bool {
        self.focus.is_focus_visible()
    }

    pub(super) fn register(
        &mut self,
        operation: &mut dyn operation::Operation,
        id: Option<&Id>,
        bounds: Rectangle,
    ) {
        self.focus.register(operation, id, bounds);
    }

    pub(super) fn focus_from_pointer(&mut self) {
        self.focus.focus_from_pointer();
    }

    pub(super) fn clear_focus(&mut self) {
        self.focus.clear();
    }

    pub(super) fn is_open(&self) -> bool {
        self.session.is_some()
    }

    pub(super) fn value(&self, fallback: Color) -> Color {
        self.session
            .map(|session| session.current)
            .unwrap_or(fallback)
    }

    pub(super) fn open_with(&mut self, value: Color) {
        self.session = Some(ColorInputSession {
            initial: value,
            current: value,
        });
    }

    pub(super) fn toggle_with(&mut self, value: Color) {
        if self.is_open() {
            self.confirm();
        } else {
            self.open_with(value);
        }
    }

    pub(super) fn apply_change(&mut self, color: Color) {
        if let Some(session) = &mut self.session {
            session.current = color;
        }
    }

    pub(super) fn confirm(&mut self) -> bool {
        self.session.take().is_some()
    }

    pub(super) fn cancel(&mut self) -> Option<Color> {
        let session = self.session.take()?;

        (!colors_match(session.initial, session.current)).then_some(session.initial)
    }
}

fn colors_match(left: Color, right: Color) -> bool {
    const EPSILON: f32 = 1.0 / 255.0;

    (left.r - right.r).abs() <= EPSILON
        && (left.g - right.g).abs() <= EPSILON
        && (left.b - right.b).abs() <= EPSILON
        && (left.a - right.a).abs() <= EPSILON
}

#[cfg(test)]
mod color_input_state_tests {
    use super::*;

    #[test]
    fn opening_sets_open() {
        let mut state = ColorInputState::default();

        state.open_with(Color::BLACK);

        assert!(state.is_open());
    }

    #[test]
    fn dismissing_sets_closed() {
        let mut state = ColorInputState::default();
        state.open_with(Color::BLACK);

        state.confirm();

        assert!(!state.is_open());
    }

    #[test]
    fn toggling_opens_and_then_closes() {
        let mut state = ColorInputState::default();

        state.toggle_with(Color::BLACK);

        assert!(state.is_open());

        state.toggle_with(Color::BLACK);

        assert!(!state.is_open());
    }

    #[test]
    fn value_returns_current_session_color_while_open() {
        let mut state = ColorInputState::default();

        state.open_with(Color::BLACK);
        state.apply_change(Color::WHITE);

        assert_eq!(state.value(Color::BLACK), Color::WHITE);
    }

    #[test]
    fn cancel_returns_initial_color_when_changed() {
        let mut state = ColorInputState::default();

        state.open_with(Color::BLACK);
        state.apply_change(Color::WHITE);

        assert_eq!(state.cancel(), Some(Color::BLACK));
        assert!(!state.is_open());
    }

    #[test]
    fn cancel_returns_none_when_unchanged() {
        let mut state = ColorInputState::default();

        state.open_with(Color::BLACK);

        assert_eq!(state.cancel(), None);
        assert!(!state.is_open());
    }

    #[test]
    fn focus_operation_tracks_focus_state() {
        let mut state = ColorInputState::default();

        assert!(!state.is_active());

        operation::Focusable::focus(&mut state.focus);

        assert!(state.is_active());

        operation::Focusable::unfocus(&mut state.focus);

        assert!(!state.is_active());
    }
}
