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

fn shadow_prominence(shadow: Shadow) -> f32 {
    shadow.color.a * shadow.blur_radius
}

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

#[test]
fn tones_are_visual_not_domain_specific() {
    let theme = Theme::Dark;

    assert_eq!(
        theme.tone(ToneRole::Accent).color,
        theme
            .control(ControlRole::Selectable, ControlState::SELECTED)
            .foreground
    );
    assert_eq!(
        theme.tone(ToneRole::Accent).border,
        theme
            .control(ControlRole::Selectable, ControlState::SELECTED)
            .border
    );
}

#[test]
fn accent_tone_reuses_the_palette_primary_pair() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let scheme = Theme::from_mode(mode).color_scheme();

        assert_eq!(scheme.tone(ToneRole::Accent), scheme.tone.primary.spec());
    }
}

#[test]
fn focused_control_uses_focus_border() {
    let theme = Theme::Dark;

    assert_eq!(
        theme
            .control(ControlRole::Standard, ControlState::FOCUSED)
            .border,
        theme.border(BorderRole::Focus)
    );
}

#[test]
fn focused_hovered_control_keeps_focus_border_and_hover_background() {
    let theme = Theme::Dark;
    let focused_hovered = ControlState::new().interaction(InteractionState::FOCUSED.hovered());
    let control = theme.control(ControlRole::Standard, focused_hovered);

    assert_eq!(control.border, theme.border(BorderRole::Focus));
    assert_eq!(
        control.background,
        theme
            .control(ControlRole::Standard, ControlState::HOVERED)
            .background
    );
}

#[test]
fn disabled_unselected_control_ignores_interaction() {
    let theme = Theme::Dark;
    let disabled = theme.control(ControlRole::Standard, ControlState::DISABLED);
    let combined = ControlState::new()
        .disabled()
        .interaction(InteractionState::PRESSED.hovered().focused());

    assert_eq!(theme.control(ControlRole::Standard, combined), disabled);
}

#[test]
fn disabled_selected_control_suppresses_hover_and_pressed() {
    let theme = Theme::Dark;
    let idle_disabled_selected = theme.control(
        ControlRole::Selectable,
        ControlState::new().selected().disabled(),
    );
    let combined = ControlState::new()
        .selected()
        .disabled()
        .interaction(InteractionState::PRESSED.hovered());

    assert_eq!(
        theme.control(ControlRole::Selectable, combined),
        idle_disabled_selected
    );
}

#[test]
fn disabled_selected_control_uses_a_canonical_dimmed_selected_fill() {
    let theme = Theme::Dark;
    let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);
    let disabled_selected = theme.control(
        ControlRole::Selectable,
        ControlState::new().selected().disabled(),
    );

    assert_eq!(
        disabled_selected.background,
        selected.background.scale_alpha(0.60)
    );
    assert_eq!(
        disabled_selected.foreground,
        selected.foreground.scale_alpha(0.60)
    );
    assert_ne!(
        disabled_selected,
        theme.control(ControlRole::Standard, ControlState::DISABLED)
    );
}

#[test]
fn selected_hover_and_pressed_intensify_the_selected_fill_centrally() {
    let theme = Theme::Dark;
    let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);
    let selected_hovered = theme.control(
        ControlRole::Selectable,
        ControlState::new()
            .selected()
            .interaction(InteractionState::HOVERED),
    );
    let selected_pressed = theme.control(
        ControlRole::Selectable,
        ControlState::new()
            .selected()
            .interaction(InteractionState::PRESSED),
    );

    assert_eq!(
        selected_hovered.background,
        selected.background.scale_alpha(1.20)
    );
    assert_eq!(
        selected_pressed.background,
        selected.background.scale_alpha(0.88)
    );
    assert_eq!(selected_hovered.foreground, selected.foreground);
    assert_eq!(selected_pressed.foreground, selected.foreground);
}

#[test]
fn selected_focus_and_selected_idle_are_visually_distinguishable() {
    let theme = Theme::Dark;
    let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);
    let selected_focused = theme.control(
        ControlRole::Selectable,
        ControlState::new()
            .selected()
            .interaction(InteractionState::FOCUSED),
    );

    assert_ne!(selected.border, selected_focused.border);
    assert_eq!(selected_focused.background, selected.background);
}

