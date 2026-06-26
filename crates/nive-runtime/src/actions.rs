use std::borrow::Cow;

use iced::keyboard;

use crate::ShortcutBinding;

/// Stable product-owned identifier for an application action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionId(&'static str);

/// A user-visible command that can be surfaced by shortcuts, toolbars, menus or
/// command palettes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action<M> {
    id: ActionId,
    label: Cow<'static, str>,
    description: Option<Cow<'static, str>>,
    shortcut: Option<ShortcutBinding>,
    enabled: bool,
    message: M,
}

/// Ordered collection of application actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionMap<M> {
    actions: Vec<Action<M>>,
}

/// Error returned when an [`ActionMap`] contains duplicate action IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DuplicateActionId {
    id: ActionId,
}

impl ActionId {
    pub const fn new(id: &'static str) -> Self {
        Self(id)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl From<&'static str> for ActionId {
    fn from(id: &'static str) -> Self {
        Self::new(id)
    }
}

impl std::fmt::Display for ActionId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.0)
    }
}

impl<M> Action<M> {
    pub fn new(id: impl Into<ActionId>, label: impl Into<Cow<'static, str>>, message: M) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
            shortcut: None,
            enabled: true,
            message,
        }
    }

    pub fn description(mut self, description: impl Into<Cow<'static, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn shortcut(mut self, shortcut: ShortcutBinding) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn disabled(self) -> Self {
        self.enabled(false)
    }

    pub fn id(&self) -> ActionId {
        self.id
    }

    pub fn label(&self) -> &str {
        self.label.as_ref()
    }

    pub fn description_text(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn shortcut_binding(&self) -> Option<&ShortcutBinding> {
        self.shortcut.as_ref()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn message(&self) -> &M {
        &self.message
    }
}

impl<M> Action<M>
where
    M: Clone,
{
    pub fn activate(&self) -> Option<M> {
        self.enabled.then(|| self.message.clone())
    }
}

impl<M> ActionMap<M> {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    pub fn action(mut self, action: Action<M>) -> Self {
        self.actions.push(action);
        self
    }

    pub fn push(&mut self, action: Action<M>) {
        self.actions.push(action);
    }

    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    pub fn len(&self) -> usize {
        self.actions.len()
    }

    pub fn actions(&self) -> &[Action<M>] {
        self.actions.as_slice()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Action<M>> {
        self.actions.iter()
    }

    pub fn find(&self, id: ActionId) -> Option<&Action<M>> {
        self.actions.iter().find(|action| action.id == id)
    }

    pub fn validate(&self) -> Result<(), DuplicateActionId> {
        for (index, action) in self.actions.iter().enumerate() {
            if self.actions[..index]
                .iter()
                .any(|previous| previous.id == action.id)
            {
                return Err(DuplicateActionId { id: action.id });
            }
        }

        Ok(())
    }
}

impl<M> ActionMap<M>
where
    M: Clone,
{
    pub(crate) fn message_for_event(&self, event: &keyboard::Event) -> Option<M> {
        self.actions
            .iter()
            .find(|action| {
                action.enabled
                    && action
                        .shortcut
                        .as_ref()
                        .is_some_and(|shortcut| shortcut.matches_event(event))
            })
            .map(|action| action.message.clone())
    }
}

impl<M> Default for ActionMap<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl DuplicateActionId {
    pub fn id(self) -> ActionId {
        self.id
    }
}

impl std::fmt::Display for DuplicateActionId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "duplicate action id: {}", self.id)
    }
}

impl std::error::Error for DuplicateActionId {}

/// Builds a [`nive_ui::widgets::CommandPaletteRow`] for each enabled action in `actions`.
///
/// Disabled actions are still included as rows so the palette can present
/// them, but their [`nive_ui::widgets::CommandPaletteRow::activated`] returns `None` so the
/// app-level submit handler ignores them. Apps that want to hide disabled
/// actions entirely can drop rows where `row.activated().is_none()` before
/// passing them to [`nive_ui::widgets::command_palette_filter`].
pub fn command_palette_rows<M>(
    actions: &ActionMap<M>,
) -> Vec<nive_ui::widgets::CommandPaletteRow<'_, M>>
where
    M: Clone,
{
    actions
        .iter()
        .map(|action| {
            let mut row = nive_ui::widgets::CommandPaletteRow::new(
                action.id().as_str(),
                action.label(),
                action.message().clone(),
            );

            if let Some(description) = action.description_text() {
                row = row.description(description);
            }

            if let Some(shortcut) = action.shortcut_binding() {
                row = row.shortcut_label(format_shortcut(shortcut));
            }

            if !action.is_enabled() {
                row = row.disabled();
            }

            row
        })
        .collect()
}

fn format_shortcut(shortcut: &ShortcutBinding) -> String {
    let mut label = String::new();
    let modifiers = shortcut.modifiers();
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
    label
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ShortcutBinding;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Message {
        Save,
        Open,
    }

    #[test]
    fn validates_unique_action_ids() {
        let actions = ActionMap::new()
            .action(Action::new("file.save", "Save", Message::Save))
            .action(Action::new("file.open", "Open", Message::Open));

        assert_eq!(actions.validate(), Ok(()));
    }

    #[test]
    fn reports_duplicate_action_id() {
        let actions = ActionMap::new()
            .action(Action::new("file.save", "Save", Message::Save))
            .action(Action::new("file.save", "Save again", Message::Open));

        assert_eq!(
            actions.validate().map_err(DuplicateActionId::id),
            Err(ActionId::new("file.save"))
        );
    }

    #[test]
    fn disabled_action_does_not_activate() {
        let action = Action::new("file.save", "Save", Message::Save).disabled();

        assert_eq!(action.activate(), None);
    }

    #[test]
    fn command_palette_rows_collects_enabled_and_disabled_actions() {
        let actions = ActionMap::new()
            .action(
                Action::new("file.save", "Save", Message::Save)
                    .description("Persist the current buffer")
                    .shortcut(ShortcutBinding::primary_character('s')),
            )
            .action(Action::new("file.open", "Open", Message::Open).disabled());

        let rows = command_palette_rows(&actions);

        assert_eq!(rows.len(), 2);

        let save = &rows[0];
        assert_eq!(save.id, "file.save");
        assert_eq!(save.label, "Save");
        assert_eq!(save.description, Some("Persist the current buffer"));
        assert_eq!(save.shortcut_label.as_deref(), Some("Cmd+S"));
        assert!(save.enabled);
        assert!(save.activated().is_some());

        let open = &rows[1];
        assert_eq!(open.id, "file.open");
        assert!(!open.enabled);
        assert!(open.activated().is_none());
    }
}
