use iced::{
    widget::button::{self, Status},
    Background,
};

use crate::advanced::control_style::{border_with_radius, disabled_alpha};
use crate::widgets::controls::button::button_control_state;

use crate::theme::{
    self, BorderRole, ControlRole, ControlState, ShapeSize, SpaceStep, SurfaceRole, TextRole,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionCardMetrics {
    pub padding: f32,
    pub radius: f32,
}

pub fn metrics() -> ActionCardMetrics {
    ActionCardMetrics {
        padding: theme::space(SpaceStep::Md),
        radius: theme::active().shape(ShapeSize::Lg).radius_value(),
    }
}

pub fn style(
    role: SurfaceRole,
    selected: bool,
    radius: f32,
) -> impl Fn(&crate::theme::Theme, Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: Status| {
        let theme = *theme;
        let surface = theme.surface(role);
        let control = theme.control(ControlRole::Standard, button_control_state(status));
        let selected_control = theme.control(ControlRole::Selectable, ControlState::SELECTED);
        let disabled_control = theme.control(ControlRole::Standard, ControlState::DISABLED);

        let background = match (selected, status) {
            (true, Status::Hovered) => selected_control.background.scale_alpha(1.20),
            (true, Status::Pressed) => selected_control.background.scale_alpha(0.88),
            (true, Status::Disabled) => disabled_control.background,
            (true, _) => selected_control.background,
            (false, Status::Hovered | Status::Pressed) => control.background,
            (false, Status::Disabled) => disabled_control.background,
            (false, _) => surface.background,
        };

        let border = if selected {
            selected_control.border
        } else if matches!(status, Status::Hovered | Status::Pressed) {
            theme.border(BorderRole::Default)
        } else {
            surface.border
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: content_color(&theme, selected, status),
            border: border_with_radius(border, radius),
            shadow: surface.shadow,
            ..button::Style::default()
        }
    }
}

fn content_color(theme: &crate::theme::Theme, selected: bool, status: Status) -> iced::Color {
    let theme = *theme;

    match (selected, status) {
        (_, Status::Disabled) => disabled_alpha(theme.text(TextRole::Muted).color),
        (true, _) => theme.text(TextRole::Primary).color,
        (false, Status::Hovered | Status::Pressed) => theme.text(TextRole::Primary).color,
        (false, _) => theme.text(TextRole::Secondary).color,
    }
}

#[cfg(test)]
mod action_card_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn selected_card_uses_app_selected_control_background() {
        let theme = Theme::Dark;
        let card_style = style(SurfaceRole::Panel, true, metrics().radius)(&theme, Status::Active);

        assert_eq!(
            background_color(&card_style),
            theme
                .control(ControlRole::Selectable, ControlState::SELECTED)
                .background
        );
    }

    fn background_color(style: &button::Style) -> iced::Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }
}