#[test]
fn embedded_role_leaves_its_host_surface_bare_when_untouched_or_disabled() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let theme = Theme::from_mode(mode);

        for state in [ControlState::ENABLED, ControlState::DISABLED] {
            let fill = theme.control(ControlRole::Embedded, state).background;

            assert_eq!(
                fill.a, 0.0,
                "{mode:?} embedded {state:?} must add no emphasis beyond its host surface"
            );
        }
    }
}

#[test]
fn body_owning_roles_still_fill_themselves_when_untouched() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let theme = Theme::from_mode(mode);

        for role in [ControlRole::Standard, ControlRole::Selectable] {
            let fill = theme.control(role, ControlState::ENABLED).background;

            assert_eq!(
                fill.a, 1.0,
                "{mode:?} {role:?} owns its body and must keep filling it at rest"
            );
        }
    }
}

#[test]
fn embedded_transient_fills_are_translucent_and_ordered() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let theme = Theme::from_mode(mode);
        let hover = theme
            .control(ControlRole::Embedded, ControlState::HOVERED)
            .background;
        let pressed = theme
            .control(ControlRole::Embedded, ControlState::PRESSED)
            .background;

        for (label, layer) in [("hover", hover), ("pressed", pressed)] {
            assert!(
                layer.a > 0.0 && layer.a < 1.0,
                "{mode:?} embedded {label} must be translucent to composite over any host, got {layer:?}"
            );
        }
        assert!(
            pressed.a > hover.a,
            "{mode:?} pressed must intensify past hover, got {} vs {}",
            pressed.a,
            hover.a
        );
    }
}

#[test]
fn embedded_emphasis_reads_the_same_on_every_host_surface() {
    // The point of a translucent layer: one emphasis weight everywhere, instead
    // of the opaque tokens' accidental weight, which grew stronger the darker
    // the host happened to be — 0.045 on Panel against 0.093 on Chrome.
    //
    // Calibrated against separation from the host rather than against the
    // opaque token. Matching that token on Panel is what left the layer at
    // ~0.043 on the darker Sidebar, where it read as nothing at all.
    const MIN_SEPARATION: f32 = 0.05;
    const MAX_SPREAD: f32 = 0.03;

    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let theme = Theme::from_mode(mode);
        let foreground = luminance(theme.text(TextRole::Primary).color);
        let mut separations = Vec::new();

        for role in [
            SurfaceRole::Panel,
            SurfaceRole::Sidebar,
            SurfaceRole::Chrome,
            SurfaceRole::Popover,
        ] {
            let host = theme.surface(role).background;
            let hovered = composite(
                theme
                    .control(ControlRole::Embedded, ControlState::HOVERED)
                    .background,
                host,
            );
            let pressed = composite(
                theme
                    .control(ControlRole::Embedded, ControlState::PRESSED)
                    .background,
                host,
            );
            let host = luminance(host);
            let separation = (luminance(hovered) - host).abs();

            assert!(
                separation > MIN_SEPARATION,
                "{mode:?} hover on {role:?} separates by only {separation:.4}"
            );
            assert_eq!(
                luminance(hovered) > host,
                foreground > host,
                "{mode:?} hover on {role:?} must move toward the foreground, not away"
            );
            assert!(
                (luminance(pressed) - host).abs() > separation,
                "{mode:?} pressed on {role:?} must intensify past hover"
            );

            separations.push(separation);
        }

        let spread = separations.iter().fold(f32::MIN, |a, b| a.max(*b))
            - separations.iter().fold(f32::MAX, |a, b| a.min(*b));
        assert!(
            spread < MAX_SPREAD,
            "{mode:?} emphasis varies by {spread:.4} across host surfaces, which is the \
             surface-dependence the layer exists to remove"
        );
    }
}

