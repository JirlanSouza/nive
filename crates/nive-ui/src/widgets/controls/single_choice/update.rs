use iced::{
    advanced::{mouse, widget::Tree, Clipboard, Layout, Shell, Widget},
    keyboard::{self, key},
    touch, Event, Rectangle,
};

use crate::widgets::controls::single_choice::{PressSource, SingleChoice, SingleChoiceState};

impl<Message> SingleChoice<'_, Message>
where
    Message: Clone,
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
        let mut content = self.content();
        content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let interactive = self.on_activate.is_some() && !self.disabled;
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<SingleChoiceState>();

        if !interactive {
            if state.focus.is_active() || state.press.is_some() {
                state.focus.clear();
                state.press = None;
                shell.request_redraw();
            }
            return;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if cursor.is_over(bounds) {
                    if self.register_focus {
                        state.focus.focus_from_pointer();
                    }
                    state.press = Some(PressSource::Pointer);
                    shell.capture_event();
                } else {
                    if self.register_focus {
                        state.focus.deactivate();
                    }
                    state.press = None;
                }
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let activates = state.press == Some(PressSource::Pointer) && cursor.is_over(bounds);
                state.press = None;
                if activates {
                    shell.publish(
                        self.on_activate
                            .clone()
                            .expect("interactive choice message"),
                    );
                    shell.capture_event();
                }
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::CursorLeft) if state.press == Some(PressSource::Pointer) => {
                state.press = None;
                shell.request_redraw();
            }
            Event::Touch(touch::Event::FingerPressed { id, position }) => {
                if bounds.contains(*position) {
                    if self.register_focus {
                        state.focus.focus_from_pointer();
                    }
                    state.press = Some(PressSource::Touch(*id));
                    shell.capture_event();
                    shell.request_redraw();
                } else {
                    if self.register_focus {
                        state.focus.deactivate();
                    }
                }
            }
            Event::Touch(touch::Event::FingerLifted { id, position }) => {
                let activates =
                    state.press == Some(PressSource::Touch(*id)) && bounds.contains(*position);
                if state.press == Some(PressSource::Touch(*id)) {
                    state.press = None;
                }
                if activates {
                    shell.publish(
                        self.on_activate
                            .clone()
                            .expect("interactive choice message"),
                    );
                    shell.capture_event();
                }
                shell.request_redraw();
            }
            Event::Touch(touch::Event::FingerLost { id, .. })
                if state.press == Some(PressSource::Touch(*id)) =>
            {
                state.press = None;
                shell.request_redraw();
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Space),
                repeat: false,
                ..
            }) if state.focus.is_active() => {
                state.focus.focus_from_keyboard();
                state.press = Some(PressSource::Space);
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Keyboard(keyboard::Event::KeyReleased {
                key: keyboard::Key::Named(key::Named::Space),
                ..
            }) if state.focus.is_active() && state.press == Some(PressSource::Space) => {
                state.press = None;
                shell.publish(
                    self.on_activate
                        .clone()
                        .expect("interactive choice message"),
                );
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Window(iced::window::Event::Unfocused) => {
                state.press = None;
                state.focus.deactivate();
                shell.request_redraw();
            }
            _ => {}
        }
    }
}
