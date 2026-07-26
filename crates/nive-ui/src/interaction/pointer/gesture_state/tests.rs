use std::time::{Duration, Instant};

use iced::{mouse, touch, Event, Point};

use super::super::{PointerButton, PointerGestureKind};
use super::PointerGestureState;

fn moved(x: f32, y: f32) -> Event {
    Event::Mouse(mouse::Event::CursorMoved {
        position: Point::new(x, y),
    })
}

fn press(button: mouse::Button) -> Event {
    Event::Mouse(mouse::Event::ButtonPressed(button))
}

fn release(button: mouse::Button) -> Event {
    Event::Mouse(mouse::Event::ButtonReleased(button))
}

fn touch_press(id: u64, x: f32, y: f32) -> Event {
    Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(id),
        position: Point::new(x, y),
    })
}

fn touch_move(id: u64, x: f32, y: f32) -> Event {
    Event::Touch(touch::Event::FingerMoved {
        id: touch::Finger(id),
        position: Point::new(x, y),
    })
}

fn touch_lift(id: u64, x: f32, y: f32) -> Event {
    Event::Touch(touch::Event::FingerLifted {
        id: touch::Finger(id),
        position: Point::new(x, y),
    })
}

fn touch_lost(id: u64, x: f32, y: f32) -> Event {
    Event::Touch(touch::Event::FingerLost {
        id: touch::Finger(id),
        position: Point::new(x, y),
    })
}

#[test]
fn maps_iced_buttons_to_pointer_buttons() {
    assert_eq!(
        PointerButton::from(mouse::Button::Left),
        PointerButton::Primary
    );
    assert_eq!(
        PointerButton::from(mouse::Button::Right),
        PointerButton::Secondary
    );
    assert_eq!(
        PointerButton::from(mouse::Button::Middle),
        PointerButton::Middle
    );
    assert_eq!(
        PointerButton::from(mouse::Button::Back),
        PointerButton::Auxiliary(4)
    );
    assert_eq!(
        PointerButton::from(mouse::Button::Forward),
        PointerButton::Auxiliary(5)
    );
    assert_eq!(
        PointerButton::from(mouse::Button::Other(8)),
        PointerButton::Auxiliary(8)
    );
}

#[test]
fn normalizes_click_count() {
    let mut state = PointerGestureState::new();
    let now = Instant::now();

    state.handle_event(&moved(1.0, 1.0), now, |_| Some("row"));
    state.handle_event(&press(mouse::Button::Left), now, |_| Some("row"));
    let first = state.handle_event(
        &release(mouse::Button::Left),
        now + Duration::from_millis(20),
        |_| Some("row"),
    );

    state.handle_event(
        &press(mouse::Button::Left),
        now + Duration::from_millis(120),
        |_| Some("row"),
    );
    let second = state.handle_event(
        &release(mouse::Button::Left),
        now + Duration::from_millis(140),
        |_| Some("row"),
    );

    assert!(matches!(
        first.last().map(|gesture| gesture.kind),
        Some(PointerGestureKind::Clicked { count: 1 })
    ));
    assert!(matches!(
        second.last().map(|gesture| gesture.kind),
        Some(PointerGestureKind::Clicked { count: 2 })
    ));
}

#[test]
fn waits_for_drag_threshold_then_moves_and_releases() {
    let mut state = PointerGestureState::new().with_drag_threshold(5.0);
    let now = Instant::now();

    state.handle_event(&moved(0.0, 0.0), now, |_| Some("row"));
    state.handle_event(&press(mouse::Button::Left), now, |_| Some("row"));
    assert!(state
        .handle_event(&moved(3.0, 0.0), now, |_| Some("row"))
        .is_empty());

    let started = state.handle_event(&moved(6.0, 0.0), now, |_| Some("row"));
    let released = state.handle_event(&release(mouse::Button::Left), now, |_| Some("row"));

    assert_eq!(
        started
            .iter()
            .map(|gesture| gesture.kind)
            .collect::<Vec<_>>(),
        vec![
            PointerGestureKind::DragStarted,
            PointerGestureKind::DragMoved
        ]
    );
    assert_eq!(
        released
            .iter()
            .map(|gesture| gesture.kind)
            .collect::<Vec<_>>(),
        vec![
            PointerGestureKind::Released,
            PointerGestureKind::DragReleased
        ]
    );
}

