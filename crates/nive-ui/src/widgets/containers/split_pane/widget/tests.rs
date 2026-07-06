use iced::{keyboard, Point};

use crate::interaction::{Orientation, PointerButton, PointerGesture, PointerGestureKind};

use super::super::state::{SplitPaneRegion, SplitPaneState};
use super::super::SplitPaneConstraints;

use super::event::{axis_position, constrained_ratio, handle_pointer_gestures};

fn gesture(kind: PointerGestureKind, position: Point) -> PointerGesture<SplitPaneRegion> {
    PointerGesture {
        kind,
        button: PointerButton::Primary,
        region: SplitPaneRegion::Grip,
        position,
        modifiers: keyboard::Modifiers::NONE,
    }
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
    assert_eq!(g.modifiers, keyboard::Modifiers::NONE);
}
