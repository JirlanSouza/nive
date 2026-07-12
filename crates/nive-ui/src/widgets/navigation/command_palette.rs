use std::borrow::Cow;

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
