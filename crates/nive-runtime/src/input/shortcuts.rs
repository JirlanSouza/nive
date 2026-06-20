use iced::keyboard;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutKey {
    Character(char),
    Named(keyboard::key::Named),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutBinding {
    key: ShortcutKey,
    modifiers: keyboard::Modifiers,
}

#[derive(Debug, Clone)]
pub struct ShortcutMap<M> {
    bindings: Vec<(ShortcutBinding, M)>,
}

impl<M> ShortcutMap<M> {
    pub fn new() -> Self {
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
}

impl ShortcutBinding {
    pub fn character(character: char, modifiers: keyboard::Modifiers) -> Self {
        Self {
            key: ShortcutKey::Character(character.to_ascii_lowercase()),
            modifiers,
        }
    }

    pub fn named(named: keyboard::key::Named, modifiers: keyboard::Modifiers) -> Self {
        Self {
            key: ShortcutKey::Named(named),
            modifiers,
        }
    }
}

impl<M: Clone> ShortcutMap<M> {
    pub(crate) fn message_for_event(&self, event: &keyboard::Event) -> Option<M> {
        let keyboard::Event::KeyPressed {
            key,
            modifiers,
            repeat,
            ..
        } = event
        else {
            return None;
        };
        if *repeat {
            return None;
        }

        self.bindings
            .iter()
            .find(|(binding, _)| binding.matches(key, *modifiers))
            .map(|(_, message)| message.clone())
    }
}

impl ShortcutBinding {
    fn matches(&self, key: &keyboard::Key, modifiers: keyboard::Modifiers) -> bool {
        if self.modifiers != modifiers {
            return false;
        }

        match (&self.key, key) {
            (ShortcutKey::Character(expected), keyboard::Key::Character(actual)) => actual
                .chars()
                .next()
                .is_some_and(|actual| actual.to_ascii_lowercase() == *expected),
            (ShortcutKey::Named(expected), keyboard::Key::Named(actual)) => expected == actual,
            _ => false,
        }
    }
}

impl<M> Default for ShortcutMap<M> {
    fn default() -> Self {
        Self::new()
    }
}
