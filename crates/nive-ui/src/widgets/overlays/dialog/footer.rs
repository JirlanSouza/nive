use std::borrow::Cow;

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    widget::row,
    Event, Length, Rectangle, Size, Vector,
};

use crate::theme::{self, ControlSize, GapRole};
use crate::widgets::controls::button;
use crate::{Element, Renderer, Theme};

/// Simple full-width Dialog footer slot. Prefer [`DialogActionFooter`] for
/// canonical action rows; use `DialogFooter` for footer content that is not
/// an action group (e.g. a single custom control).
pub struct DialogFooter<'a, Message> {
    content: Element<'a, Message>,
}

impl<'a, Message> DialogFooter<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
        }
    }

    fn into_element(self) -> Element<'a, Message> {
        row![self.content].width(Length::Fill).into()
    }
}

impl<'a, Message> From<DialogFooter<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(footer: DialogFooter<'a, Message>) -> Self {
        footer.into_element()
    }
}

/// Typed semantic role of a [`DialogAction`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DialogActionRole {
    Cancel,
    Secondary,
    Primary,
    Destructive,
}

/// A single labeled Dialog action with a typed role.
///
/// Preceding actions (rendered before the terminal action) must be
/// [`DialogActionRole::Cancel`] or [`DialogActionRole::Secondary`]; the
/// terminal action must be [`DialogActionRole::Primary`] or
/// [`DialogActionRole::Destructive`] and is constructed through
/// [`DialogTerminalAction`] so it cannot be mis-typed as a preceding action.
pub struct DialogAction<'a, Message> {
    label: Cow<'a, str>,
    role: DialogActionRole,
    message: Message,
    disabled: bool,
    id: Option<iced::widget::Id>,
}

impl<'a, Message> DialogAction<'a, Message> {
    fn new(role: DialogActionRole, label: impl Into<Cow<'a, str>>, message: Message) -> Self {
        Self {
            label: label.into(),
            role,
            message,
            disabled: false,
            id: None,
        }
    }

    pub fn cancel(label: impl Into<Cow<'a, str>>, message: Message) -> Self {
        Self::new(DialogActionRole::Cancel, label, message)
    }

    pub fn secondary(label: impl Into<Cow<'a, str>>, message: Message) -> Self {
        Self::new(DialogActionRole::Secondary, label, message)
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable [`iced::widget::Id`], e.g. so a
    /// [`crate::widgets::overlays::DialogInitialFocus::Target`] can name
    /// this action directly.
    pub fn id(mut self, id: impl Into<iced::widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn role(&self) -> DialogActionRole {
        self.role
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn is_safe_preceding_role(&self) -> bool {
        matches!(
            self.role,
            DialogActionRole::Cancel | DialogActionRole::Secondary
        )
    }
}

impl<'a, Message: Clone> Clone for DialogAction<'a, Message> {
    fn clone(&self) -> Self {
        Self {
            label: self.label.clone(),
            role: self.role,
            message: self.message.clone(),
            disabled: self.disabled,
            id: self.id.clone(),
        }
    }
}

/// A terminal (final) Dialog action: always `Primary` or `Destructive`,
/// constructed only through this type so [`DialogActionFooter`] can enforce
/// the bounded action model at the type level.
pub struct DialogTerminalAction<'a, Message>(DialogAction<'a, Message>);

impl<'a, Message> DialogTerminalAction<'a, Message> {
    pub fn primary(label: impl Into<Cow<'a, str>>, message: Message) -> Self {
        Self(DialogAction::new(DialogActionRole::Primary, label, message))
    }

    pub fn destructive(label: impl Into<Cow<'a, str>>, message: Message) -> Self {
        Self(DialogAction::new(
            DialogActionRole::Destructive,
            label,
            message,
        ))
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.0.disabled = disabled;
        self
    }

    /// Sets a stable [`iced::widget::Id`], e.g. so a
    /// [`crate::widgets::overlays::DialogInitialFocus::Target`] can name
    /// this action directly.
    pub fn id(mut self, id: impl Into<iced::widget::Id>) -> Self {
        self.0.id = Some(id.into());
        self
    }
}

/// Construction error for a dynamically assembled [`DialogActionFooter`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DialogActionFooterError {
    /// More than two preceding actions were supplied.
    TooManyPrecedingActions(usize),
    /// A preceding action used a role other than Cancel or Secondary.
    InvalidPrecedingRole(DialogActionRole),
}

impl std::fmt::Display for DialogActionFooterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyPrecedingActions(count) => write!(
                f,
                "DialogActionFooter admits at most two preceding actions, got {count}"
            ),
            Self::InvalidPrecedingRole(role) => write!(
                f,
                "DialogActionFooter preceding actions must be Cancel or Secondary, got {role:?}"
            ),
        }
    }
}

