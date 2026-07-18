//! Toolkit-neutral application actions and keyboard shortcut contracts.

use std::borrow::Cow;
use std::fmt;
use std::ops::{BitOr, BitOrAssign};

/// Stable product-owned identifier for an application action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionId(&'static str);

/// A user-visible command shared by shortcuts and action-aware controls.
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

/// Toolkit-neutral key used by a [`ShortcutBinding`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutKey {
    Character(char),
    Named(NamedShortcutKey),
}

/// Named keys that Nive can match and label consistently across toolkits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamedShortcutKey {
    Enter,
    Tab,
    Space,
    Escape,
    Backspace,
    Delete,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

/// Modifier set for toolkit-neutral shortcut bindings.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ShortcutModifiers(u8);

/// A toolkit-neutral keyboard shortcut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShortcutBinding {
    key: ShortcutKey,
    modifiers: ShortcutModifiers,
}

/// Ordered fallback keyboard bindings that are not represented by actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutMap<M> {
    bindings: Vec<(ShortcutBinding, M)>,
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

impl fmt::Display for ActionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl<M: Clone> Action<M> {
    pub fn activate(&self) -> Option<M> {
        self.enabled.then(|| self.message.clone())
    }
}

impl<M> ActionMap<M> {
    pub const fn new() -> Self {
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

impl<M> Default for ActionMap<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl DuplicateActionId {
    pub const fn id(self) -> ActionId {
        self.id
    }
}

impl fmt::Display for DuplicateActionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "duplicate action id: {}", self.id)
    }
}

impl std::error::Error for DuplicateActionId {}

impl ShortcutKey {
    pub fn label(self) -> Cow<'static, str> {
        match self {
            Self::Character(character) => Cow::Owned(character.to_ascii_uppercase().to_string()),
            Self::Named(named) => Cow::Borrowed(named.label()),
        }
    }
}

impl NamedShortcutKey {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Enter => "Enter",
            Self::Tab => "Tab",
            Self::Space => "Space",
            Self::Escape => "Esc",
            Self::Backspace => "Backspace",
            Self::Delete => "Delete",
            Self::ArrowUp => "Up",
            Self::ArrowDown => "Down",
            Self::ArrowLeft => "Left",
            Self::ArrowRight => "Right",
            Self::Home => "Home",
            Self::End => "End",
            Self::PageUp => "PageUp",
            Self::PageDown => "PageDown",
            Self::F1 => "F1",
            Self::F2 => "F2",
            Self::F3 => "F3",
            Self::F4 => "F4",
            Self::F5 => "F5",
            Self::F6 => "F6",
            Self::F7 => "F7",
            Self::F8 => "F8",
            Self::F9 => "F9",
            Self::F10 => "F10",
            Self::F11 => "F11",
            Self::F12 => "F12",
        }
    }
}

impl ShortcutModifiers {
    pub const NONE: Self = Self(0);
    pub const CONTROL: Self = Self(1 << 0);
    pub const ALT: Self = Self(1 << 1);
    pub const SHIFT: Self = Self(1 << 2);
    pub const LOGO: Self = Self(1 << 3);
    pub const PRIMARY: Self = Self(1 << 4);

    pub const fn control(self) -> bool {
        self.contains(Self::CONTROL)
    }

    pub const fn alt(self) -> bool {
        self.contains(Self::ALT)
    }

    pub const fn shift(self) -> bool {
        self.contains(Self::SHIFT)
    }

    pub const fn logo(self) -> bool {
        self.contains(Self::LOGO)
    }

    pub const fn primary(self) -> bool {
        self.contains(Self::PRIMARY)
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn resolved_primary(self) -> Self {
        if !self.primary() {
            return self;
        }

        let without_primary = Self(self.0 & !Self::PRIMARY.0);
        if cfg!(target_os = "macos") {
            without_primary.union(Self::LOGO)
        } else {
            without_primary.union(Self::CONTROL)
        }
    }
}

impl BitOr for ShortcutModifiers {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl BitOrAssign for ShortcutModifiers {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.union(rhs);
    }
}

impl ShortcutBinding {
    pub const fn primary_character(character: char) -> Self {
        Self::character(character, ShortcutModifiers::PRIMARY)
    }

    pub const fn character(character: char, modifiers: ShortcutModifiers) -> Self {
        Self {
            key: ShortcutKey::Character(character.to_ascii_lowercase()),
            modifiers,
        }
    }

    pub const fn named(named: NamedShortcutKey, modifiers: ShortcutModifiers) -> Self {
        Self {
            key: ShortcutKey::Named(named),
            modifiers,
        }
    }

    pub const fn key(self) -> ShortcutKey {
        self.key
    }

    pub const fn modifiers(self) -> ShortcutModifiers {
        self.modifiers
    }

    pub fn matches(self, key: ShortcutKey, modifiers: ShortcutModifiers) -> bool {
        self.modifiers.resolved_primary() == modifiers.resolved_primary()
            && match (self.key, key) {
                (ShortcutKey::Character(expected), ShortcutKey::Character(actual)) => {
                    expected == actual.to_ascii_lowercase()
                }
                (ShortcutKey::Named(expected), ShortcutKey::Named(actual)) => expected == actual,
                _ => false,
            }
    }
}

impl<M> ShortcutMap<M> {
    pub const fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    pub fn bind(mut self, binding: ShortcutBinding, message: M) -> Self {
        self.bindings.push((binding, message));
        self
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ShortcutBinding, &M)> {
        self.bindings
            .iter()
            .map(|(binding, message)| (binding, message))
    }
}

impl<M> Default for ShortcutMap<M> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Message {
        Save,
        Open,
    }

    #[test]
    fn action_map_preserves_order_lookup_and_unique_validation() {
        let actions = ActionMap::new()
            .action(Action::new("file.save", "Save", Message::Save))
            .action(Action::new("file.open", "Open", Message::Open));

        let ids: Vec<_> = actions.iter().map(Action::id).collect();
        assert_eq!(
            ids,
            [ActionId::new("file.save"), ActionId::new("file.open")]
        );
        assert_eq!(
            actions
                .find(ActionId::new("file.open"))
                .map(Action::message),
            Some(&Message::Open)
        );
        assert_eq!(actions.validate(), Ok(()));
    }

    #[test]
    fn action_map_reports_duplicate_identifier() {
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
    fn character_shortcuts_normalize_case_and_label() {
        let binding = ShortcutBinding::character('S', ShortcutModifiers::CONTROL);

        assert_eq!(binding.key(), ShortcutKey::Character('s'));
        assert_eq!(binding.key().label(), "S");
        assert!(binding.matches(ShortcutKey::Character('S'), ShortcutModifiers::CONTROL));
    }

    #[test]
    fn named_shortcuts_have_stable_labels() {
        assert_eq!(NamedShortcutKey::Escape.label(), "Esc");
        assert_eq!(NamedShortcutKey::ArrowDown.label(), "Down");
        assert_eq!(NamedShortcutKey::F12.label(), "F12");
    }

    #[test]
    fn primary_modifier_resolves_for_target_platform() {
        let resolved = ShortcutModifiers::PRIMARY.resolved_primary();

        if cfg!(target_os = "macos") {
            assert_eq!(resolved, ShortcutModifiers::LOGO);
        } else {
            assert_eq!(resolved, ShortcutModifiers::CONTROL);
        }
    }

    #[test]
    fn modifiers_compose_without_losing_flags() {
        let modifiers = ShortcutModifiers::PRIMARY | ShortcutModifiers::SHIFT;

        assert!(modifiers.primary());
        assert!(modifiers.shift());
        assert!(!modifiers.alt());
    }
}
