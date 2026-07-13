use iced::{
    keyboard::{
        key::{Code, Named, Physical},
        Key, Location, Modifiers,
    },
    mouse, touch, Event, Point, Size,
};

use crate::interaction::{Orientation, PointerButton, PointerGesture, PointerGestureKind};
use crate::theme::ControlSize;

use super::super::super::state::{SplitPaneRegion, SplitPaneState};
use super::super::super::SplitPaneConstraints;
use super::super::event::{
    axis_position, constrained_ratio, handle_pointer_gestures, has_primary_gesture,
};
use super::support::{Harness, Message};

fn gesture(kind: PointerGestureKind, position: Point) -> PointerGesture<SplitPaneRegion> {
    PointerGesture {
        kind,
        button: PointerButton::Primary,
        region: SplitPaneRegion::Grip,
        position,
        modifiers: Modifiers::NONE,
    }
}

fn touch_press(id: u64, position: Point) -> Event {
    Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(id),
        position,
    })
}

fn touch_move(id: u64, position: Point) -> Event {
    Event::Touch(touch::Event::FingerMoved {
        id: touch::Finger(id),
        position,
    })
}

fn key_pressed(key: Named, code: Code) -> Event {
    let key = Key::Named(key);

    Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: Physical::Code(code),
        location: Location::Standard,
        modifiers: Modifiers::NONE,
        text: None,
        repeat: false,
    })
}

#[test]
fn drag_moved_emits_clamped_ratio() {
    let mut state = SplitPaneState {
        available_length: 100.0,
        ..SplitPaneState::default()
    };
    let gestures = [
        gesture(PointerGestureKind::DragStarted, Point::new(20.0, 0.0)),
        gesture(PointerGestureKind::DragMoved, Point::new(120.0, 0.0)),
    ];
    let ratios = handle_pointer_gestures(
        &mut state,
        &gestures,
        Orientation::Horizontal,
        0.5,
        SplitPaneConstraints::new(0.0, 20.0),
        None,
        false,
    );

    assert_eq!(ratios, vec![0.8]);
}

#[test]
fn double_click_emits_reset() {
    let mut state = SplitPaneState {
        available_length: 100.0,
        ..SplitPaneState::default()
    };
    let ratios = handle_pointer_gestures(
        &mut state,
        &[gesture(
            PointerGestureKind::Clicked { count: 2 },
            Point::new(0.0, 0.0),
        )],
        Orientation::Horizontal,
        0.2,
        SplitPaneConstraints::new(0.0, 0.0),
        None,
        false,
    );

    assert_eq!(ratios, vec![0.5]);
}

#[test]
fn locked_emits_nothing() {
    let mut state = SplitPaneState {
        available_length: 100.0,
        ..SplitPaneState::default()
    };
    let ratios = handle_pointer_gestures(
        &mut state,
        &[gesture(
            PointerGestureKind::DragMoved,
            Point::new(100.0, 0.0),
        )],
        Orientation::Horizontal,
        0.5,
        SplitPaneConstraints::new(0.0, 0.0),
        None,
        true,
    );

    assert!(ratios.is_empty());
}

#[test]
fn cancel_keeps_last_ratio() {
    let mut state = SplitPaneState {
        available_length: 100.0,
        ..SplitPaneState::default()
    };
    let ratios = handle_pointer_gestures(
        &mut state,
        &[
            gesture(PointerGestureKind::DragStarted, Point::new(0.0, 0.0)),
            gesture(PointerGestureKind::DragMoved, Point::new(10.0, 0.0)),
            gesture(PointerGestureKind::DragCancelled, Point::new(10.0, 0.0)),
        ],
        Orientation::Horizontal,
        0.5,
        SplitPaneConstraints::new(0.0, 0.0),
        None,
        false,
    );

    assert_eq!(ratios, vec![0.6]);
    assert!(state.drag.is_none());
}

#[test]
fn focus_and_unfocus_toggle_state() {
    use iced::advanced::widget::operation;

    let mut state = SplitPaneState::default();

    operation::Focusable::focus(&mut state);
    assert!(operation::Focusable::is_focused(&state));

    operation::Focusable::unfocus(&mut state);
    assert!(!operation::Focusable::is_focused(&state));
}

#[test]
fn press_on_grip_focuses() {
    let mut state = SplitPaneState::default();

    handle_pointer_gestures(
        &mut state,
        &[gesture(PointerGestureKind::Pressed, Point::new(0.0, 0.0))],
        Orientation::Horizontal,
        0.5,
        SplitPaneConstraints::new(0.0, 0.0),
        None,
        false,
    );

    assert!(state.focused);
}

#[test]
fn axis_position_uses_orientation_axis() {
    assert_eq!(
        axis_position(Orientation::Horizontal, Point::new(7.0, 4.0)),
        7.0
    );
    assert_eq!(
        axis_position(Orientation::Vertical, Point::new(7.0, 4.0)),
        4.0
    );
}

#[test]
fn constrained_ratio_applies_snap_and_clamp() {
    let constraints = SplitPaneConstraints::new(10.0, 10.0);

    assert_eq!(constrained_ratio(0.05, constraints, 100.0, None), 0.1);
    assert_eq!(constrained_ratio(0.97, constraints, 100.0, None), 0.9);
}

#[test]
fn gesture_helper_propagates_modifiers_none() {
    let g = gesture(PointerGestureKind::Pressed, Point::ORIGIN);

    assert_eq!(g.button, PointerButton::Primary);
    assert_eq!(g.modifiers, Modifiers::NONE);
}

