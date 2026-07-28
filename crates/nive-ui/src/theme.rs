mod active;
mod builder;
pub mod color;
pub mod color_scheme;
pub mod component;
pub mod density;
pub mod mode;
pub mod palette;
pub mod roles;
pub mod scheme;
pub mod shape;
pub mod spacing;
pub mod surface;
pub mod text;
pub mod typography;

mod catalog;
pub(crate) mod choice;

pub use active::{
    active, control_metrics, controls, density, form_control_metrics, gap, padding, space, spacing,
};
pub use builder::ThemeBuilder;
pub use catalog::{
    ButtonClass, CheckboxClass, ContainerClass, FieldValidation, MenuClass, PickListClass,
    ProgressBarClass, RuleClass, ScrollableClass, TextClass, TextInputClass, TogglerClass,
};
pub use color_scheme::{BorderSpec, SurfaceSpec};
pub use component::{ControlMetrics, ControlMetricsScale, ControlSize, FormControlMetrics};
pub use density::ThemeDensity;
pub use mode::{ThemeMode, ThemePreference};
pub use roles::{
    BorderRole, ControlRole, ControlState, InteractionState, SurfaceRole, TextRole, ToneRole,
};
pub use scheme::{Theme, ThemeCatalog, ThemeData, ThemeId};
pub use shape::{ShapeScale, ShapeSize, ShapeSpec};
pub use spacing::{GapRole, PaddingRole, SpaceStep, SpacingScale};
pub use typography::{typography, TextStyle, TypographyRole, TypographyScale};

/// The six colors an application supplies. Every semantic color — surface,
/// border, control, and tone — is derived from these, never set directly.
///
/// Re-exported under a qualified name because a bare `Palette` sits next to
/// `CommandPalette` in the prelude and reads as the wrong thing.
pub use iced::theme::Palette as ThemePalette;

pub use crate::tokens::color::{format_hex_color, format_rgb_hex_color, hex, parse_hex_color};

#[doc(hidden)]
pub mod runtime {
    use super::Theme;

    pub fn set_active(theme: Theme) {
        super::active::set_active(theme);
    }
}

#[doc(hidden)]
pub mod testing {
    pub use super::active::ThemeTestGuard;
}