fn dragging_state() -> (PointerGestureState<&'static str>, Instant) {
    let mut state = PointerGestureState::new().with_drag_threshold(1.0);
    let now = Instant::now();

    state.handle_event(&moved(0.0, 0.0), now, |_| Some("row"));
    state.handle_event(&press(mouse::Button::Left), now, |_| Some("row"));
    state.handle_event(&moved(2.0, 0.0), now, |_| Some("row"));

    (state, now)
}

#[test]
fn leaving_the_window_suspends_the_drag_instead_of_cancelling_it() {
    let (mut state, now) = dragging_state();

    let left = state.handle_event(&Event::Mouse(mouse::Event::CursorLeft), now, |_| {
        Some("row")
    });
    assert!(left.is_empty(), "leaving the window emitted {left:?}");

    // The button never came up, so moving back inside carries on the same drag.
    let resumed = state.handle_event(&moved(40.0, 0.0), now, |_| Some("row"));

    assert_eq!(
        resumed.iter().map(|g| g.kind).collect::<Vec<_>>(),
        vec![PointerGestureKind::DragMoved]
    );
    assert_eq!(resumed[0].position.x, 40.0);
}

#[test]
fn releasing_after_a_window_round_trip_still_terminates_the_drag() {
    let (mut state, now) = dragging_state();

    state.handle_event(&Event::Mouse(mouse::Event::CursorLeft), now, |_| {
        Some("row")
    });
    state.handle_event(&Event::Mouse(mouse::Event::CursorEntered), now, |_| {
        Some("row")
    });
    state.handle_event(&moved(40.0, 0.0), now, |_| Some("row"));
    let released = state.handle_event(&release(mouse::Button::Left), now, |_| Some("row"));

    assert_eq!(
        released.iter().map(|g| g.kind).collect::<Vec<_>>(),
        vec![
            PointerGestureKind::Released,
            PointerGestureKind::DragReleased
        ]
    );
}

#[test]
fn a_press_that_never_became_a_drag_also_survives_leaving() {
    let mut state = PointerGestureState::new().with_drag_threshold(1.0);
    let now = Instant::now();

    state.handle_event(&moved(0.0, 0.0), now, |_| Some("row"));
    state.handle_event(&press(mouse::Button::Left), now, |_| Some("row"));
    state.handle_event(&Event::Mouse(mouse::Event::CursorLeft), now, |_| {
        Some("row")
    });
    let released = state.handle_event(&release(mouse::Button::Left), now, |_| Some("row"));

    assert!(
        released
            .iter()
            .any(|g| g.kind == PointerGestureKind::Released),
        "the press was lost: {released:?}"
    );
}

#[test]
fn losing_window_focus_cancels_an_active_drag() {
    let (mut state, now) = dragging_state();

    let cancelled = state.handle_event(&Event::Window(iced::window::Event::Unfocused), now, |_| {
        Some("row")
    });

    assert_eq!(cancelled[0].kind, PointerGestureKind::DragCancelled);
    // The session is gone, so a later move proposes nothing.
    assert!(state
        .handle_event(&moved(80.0, 0.0), now, |_| Some("row"))
        .is_empty());
}

#[test]
fn captures_current_modifiers() {
    use iced::keyboard;

    let mut state = PointerGestureState::new();
    let now = Instant::now();

    state.handle_event(
        &Event::Keyboard(keyboard::Event::ModifiersChanged(
            keyboard::Modifiers::SHIFT,
        )),
        now,
        |_| Some("row"),
    );
    state.handle_event(&moved(0.0, 0.0), now, |_| Some("row"));
    let gestures = state.handle_event(&press(mouse::Button::Left), now, |_| Some("row"));

    assert_eq!(gestures[0].modifiers, keyboard::Modifiers::SHIFT);
}

