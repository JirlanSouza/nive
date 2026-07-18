use iced::{
    widget::{button, container},
    Background, Color, Shadow,
};

use crate::advanced::control_style::{transparent_border, transparent_border_with_radius};
use crate::theme::{BorderRole, ControlRole, ControlState, TextRole, ToneRole};
use crate::widgets::controls::button::button_control_state;

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

pub(super) fn item_style(
    selected: bool,
    destructive: bool,
    explicitly_disabled: bool,
    radius: f32,
) -> impl Fn(&crate::theme::Theme, button::Status) -> button::Style {
    move |theme, status| {
        let status = if status == button::Status::Disabled && !explicitly_disabled {
            button::Status::Active
        } else {
            status
        };
        let control = theme.control(ControlRole::Standard, button_control_state(status));
        let selected_control = theme.control(ControlRole::Selectable, ControlState::SELECTED);
        let disabled_control = theme.control(ControlRole::Standard, ControlState::DISABLED);
        let danger = theme.tone(ToneRole::Danger);

        let background = match (selected, status) {
            (true, button::Status::Hovered) => selected_control.background.scale_alpha(1.20),
            (true, button::Status::Pressed) => selected_control.background.scale_alpha(0.88),
            (true, button::Status::Disabled) => selected_control.background.scale_alpha(0.55),
            (true, _) => selected_control.background,
            (false, button::Status::Hovered | button::Status::Pressed) => control.background,
            (false, _) => Color::TRANSPARENT,
        };
        let text_color = match (destructive, selected, status) {
            (_, _, button::Status::Disabled) => disabled_control.foreground,
            (true, _, _) => danger.color,
            (false, true, _) => selected_control.foreground,
            (false, false, button::Status::Hovered | button::Status::Pressed) => {
                theme.text(TextRole::Primary).color
            }
            (false, false, _) => theme.text(TextRole::Secondary).color,
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color,
            border: transparent_border_with_radius(radius),
            shadow: Shadow::default(),
            ..button::Style::default()
        }
    }
}
