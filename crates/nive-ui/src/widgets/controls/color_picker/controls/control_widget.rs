use iced::{
    advanced::{
        layout, mouse,
        widget::{operation, tree, Tree},
        Layout, Shell,
    },
    Event, Length, Rectangle, Size,
};

use super::super::event::{ColorPickerControl, ColorPickerEvent};
use super::{control_state::ControlState, drag::pressed_inside};

pub(super) fn tag() -> tree::Tag {
    ControlState::tag()
}

pub(super) fn state() -> tree::State {
    ControlState::new_state()
}

pub(super) fn fixed_layout(limits: &layout::Limits, size: Size<Length>) -> layout::Node {
    layout::Node::new(limits.resolve(size.width, size.height, Size::ZERO))
}

pub(super) fn focus_on_press(
    event: &Event,
    bounds: Rectangle,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, ColorPickerEvent>,
    control: ColorPickerControl,
) {
    if pressed_inside(event, bounds, cursor) {
        shell.publish(ColorPickerEvent::FocusControl(control));
    }
}

pub(super) fn operate_focus(
    tree: &mut Tree,
    layout: Layout<'_>,
    disabled: bool,
    operation: &mut dyn operation::Operation,
    control: ColorPickerControl,
) {
    let state = tree.state.downcast_mut::<ControlState>();
    if disabled {
        state.focus().clear();
        return;
    }
    let id = control.id();

    state
        .focus()
        .register(operation, Some(&id), layout.bounds());
}

pub(super) fn handle_keyboard(
    disabled: bool,
    state: &ControlState,
    event: &Event,
    shell: &mut Shell<'_, ColorPickerEvent>,
    message: impl FnOnce(&Event) -> Option<ColorPickerEvent>,
) {
    if disabled || !state.is_active() {
        return;
    }

    let Some(message) = message(event) else {
        return;
    };

    shell.publish(message);
    shell.capture_event();
    shell.request_redraw();
}