#[test]
fn touch_press_emits_pressed() {
    let mut state = PointerGestureState::new();
    let now = Instant::now();
    let gestures = state.handle_event(&touch_press(1, 1.0, 1.0), now, |_| Some("row"));

    assert_eq!(gestures[0].kind, PointerGestureKind::Pressed);
    assert_eq!(gestures[0].button, PointerButton::Primary);
}

#[test]
fn touch_move_crossing_threshold_emits_drag() {
    let mut state = PointerGestureState::new().with_drag_threshold(5.0);
    let now = Instant::now();

    state.handle_event(&touch_press(1, 0.0, 0.0), now, |_| Some("row"));
    assert!(state
        .handle_event(&touch_move(1, 3.0, 0.0), now, |_| Some("row"))
        .is_empty());

    let gestures = state.handle_event(&touch_move(1, 6.0, 0.0), now, |_| Some("row"));

    assert_eq!(
        gestures
            .iter()
            .map(|gesture| gesture.kind)
            .collect::<Vec<_>>(),
        vec![
            PointerGestureKind::DragStarted,
            PointerGestureKind::DragMoved
        ]
    );
}

#[test]
fn touch_lift_emits_release_and_drag_release() {
    let mut state = PointerGestureState::new().with_drag_threshold(1.0);
    let now = Instant::now();

    state.handle_event(&touch_press(1, 0.0, 0.0), now, |_| Some("row"));
    state.handle_event(&touch_move(1, 2.0, 0.0), now, |_| Some("row"));
    let gestures = state.handle_event(&touch_lift(1, 2.0, 0.0), now, |_| Some("row"));

    assert_eq!(
        gestures
            .iter()
            .map(|gesture| gesture.kind)
            .collect::<Vec<_>>(),
        vec![
            PointerGestureKind::Released,
            PointerGestureKind::DragReleased
        ]
    );
}

#[test]
fn touch_lost_cancels_drag() {
    let mut state = PointerGestureState::new().with_drag_threshold(1.0);
    let now = Instant::now();

    state.handle_event(&touch_press(1, 0.0, 0.0), now, |_| Some("row"));
    state.handle_event(&touch_move(1, 2.0, 0.0), now, |_| Some("row"));
    let gestures = state.handle_event(&touch_lost(1, 2.0, 0.0), now, |_| Some("row"));

    assert_eq!(gestures[0].kind, PointerGestureKind::DragCancelled);
}

#[test]
fn mouse_events_are_ignored_during_active_touch() {
    let mut state = PointerGestureState::new().with_drag_threshold(1.0);
    let now = Instant::now();

    state.handle_event(&touch_press(1, 0.0, 0.0), now, |_| Some("row"));

    assert!(state
        .handle_event(&moved(2.0, 0.0), now, |_| Some("row"))
        .is_empty());
    assert!(state
        .handle_event(&press(mouse::Button::Left), now, |_| Some("row"))
        .is_empty());
}

#[test]
fn touch_events_are_ignored_during_mouse_drag() {
    let mut state = PointerGestureState::new().with_drag_threshold(1.0);
    let now = Instant::now();

    state.handle_event(&moved(0.0, 0.0), now, |_| Some("row"));
    state.handle_event(&press(mouse::Button::Left), now, |_| Some("row"));
    state.handle_event(&moved(2.0, 0.0), now, |_| Some("row"));

    assert!(state
        .handle_event(&touch_press(1, 0.0, 0.0), now, |_| Some("row"))
        .is_empty());
    assert!(state
        .handle_event(&touch_move(1, 2.0, 0.0), now, |_| Some("row"))
        .is_empty());
}
