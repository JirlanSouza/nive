use iced::{widget::container, Background, Color, Shadow};

use crate::advanced::control_style::{transparent_border, transparent_border_with_radius};
use crate::theme::{
    choice::ResolvedChoiceState, BorderRole, ControlRole, ControlState, TextRole, ToneRole,
};

pub(super) fn separator_style() -> impl Fn(&crate::theme::Theme) -> container::Style {
    |theme: &crate::theme::Theme| {
        let border = theme.border(BorderRole::Subtle);

        container::Style {
            background: Some(Background::Color(border.color)),
            border: transparent_border(),
            ..container::Style::default()
        }
    }
}

pub(crate) fn row_style(
    selected: bool,
    destructive: bool,
    explicitly_disabled: bool,
    radius: f32,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let selected_control = theme.control(ControlRole::Selectable, ControlState::SELECTED);
        let disabled_control = theme.control(ControlRole::Standard, ControlState::DISABLED);
        let danger = theme.tone(ToneRole::Danger);

        let text_color = match (explicitly_disabled, destructive, selected) {
            (true, _, _) => disabled_control.foreground,
            (false, true, _) => danger.color,
            (false, false, true) => selected_control.foreground,
            (false, false, false) => theme.text(TextRole::Secondary).color,
        };

        container::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: Some(text_color),
            border: transparent_border_with_radius(radius),
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

/// The one fill every popup row paints for its resolved state, shared by Menu,
/// Select and Autocomplete.
///
/// Rows sit on the Popover surface and own no chrome, so [`ControlRole::Embedded`]
/// leaves an untouched or disabled row bare and projects hover/pressed as
/// translucent layers that read as emphasis over that surface rather than
/// opaque fills calibrated for a different one. Selection keeps the shared
/// selected ladder, which is role-independent.
pub(crate) fn row_fill(theme: &crate::theme::Theme, resolved: ResolvedChoiceState) -> Color {
    theme
        .control(ControlRole::Embedded, resolved.control)
        .background
}

/// [`row_style`] plus [`row_fill`], for popup lists that paint the fill through
/// the row container rather than in `draw`.
///
/// Menu and Select fill their rows in `draw` because the highlight lives in
/// widget state that only exists there; Autocomplete carries its highlight in a
/// shared cell and so resolves the whole row here.
///
/// Every flag comes from `resolved`, so no caller can transpose one: suggestion
/// rows have no destructive variant, and their highlight is a fill rather than a
/// text tone.
pub(crate) fn row_style_with_fill(
    theme: &crate::theme::Theme,
    resolved: ResolvedChoiceState,
    radius: f32,
) -> container::Style {
    container::Style {
        background: Some(Background::Color(row_fill(theme, resolved))),
        ..row_style(
            resolved.control.selected,
            false,
            !resolved.control.enabled,
            radius,
        )(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{
        choice::{ChoicePersistentState, ChoiceStateInput},
        Theme, ThemeMode,
    };

    #[test]
    fn row_and_separator_styles_resolve_from_the_supplied_custom_theme() {
        let theme = Theme::builder("Menu Test", ThemeMode::Dark)
            .text(Color::from_rgb8(0xEE, 0xED, 0xF4))
            .danger(Color::from_rgb8(0xFF, 0x66, 0x77))
            .build();

        let ordinary = row_style(false, false, false, 4.0)(&theme);
        let destructive = row_style(false, true, false, 4.0)(&theme);
        let separator = separator_style()(&theme);

        assert_eq!(
            ordinary.text_color,
            Some(theme.text(TextRole::Secondary).color)
        );
        assert_eq!(
            destructive.text_color,
            Some(theme.tone(ToneRole::Danger).color)
        );
        assert_eq!(
            separator.background,
            Some(Background::Color(theme.border(BorderRole::Subtle).color))
        );
    }

    fn resolved(
        persistent: ChoicePersistentState,
        disabled: bool,
        hovered: bool,
        pressed: bool,
    ) -> ResolvedChoiceState {
        crate::theme::choice::resolve_state(ChoiceStateInput {
            persistent,
            validation: crate::theme::FieldValidation::Valid,
            callback_present: !disabled,
            disabled,
            hovered,
            pressed,
            focused: false,
        })
    }

    #[test]
    fn an_untouched_or_disabled_row_lets_its_popover_surface_show_through() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = Theme::from_mode(mode);

            for (label, state) in [
                (
                    "untouched",
                    resolved(ChoicePersistentState::Unselected, false, false, false),
                ),
                (
                    "disabled",
                    resolved(ChoicePersistentState::Unselected, true, false, false),
                ),
                (
                    // Disabled must suppress the highlight rather than fill.
                    "disabled and hovered",
                    resolved(ChoicePersistentState::Unselected, true, true, false),
                ),
            ] {
                assert_eq!(
                    row_fill(&theme, state).a,
                    0.0,
                    "{mode:?} {label} row must not paint over the Popover surface"
                );
            }
        }
    }

    #[test]
    fn highlight_press_and_selection_each_fill_a_row() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = Theme::from_mode(mode);
            let untouched = row_fill(
                &theme,
                resolved(ChoicePersistentState::Unselected, false, false, false),
            );
            let hovered = row_fill(
                &theme,
                resolved(ChoicePersistentState::Unselected, false, true, false),
            );
            let pressed = row_fill(
                &theme,
                resolved(ChoicePersistentState::Unselected, false, true, true),
            );
            let selected = row_fill(
                &theme,
                resolved(ChoicePersistentState::Selected, false, false, false),
            );

            for (label, fill) in [
                ("hovered", hovered),
                ("pressed", pressed),
                ("selected", selected),
            ] {
                assert!(
                    fill.a > untouched.a,
                    "{mode:?} {label} row must be filled, got {fill:?}"
                );
            }
            assert!(
                pressed.a > hovered.a,
                "{mode:?} pressed must intensify past the highlight"
            );
            // The highlight is neutral and the selection is toned, so a
            // highlighted row is never mistakable for a committed one.
            assert_ne!(
                (hovered.r, hovered.g, hovered.b),
                (selected.r, selected.g, selected.b),
                "{mode:?} transient highlight must not read as committed selection"
            );
        }
    }

    #[test]
    fn a_highlighted_row_is_filled_and_keeps_ordinary_text() {
        let theme = Theme::Dark;
        let untouched = row_style_with_fill(
            &theme,
            resolved(ChoicePersistentState::Unselected, false, false, false),
            4.0,
        );
        let highlighted = row_style_with_fill(
            &theme,
            resolved(ChoicePersistentState::Unselected, false, true, false),
            4.0,
        );

        assert_eq!(
            untouched.background,
            Some(Background::Color(Color::TRANSPARENT))
        );
        assert_ne!(highlighted.background, untouched.background);
        // A highlight is a neutral fill, never a destructive text tone.
        assert_eq!(highlighted.text_color, untouched.text_color);
        assert_ne!(
            highlighted.text_color,
            Some(theme.tone(ToneRole::Danger).color)
        );
    }
}
