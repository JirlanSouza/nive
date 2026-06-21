use iced::keyboard;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutKey {
    Character(char),
    Named(keyboard::key::Named),
}

impl ShortcutKey {
    pub fn label(&self) -> std::borrow::Cow<'static, str> {
        match self {
            ShortcutKey::Character(character) => {
                let mut label = String::new();
                label.push(character.to_ascii_uppercase());
                std::borrow::Cow::Owned(label)
            }
            ShortcutKey::Named(named) => std::borrow::Cow::Borrowed(named_label(*named)),
        }
    }
}

fn named_label(named: keyboard::key::Named) -> &'static str {
    use keyboard::key::Named;
    match named {
        Named::Enter => "Enter",
        Named::Tab => "Tab",
        Named::Space => "Space",
        Named::Escape => "Esc",
        Named::Backspace => "Backspace",
        Named::Delete => "Delete",
        Named::ArrowUp => "Up",
        Named::ArrowDown => "Down",
        Named::ArrowLeft => "Left",
        Named::ArrowRight => "Right",
        Named::Home => "Home",
        Named::End => "End",
        Named::PageUp => "PageUp",
        Named::PageDown => "PageDown",
        Named::F1 => "F1",
        Named::F2 => "F2",
        Named::F3 => "F3",
        Named::F4 => "F4",
        Named::F5 => "F5",
        Named::F6 => "F6",
        Named::F7 => "F7",
        Named::F8 => "F8",
        Named::F9 => "F9",
        Named::F10 => "F10",
        Named::F11 => "F11",
        Named::F12 => "F12",
        _ => "Key",
    }
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
    pub fn primary_character(character: char) -> Self {
        Self::character(character, primary_modifier())
    }

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

    pub fn key(&self) -> ShortcutKey {
        self.key.clone()
    }

    pub fn modifiers(&self) -> keyboard::Modifiers {
        self.modifiers
    }
}

fn primary_modifier() -> keyboard::Modifiers {
    if cfg!(target_os = "macos") {
        keyboard::Modifiers::COMMAND
    } else {
        keyboard::Modifiers::CTRL
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
    pub(crate) fn matches_event(&self, event: &keyboard::Event) -> bool {
        let keyboard::Event::KeyPressed {
            key,
            modifiers,
            repeat,
            ..
        } = event
        else {
            return false;
        };
        if *repeat {
            return false;
        }

        self.matches(key, *modifiers)
    }

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
