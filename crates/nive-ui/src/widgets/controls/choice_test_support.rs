use std::borrow::Cow;

use iced::{
    keyboard::{self, key},
    mouse, touch, Event, Length, Point,
};

use crate::test_support::{named_probe, WidgetHarness};
use crate::theme::TypographyRole;
use crate::widgets::display::measured_text::{EllipsisStrategy, MeasuredText};
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ChoiceStateFixture {
    pub(super) selected: bool,
    pub(super) mixed: bool,
    pub(super) hovered: bool,
    pub(super) pressed: bool,
    pub(super) focused: bool,
    pub(super) invalid: bool,
    pub(super) disabled: bool,
    pub(super) interactive: bool,
}

impl ChoiceStateFixture {
    pub(super) const INTERACTION: [Self; 4] = [
        Self::enabled(),
        Self {
            hovered: true,
            ..Self::enabled()
        },
        Self {
            pressed: true,
            ..Self::enabled()
        },
        Self {
            focused: true,
            ..Self::enabled()
        },
    ];

    pub(super) const fn enabled() -> Self {
        Self {
            selected: false,
            mixed: false,
            hovered: false,
            pressed: false,
            focused: false,
            invalid: false,
            disabled: false,
            interactive: true,
        }
    }

    pub(super) const fn display_only() -> Self {
        Self {
            interactive: false,
            ..Self::enabled()
        }
    }

    pub(super) const fn disabled() -> Self {
        Self {
            disabled: true,
            ..Self::enabled()
        }
    }
}

pub(super) fn choice_probe<'a, Message>(
    name: &'static str,
    width: Length,
    height: Length,
) -> Element<'a, Message>
where
    Message: 'a,
{
    named_probe(name, iced::widget::Space::new().width(width).height(height))
}

pub(super) fn measured_choice_text<'a, Message>(
    name: &'static str,
    text: impl Into<Cow<'a, str>>,
    typography: TypographyRole,
    maximum_width: f32,
) -> Element<'a, Message>
where
    Message: 'a,
{
    named_probe(
        name,
        MeasuredText::new_inherited(text, EllipsisStrategy::End, typography)
            .max_width(maximum_width),
    )
}

pub(crate) fn pointer_move(position: Point) -> Event {
    Event::Mouse(mouse::Event::CursorMoved { position })
}

pub(super) fn pointer_press() -> Event {
    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
}

pub(super) fn pointer_release() -> Event {
    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
}

pub(super) fn touch_press(id: u64, position: Point) -> Event {
    Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(id),
        position,
    })
}

pub(super) fn touch_move(id: u64, position: Point) -> Event {
    Event::Touch(touch::Event::FingerMoved {
        id: touch::Finger(id),
        position,
    })
}

pub(super) fn touch_lift(id: u64, position: Point) -> Event {
    Event::Touch(touch::Event::FingerLifted {
        id: touch::Finger(id),
        position,
    })
}

pub(super) fn touch_lost(id: u64, position: Point) -> Event {
    Event::Touch(touch::Event::FingerLost {
        id: touch::Finger(id),
        position,
    })
}

pub(super) fn key_pressed(named: key::Named, code: key::Code) -> Event {
    let key = keyboard::Key::Named(named);

    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: key::Physical::Code(code),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
        text: None,
        repeat: false,
    })
}

pub(super) fn key_released(named: key::Named, code: key::Code) -> Event {
    let key = keyboard::Key::Named(named);

    Event::Keyboard(keyboard::Event::KeyReleased {
        key: key.clone(),
        modified_key: key,
        physical_key: key::Physical::Code(code),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
    })
}

pub(super) fn pointer_click<Message>(
    harness: &mut WidgetHarness<'_, Message>,
    position: Point,
) -> Vec<Message> {
    let mut messages = Vec::new();
    harness.set_cursor(position);

    for event in [pointer_move(position), pointer_press(), pointer_release()] {
        messages.extend(harness.update(event).messages);
    }

    messages
}

pub(super) fn touch_tap<Message>(
    harness: &mut WidgetHarness<'_, Message>,
    id: u64,
    position: Point,
) -> Vec<Message> {
    let mut messages = Vec::new();

    for event in [touch_press(id, position), touch_lift(id, position)] {
        messages.extend(harness.update(event).messages);
    }

    messages
}

#[cfg(test)]
mod tests {
    use iced::{keyboard::key, Size};

    use super::*;

    #[test]
    fn fixtures_cover_choice_state_and_named_layout() {
        assert!(ChoiceStateFixture::INTERACTION
            .iter()
            .any(|state| state.hovered));
        assert!(ChoiceStateFixture::INTERACTION
            .iter()
            .any(|state| state.pressed));
        assert!(ChoiceStateFixture::INTERACTION
            .iter()
            .any(|state| state.focused));
        assert!(!ChoiceStateFixture::display_only().interactive);
        assert!(ChoiceStateFixture::disabled().disabled);

        let mut harness = WidgetHarness::<()>::new(
            choice_probe("choice-target", Length::Fixed(80.0), Length::Fixed(28.0)),
            Size::new(320.0, 200.0),
        );

        assert_eq!(
            harness
                .named_bounds("choice-target")
                .map(|bounds| bounds.size()),
            Some(Size::new(80.0, 28.0))
        );
    }

    #[test]
    fn event_and_measured_text_fixtures_are_deterministic() {
        let events = [
            pointer_move(Point::new(8.0, 8.0)),
            pointer_press(),
            pointer_release(),
            touch_press(1, Point::new(8.0, 8.0)),
            touch_move(1, Point::new(12.0, 8.0)),
            touch_lift(1, Point::new(12.0, 8.0)),
            touch_lost(2, Point::new(16.0, 8.0)),
            key_pressed(key::Named::Space, key::Code::Space),
            key_released(key::Named::Space, key::Code::Space),
        ];

        assert_eq!(events.len(), 9);

        let mut harness = WidgetHarness::<()>::new(
            measured_choice_text(
                "choice-label",
                "A measured selection label",
                TypographyRole::ControlStrong,
                48.0,
            ),
            Size::new(320.0, 200.0),
        );

        assert!(harness
            .named_bounds("choice-label")
            .is_some_and(|bounds| bounds.width <= 48.0));
    }
}
