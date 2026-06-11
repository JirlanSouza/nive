use iced::{
    widget::{button, container},
    Background, Color, Shadow,
};

use crate::theme::{
    self, control_metrics, BorderRole, ControlRole, ControlSize, ControlState, SurfaceRole,
    TextRole, ToneRole,
};

use super::super::button::button_control_state;
use super::super::control_style::{
    border_with_radius, transparent_border, transparent_border_with_radius,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuMetrics {
    pub item_height: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub gap: f32,
    pub radius: f32,
    pub padding: f32,
    pub item_padding_h: f32,
    pub separator_height: f32,
}

pub fn metrics(size: ControlSize) -> MenuMetrics {
    let control = control_metrics(size);
    let spacing = theme::spacing();

    MenuMetrics {
        item_height: control.height,
        font_size: control.font_size,
        icon_size: control.icon_size,
        gap: control.gap,
        radius: control.radius,
        padding: spacing.xs,
        item_padding_h: match size {
            ControlSize::Xs => spacing.sm,
            ControlSize::Sm => spacing.md,
            ControlSize::Md => spacing.md + spacing.xxs,
            ControlSize::Lg => spacing.lg,
        },
        separator_height: 1.0,
    }
}

pub fn menu_style(radius: f32) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        let surface = theme.surface(SurfaceRole::Popover);

        container::Style {
            text_color: Some(surface.foreground),
            background: Some(Background::Color(surface.background)),
            border: border_with_radius(surface.border, radius),
            shadow: surface.shadow,
            ..container::Style::default()
        }
    }
}

pub fn separator_style() -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        let border = theme.border(BorderRole::Subtle);

        container::Style {
            text_color: None,
            background: Some(Background::Color(border.color)),
            border: transparent_border(),
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

pub fn item_style(
    selected: bool,
    destructive: bool,
    radius: f32,
) -> impl Fn(&crate::theme::Theme, button::Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: button::Status| {
        let theme = *theme;
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

#[cfg(test)]
mod menu_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn selected_item_uses_app_selected_control_background() {
        let theme = Theme::Dark;
        let item = item_style(true, false, 6.0)(&theme, button::Status::Active);

        assert_eq!(
            background_color(&item),
            theme
                .control(ControlRole::Selectable, ControlState::SELECTED)
                .background
        );
    }

    fn background_color(style: &button::Style) -> Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }
}
