use iced::{
    advanced::{mouse, widget::Tree, Layout, Shell},
    touch, Event, Point, Rectangle,
};

use super::super::event::ColorPickerEvent;
use super::control_state::ControlState;

pub(super) fn handle_drag(
    event: &Event,
    bounds: Rectangle,
    cursor: mouse::Cursor,
    state: &mut ControlState,
    disabled: bool,
    shell: &mut Shell<'_, ColorPickerEvent>,
    message: impl Fn(Point) -> ColorPickerEvent,
) {
    if disabled {
        return;
    }

    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
            if let Some(position) = cursor.position_over(bounds) {
                state.set_dragging(true);
                publish_drag(bounds, position, shell, message);
            }
        }
        Event::Mouse(mouse::Event::CursorMoved { position }) if state.is_dragging() => {
            publish_drag(bounds, *position, shell, message);
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.is_dragging() => {
            state.set_dragging(false);
            shell.capture_event();
            shell.request_redraw();
        }
        Event::Touch(touch::Event::FingerPressed { position, .. })
            if bounds.contains(*position) =>
        {
            state.set_dragging(true);
            publish_drag(bounds, *position, shell, message);
        }
        Event::Touch(touch::Event::FingerMoved { position, .. }) if state.is_dragging() => {
            publish_drag(bounds, *position, shell, message);
        }
        Event::Touch(touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. })
            if state.is_dragging() =>
        {
            state.set_dragging(false);
            shell.capture_event();
            shell.request_redraw();
        }
        _ => {}
    }
}

pub(super) fn pressed_inside(event: &Event, bounds: Rectangle, cursor: mouse::Cursor) -> bool {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
            cursor.position_over(bounds).is_some()
        }
        Event::Touch(touch::Event::FingerPressed { position, .. }) => bounds.contains(*position),
        _ => false,
    }
}

pub(super) fn interaction(
    disabled: bool,
    tree: &Tree,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    active: mouse::Interaction,
) -> mouse::Interaction {
    if disabled {
        return mouse::Interaction::None;
    }

    let state = tree.state.downcast_ref::<ControlState>();

    if state.is_dragging() {
        mouse::Interaction::Grabbing
    } else if cursor.is_over(layout.bounds()) {
        active
    } else {
        mouse::Interaction::None
    }
}

fn publish_drag(
    bounds: Rectangle,
    position: Point,
    shell: &mut Shell<'_, ColorPickerEvent>,
    message: impl Fn(Point) -> ColorPickerEvent,
) {
    let local = Point::new(
        (position.x - bounds.x).clamp(0.0, bounds.width),
        (position.y - bounds.y).clamp(0.0, bounds.height),
    );

    shell.publish(message(local));
    shell.capture_event();
    shell.request_redraw();
}