impl std::error::Error for DialogActionFooterError {}

/// Canonical bounded Dialog action footer: at most two preceding
/// Cancel/Secondary actions plus one required terminal Primary or
/// Destructive action, with optional leading status/help content and
/// measured responsive reflow.
pub struct DialogActionFooter<'a, Message> {
    status: Option<Element<'a, Message>>,
    preceding: Vec<DialogAction<'a, Message>>,
    terminal: DialogAction<'a, Message>,
}

impl<'a, Message> DialogActionFooter<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(terminal: DialogTerminalAction<'a, Message>) -> Self {
        Self {
            status: None,
            preceding: Vec::new(),
            terminal: terminal.0,
        }
    }

    pub fn with_one(
        preceding: DialogAction<'a, Message>,
        terminal: DialogTerminalAction<'a, Message>,
    ) -> Self {
        Self {
            status: None,
            preceding: vec![preceding],
            terminal: terminal.0,
        }
    }

    pub fn with_two(
        preceding: [DialogAction<'a, Message>; 2],
        terminal: DialogTerminalAction<'a, Message>,
    ) -> Self {
        Self {
            status: None,
            preceding: preceding.into(),
            terminal: terminal.0,
        }
    }

    /// Builds a footer from a dynamically sized preceding-action collection.
    /// Returns a typed error instead of truncating when the collection is
    /// invalid.
    pub fn try_from_parts(
        preceding: Vec<DialogAction<'a, Message>>,
        terminal: DialogTerminalAction<'a, Message>,
    ) -> Result<Self, DialogActionFooterError> {
        if preceding.len() > 2 {
            return Err(DialogActionFooterError::TooManyPrecedingActions(
                preceding.len(),
            ));
        }

        for action in &preceding {
            if !action.is_safe_preceding_role() {
                return Err(DialogActionFooterError::InvalidPrecedingRole(action.role));
            }
        }

        Ok(Self {
            status: None,
            preceding,
            terminal: terminal.0,
        })
    }

    pub fn status(mut self, status: impl Into<Element<'a, Message>>) -> Self {
        self.status = Some(status.into());
        self
    }

    /// The terminal action's message, published on unconsumed non-repeated
    /// Enter when the terminal action is `Primary` and enabled. Never
    /// exposed for a `Destructive` terminal action.
    pub(crate) fn enter_default_message(&self) -> Option<&Message> {
        if self.terminal.disabled || self.terminal.role != DialogActionRole::Primary {
            return None;
        }

        Some(&self.terminal.message)
    }

    #[cfg(test)]
    fn all_actions(&self) -> impl Iterator<Item = &DialogAction<'a, Message>> {
        self.preceding.iter().chain(std::iter::once(&self.terminal))
    }

    fn into_element(self) -> Element<'a, Message> {
        let enter_default = self.enter_default_message().cloned();
        let mut actions: Vec<Element<'a, Message>> =
            self.preceding.into_iter().map(action_button).collect();
        // Tagged so a modal host's initial-focus resolution can recognize
        // and skip the terminal action (`DialogInitialFocus::First` must
        // never land on it, Primary or Destructive) without needing to know
        // Dialog's internal anatomy.
        actions.push(TerminalActionMarker::wrap(action_button(self.terminal)));

        DialogActionFooterWidget {
            status: self.status,
            actions,
            enter_default,
        }
        .into()
    }
}

