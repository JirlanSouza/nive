use super::*;

#[test]
fn light_and_dark_modes_are_explicit() {
    assert!(!Theme::Light.color_scheme().is_dark());
    assert!(Theme::Dark.color_scheme().is_dark());
}

#[test]
fn surfaces_use_distinct_shell_chrome() {
    let theme = Theme::Dark;

    assert_ne!(
        theme.surface(SurfaceRole::Chrome).background,
        theme.surface(SurfaceRole::Panel).background
    );
    assert_ne!(
        theme.surface(SurfaceRole::Sidebar).background,
        theme.surface(SurfaceRole::Chrome).background
    );
}

const ORDERED_STRUCTURAL_SURFACES: [SurfaceRole; 7] = [
    SurfaceRole::App,
    SurfaceRole::Chrome,
    SurfaceRole::Sidebar,
    SurfaceRole::Canvas,
    SurfaceRole::Panel,
    SurfaceRole::Elevated,
    SurfaceRole::Popover,
];
const MIN_ADJACENT_DELTA_L: f32 = 1.5;

#[test]
fn dialog_no_longer_shares_the_panel_fill() {
    let theme = Theme::Dark;

    assert_ne!(
        theme.surface(SurfaceRole::Dialog).background,
        theme.surface(SurfaceRole::Panel).background
    );
}

#[test]
fn elevation_shadow_ramp_increases_toward_dialog() {
    let theme = Theme::Dark;

    let elevated = shadow_prominence(theme.surface(SurfaceRole::Elevated).shadow);
    let popover = shadow_prominence(theme.surface(SurfaceRole::Popover).shadow);
    let dialog = shadow_prominence(theme.surface(SurfaceRole::Dialog).shadow);

    assert!(
        elevated < popover,
        "Elevated should be less prominent than Popover"
    );
    assert!(
        popover < dialog,
        "Popover should be less prominent than Dialog"
    );
}

#[test]
fn dark_surface_ramp_preserves_semantic_lightness_order() {
    let theme = Theme::Dark;
    let lightness = |role: SurfaceRole| {
        crate::theme::color::perceptual_lightness(theme.surface(role).background)
    };

    for pair in ORDERED_STRUCTURAL_SURFACES.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        assert!(
            lightness(a) < lightness(b),
            "expected {a:?} lightness to be less than {b:?}"
        );
    }
}

#[test]
fn dark_surface_ramp_clears_the_adjacency_floor() {
    let theme = Theme::Dark;
    let lightness = |role: SurfaceRole| {
        crate::theme::color::perceptual_lightness(theme.surface(role).background)
    };

    for pair in ORDERED_STRUCTURAL_SURFACES.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let delta = lightness(b) - lightness(a);

        assert!(
            delta >= MIN_ADJACENT_DELTA_L,
            "{a:?} -> {b:?} ΔL* {delta:.2} is below the {MIN_ADJACENT_DELTA_L:.2} floor"
        );
    }
}
