use super::*;

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