/// Zero-sized tag pushed to any [`iced::advanced::widget::Operation`] that
/// reaches the terminal action, via [`iced::advanced::widget::Operation::custom`].
pub(crate) struct TerminalActionTag;

/// Transparent single-child wrapper that announces [`TerminalActionTag`]
/// before delegating `operate()` to its inner action button. Every other
/// [`Widget`] method passes through unchanged.
struct TerminalActionMarker<'a, Message>(Element<'a, Message>);

impl<'a, Message> TerminalActionMarker<'a, Message>
where
    Message: Clone + 'a,
{
    fn wrap(inner: Element<'a, Message>) -> Element<'a, Message> {
        Element::new(Self(inner))
    }
}

impl<'a, Message> Widget<Message, Theme, Renderer> for TerminalActionMarker<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        self.0.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.0.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.0.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.0.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.0.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.0.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.0.as_widget_mut().layout(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        operation.custom(None, layout.bounds(), &mut TerminalActionTag);
        self.0
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.0.as_widget_mut().update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.0
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.0.as_widget().draw(
            tree,
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
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.0
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

/// A non-repeated Enter press not already consumed by a body control,
/// editor, or nested overlay. Never matches on a repeated (held-key) press.
fn is_unconsumed_confirm_enter(event: &Event) -> bool {
    matches!(
        event,
        Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter),
            repeat: false,
            ..
        })
    )
}

fn action_button<'a, Message: Clone + 'a>(
    action: DialogAction<'a, Message>,
) -> Element<'a, Message> {
    let mut button = match action.role {
        DialogActionRole::Cancel | DialogActionRole::Secondary => button::secondary(action.label),
        DialogActionRole::Primary => button::primary(action.label),
        DialogActionRole::Destructive => button::destructive(action.label),
    }
    .size(ControlSize::Md)
    .disabled(action.disabled)
    .on_press(action.message);

    if let Some(id) = action.id {
        button = button.id(id);
    }

    button.into()
}

struct DialogActionFooterWidget<'a, Message> {
    status: Option<Element<'a, Message>>,
    actions: Vec<Element<'a, Message>>,
    enter_default: Option<Message>,
}

enum ReflowLayout {
    /// Status leading, actions trailing, on one row.
    SingleRow,
    /// Status above a complete actions row.
    StackedStatus,
    /// Status (if any) above each action stacked full-width.
    StackedActions,
}

impl<'a, Message> DialogActionFooterWidget<'a, Message>
where
    Message: Clone + 'a,
{
    fn slots(&self) -> Vec<&Element<'a, Message>> {
        let mut slots = Vec::with_capacity(self.actions.len() + 1);
        if let Some(status) = &self.status {
            slots.push(status);
        }
        slots.extend(self.actions.iter());
        slots
    }

    fn slots_mut(&mut self) -> Vec<&mut Element<'a, Message>> {
        let mut slots = Vec::with_capacity(self.actions.len() + 1);
        if let Some(status) = &mut self.status {
            slots.push(status);
        }
        slots.extend(self.actions.iter_mut());
        slots
    }
}

