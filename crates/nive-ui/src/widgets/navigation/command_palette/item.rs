use std::borrow::Cow;

use nive_core::{Action, ShortcutBinding};

use crate::icons::IconRole;

/// One command presented inside a [`super::CommandPalette`].
///
/// Apps build these from their own action/command catalogs; the widget
/// consumes only the public fields, so the item type stays decoupled from
/// `nive-runtime`. Activation is carried on the item: a disabled item
/// renders but carries no message, mirroring `MenuCommand`.
#[derive(Clone)]
pub struct CommandPaletteItem<'a, M> {
    pub id: &'a str,
    pub icon: Option<IconRole>,
    pub label: &'a str,
    pub description: Option<&'a str>,
    pub shortcut_label: Option<Cow<'a, str>>,
    pub enabled: bool,
    pub message: Option<M>,
}

impl<'a, M> CommandPaletteItem<'a, M> {
    pub fn new(id: &'a str, label: &'a str, message: M) -> Self {
        Self {
            id,
            icon: None,
            label,
            description: None,
            shortcut_label: None,
            enabled: true,
            message: Some(message),
        }
    }

    pub fn icon(mut self, icon: IconRole) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn shortcut_label(mut self, shortcut_label: impl Into<Cow<'a, str>>) -> Self {
        self.shortcut_label = Some(shortcut_label.into());
        self
    }

    /// Sets whether the item is disabled.
    ///
    /// Disabled items do not produce activation messages.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.enabled = !disabled;
        if disabled {
            self.message = None;
        }
        self
    }

    /// Returns the item message only when the item is enabled.
    pub fn activated(&self) -> Option<&M> {
        if self.enabled {
            self.message.as_ref()
        } else {
            None
        }
    }
}

impl<'a, M: Clone> CommandPaletteItem<'a, M> {
    /// Projects a shared application action into a command-palette item.
    pub fn from_action(action: &'a Action<M>) -> Self {
        Self {
            id: action.id().as_str(),
            icon: None,
            label: action.label(),
            description: action.description_text(),
            shortcut_label: action.shortcut_binding().map(format_shortcut),
            enabled: action.is_enabled(),
            message: action.activate(),
        }
    }
}

pub(in crate::widgets::navigation) fn format_shortcut(
    shortcut: &ShortcutBinding,
) -> Cow<'static, str> {
    let mut label = String::new();
    let modifiers = shortcut.modifiers().resolved_primary();

    if modifiers.control() {
        label.push_str("Ctrl+");
    }
    if modifiers.alt() {
        label.push_str("Alt+");
    }
    if modifiers.shift() {
        label.push_str("Shift+");
    }
    if modifiers.logo() {
        label.push_str("Cmd+");
    }
    label.push_str(shortcut.key().label().as_ref());

    Cow::Owned(label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nive_core::{ShortcutBinding, ShortcutModifiers};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Message {
        Save,
        Open,
    }

    #[test]
    fn projects_action_metadata_and_activation() {
        let action = Action::new("file.save", "Save", Message::Save)
            .description("Persist the current buffer")
            .shortcut(ShortcutBinding::primary_character('s'));

        let item = CommandPaletteItem::from_action(&action);

        assert_eq!(item.id, "file.save");
        assert_eq!(item.label, "Save");
        assert_eq!(item.description, Some("Persist the current buffer"));
        if cfg!(target_os = "macos") {
            assert_eq!(item.shortcut_label.as_deref(), Some("Cmd+S"));
        } else {
            assert_eq!(item.shortcut_label.as_deref(), Some("Ctrl+S"));
        }
        assert_eq!(item.activated(), Some(&Message::Save));
    }

    #[test]
    fn disabled_action_remains_visible_without_activation() {
        let action = Action::new("file.open", "Open", Message::Open).disabled();

        let item = CommandPaletteItem::from_action(&action);

        assert!(!item.enabled);
        assert_eq!(item.label, "Open");
        assert_eq!(item.activated(), None);
    }

    #[test]
    fn formats_composed_explicit_modifiers() {
        let action = Action::new("file.save_as", "Save as", Message::Save).shortcut(
            ShortcutBinding::character('s', ShortcutModifiers::CONTROL | ShortcutModifiers::SHIFT),
        );

        let item = CommandPaletteItem::from_action(&action);

        assert_eq!(item.shortcut_label.as_deref(), Some("Ctrl+Shift+S"));
    }

    #[test]
    fn macos_shortcut_label_uses_the_command_glyph() {
        if !cfg!(target_os = "macos") {
            return;
        }
        let action = Action::new("file.save", "Save", Message::Save)
            .shortcut(ShortcutBinding::primary_character('s'));

        let item = CommandPaletteItem::from_action(&action);

        assert_eq!(item.shortcut_label.as_deref(), Some("Cmd+S"));
    }

    #[test]
    fn non_macos_shortcut_label_uses_ctrl() {
        if cfg!(target_os = "macos") {
            return;
        }
        let action = Action::new("file.save", "Save", Message::Save)
            .shortcut(ShortcutBinding::primary_character('s'));

        let item = CommandPaletteItem::from_action(&action);

        assert_eq!(item.shortcut_label.as_deref(), Some("Ctrl+S"));
    }
}