#[test]
fn embedded_and_body_owning_roles_share_one_selected_ladder() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let theme = Theme::from_mode(mode);

        for interaction in [
            InteractionState::NONE,
            InteractionState::HOVERED,
            InteractionState::PRESSED,
            InteractionState::FOCUSED,
        ] {
            for enabled in [true, false] {
                let mut state = ControlState::new().selected().interaction(interaction);
                if !enabled {
                    state = state.disabled();
                }

                assert_eq!(
                    theme.control(ControlRole::Embedded, state),
                    theme.control(ControlRole::Selectable, state),
                    "{mode:?} selection is role-independent, but {state:?} diverged"
                );
            }
        }
    }
}

/// Source-over composite of a translucent layer onto an opaque host, matching
/// what the renderer does with the fill the theme hands back.
fn composite(layer: Color, host: Color) -> Color {
    super::helpers::mix(host, layer, layer.a)
}

#[test]
fn focus_border_is_brighter_than_accent() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let theme = Theme::from_mode(mode);
        let focus = theme.border(BorderRole::Focus).color;
        let accent = theme.tone(ToneRole::Accent).color;

        assert!(luminance(focus) > luminance(accent));
        assert!(focus.a > 0.90);
    }
}

#[test]
fn focus_border_stays_visually_close_to_accent() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let theme = Theme::from_mode(mode);
        let focus = theme.border(BorderRole::Focus).color;
        let accent = theme.tone(ToneRole::Accent).color;

        assert!(color_distance(focus, accent) < 0.22);
    }
}

#[test]
fn text_roles_keep_readable_contrast_on_representative_surfaces() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let theme = Theme::from_mode(mode);

        for surface_role in REPRESENTATIVE_SURFACES {
            let surface = theme.surface(surface_role);

            for (text_role, minimum) in TEXT_ROLE_CASES {
                let text = theme.text(text_role);
                assert_contrast_at_least(
                    format!("{mode:?} {text_role:?} over {surface_role:?}"),
                    text.color,
                    surface.background,
                    minimum,
                );
            }
        }
    }
}

#[test]
fn muted_text_clears_the_3_to_1_floor_on_base_surfaces() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let theme = Theme::from_mode(mode);
        let muted = theme.text(TextRole::Muted).color;

        for surface_role in MUTED_FLOOR_SURFACES {
            let background = theme.surface(surface_role).background;
            assert_contrast_at_least(
                format!("{mode:?} Muted over {surface_role:?}"),
                muted,
                background,
                MUTED_CONTRAST_FLOOR,
            );
        }
    }
}

#[test]
fn muted_text_stays_below_secondary_contrast() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let theme = Theme::from_mode(mode);
        let muted = theme.text(TextRole::Muted).color;
        let secondary = theme.text(TextRole::Secondary).color;

        for surface_role in MUTED_FLOOR_SURFACES {
            let background = theme.surface(surface_role).background;
            let muted_contrast = crate::theme::color::contrast_ratio(muted, background);
            let secondary_contrast = crate::theme::color::contrast_ratio(secondary, background);

            assert!(
                muted_contrast < secondary_contrast,
                "{mode:?} Muted over {surface_role:?} contrast {muted_contrast:.2} \
                 should stay below Secondary contrast {secondary_contrast:.2}"
            );
        }
    }
}

#[test]
fn tone_roles_keep_readable_foreground_background_contrast() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let theme = Theme::from_mode(mode);
        let panel_background = theme.surface(SurfaceRole::Panel).background;

        for tone_role in TONE_ROLES {
            let tone = theme.tone(tone_role);
            let background = composite_over(tone.container, panel_background);

            assert_contrast_at_least(
                format!("{mode:?} {tone_role:?} tone"),
                tone.color,
                background,
                MIN_TONE_CONTRAST,
            );
        }
    }
}

#[test]
fn on_accent_text_reads_over_accent_in_light_and_dark_modes() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let theme = Theme::from_mode(mode);
        let accent = theme.tone(ToneRole::Accent).color;

        assert_contrast_at_least(
            format!("{mode:?} on Accent"),
            theme.tone(ToneRole::Accent).on_color,
            accent,
            MIN_ON_ACCENT_CONTRAST,
        );
    }
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
