use iced::{
    border::Radius,
    widget::{button, container},
    Background, Color, Shadow,
};

use super::super::button::button_control_state;
use super::super::control_style::{border_with_radius, transparent_border_with_radius};

use crate::theme::{
    self, control_metrics, BorderSpec, ControlRole, ControlSize, ControlState, SurfaceRole,
    TextRole,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TabPart {
    Full,
    Leading,
    Trailing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TabBarMetrics {
    pub height: f32,
    pub tab_height: f32,
    pub radius: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub close_icon_size: f32,
    pub close_side: f32,
    pub gap: f32,
    pub padding_h: f32,
    pub bar_padding_h: f32,
    pub bar_padding_v: f32,
    pub dirty_size: f32,
    pub tab_gap: f32,
}

pub fn metrics(size: ControlSize) -> TabBarMetrics {
    let control = control_metrics(size);
    let spacing = theme::spacing();
    let bar_padding_v = spacing.xs;

    TabBarMetrics {
        height: control.height + bar_padding_v * 2.0,
        tab_height: control.height,
        radius: control.radius,
        font_size: control.font_size,
        icon_size: control.icon_size,
        close_icon_size: (control.icon_size - 2.0).max(10.0),
        close_side: control.height,
        gap: control.gap,
        padding_h: match size {
            ControlSize::Xs => spacing.sm,
            ControlSize::Sm => spacing.md,
            ControlSize::Md => spacing.md + spacing.xxs,
            ControlSize::Lg => spacing.lg,
        },
        bar_padding_h: spacing.md,
        bar_padding_v,
        dirty_size: match size {
            ControlSize::Xs => 5.0,
            ControlSize::Sm => 6.0,
            ControlSize::Md | ControlSize::Lg => 7.0,
        },
        tab_gap: spacing.xxs,
    }
}

pub fn bar_style(role: SurfaceRole) -> impl Fn(&crate::theme::Theme) -> container::Style {
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

pub(crate) fn tab_style(
    selected: bool,
    part: TabPart,
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
            (true, button::Status::Disabled) => selected_control.background.scale_alpha(0.68),
            (true, _) => selected_control.background,
            (false, button::Status::Hovered | button::Status::Pressed) => control.background,
            (false, button::Status::Disabled) => Color::TRANSPARENT,
            (false, _) => Color::TRANSPARENT,
        };

        let text_color = match (selected, status) {
            (true, button::Status::Disabled) => selected_control.foreground.scale_alpha(0.62),
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
            border: border_with_radius(border, part_radius(part, radius)),
            shadow: Shadow::default(),
            ..button::Style::default()
        }
    }
}

pub fn dirty_indicator_style(size: f32) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        let accent = theme.control(ControlRole::Selectable, ControlState::SELECTED);

        container::Style {
            text_color: None,
            background: Some(Background::Color(accent.foreground)),
            border: transparent_border_with_radius(size / 2.0),
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

fn part_radius(part: TabPart, radius: f32) -> Radius {
    match part {
        TabPart::Full => Radius::new(radius),
        TabPart::Leading => Radius::default().left(radius),
        TabPart::Trailing => Radius::default().right(radius),
    }
}

#[cfg(test)]
mod tabs_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn metrics_follow_control_size() {
        assert_eq!(
            metrics(ControlSize::Sm).tab_height,
            control_metrics(ControlSize::Sm).height
        );
        assert!(metrics(ControlSize::Xs).height < metrics(ControlSize::Lg).height);
    }

    #[test]
    fn selected_tab_uses_app_selected_control_background() {
        let theme = Theme::Dark;
        let tab = tab_style(true, TabPart::Full, 6.0)(&theme, button::Status::Active);

        assert_eq!(
            background_color(&tab),
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
