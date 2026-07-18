use iced::{
    advanced::widget::operation::focusable,
    keyboard::{self, key, Location},
    mouse, Event, Point, Size,
};

use super::super::ColorInput;
use crate::{accessibility::FocusRoot, test_support::WidgetHarness, Element};

fn tab_pressed() -> Event {
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: keyboard::Key::Named(key::Named::Tab),
        modified_key: keyboard::Key::Named(key::Named::Tab),
        physical_key: keyboard::key::Physical::Code(key::Code::Tab),
        location: Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
        text: None,
        repeat: false,
    })
}

#[test]
fn compatibility_trap_enters_once_and_visits_every_picker_control() {
    let input: Element<'_, ()> = ColorInput::new(iced::Color::BLACK).on_change(|_| ()).into();
    let mut harness = WidgetHarness::new(FocusRoot::new(input).into(), Size::new(360.0, 280.0));
    harness.set_cursor(Point::new(10.0, 10.0));
    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    assert!(harness.has_overlay());

    for _ in 0..2 {
        harness
            .update_overlay(Event::Window(iced::window::Event::RedrawRequested(
                iced::time::Instant::now(),
            )))
            .expect("open ColorInput overlay");
        assert_eq!(
            harness.focused_overlay_count(),
            Some(focusable::Count {
                total: 5,
                focused: None,
            })
        );
    }

    for expected in [0, 1, 2, 3, 4, 0] {
        let result = harness
            .update_overlay(tab_pressed())
            .expect("open ColorInput overlay");
        assert!(result.captured);
        assert_eq!(
            harness
                .focused_overlay_count()
                .and_then(|count| count.focused),
            Some(expected)
        );
    }
}
