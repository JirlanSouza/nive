use nive_core::Action;

use super::style as theme_toolbar;
use crate::widgets::controls::button::{self, GroupedItemKind, GroupedItemSpec, SelectionChrome};
use crate::Element;
use crate::IconRef;

/// Action rendered inside a `Toolbar`.
///
/// Toolbar actions use `destructive()` for destructive semantics and do not
/// expose `danger()` or `suggested()` shortcuts.
pub struct ToolbarAction<'a, Message> {
    label: Option<&'a str>,
    icon: Option<IconRef>,
    selected: bool,
    destructive: bool,
    disabled: bool,
    loading: bool,
    reserve_loading_indicator: bool,
    on_press: Option<Message>,
    tooltip: Option<&'a str>,
}

impl<'a, Message: Clone + 'a> ToolbarAction<'a, Message> {
    /// Projects a shared action into a text toolbar action.
    pub fn from_action(action: &'a Action<Message>) -> Self {
        Self::from_action_parts(action, None)
    }

    /// Projects a shared action while adding a UI-owned icon.
    pub fn from_action_with_icon(action: &'a Action<Message>, icon: impl Into<IconRef>) -> Self {
        Self::from_action_parts(action, Some(icon.into()))
    }

    fn from_action_parts(action: &'a Action<Message>, icon: Option<IconRef>) -> Self {
        Self {
            label: Some(action.label()),
            icon,
            selected: false,
            destructive: false,
            disabled: !action.is_enabled(),
            loading: false,
            reserve_loading_indicator: false,
            on_press: action.activate(),
            tooltip: action.description_text(),
        }
    }

    pub fn icon(icon: impl Into<IconRef>) -> Self {
        Self {
            label: None,
            icon: Some(icon.into()),
            selected: false,
            destructive: false,
            disabled: false,
            loading: false,
            reserve_loading_indicator: false,
            on_press: None,
            tooltip: None,
        }
    }

    pub fn label(label: &'a str) -> Self {
        Self {
            label: Some(label),
            icon: None,
            selected: false,
            destructive: false,
            disabled: false,
            loading: false,
            reserve_loading_indicator: false,
            on_press: None,
            tooltip: None,
        }
    }

    pub fn icon_label(icon: impl Into<IconRef>, label: &'a str) -> Self {
        Self {
            label: Some(label),
            icon: Some(icon.into()),
            selected: false,
            destructive: false,
            disabled: false,
            loading: false,
            reserve_loading_indicator: false,
            on_press: None,
            tooltip: None,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Marks this action as destructive.
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self.reserve_loading_indicator = true;
        self
    }

    pub fn tooltip(mut self, tooltip: &'a str) -> Self {
        self.tooltip = Some(tooltip);
        self
    }

    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    pub(super) fn into_element(
        self,
        metrics: theme_toolbar::ToolbarMetrics,
    ) -> Element<'a, Message> {
        let is_icon_only = self.label.is_none();
        let mut button = match (self.icon, self.label) {
            (Some(icon), Some(label)) => button::secondary(label).leading_icon(icon),
            (Some(icon), None) => button::icon(icon, self.tooltip.unwrap_or("Toolbar action")),
            (None, Some(label)) => button::secondary(label),
            (None, None) => button::secondary(""),
        }
        .disabled(self.disabled)
        .on_press_maybe(self.on_press)
        .tooltip_maybe(self.tooltip);

        if self.loading || self.reserve_loading_indicator {
            button = button.loading(self.loading);
        }

        if is_icon_only {
            button = button.width(iced::Length::Fixed(metrics.action_height));
        }

        button.into_grouped_item(GroupedItemSpec {
            size: metrics.size,
            radius: metrics.radius.into(),
            height: metrics.action_height,
            padding_h: metrics.action_padding_h,
            selected: self.selected,
            selection: SelectionChrome::Outlined,
            destructive: self.destructive,
            kind: GroupedItemKind::Toolbar,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use crate::IconRole;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Message {
        Save,
        Delete,
    }

    #[test]
    fn projects_action_and_keeps_icon_in_ui() {
        let action = Action::new("file.save", "Save", Message::Save)
            .description("Persist the current buffer");

        let projected = ToolbarAction::from_action_with_icon(&action, IconRole::ActionConfirm);

        assert_eq!(projected.label, Some("Save"));
        assert_eq!(projected.icon, Some(IconRole::ActionConfirm.into()));
        assert_eq!(projected.tooltip, Some("Persist the current buffer"));
        assert_eq!(projected.on_press, Some(Message::Save));
        assert!(!projected.disabled);
    }

    #[test]
    fn disabled_action_has_no_press_message() {
        let action = Action::new("file.delete", "Delete", Message::Delete).disabled();

        let projected = ToolbarAction::from_action(&action);

        assert!(projected.disabled);
        assert_eq!(projected.on_press, None);
    }
}
