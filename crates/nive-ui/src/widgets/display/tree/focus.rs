use std::time::Duration;

use iced::{keyboard, Point};

use crate::advanced::focus::FocusState;
use crate::interaction::keyboard::TypeAhead;
use crate::Element;

use super::widget::Tree;

mod pointer_drag;

#[cfg(test)]
#[path = "focus_tests.rs"]
mod focus_tests;

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
    inactive_content: Element<'a, Message>,
    visible_content: Element<'a, Message>,
    tree: Tree<'a, Id, Message>,
}

#[derive(Debug)]
struct TreeFocusState {
    focus: FocusState,
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
            focus: FocusState::default(),
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
    pub(super) fn new(
        tree: Tree<'a, Id, Message>,
        inactive_content: Element<'a, Message>,
        visible_content: Element<'a, Message>,
    ) -> Self {
        Self {
            inactive_content,
            visible_content,
            tree,
        }
    }

    fn content(&self, focus_visible: bool) -> &Element<'a, Message> {
        if focus_visible {
            &self.visible_content
        } else {
            &self.inactive_content
        }
    }

    fn content_mut(&mut self, focus_visible: bool) -> &mut Element<'a, Message> {
        if focus_visible {
            &mut self.visible_content
        } else {
            &mut self.inactive_content
        }
    }
}

mod widget;

impl<'a, Id, Message> From<TreeFocus<'a, Id, Message>> for Element<'a, Message>
where
    Id: Clone + Ord + 'a,
    Message: Clone + 'a,
{
    fn from(value: TreeFocus<'a, Id, Message>) -> Self {
        Element::new(value)
    }
}
