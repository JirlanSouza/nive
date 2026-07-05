use iced::{
    advanced::mouse,
    advanced::Shell,
    keyboard::{self, key, Key},
    touch, Color, Event, Rectangle,
};

use super::state::ColorInputState;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum ColorInputEvent {
    Dismiss,
    Confirm,
    Cancel,
    Change(Color),
}

pub(super) fn trigger_pressed(
    enabled: bool,
    event: &Event,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> bool {
    if !enabled {
        return false;
    }

    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => cursor.is_over(bounds),
        Event::Touch(touch::Event::FingerPressed { position, .. }) => bounds.contains(*position),
        _ => false,
    }
}

pub(super) fn popover_key_pressed(event: &keyboard::Event) -> Option<ColorInputEvent> {
    match event {
        keyboard::Event::KeyPressed {
            key: Key::Named(key::Named::Enter),
            ..
        } => Some(ColorInputEvent::Confirm),
        keyboard::Event::KeyPressed {
            key: Key::Named(key::Named::Escape),
            ..
        } => Some(ColorInputEvent::Cancel),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct ColorInputTransition {
    changed: Option<Color>,
    redraw: bool,
}

impl ColorInputEvent {
    pub(super) fn apply(self, state: &mut ColorInputState) -> ColorInputTransition {
        match self {
            Self::Dismiss | Self::Confirm => {
                if state.confirm() {
                    ColorInputTransition {
                        redraw: true,
                        ..ColorInputTransition::default()
                    }
                } else {
                    ColorInputTransition::default()
                }
            }
            Self::Cancel => {
                let open = state.is_open();
                let changed = state.cancel();

                ColorInputTransition {
                    changed,
                    redraw: open,
                }
            }
            Self::Change(color) => {
                state.apply_change(color);

                ColorInputTransition {
                    changed: Some(color),
                    redraw: true,
                }
            }
        }
    }
}

impl ColorInputTransition {
    pub(super) fn relay<Message>(
        self,
        on_change: Option<&dyn Fn(Color) -> Message>,
        shell: &mut Shell<'_, Message>,
    ) {
        if let Some(color) = self.changed {
            if let Some(on_change) = on_change {
                shell.publish(on_change(color));
            }
        }

        if self.redraw {
            shell.invalidate_layout();
            shell.request_redraw();
        }
    }
}

#[cfg(test)]
mod color_input_event_tests {
    use super::*;

    use iced::keyboard::key::{Code, Physical};
    use iced::keyboard::Location;

    fn key_pressed(key: Key) -> keyboard::Event {
        keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: Physical::Code(Code::Enter),
            location: Location::Standard,
            modifiers: keyboard::Modifiers::NONE,
            text: None,
            repeat: false,
        }
    }

    #[test]
    fn enter_confirms_popover() {
        assert_eq!(
            popover_key_pressed(&key_pressed(Key::Named(key::Named::Enter))),
            Some(ColorInputEvent::Confirm)
        );
    }

    #[test]
    fn escape_cancels_popover() {
        assert_eq!(
            popover_key_pressed(&key_pressed(Key::Named(key::Named::Escape))),
            Some(ColorInputEvent::Cancel)
        );
    }

    #[test]
    fn cancel_restores_initial_color_when_changed() {
        let mut state = ColorInputState::default();
        state.open_with(Color::BLACK);

        ColorInputEvent::Change(Color::WHITE).apply(&mut state);
        let transition = ColorInputEvent::Cancel.apply(&mut state);

        assert_eq!(transition.changed, Some(Color::BLACK));
        assert!(!state.is_open());
    }

    #[test]
    fn confirm_closes_without_republishing_color() {
        let mut state = ColorInputState::default();
        state.open_with(Color::BLACK);

        ColorInputEvent::Change(Color::WHITE).apply(&mut state);
        let transition = ColorInputEvent::Confirm.apply(&mut state);

        assert_eq!(transition.changed, None);
        assert!(!state.is_open());
    }
}
