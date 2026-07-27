/// Semantic surface roles resolved by [`crate::theme::Theme`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceRole {
    App,
    Chrome,
    Sidebar,
    Panel,
    Elevated,
    Canvas,
    Dialog,
    Popover,
    Scrim,
}

/// Text hierarchy roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRole {
    Primary,
    Secondary,
    Muted,
    Disabled,
}

/// Semantic border roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderRole {
    Default,
    Subtle,
    Strong,
    Accent,
    Focus,
    Danger,
}

/// Semantic tone roles for status and accent color.
///
/// Use `Accent` for the product/system accent tone. `Primary` remains only in
/// text hierarchy vocabulary as [`TextRole::Primary`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToneRole {
    Neutral,
    /// Product/system accent tone.
    Accent,
    Info,
    Success,
    Warning,
    Danger,
}

/// Control chrome roles.
///
/// The role answers one question: does the control own a body of its own, or
/// does it paint on top of a host surface? It never changes state precedence —
/// only which neutral fills that precedence draws from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlRole {
    /// The control owns its body and fills it at rest: fields, the Select
    /// trigger, ordinary buttons.
    Standard,
    /// The control paints on top of whichever surface hosts it and owns no
    /// chrome: popup rows, toolbar actions, card and content actions. Untouched
    /// and disabled render nothing at all, and hover/pressed are translucent
    /// layers that read as emphasis over every host surface rather than opaque
    /// fills calibrated for one of them.
    Embedded,
    /// Like [`ControlRole::Standard`], but for a control whose body also
    /// carries durable selection: choice anchors, tabs, rail items.
    Selectable,
}

/// Pointer/keyboard interaction flags for control state resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InteractionState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub dragged: bool,
}

impl InteractionState {
    pub const NONE: Self = Self::new();
    pub const HOVERED: Self = Self::new().hovered();
    pub const PRESSED: Self = Self::new().pressed();
    pub const FOCUSED: Self = Self::new().focused();
    pub const DRAGGED: Self = Self::new().dragged();

    pub const fn new() -> Self {
        Self {
            hovered: false,
            pressed: false,
            focused: false,
            dragged: false,
        }
    }

    pub const fn hovered(self) -> Self {
        Self {
            hovered: true,
            ..self
        }
    }

    pub const fn pressed(self) -> Self {
        Self {
            pressed: true,
            ..self
        }
    }

    pub const fn focused(self) -> Self {
        Self {
            focused: true,
            ..self
        }
    }

    pub const fn dragged(self) -> Self {
        Self {
            dragged: true,
            ..self
        }
    }
}

/// Combined enabled/selected/interaction state for control styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlState {
    pub enabled: bool,
    pub selected: bool,
    pub interaction: InteractionState,
}

impl Default for ControlState {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlState {
    pub const ENABLED: Self = Self::new();
    pub const DISABLED: Self = Self::new().disabled();
    pub const SELECTED: Self = Self::new().selected();
    pub const HOVERED: Self = Self::new().interaction(InteractionState::HOVERED);
    pub const PRESSED: Self = Self::new().interaction(InteractionState::PRESSED);
    pub const FOCUSED: Self = Self::new().interaction(InteractionState::FOCUSED);

    pub const fn new() -> Self {
        Self {
            enabled: true,
            selected: false,
            interaction: InteractionState::NONE,
        }
    }

    pub const fn disabled(self) -> Self {
        Self {
            enabled: false,
            ..self
        }
    }

    pub const fn selected(self) -> Self {
        Self {
            selected: true,
            ..self
        }
    }

    pub const fn interaction(self, interaction: InteractionState) -> Self {
        Self {
            interaction,
            ..self
        }
    }
}

#[cfg(test)]
mod role_tests {
    use super::*;

    #[test]
    fn interaction_state_preserves_combined_flags() {
        let interaction = InteractionState::FOCUSED.hovered().dragged();

        assert!(interaction.focused);
        assert!(interaction.hovered);
        assert!(interaction.dragged);
        assert!(!interaction.pressed);
    }

    #[test]
    fn control_state_preserves_selection_and_interaction() {
        let state = ControlState::new()
            .selected()
            .interaction(InteractionState::FOCUSED.hovered());

        assert!(state.enabled);
        assert!(state.selected);
        assert!(state.interaction.focused);
        assert!(state.interaction.hovered);
    }
}
