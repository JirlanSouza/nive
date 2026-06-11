use iced::{
    widget::{button, container},
    Background, Color, Shadow,
};

use crate::theme::{
    self, control_metrics, BorderRole, BorderSpec, ControlRole, ControlSize, ControlState,
    SurfaceRole, TextRole,
};

use super::super::button::button_control_state;
use super::super::control_style::{border_with_radius, transparent_border};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToolbarMetrics {
    pub height: f32,
    pub action_height: f32,
    pub radius: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub gap: f32,
    pub action_padding_h: f32,
    pub toolbar_padding_h: f32,
    pub toolbar_padding_v: f32,
    pub group_padding: f32,
    pub group_gap: f32,
    pub item_gap: f32,
    pub separator_height: f32,
    pub separator_width: f32,
}

pub fn metrics(size: ControlSize) -> ToolbarMetrics {
    let control = control_metrics(size);
    let spacing = theme::spacing();
    let toolbar_padding_v = spacing.xs;
    let group_padding = spacing.xxs;

    ToolbarMetrics {
        height: control.height + group_padding * 2.0 + toolbar_padding_v * 2.0,
        action_height: control.height,
        radius: control.radius,
        font_size: control.font_size,
        icon_size: control.icon_size,
        gap: control.gap,
        action_padding_h: match size {
            ControlSize::Xs => spacing.sm,
            ControlSize::Sm => spacing.md,
            ControlSize::Md => spacing.md + spacing.xxs,
            ControlSize::Lg => spacing.lg,
        },
        toolbar_padding_h: spacing.md,
        toolbar_padding_v,
        group_padding,
        group_gap: spacing.md,
        item_gap: spacing.xxs,
        separator_height: control.height - spacing.md,
        separator_width: 1.0,
    }
}

pub fn toolbar_style(role: SurfaceRole) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        let surface = theme.surface(role);

        container::Style {
            text_color: Some(surface.foreground),
            background: Some(Background::Color(surface.background)),
            border: border_with_radius(surface.border, 0.0),
            shadow: surface.shadow,
            ..container::Style::default()
        }
    }
}

pub fn group_style(radius: f32) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        let theme = *theme;
        let control = theme.control(ControlRole::Standard, ControlState::ENABLED);

        container::Style {
            text_color: Some(control.foreground),
            background: Some(Background::Color(control.background)),
            border: border_with_radius(control.border, radius),
            shadow: Shadow::default(),
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

pub fn action_style(
    selected: bool,
    radius: f32,
) -> impl Fn(&crate::theme::Theme, button::Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: button::Status| {
        let theme = *theme;
        let control = theme.control(ControlRole::Standard, button_control_state(status));
        let selected_control = theme.control(ControlRole::Selectable, ControlState::SELECTED);
        let disabled_control = theme.control(ControlRole::Standard, ControlState::DISABLED);

        let background = match (selected, status) {
            (true, button::Status::Hovered) => selected_control.background.scale_alpha(1.20),
            (true, button::Status::Pressed) => selected_control.background.scale_alpha(0.88),
            (true, button::Status::Disabled) => selected_control.background.scale_alpha(0.60),
            (true, _) => selected_control.background,
            (false, button::Status::Hovered | button::Status::Pressed) => control.background,
            (false, button::Status::Disabled) => Color::TRANSPARENT,
            (false, _) => Color::TRANSPARENT,
        };

        let text_color = match (selected, status) {
            (true, button::Status::Disabled) => selected_control.foreground.scale_alpha(0.60),
            (true, _) => selected_control.foreground,
            (false, button::Status::Hovered | button::Status::Pressed) => {
                theme.text(TextRole::Primary).color
            }
            (false, button::Status::Disabled) => disabled_control.foreground,
            (false, _) => theme.text(TextRole::Secondary).color,
        };

        let border = if selected {
            selected_control.border
        } else {
            BorderSpec::none()
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color,
            border: border_with_radius(border, radius),
            shadow: Shadow::default(),
            ..button::Style::default()
        }
    }
}

#[cfg(test)]
mod toolbar_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn metrics_follow_control_size() {
        assert_eq!(
            metrics(ControlSize::Sm).action_height,
            control_metrics(ControlSize::Sm).height
        );
        assert!(metrics(ControlSize::Xs).height < metrics(ControlSize::Lg).height);
    }

    #[test]
    fn selected_action_uses_app_selected_control_background() {
        let theme = Theme::Dark;
        let action = action_style(true, 6.0)(&theme, button::Status::Active);

        assert_eq!(
            background_color(&action),
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