impl<'a, Message> Widget<Message, Theme, Renderer> for DialogActionFooterWidget<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    fn state(&self) -> tree::State {
        tree::State::None
    }

    fn children(&self) -> Vec<Tree> {
        self.slots().iter().map(|slot| Tree::new(*slot)).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let widgets: Vec<_> = self.slots().iter().map(|slot| slot.as_widget()).collect();
        tree.diff_children(&widgets);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn size_hint(&self) -> Size<Length> {
        self.size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let gap = theme::gap(GapRole::Related);
        let max_width = limits.max().width;
        let unbounded = layout::Limits::new(Size::ZERO, Size::new(f32::INFINITY, f32::INFINITY));

        let action_start = if self.status.is_some() { 1 } else { 0 };

        let status_node = self.status.as_mut().map(|status| {
            status
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &unbounded)
        });

        let action_nodes: Vec<_> = self
            .actions
            .iter_mut()
            .enumerate()
            .map(|(index, action)| {
                action.as_widget_mut().layout(
                    &mut tree.children[action_start + index],
                    renderer,
                    &unbounded,
                )
            })
            .collect();

        let actions_width: f32 = action_nodes
            .iter()
            .map(|node| node.size().width)
            .sum::<f32>()
            + gap * action_nodes.len().saturating_sub(1) as f32;
        let actions_height = action_nodes
            .iter()
            .map(|node| node.size().height)
            .fold(0.0_f32, f32::max);
        let status_width = status_node.as_ref().map_or(0.0, |node| node.size().width);
        let status_height = status_node.as_ref().map_or(0.0, |node| node.size().height);

        let single_row_width = if status_node.is_some() {
            status_width + gap + actions_width
        } else {
            actions_width
        };

        let reflow = if single_row_width <= max_width {
            ReflowLayout::SingleRow
        } else if actions_width <= max_width {
            ReflowLayout::StackedStatus
        } else {
            ReflowLayout::StackedActions
        };

        match reflow {
            ReflowLayout::SingleRow => {
                let mut children = Vec::with_capacity(1 + action_nodes.len());

                if let Some(status_node) = status_node {
                    children.push(status_node.move_to(iced::Point::new(0.0, 0.0)));
                }

                let row_height = actions_height.max(status_height);
                let actions_x_start = max_width - actions_width;
                let mut x = actions_x_start.max(0.0);
                for node in action_nodes {
                    let y = (row_height - node.size().height) / 2.0;
                    children.push(node.move_to(iced::Point::new(x, y.max(0.0))));
                    x += children.last().unwrap().size().width + gap;
                }

                layout::Node::with_children(Size::new(max_width, row_height), children)
            }
            ReflowLayout::StackedStatus => {
                let mut children = Vec::with_capacity(1 + action_nodes.len());
                let mut y = 0.0;

                if let Some(status_node) = status_node {
                    let height = status_node.size().height;
                    children.push(status_node.move_to(iced::Point::new(0.0, 0.0)));
                    y = height + gap;
                }

                let actions_x_start = (max_width - actions_width).max(0.0);
                let mut x = actions_x_start;
                for node in action_nodes {
                    children.push(node.move_to(iced::Point::new(x, y)));
                    x += children.last().unwrap().size().width + gap;
                }

                let total_height = y + actions_height;
                layout::Node::with_children(Size::new(max_width, total_height), children)
            }
            ReflowLayout::StackedActions => {
                let mut children = Vec::with_capacity(1 + action_nodes.len());
                let mut y = 0.0;

                if let Some(status_node) = status_node {
                    let height = status_node.size().height;
                    children.push(status_node.move_to(iced::Point::new(0.0, 0.0)));
                    y = height + gap;
                }

                // Full-width stacking changes each action's own layout (not
                // just its outer bounds), so every action is re-laid-out
                // against its real, persistent tree slot with a fixed-width
                // Limits rather than reusing the natural-width measurement.
                let stretched = layout::Limits::new(
                    Size::new(max_width, 0.0),
                    Size::new(max_width, f32::INFINITY),
                );
                for (index, action) in self.actions.iter_mut().enumerate() {
                    let node = action.as_widget_mut().layout(
                        &mut tree.children[action_start + index],
                        renderer,
                        &stretched,
                    );
                    let height = node.size().height;
                    children.push(node.move_to(iced::Point::new(0.0, y)));
                    y += height + gap;
                }

                let total_height = if children.is_empty() { 0.0 } else { y - gap };
                layout::Node::with_children(Size::new(max_width, total_height.max(0.0)), children)
            }
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        for ((slot, state), child_layout) in self
            .slots_mut()
            .into_iter()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
        {
            slot.as_widget_mut()
                .operate(state, child_layout, renderer, operation);
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for ((slot, state), child_layout) in self
            .slots_mut()
            .into_iter()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
        {
            slot.as_widget_mut().update(
                state,
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );

            if shell.is_event_captured() {
                return;
            }
        }

        if let Some(message) = self.enter_default.clone() {
            if is_unconsumed_confirm_enter(event) {
                shell.publish(message);
                shell.capture_event();
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.slots()
            .into_iter()
            .zip(tree.children.iter())
            .zip(layout.children())
            .map(|((slot, state), child_layout)| {
                slot.as_widget()
                    .mouse_interaction(state, child_layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((slot, state), child_layout) in self
            .slots()
            .into_iter()
            .zip(tree.children.iter())
            .zip(layout.children())
        {
            slot.as_widget().draw(
                state,
                renderer,
                theme,
                inherited_style,
                child_layout,
                cursor,
                viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let _ = (tree, layout, renderer, viewport, translation);
        None
    }
}

impl<'a, Message> From<DialogActionFooterWidget<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(widget: DialogActionFooterWidget<'a, Message>) -> Self {
        Element::new(widget)
    }
}

impl<'a, Message> From<DialogActionFooter<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(footer: DialogActionFooter<'a, Message>) -> Self {
        footer.into_element()
    }
}

#[cfg(test)]
mod dialog_action_footer_tests {
    use super::*;

    #[test]
    fn new_has_zero_preceding_actions() {
        let footer = DialogActionFooter::<()>::new(DialogTerminalAction::primary("Save", ()));
        assert!(footer.preceding.is_empty());
        assert_eq!(footer.terminal.role, DialogActionRole::Primary);
    }

    #[test]
    fn with_one_and_with_two_bound_preceding_count() {
        let one = DialogActionFooter::with_one(
            DialogAction::cancel("Cancel", ()),
            DialogTerminalAction::primary("Save", ()),
        );
        assert_eq!(one.preceding.len(), 1);

        let two = DialogActionFooter::with_two(
            [
                DialogAction::cancel("Cancel", ()),
                DialogAction::secondary("More", ()),
            ],
            DialogTerminalAction::primary("Save", ()),
        );
        assert_eq!(two.preceding.len(), 2);
    }

    #[test]
    fn try_from_parts_rejects_more_than_two_preceding_actions() {
        let result = DialogActionFooter::try_from_parts(
            vec![
                DialogAction::cancel("A", ()),
                DialogAction::secondary("B", ()),
                DialogAction::secondary("C", ()),
            ],
            DialogTerminalAction::primary("Save", ()),
        );

        match result {
            Err(error) => {
                assert_eq!(error, DialogActionFooterError::TooManyPrecedingActions(3));
            }
            Ok(_) => panic!("expected TooManyPrecedingActions"),
        }
    }

    #[test]
    fn try_from_parts_rejects_invalid_preceding_role() {
        let invalid = DialogAction::new(DialogActionRole::Destructive, "Delete", ());
        let result = DialogActionFooter::try_from_parts(
            vec![invalid],
            DialogTerminalAction::primary("Save", ()),
        );

        match result {
            Err(error) => {
                assert_eq!(
                    error,
                    DialogActionFooterError::InvalidPrecedingRole(DialogActionRole::Destructive)
                );
            }
            Ok(_) => panic!("expected InvalidPrecedingRole"),
        }
    }

    #[test]
    fn try_from_parts_accepts_zero_to_two_valid_preceding_actions() {
        assert!(DialogActionFooter::try_from_parts(
            vec![],
            DialogTerminalAction::primary("Save", ())
        )
        .is_ok());
        assert!(DialogActionFooter::try_from_parts(
            vec![DialogAction::cancel("Cancel", ())],
            DialogTerminalAction::primary("Save", ())
        )
        .is_ok());
    }

    #[test]
    fn enter_default_message_is_none_for_destructive_terminal() {
        let footer = DialogActionFooter::new(DialogTerminalAction::destructive("Delete", "delete"));
        assert!(footer.enter_default_message().is_none());
    }

    #[test]
    fn enter_default_message_is_none_when_primary_disabled() {
        let footer =
            DialogActionFooter::new(DialogTerminalAction::primary("Save", "save").disabled(true));
        assert!(footer.enter_default_message().is_none());
    }

    #[test]
    fn enter_default_message_is_the_enabled_primary_message() {
        let footer = DialogActionFooter::new(DialogTerminalAction::primary("Save", "save"));
        assert_eq!(footer.enter_default_message(), Some(&"save"));
    }

    #[test]
    fn ordering_places_preceding_actions_before_the_terminal_action() {
        let footer = DialogActionFooter::with_one(
            DialogAction::cancel("Cancel", 1),
            DialogTerminalAction::primary("Save", 2),
        );
        let ordered: Vec<_> = footer.all_actions().map(|action| action.message).collect();

        assert_eq!(ordered, vec![1, 2]);
    }
}

#[cfg(test)]
mod dialog_action_footer_layout_tests {
    use super::*;

    fn node_for(footer: DialogActionFooter<'static, ()>, width: f32) -> layout::Node {
        crate::test_support::layout(footer.into(), Size::new(width, 1000.0))
    }

    #[test]
    fn wide_footer_renders_a_single_row() {
        let footer = DialogActionFooter::with_one(
            DialogAction::cancel("Cancel", ()),
            DialogTerminalAction::primary("Save", ()),
        )
        .status(iced::widget::text("Autosaved"));

        let node = node_for(footer, 900.0);
        assert_eq!(node.children().len(), 3);
    }

    #[test]
    fn narrow_footer_stacks_status_above_actions() {
        let footer = DialogActionFooter::with_one(
            DialogAction::cancel("Cancel", ()),
            DialogTerminalAction::primary("Save", ()),
        )
        .status(iced::widget::text(
            "A rather long status message that will not fit beside the action buttons",
        ));

        let single_row = node_for(
            DialogActionFooter::with_one(
                DialogAction::cancel("Cancel", ()),
                DialogTerminalAction::primary("Save", ()),
            )
            .status(iced::widget::text("Short")),
            900.0,
        );
        let stacked = node_for(footer, 320.0);

        assert!(stacked.size().height > single_row.size().height);
    }
}

#[cfg(test)]
mod dialog_action_footer_enter_tests {
    use super::*;
    use iced::keyboard;

    fn enter(repeat: bool) -> Event {
        let key = keyboard::Key::Named(keyboard::key::Named::Enter);
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Enter),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::NONE,
            text: None,
            repeat,
        })
    }

    fn messages(
        footer: DialogActionFooter<'static, &'static str>,
        event: Event,
    ) -> Vec<&'static str> {
        crate::test_support::event_messages(footer.into(), Size::new(900.0, 200.0), event)
    }

    #[test]
    fn unconsumed_enter_activates_the_enabled_primary_action() {
        let footer = DialogActionFooter::with_one(
            DialogAction::cancel("Cancel", "cancel"),
            DialogTerminalAction::primary("Save", "save"),
        );

        assert_eq!(messages(footer, enter(false)), vec!["save"]);
    }

    #[test]
    fn repeated_enter_publishes_nothing() {
        let footer = DialogActionFooter::new(DialogTerminalAction::primary("Save", "save"));

        assert!(messages(footer, enter(true)).is_empty());
    }

    #[test]
    fn disabled_primary_publishes_nothing_on_enter() {
        let footer =
            DialogActionFooter::new(DialogTerminalAction::primary("Save", "save").disabled(true));

        assert!(messages(footer, enter(false)).is_empty());
    }

    #[test]
    fn destructive_terminal_is_never_an_implicit_enter_default() {
        let footer = DialogActionFooter::new(DialogTerminalAction::destructive("Delete", "delete"));

        assert!(messages(footer, enter(false)).is_empty());
    }
}
