use super::helpers::luminance;
use super::*;
use crate::theme::{
    BorderRole, ControlRole, ControlState, InteractionState, SurfaceRole, TextRole, Theme,
    ThemeMode, ToneRole,
};

const REPRESENTATIVE_SURFACES: [SurfaceRole; 8] = [
    SurfaceRole::App,
    SurfaceRole::Chrome,
    SurfaceRole::Sidebar,
    SurfaceRole::Panel,
    SurfaceRole::Elevated,
    SurfaceRole::Canvas,
    SurfaceRole::Dialog,
    SurfaceRole::Popover,
];
const MUTED_FLOOR_SURFACES: [SurfaceRole; 5] = [
    SurfaceRole::App,
    SurfaceRole::Chrome,
    SurfaceRole::Sidebar,
    SurfaceRole::Canvas,
    SurfaceRole::Panel,
];
const MUTED_CONTRAST_FLOOR: f32 = 3.0;
const TEXT_ROLE_CASES: [(TextRole, f32); 4] = [
    (TextRole::Primary, 4.5),
    (TextRole::Secondary, 4.5),
    (TextRole::Muted, 2.25),
    (TextRole::Disabled, 2.0),
];
const TONE_ROLES: [ToneRole; 6] = [
    ToneRole::Neutral,
    ToneRole::Accent,
    ToneRole::Info,
    ToneRole::Success,
    ToneRole::Warning,
    ToneRole::Danger,
];
const MIN_TONE_CONTRAST: f32 = 2.25;
const MIN_ON_ACCENT_CONTRAST: f32 = 4.5;

fn shadow_prominence(shadow: Shadow) -> f32 {
    shadow.color.a * shadow.blur_radius
}

/// Source-over composite of a translucent layer onto an opaque host, matching
/// what the renderer does with the fill the theme hands back.
fn composite(layer: Color, host: Color) -> Color {
    super::helpers::mix(host, layer, layer.a)
}

fn assert_contrast_at_least(label: String, foreground: Color, background: Color, minimum: f32) {
    let contrast = crate::theme::color::contrast_ratio(foreground, background);

    assert!(
        contrast >= minimum,
        "{label} contrast {contrast:.2} is below {minimum:.2}"
    );
}

fn color_distance(first: Color, second: Color) -> f32 {
    let dr = first.r - second.r;
    let dg = first.g - second.g;
    let db = first.b - second.b;

    (dr * dr + dg * dg + db * db).sqrt()
}

fn composite_over(foreground: Color, background: Color) -> Color {
    Color {
        r: foreground.r * foreground.a + background.r * (1.0 - foreground.a),
        g: foreground.g * foreground.a + background.g * (1.0 - foreground.a),
        b: foreground.b * foreground.a + background.b * (1.0 - foreground.a),
        a: foreground.a + background.a * (1.0 - foreground.a),
    }
}

mod control;
mod surfaces;
mod text;
mod tone;
