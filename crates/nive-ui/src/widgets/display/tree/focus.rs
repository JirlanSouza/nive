use std::time::{Duration, Instant};

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree as WidgetTree},
        Clipboard, Layout, Shell, Widget,
    },
    keyboard, Event, Length, Point, Rectangle, Size, Vector,
};

use crate::interaction::keyboard::TypeAhead;
use crate::Element;

use super::keymap::{key_action, TreeClipboardAction, TreeKeyAction, TreeNavKey};
use super::widget::Tree;

mod pointer_drag;

use pointer_drag::update_modifiers;

const TYPE_AHEAD_TIMEOUT: Duration = Duration::from_secs(1);

/// Focus-tracking wrapper around the composed tree row content.
///
/// `Tree` renders as composed widgets with no private mutable state, so it
/// cannot by itself know whether this rendered instance currently holds
/// keyboard focus, nor intercept raw keyboard events. `TreeFocus` holds that
/// per-instance focus flag and the type-ahead buffer, and translates
/// unfocused/uncaptured keyboard events into `TreeEvent`s using the pure
/// navigation helpers on `Tree`.
pub(super) struct TreeFocus<'a, Id, Message> {
    content: Element<'a, Message>,
    tree: Tree<'a, Id, Message>,
}

#[derive(Debug)]
struct TreeFocusState {
    focused: bool,
    type_ahead: TypeAhead,
    modifiers: keyboard::Modifiers,
    drag: PointerDragState,
}

#[derive(Debug, Default)]
struct PointerDragState {
    origin: Option<Point>,
    active: bool,
}

impl Default for TreeFocusState {
    fn default() -> Self {
        Self {
            focused: false,
            type_ahead: TypeAhead::new(TYPE_AHEAD_TIMEOUT),
            modifiers: keyboard::Modifiers::NONE,
            drag: PointerDragState::default(),
        }
    }
}

impl<'a, Id, Message> TreeFocus<'a, Id, Message>
where
    Id: Clone + Ord + 'a,
    Message: Clone + 'a,
{
    pub(super) fn new(tree: Tree<'a, Id, Message>, content: Element<'a, Message>) -> Self {
        Self { content, tree }
    }
}

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
        vec![WidgetTree::new(&self.content)]
    }

    fn diff(&self, tree: &mut WidgetTree) {
        tree.diff_children(&[self.content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut WidgetTree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut WidgetTree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<TreeFocusState>();

        operation.focusable(self.tree.id_ref(), layout.bounds(), state);
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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

        self.content.as_widget_mut().update(
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

        if !state.focused {
            return;
        }

        let Some(action) = key_action(event) else {
            return;
        };

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
        self.content.as_widget().mouse_interaction(
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
        tree: &'b mut WidgetTree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl operation::Focusable for TreeFocusState {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }
}

impl<'a, Id, Message> From<TreeFocus<'a, Id, Message>> for Element<'a, Message>
where
    Id: Clone + Ord + 'a,
    Message: Clone + 'a,
{
    fn from(value: TreeFocus<'a, Id, Message>) -> Self {
        Element::new(value)
    }
}
