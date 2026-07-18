use std::borrow::Cow;

use nive_core::{Action, ShortcutBinding};

mod filter;
mod view;

pub use filter::command_palette_filter;
pub use view::command_palette_view;

/// One row presented inside a command palette.
///
/// Apps build these from their own action/command catalogs. The widget layer
/// only consumes the public fields, so the row type stays decoupled from
/// `nive-runtime`.
/// One command row rendered by `command_palette_view`.
#[derive(Clone)]
pub struct CommandPaletteRow<'a, M> {
    pub id: &'a str,
    pub label: &'a str,
    pub description: Option<&'a str>,
    pub shortcut_label: Option<Cow<'a, str>>,
    pub enabled: bool,
    pub message: Option<M>,
}

impl<'a, M> CommandPaletteRow<'a, M> {
    pub fn new(id: &'a str, label: &'a str, message: M) -> Self {
        Self {
            id,
            label,
            description: None,
            shortcut_label: None,
            enabled: true,
            message: Some(message),
        }
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn shortcut_label(mut self, shortcut_label: impl Into<Cow<'a, str>>) -> Self {
        self.shortcut_label = Some(shortcut_label.into());
        self
    }

    /// Sets whether the row is disabled.
    ///
    /// Disabled rows do not produce activation messages.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.enabled = !disabled;
        if disabled {
            self.message = None;
        }
        self
    }

    /// Returns the row message only when the row is enabled.
    pub fn activated(&self) -> Option<&M> {
        if self.enabled {
            self.message.as_ref()
        } else {
            None
        }
    }
}

impl<'a, M: Clone> CommandPaletteRow<'a, M> {
    /// Projects a shared application action into a command-palette row.
    pub fn from_action(action: &'a Action<M>) -> Self {
        Self {
            id: action.id().as_str(),
            label: action.label(),
            description: action.description_text(),
            shortcut_label: action.shortcut_binding().map(format_shortcut),
            enabled: action.is_enabled(),
            message: action.activate(),
        }
    }
}

pub(super) fn format_shortcut(shortcut: &ShortcutBinding) -> Cow<'static, str> {
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

        let row = CommandPaletteRow::from_action(&action);

        assert_eq!(row.id, "file.save");
        assert_eq!(row.label, "Save");
        assert_eq!(row.description, Some("Persist the current buffer"));
        if cfg!(target_os = "macos") {
            assert_eq!(row.shortcut_label.as_deref(), Some("Cmd+S"));
        } else {
            assert_eq!(row.shortcut_label.as_deref(), Some("Ctrl+S"));
        }
        assert_eq!(row.activated(), Some(&Message::Save));
    }

    #[test]
    fn disabled_action_remains_visible_without_activation() {
        let action = Action::new("file.open", "Open", Message::Open).disabled();

        let row = CommandPaletteRow::from_action(&action);

        assert!(!row.enabled);
        assert_eq!(row.label, "Open");
        assert_eq!(row.activated(), None);
    }

    #[test]
    fn formats_composed_explicit_modifiers() {
        let action = Action::new("file.save_as", "Save as", Message::Save).shortcut(
            ShortcutBinding::character('s', ShortcutModifiers::CONTROL | ShortcutModifiers::SHIFT),
        );

        let row = CommandPaletteRow::from_action(&action);

        assert_eq!(row.shortcut_label.as_deref(), Some("Ctrl+Shift+S"));
    }
}
