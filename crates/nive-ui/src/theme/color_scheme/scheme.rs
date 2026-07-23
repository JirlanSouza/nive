mod build;
mod helpers;

#[cfg(test)]
mod color_scheme_tests;

use iced::{Color, Shadow};

/// Combined-selected-state alpha ladder for [`ColorScheme::selected_control_spec`].
/// Centralized here so no widget scales these fills locally.
const SELECTED_HOVER_ALPHA: f32 = 1.20;
const SELECTED_PRESSED_ALPHA: f32 = 0.88;
const DISABLED_SELECTED_ALPHA: f32 = 0.60;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderSpec {
    pub color: Color,
    pub width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceSpec {
    pub background: Color,
    pub foreground: Color,
    pub border: BorderSpec,
    pub shadow: Shadow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextSpec {
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlSpec {
    pub background: Color,
    pub foreground: Color,
    pub border: BorderSpec,
    pub focus: BorderSpec,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToneSpec {
    pub color: Color,
    pub on_color: Color,
    pub container: Color,
    pub on_container: Color,
    pub border: BorderSpec,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorScheme {
    is_dark: bool,
    surface: SurfaceColors,
    text: TextColors,
    border: BorderColors,
    control: ControlColors,
    tone: ToneColors,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SurfaceColors {
    app: Color,
    chrome: Color,
    sidebar: Color,
    panel: Color,
    elevated: Color,
    overlay: Color,
    canvas: Color,
    scrim: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TextColors {
    primary: Color,
    secondary: Color,
    muted: Color,
    disabled: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct BorderColors {
    subtle: Color,
    default: Color,
    strong: Color,
    focus: Color,
    accent: Color,
    danger: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ControlColors {
    active: Color,
    hover: Color,
    pressed: Color,
    disabled: Color,
    selected: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ToneColors {
    neutral: ToneColorsForRole,
    primary: ToneColorsForRole,
    info: ToneColorsForRole,
    success: ToneColorsForRole,
    warning: ToneColorsForRole,
    danger: ToneColorsForRole,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ToneColorsForRole {
    color: Color,
    on_color: Color,
    container: Color,
    on_container: Color,
    border: Color,
}
