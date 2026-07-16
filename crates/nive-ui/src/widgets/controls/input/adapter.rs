use std::borrow::Cow;

use iced::{
    advanced::{
        input_method, layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    keyboard::{self, key::Named, Key},
    Event, Length, Rectangle, Size, Vector,
};

use crate::Element;

#[derive(Debug, Clone)]
pub(super) enum InputEvent {
    Changed(String),
    Submit,
}

pub(super) struct TextInputAdapter<'a, Message> {
    pub(super) content: Element<'a, InputEvent>,
    pub(super) on_change: Option<Box<dyn Fn(String) -> Message + 'a>>,
    pub(super) on_submit: Option<Message>,
    pub(super) semantic_name: Option<Cow<'a, str>>,
    pub(super) read_only: bool,
    pub(super) disabled: bool,
}

#[derive(Debug)]
struct AdapterState {
    disabled: bool,
}

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for TextInputAdapter<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<AdapterState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(AdapterState {
            disabled: self.disabled,
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<AdapterState>();

        if self.disabled && !state.disabled {
            tree.children[0] = Tree::new(&self.content);
        } else {
            tree.children[0].diff(self.content.as_widget());
        }

        state.disabled = self.disabled;
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        if !self.disabled {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        }
    }

    fn update(
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
        if self.disabled {
            return;
        }

        if self.read_only && is_mutating_event(event) {
            shell.capture_event();
            return;
        }

        if self.read_only
            && matches!(
                event,
                Event::Window(iced::window::Event::RedrawRequested(_))
            )
        {
            return;
        }

        let mut input_events = Vec::new();
        let mut child_shell = Shell::new(&mut input_events);
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            &mut child_shell,
            viewport,
        );

        let captured = child_shell.is_event_captured();
        let redraw = child_shell.redraw_request();
        let layout_invalid = child_shell.is_layout_invalid();
        let widgets_invalid = child_shell.are_widgets_invalid();
        let input_method = child_shell.input_method().clone();
        drop(child_shell);

        if captured {
            shell.capture_event();
        }
        shell.request_redraw_at(redraw);
        if layout_invalid {
            shell.invalidate_layout();
        }
        if widgets_invalid {
            shell.invalidate_widgets();
        }
        if !self.read_only {
            shell.request_input_method(&input_method);
        }

        for event in input_events {
            match event {
                InputEvent::Changed(value) => {
                    if let Some(on_change) = &self.on_change {
                        shell.publish(on_change(value));
                    }
                }
                InputEvent::Submit => {
                    if let Some(on_submit) = &self.on_submit {
                        shell.publish(on_submit.clone());
                    }
                }
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if self.disabled {
            mouse::Interaction::None
        } else {
            self.content.as_widget().mouse_interaction(
                &tree.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let _semantic_name = self.semantic_name.as_deref();
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        let overlay = self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        );
        debug_assert!(
            overlay.is_none(),
            "Iced TextInput unexpectedly produced an overlay"
        );
        None
    }
}

fn is_mutating_event(event: &Event) -> bool {
    match event {
        Event::InputMethod(
            input_method::Event::Opened
            | input_method::Event::Preedit(..)
            | input_method::Event::Commit(_),
        ) => true,
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            modifiers,
            text,
            ..
        }) => {
            if matches!(key, Key::Named(Named::Backspace | Named::Delete)) {
                return true;
            }

            let clipboard_command = modifiers.control() || modifiers.logo();

            if clipboard_command
                && matches!(key, Key::Character(value) if value.eq_ignore_ascii_case("x") || value.eq_ignore_ascii_case("v"))
            {
                return true;
            }

            !clipboard_command
                && text
                    .as_deref()
                    .is_some_and(|text| text.chars().any(|character| !character.is_control()))
        }
        _ => false,
    }
}

#[cfg(test)]
mod adapter_tests {
    use iced::keyboard::{
        key::{Code, Physical},
        Location, Modifiers,
    };

    use super::*;

    fn key_pressed(key: Key, modifiers: Modifiers, text: Option<&str>) -> Event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: Physical::Code(Code::KeyA),
            location: Location::Standard,
            modifiers,
            text: text.map(Into::into),
            repeat: false,
        })
    }

    #[test]
    fn read_only_filter_blocks_mutation_but_not_copy_or_navigation() {
        assert!(is_mutating_event(&key_pressed(
            Key::Character("a".into()),
            Modifiers::NONE,
            Some("a"),
        )));
        assert!(is_mutating_event(&key_pressed(
            Key::Named(Named::Backspace),
            Modifiers::NONE,
            None,
        )));
        assert!(is_mutating_event(&key_pressed(
            Key::Character("v".into()),
            Modifiers::CTRL,
            None,
        )));
        assert!(!is_mutating_event(&key_pressed(
            Key::Character("c".into()),
            Modifiers::CTRL,
            None,
        )));
        assert!(!is_mutating_event(&key_pressed(
            Key::Named(Named::ArrowLeft),
            Modifiers::NONE,
            None,
        )));
    }

    #[test]
    fn read_only_filter_blocks_ime_mutation() {
        assert!(is_mutating_event(&Event::InputMethod(
            input_method::Event::Opened,
        )));
        assert!(is_mutating_event(&Event::InputMethod(
            input_method::Event::Preedit("draft".into(), None),
        )));
        assert!(is_mutating_event(&Event::InputMethod(
            input_method::Event::Commit("value".into()),
        )));
        assert!(!is_mutating_event(&Event::InputMethod(
            input_method::Event::Closed,
        )));
    }
}
