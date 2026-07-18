use std::{any::Any, borrow::Cow, cell::Cell, rc::Rc};

use iced::{
    advanced::{
        input_method, layout, mouse, overlay, renderer,
        widget::{
            operation::{self, Focusable, Operation, Scrollable, TextInput},
            tree, Tree,
        },
        Clipboard, Layout, Shell, Widget,
    },
    keyboard::{self, key::Named, Key},
    widget::Id,
    Event, Length, Rectangle, Size, Vector,
};

use crate::advanced::focus::{FocusState, FocusVisibility};
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
    pub(super) on_blur: Option<Message>,
    pub(super) focus_tracker: Option<Rc<Cell<bool>>>,
    pub(super) semantic_name: Option<Cow<'a, str>>,
    pub(super) read_only: bool,
    pub(super) disabled: bool,
    pub(super) focus_identity: Option<Id>,
}

#[derive(Debug)]
struct AdapterState {
    disabled: bool,
    focus: FocusState,
    visual_focus: bool,
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
            focus: FocusState::new(FocusVisibility::AlwaysWhileActive),
            visual_focus: false,
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<AdapterState>();

        if self.disabled && !state.disabled {
            tree.children[0] = Tree::new(&self.content);
            state.focus.clear();
            state.visual_focus = false;
            self.sync_focus_tracker(false);
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
            let state = tree.state.downcast_mut::<AdapterState>();
            operation.custom(None, layout.bounds(), state);
            state
                .focus
                .expose(operation, self.focus_identity.as_ref(), layout.bounds());
            let mut operation = LogicalFocusOperation {
                operation,
                focus: &mut state.focus,
            };
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut operation,
            );
            state.visual_focus =
                child_has_native_focus(&mut self.content, &mut tree.children[0], layout, renderer);
            self.sync_focus_tracker(state.visual_focus);
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

        let state = tree.state.downcast_mut::<AdapterState>();
        let was_focused =
            child_has_native_focus(&mut self.content, &mut tree.children[0], layout, renderer);

        if is_primary_press(event) {
            if cursor.is_over(layout.bounds()) {
                state.focus.focus_from_pointer();
            }
        } else if matches!(event, Event::Window(iced::window::Event::Unfocused)) {
            state.focus.deactivate();
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

        let is_focused =
            child_has_native_focus(&mut self.content, &mut tree.children[0], layout, renderer);
        if is_focused && !state.focus.is_active() {
            operation::Focusable::focus(&mut state.focus);
        } else if was_focused && !is_focused {
            state.focus.deactivate();
        }
        state.visual_focus = is_focused;
        self.sync_focus_tracker(is_focused);

        if was_focused && !is_focused {
            if let Some(on_blur) = &self.on_blur {
                shell.publish(on_blur.clone());
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

impl<Message> TextInputAdapter<'_, Message> {
    fn sync_focus_tracker(&self, focused: bool) {
        if let Some(tracker) = &self.focus_tracker {
            tracker.set(focused);
        }
    }
}

struct LogicalFocusOperation<'a> {
    operation: &'a mut dyn Operation,
    focus: &'a mut FocusState,
}

impl Operation for LogicalFocusOperation<'_> {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        self.operation.traverse(&mut |operation| {
            operate(&mut LogicalFocusOperation {
                operation,
                focus: self.focus,
            });
        });
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        self.operation.container(id, bounds);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        state: &mut dyn Scrollable,
    ) {
        self.operation
            .scrollable(id, bounds, content_bounds, translation, state);
    }

    fn focusable(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn Focusable) {
        self.operation.focusable(
            id,
            bounds,
            &mut LogicalFocus {
                native: state,
                focus: self.focus,
            },
        );
    }

    fn text_input(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn TextInput) {
        self.operation.text_input(id, bounds, state);
    }

    fn text(&mut self, id: Option<&Id>, bounds: Rectangle, text: &str) {
        self.operation.text(id, bounds, text);
    }

    fn custom(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn Any) {
        self.operation.custom(id, bounds, state);
    }
}

struct LogicalFocus<'a> {
    native: &'a mut dyn Focusable,
    focus: &'a mut FocusState,
}

impl Focusable for LogicalFocus<'_> {
    fn is_focused(&self) -> bool {
        self.native.is_focused() || Focusable::is_focused(self.focus)
    }

    fn focus(&mut self) {
        self.native.focus();
        Focusable::focus(self.focus);
    }

    fn unfocus(&mut self) {
        self.native.unfocus();
        Focusable::unfocus(self.focus);
    }
}

fn child_has_native_focus(
    content: &mut Element<'_, InputEvent>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &iced::Renderer,
) -> bool {
    let mut count = operation::focusable::count();
    content.as_widget_mut().operate(
        tree,
        layout,
        renderer,
        &mut operation::black_box(&mut count),
    );

    matches!(
        count.finish(),
        operation::Outcome::Some(operation::focusable::Count {
            focused: Some(_),
            ..
        })
    )
}

fn is_primary_press(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(iced::touch::Event::FingerPressed { .. })
    )
}

pub(in crate::widgets::controls) fn content_has_visual_focus<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &iced::Renderer,
) -> bool {
    let mut visual_focus = VisualFocus(false);
    content.as_widget_mut().operate(
        tree,
        layout,
        renderer,
        &mut operation::black_box(&mut visual_focus),
    );

    matches!(visual_focus.finish(), operation::Outcome::Some(true))
}

struct VisualFocus(bool);

impl Operation<bool> for VisualFocus {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<bool>)) {
        operate(self);
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Any) {
        if let Some(state) = state.downcast_ref::<AdapterState>() {
            self.0 |= state.visual_focus;
        }
    }

    fn finish(&self) -> operation::Outcome<bool> {
        operation::Outcome::Some(self.0)
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
    fn visual_focus_query_ignores_the_logical_navigation_anchor() {
        let mut operation = VisualFocus(false);
        let mut state = AdapterState {
            disabled: false,
            focus: FocusState::default(),
            visual_focus: false,
        };
        Focusable::focus(&mut state.focus);

        operation.custom(None, Rectangle::default(), &mut state);

        assert!(matches!(
            operation.finish(),
            operation::Outcome::Some(false)
        ));
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
