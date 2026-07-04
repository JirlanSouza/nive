use iced::keyboard::key::Named;

/// Activation behavior presets.
///
/// This enum is non-exhaustive; app matches should include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum ActivationBehavior {
    /// Resolve to the desktop-platform default.
    #[default]
    Platform,
    /// Double-click activates.
    DoubleClick,
    /// Enter activates.
    Enter,
    /// Space activates.
    Space,
    /// Enter and double-click activate.
    EnterAndDoubleClick,
    /// Space and double-click activate.
    SpaceAndDoubleClick,
    /// Enter, Space, and double-click activate.
    EnterSpaceAndDoubleClick,
    /// Command+O, Command+Down, and double-click activate.
    CommandOpenAndDoubleClick,
    /// Space, Command+O, Command+Down, and double-click activate.
    SpaceCommandOpenAndDoubleClick,
}

impl ActivationBehavior {
    /// Resolves [`ActivationBehavior::Platform`] for the current target OS.
    pub fn resolve(self) -> Self {
        match self {
            Self::Platform if cfg!(target_os = "macos") => Self::SpaceCommandOpenAndDoubleClick,
            Self::Platform => Self::EnterAndDoubleClick,
            behavior => behavior,
        }
    }

    /// Returns whether this behavior includes the provided trigger.
    pub fn includes(self, trigger: ActivationTrigger) -> bool {
        match self.resolve() {
            Self::Platform => false,
            Self::DoubleClick => trigger == ActivationTrigger::DoubleClick,
            Self::Enter => trigger == ActivationTrigger::Enter,
            Self::Space => trigger == ActivationTrigger::Space,
            Self::EnterAndDoubleClick => matches!(
                trigger,
                ActivationTrigger::Enter | ActivationTrigger::DoubleClick
            ),
            Self::SpaceAndDoubleClick => matches!(
                trigger,
                ActivationTrigger::Space | ActivationTrigger::DoubleClick
            ),
            Self::EnterSpaceAndDoubleClick => matches!(
                trigger,
                ActivationTrigger::Enter
                    | ActivationTrigger::Space
                    | ActivationTrigger::DoubleClick
            ),
            Self::CommandOpenAndDoubleClick => matches!(
                trigger,
                ActivationTrigger::CommandO
                    | ActivationTrigger::CommandDown
                    | ActivationTrigger::DoubleClick
            ),
            Self::SpaceCommandOpenAndDoubleClick => matches!(
                trigger,
                ActivationTrigger::Space
                    | ActivationTrigger::CommandO
                    | ActivationTrigger::CommandDown
                    | ActivationTrigger::DoubleClick
            ),
        }
    }

    /// Returns whether the given trigger should emit an activation event.
    pub fn should_activate(self, trigger: ActivationTrigger) -> bool {
        self.includes(trigger)
    }

    /// Maps an iced keyboard event to an [`ActivationTrigger`] if it matches.
    pub fn trigger_from_key_event(
        self,
        event: &iced::keyboard::Event,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<ActivationTrigger> {
        use iced::keyboard::{Event, Key};

        let Event::KeyPressed {
            key, repeat: false, ..
        } = event
        else {
            return None;
        };

        let trigger = match key {
            Key::Named(Named::Enter) => ActivationTrigger::Enter,
            Key::Named(Named::Space) => ActivationTrigger::Space,
            Key::Character(c) if c == "o" && modifiers.command() => ActivationTrigger::CommandO,
            Key::Named(Named::ArrowDown) if modifiers.command() => ActivationTrigger::CommandDown,
            _ => return None,
        };

        if self.should_activate(trigger) {
            Some(trigger)
        } else {
            None
        }
    }
}

/// Input source that caused an activation intent.
///
/// This enum is non-exhaustive; app matches should include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ActivationTrigger {
    /// Pointer double-click.
    DoubleClick,
    /// Enter or Return activation.
    Enter,
    /// Space activation.
    Space,
    /// Command+O activation.
    CommandO,
    /// Command+Down activation.
    CommandDown,
}

/// Rename behavior presets.
///
/// This enum is non-exhaustive; app matches should include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum RenameBehavior {
    /// Resolve to the desktop-platform default.
    #[default]
    Platform,
    /// Keyboard rename is disabled.
    Disabled,
    /// F2 requests rename.
    F2,
    /// Return requests rename.
    Return,
    /// F2 and Return request rename.
    F2OrReturn,
}

impl RenameBehavior {
    /// Resolves [`RenameBehavior::Platform`] for the current target OS.
    pub fn resolve(self) -> Self {
        match self {
            Self::Platform if cfg!(target_os = "macos") => Self::Return,
            Self::Platform => Self::F2,
            behavior => behavior,
        }
    }

    /// Returns whether this behavior requests rename from F2.
    pub fn includes_f2(self) -> bool {
        matches!(self.resolve(), Self::F2 | Self::F2OrReturn)
    }

    /// Returns whether this behavior requests rename from Return.
    pub fn includes_return(self) -> bool {
        matches!(self.resolve(), Self::Return | Self::F2OrReturn)
    }

    /// Returns whether the given key should emit a rename request.
    pub fn should_rename(self, key: Named) -> bool {
        match key {
            Named::F2 => self.includes_f2(),
            Named::Enter => self.includes_return(),
            _ => false,
        }
    }

    /// Maps an iced keyboard event to `true` if it should emit a rename request.
    pub fn is_rename_key_event(self, event: &iced::keyboard::Event) -> bool {
        use iced::keyboard::{Event, Key};

        let Event::KeyPressed {
            key, repeat: false, ..
        } = event
        else {
            return false;
        };

        match key {
            Key::Named(named) => self.should_rename(*named),
            _ => false,
        }
    }
}

mod type_ahead;

#[cfg(test)]
mod keyboard_tests;

pub(crate) use type_ahead::TypeAhead;