#[test]
fn only_primary_gestures_claim_splitter_input() {
    let secondary = PointerGesture {
        button: PointerButton::Secondary,
        ..gesture(PointerGestureKind::Pressed, Point::ORIGIN)
    };

    assert!(!has_primary_gesture(&[secondary]));
    assert!(has_primary_gesture(&[gesture(
        PointerGestureKind::Pressed,
        Point::ORIGIN,
    )]));
}

#[test]
fn primary_pointer_inside_hit_target_precedes_adjacent_children() {
    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let mut harness = Harness::new(
            orientation,
            ControlSize::Sm,
            Size::new(200.0, 120.0),
            0.5,
            false,
        );
        let point = harness.hit_only_point();

        assert!(harness.child_bounds(0).contains(point));

        let result = harness.press(mouse::Button::Left, point);

        assert!(result.captured);
        assert!(result.messages.is_empty());
        assert!(harness.state().focused);
    }
}

#[test]
fn touch_inside_hit_target_outside_seam_resizes_and_focuses() {
    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let mut harness = Harness::new(
            orientation,
            ControlSize::Sm,
            Size::new(200.0, 120.0),
            0.5,
            false,
        );
        let point = harness.hit_only_point();
        let pressed = harness.update(touch_press(1, point));

        assert!(pressed.captured);
        assert!(pressed.messages.is_empty());
        assert!(harness.state().focused);

        let moved = match orientation {
            Orientation::Horizontal => Point::new(point.x + 10.0, point.y),
            Orientation::Vertical => Point::new(point.x, point.y + 10.0),
        };
        let result = harness.update(touch_move(1, moved));

        assert!(result.captured);
        assert!(matches!(result.messages.as_slice(), [Message::Ratio(_)]));
    }
}

#[test]
fn outside_non_primary_and_locked_input_continue_to_children() {
    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let mut outside = Harness::new(
            orientation,
            ControlSize::Sm,
            Size::new(200.0, 120.0),
            0.5,
            false,
        );
        let bounds = outside.bounds();
        let point = Point::new(bounds.x + 1.0, bounds.y + 1.0);
        let result = outside.press(mouse::Button::Left, point);

        assert!(!result.captured);
        assert_eq!(result.messages, vec![Message::Leading, Message::Trailing]);

        let mut non_primary = Harness::new(
            orientation,
            ControlSize::Sm,
            Size::new(200.0, 120.0),
            0.5,
            false,
        );
        let point = non_primary.hit_only_point();
        let result = non_primary.press(mouse::Button::Right, point);

        assert!(!result.captured);
        assert_eq!(result.messages, vec![Message::Leading, Message::Trailing]);

        let mut locked = Harness::new(
            orientation,
            ControlSize::Sm,
            Size::new(200.0, 120.0),
            0.5,
            true,
        );
        let point = locked.hit_only_point();
        let result = locked.press(mouse::Button::Left, point);

        assert!(!result.captured);
        assert_eq!(result.messages, vec![Message::Leading, Message::Trailing]);
        assert!(!locked.state().focused);
    }
}

#[test]
fn resize_cursor_uses_hit_target_and_respects_locking() {
    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let mut harness = Harness::new(
            orientation,
            ControlSize::Sm,
            Size::new(200.0, 120.0),
            0.5,
            false,
        );
        let point = harness.hit_only_point();
        let _ = harness.move_to(point);

        let expected = match orientation {
            Orientation::Horizontal => mouse::Interaction::ResizingColumn,
            Orientation::Vertical => mouse::Interaction::ResizingRow,
        };
        assert_eq!(harness.mouse_interaction(), expected);

        let bounds = harness.bounds();
        let _ = harness.move_to(Point::new(bounds.x + 1.0, bounds.y + 1.0));
        assert_eq!(harness.mouse_interaction(), mouse::Interaction::None);

        let mut locked = Harness::new(
            orientation,
            ControlSize::Sm,
            Size::new(200.0, 120.0),
            0.5,
            true,
        );
        let _ = locked.move_to(locked.hit_only_point());
        assert_eq!(locked.mouse_interaction(), mouse::Interaction::None);
    }
}

#[test]
fn keyboard_resize_uses_focused_splitter_and_layout_thickness() {
    let mut harness = Harness::new(
        Orientation::Horizontal,
        ControlSize::Sm,
        Size::new(200.0, 120.0),
        0.5,
        false,
    );
    let _ = harness.press(mouse::Button::Left, harness.hit_only_point());

    let result = harness.update(key_pressed(Named::ArrowRight, Code::ArrowRight));

    assert!(result.captured);
    assert!(
        matches!(result.messages.as_slice(), [Message::Ratio(ratio)] if (*ratio - 0.51).abs() < f32::EPSILON)
    );
    assert_eq!(harness.state().available_length, 199.0);
}

#[test]
fn primary_press_outside_hit_releases_focus_before_forwarding() {
    let mut harness = Harness::new(
        Orientation::Horizontal,
        ControlSize::Sm,
        Size::new(200.0, 120.0),
        0.5,
        false,
    );
    let _ = harness.press(mouse::Button::Left, harness.hit_only_point());
    assert!(harness.state().focused);

    let bounds = harness.bounds();
    let result = harness.press(
        mouse::Button::Left,
        Point::new(bounds.x + 1.0, bounds.y + 1.0),
    );

    assert!(!result.captured);
    assert_eq!(result.messages, vec![Message::Leading, Message::Trailing]);
    assert!(!harness.state().focused);
}
