use super::*;

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
