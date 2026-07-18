use iced::{widget::container, Background, Color, Shadow};

use crate::advanced::control_style::{transparent_border, transparent_border_with_radius};
use crate::theme::{BorderRole, ControlRole, ControlState, TextRole, ToneRole};

pub(in crate::widgets::navigation) fn surface_style(
    theme: &crate::theme::Theme,
) -> container::Style {
    let surface = theme.surface(crate::theme::SurfaceRole::Popover);

    container::Style {
        text_color: Some(surface.foreground),
        background: Some(Background::Color(surface.background)),
        border: crate::advanced::control_style::border_with_radius(surface.border, 8.0),
        shadow: surface.shadow,
        ..container::Style::default()
    }
}

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

pub(super) fn row_style(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Theme, ThemeMode};

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
}
