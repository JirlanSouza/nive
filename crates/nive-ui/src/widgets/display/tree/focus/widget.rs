use std::time::Instant;

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree as WidgetTree},
        Clipboard, Layout, Shell, Widget,
    },
    Event, Length, Rectangle, Size, Vector,
};

use super::{pointer_drag::update_modifiers, TreeFocus, TreeFocusState};
use crate::widgets::display::tree::keymap::{
    key_action, TreeClipboardAction, TreeKeyAction, TreeNavKey,
};

impl<'a, Id, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for TreeFocus<'a, Id, Message>
where
    Id: Clone + Ord + 'a,
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TreeFocusState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TreeFocusState::default())
    }

    fn children(&self) -> Vec<WidgetTree> {
        vec![WidgetTree::new(&self.inactive_content)]
    }

    fn diff(&self, tree: &mut WidgetTree) {
        let focus_visible = tree
            .state
            .downcast_ref::<TreeFocusState>()
            .focus
            .is_focus_visible();
        tree.diff_children(&[self.content(focus_visible).as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        self.inactive_content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.inactive_content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut WidgetTree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let focus_visible = tree
            .state
            .downcast_ref::<TreeFocusState>()
            .focus
            .is_focus_visible();
        self.content_mut(focus_visible).as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            limits,
        )
    }

    fn operate(
        &mut self,
        tree: &mut WidgetTree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<TreeFocusState>();

        state
            .focus
            .register(operation, self.tree.id_ref(), layout.bounds());
        let focus_visible = state.focus.is_focus_visible();
        self.content_mut(focus_visible).as_widget_mut().operate(
            &mut tree.children[0],
            layout,
            renderer,
            operation,
        );
    }

    fn update(
        &mut self,
        tree: &mut WidgetTree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<TreeFocusState>();
        update_modifiers(state, event);

        match event {
            Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                if cursor.is_over(layout.bounds()) {
                    state.focus.focus_from_pointer();
                } else {
                    state.focus.deactivate();
                }
            }
            Event::Touch(iced::touch::Event::FingerPressed { position, .. }) => {
                if layout.bounds().contains(*position) {
                    state.focus.focus_from_pointer();
                } else {
                    state.focus.deactivate();
                }
            }
            Event::Window(iced::window::Event::Unfocused) => state.focus.deactivate(),
            _ => {}
        }

        if let Some(message) = self.pointer_context_message(event, layout, cursor) {
            shell.publish(message);
            shell.capture_event();
            shell.invalidate_layout();
            shell.request_redraw();
            return;
        }

        if let Some(message) = self.pointer_drag_message(state, event, layout, cursor) {
            shell.publish(message);
            shell.capture_event();
            shell.invalidate_layout();
            shell.request_redraw();
            return;
        }

        let focus_visible = state.focus.is_focus_visible();
        self.content_mut(focus_visible).as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if shell.is_event_captured() {
            return;
        }

        if !state.focus.is_active() {
            return;
        }

        let Some(action) = key_action(event) else {
            return;
        };
        state.focus.focus_from_keyboard();

        let tree_state = self.tree.state_or_default();

        let tree_event = match action {
            TreeKeyAction::Navigate(nav, modifiers) => {
                state.type_ahead.reset();

                match nav {
                    TreeNavKey::Up => self.tree.handle_key_up_tree(&tree_state, modifiers),
                    TreeNavKey::Down => self.tree.handle_key_down_tree(&tree_state, modifiers),
                    TreeNavKey::Left => self.tree.handle_key_left_tree(&tree_state),
                    TreeNavKey::Right => self.tree.handle_key_right_tree(&tree_state),
                    TreeNavKey::Home => self.tree.handle_key_home_tree(&tree_state, modifiers),
                    TreeNavKey::End => self.tree.handle_key_end_tree(&tree_state, modifiers),
                    TreeNavKey::PageUp => self.tree.handle_key_page_up_tree(&tree_state, modifiers),
                    TreeNavKey::PageDown => {
                        self.tree.handle_key_page_down_tree(&tree_state, modifiers)
                    }
                }
            }
            TreeKeyAction::TypeAhead(ch) => {
                if self.tree.type_ahead_enabled() {
                    let buffer = state.type_ahead.push(ch, Instant::now()).to_owned();
                    self.tree.handle_type_ahead_tree(&tree_state, &buffer)
                } else {
                    None
                }
            }
            TreeKeyAction::Escape => {
                state.type_ahead.reset();
                self.tree.handle_escape_tree(&tree_state)
            }
            TreeKeyAction::Context => {
                state.type_ahead.reset();
                let Some(message) = self.tree.context_request_event_for_keyboard(&tree_state)
                else {
                    return;
                };
                shell.publish(message);
                shell.capture_event();
                shell.invalidate_layout();
                shell.request_redraw();
                return;
            }
            TreeKeyAction::Clipboard(action) => match action {
                TreeClipboardAction::Copy => self.tree.handle_copy_tree(&tree_state),
                TreeClipboardAction::Cut => self.tree.handle_cut_tree(&tree_state),
                TreeClipboardAction::Paste => self.tree.handle_paste_tree(&tree_state),
            },
        };

        let Some(tree_event) = tree_event else {
            return;
        };

        let Some(message) = self.tree.publish(tree_event) else {
            return;
        };

        shell.publish(message);
        shell.capture_event();
        shell.invalidate_layout();
        shell.request_redraw();
    }

    fn mouse_interaction(
        &self,
        tree: &WidgetTree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let focus_visible = tree
            .state
            .downcast_ref::<TreeFocusState>()
            .focus
            .is_focus_visible();
        self.content(focus_visible).as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &WidgetTree,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let focus_visible = tree
            .state
            .downcast_ref::<TreeFocusState>()
            .focus
            .is_focus_visible();
        self.content(focus_visible).as_widget().draw(
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
        tree: &'b mut WidgetTree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        let focus_visible = tree
            .state
            .downcast_ref::<TreeFocusState>()
            .focus
            .is_focus_visible();
        self.content_mut(focus_visible).as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}
