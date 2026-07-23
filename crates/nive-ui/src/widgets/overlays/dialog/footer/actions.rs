use std::borrow::Cow;

use iced::{widget::row, Length};

use super::{
    DialogAction, DialogActionFooterError, DialogActionRole, DialogFooter, DialogTerminalAction,
};
use crate::Element;

impl<'a, Message> DialogFooter<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
        }
    }

    pub(super) fn into_element(self) -> Element<'a, Message> {
        row![self.content].width(Length::Fill).into()
    }
}

impl<'a, Message> DialogAction<'a, Message> {
    pub(super) fn new(
        role: DialogActionRole,
        label: impl Into<Cow<'a, str>>,
        message: Message,
    ) -> Self {
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

    pub(super) fn is_safe_preceding_role(&self) -> bool {
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
