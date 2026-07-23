mod action_footer;
mod actions;
mod footer_widget;
mod marker;

#[cfg(test)]
mod dialog_action_footer_enter_tests;
#[cfg(test)]
mod dialog_action_footer_layout_tests;
#[cfg(test)]
mod dialog_action_footer_tests;

use std::borrow::Cow;

use crate::Element;

/// Simple full-width Dialog footer slot. Prefer [`DialogActionFooter`] for
/// canonical action rows; use `DialogFooter` for footer content that is not
/// an action group (e.g. a single custom control).
pub struct DialogFooter<'a, Message> {
    content: Element<'a, Message>,
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

/// A terminal (final) Dialog action: always `Primary` or `Destructive`,
/// constructed only through this type so [`DialogActionFooter`] can enforce
/// the bounded action model at the type level.
pub struct DialogTerminalAction<'a, Message>(DialogAction<'a, Message>);

/// Construction error for a dynamically assembled [`DialogActionFooter`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DialogActionFooterError {
    /// More than two preceding actions were supplied.
    TooManyPrecedingActions(usize),
    /// A preceding action used a role other than Cancel or Secondary.
    InvalidPrecedingRole(DialogActionRole),
}

/// Canonical bounded Dialog action footer: at most two preceding
/// Cancel/Secondary actions plus one required terminal Primary or
/// Destructive action, with optional leading status/help content and
/// measured responsive reflow.
pub struct DialogActionFooter<'a, Message> {
    status: Option<Element<'a, Message>>,
    preceding: Vec<DialogAction<'a, Message>>,
    terminal: DialogAction<'a, Message>,
}

/// Zero-sized tag pushed to any [`iced::advanced::widget::Operation`] that
/// reaches the terminal action, via [`iced::advanced::widget::Operation::custom`].
pub(crate) struct TerminalActionTag;

/// Transparent single-child wrapper that announces [`TerminalActionTag`]
/// before delegating `operate()` to its inner action button. Every other
/// [`Widget`] method passes through unchanged.
struct TerminalActionMarker<'a, Message>(Element<'a, Message>);

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
