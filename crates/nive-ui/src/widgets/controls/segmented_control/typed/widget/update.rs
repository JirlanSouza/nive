use iced::{
    advanced::{mouse, widget::Tree, Clipboard, Layout, Shell},
    keyboard::{self, key},
    touch, Event, Rectangle,
};

use crate::widgets::controls::segmented_control::typed::{SegmentedControl, SegmentedState};

impl<'a, T, Message> SegmentedControl<'a, T, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    #[allow(clippy::too_many_arguments)]
    pub(super) fn update_impl(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for (index, (tree, child_layout)) in
            tree.children.iter_mut().zip(layout.children()).enumerate()
        {
            self.contents[index].as_widget_mut().update(
                tree,
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        let interactive = self.interactive();
        let state = tree.state.downcast_mut::<SegmentedState>();
        if !interactive {
            state.focus.clear();
            state.focused_index = None;
            state.pressed_index = None;
            state.touch = None;
            return;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let hit = cursor
                    .position()
                    .and_then(|point| self.item_at(state, layout, point));
                if let Some(index) = hit.filter(|index| !self.options[*index].disabled) {
                    state.focus.focus_from_pointer();
                    state.focused_index = Some(index);
                    state.pressed_index = Some(index);
                    shell.capture_event();
                } else {
                    state.focus.deactivate();
                    state.pressed_index = None;
                }
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let hit = cursor
                    .position()
                    .and_then(|point| self.item_at(state, layout, point));
                let activates = state.pressed_index.filter(|index| Some(*index) == hit);
                state.pressed_index = None;
                if let Some(index) = activates {
                    self.publish_if_changed(index, shell);
                    shell.capture_event();
                }
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::CursorLeft) => {
                state.pressed_index = None;
                shell.request_redraw();
            }
            Event::Touch(touch::Event::FingerPressed { id, position }) => {
                if let Some(index) = self
                    .item_at(state, layout, *position)
                    .filter(|index| !self.options[*index].disabled)
                {
                    state.focus.focus_from_pointer();
                    state.focused_index = Some(index);
                    state.touch = Some((*id, index));
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Touch(touch::Event::FingerLifted { id, position }) => {
                let hit = self.item_at(state, layout, *position);
                let activates = state
                    .touch
                    .filter(|(finger, index)| finger == id && Some(*index) == hit)
                    .map(|(_, index)| index);
                if state.touch.is_some_and(|(finger, _)| finger == *id) {
                    state.touch = None;
                }
                if let Some(index) = activates {
                    self.publish_if_changed(index, shell);
                    shell.capture_event();
                }
                shell.request_redraw();
            }
            Event::Touch(touch::Event::FingerLost { id, .. })
                if state.touch.is_some_and(|(finger, _)| finger == *id) =>
            {
                state.touch = None;
                shell.request_redraw();
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(named),
                repeat: false,
                ..
            }) if state.focus.is_active() => {
                let focus_key = matches!(
                    named,
                    key::Named::ArrowLeft
                        | key::Named::ArrowRight
                        | key::Named::Home
                        | key::Named::End
                        | key::Named::Space
                        | key::Named::Enter
                );
                if focus_key {
                    state.focus.focus_from_keyboard();
                }
                let target = match named {
                    key::Named::ArrowLeft => self.move_bounded(state, -1),
                    key::Named::ArrowRight => self.move_bounded(state, 1),
                    key::Named::Home => self.options.iter().position(|option| !option.disabled),
                    key::Named::End => self.options.iter().rposition(|option| !option.disabled),
                    key::Named::Space | key::Named::Enter => self.reconciled_focus(state),
                    _ => None,
                };
                if let Some(index) = target {
                    state.focused_index = Some(index);
                    self.publish_if_changed(index, shell);
                    shell.capture_event();
                    shell.request_redraw();
                } else if focus_key {
                    shell.request_redraw();
                }
            }
            Event::Window(iced::window::Event::Unfocused) => {
                state.focus.deactivate();
                state.pressed_index = None;
                state.touch = None;
                shell.request_redraw();
            }
            _ => {}
        }
    }
}
