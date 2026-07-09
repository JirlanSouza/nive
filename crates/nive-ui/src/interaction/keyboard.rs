use iced::keyboard::{key::Named, Key, Modifiers};

use super::Orientation;

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
    /// Primary pointer click.
    Click,
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

/// Reusable arrow-key step adjustment for continuous values, parameterized by
/// [`Orientation`].
///
/// `ArrowRight`/`ArrowDown` produce a positive delta; `ArrowLeft`/`ArrowUp`
/// produce a negative delta. Arrow keys off the given orientation (e.g.
/// `ArrowUp`/`ArrowDown` for `Orientation::Horizontal`) return `None`.
///
/// # Examples
///
/// ```
/// use nive_ui::interaction::{Orientation, StepAdjustment};
/// use iced::keyboard::{key::Named, Key, Modifiers};
///
/// let step = StepAdjustment::new(0.01, 0.1);
/// let delta = step.delta(
///     &Key::Named(Named::ArrowRight),
///     Modifiers::default(),
///     Orientation::Horizontal,
/// );
///
/// assert_eq!(delta, Some(0.01));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepAdjustment {
    /// Step used with no modifiers held.
    pub step: f32,
    /// Step used when Shift is held.
    pub large_step: f32,
    /// Step used when Command (macOS) or Ctrl (other) is held. Defaults to
    /// `large_step * 2.0` when not configured via [`Self::with_modifier_step`].
    pub modifier_step: Option<f32>,
}

impl StepAdjustment {
    /// Creates a step adjustment with the given normal and large (Shift) steps.
    pub const fn new(step: f32, large_step: f32) -> Self {
        Self {
            step,
            large_step,
            modifier_step: None,
        }
    }

    /// Sets an explicit step used when Command/Ctrl is held.
    pub const fn with_modifier_step(mut self, modifier_step: f32) -> Self {
        self.modifier_step = Some(modifier_step);
        self
    }

    /// Computes the signed step delta for an arrow key given the current
    /// modifiers and orientation. Returns `None` if the key is not an arrow
    /// key for the given orientation.
    pub fn delta(&self, key: &Key, modifiers: Modifiers, orientation: Orientation) -> Option<f32> {
        let Key::Named(named) = key else {
            return None;
        };

        let sign = match (orientation, named) {
            (Orientation::Horizontal, Named::ArrowRight) => 1.0,
            (Orientation::Horizontal, Named::ArrowLeft) => -1.0,
            (Orientation::Vertical, Named::ArrowDown) => 1.0,
            (Orientation::Vertical, Named::ArrowUp) => -1.0,
            _ => return None,
        };

        let magnitude = if modifiers.command() {
            self.modifier_step.unwrap_or(self.large_step * 2.0)
        } else if modifiers.shift() {
            self.large_step
        } else {
            self.step
        };

        Some(sign * magnitude)
    }
}

mod type_ahead;

#[cfg(test)]
mod keyboard_tests;

pub(crate) use type_ahead::TypeAhead;
