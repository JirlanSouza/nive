use iced::{widget::container, Background, Shadow};

use crate::theme::{self, BorderRole, ControlSize, SurfaceRole, Theme};

use crate::advanced::control_style::{border_with_radius, transparent_border};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToolbarMetrics {
    pub size: ControlSize,
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
    metrics_for_theme(theme::active(), size)
}

fn metrics_for_theme(theme: Theme, size: ControlSize) -> ToolbarMetrics {
    let control = theme.control_metrics(size);
    let spacing = theme.spacing();
    let toolbar_padding_v = spacing.xs;
    ToolbarMetrics {
        size,
        height: control.height + toolbar_padding_v * 2.0,
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
        group_padding: 0.0,
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

#[cfg(test)]
mod toolbar_tests {
    use super::*;

    #[test]
    fn metrics_follow_control_size() {
        assert_eq!(
            metrics(ControlSize::Sm).action_height,
            theme::control_metrics(ControlSize::Sm).height
        );
        assert!(metrics(ControlSize::Xs).height < metrics(ControlSize::Lg).height);
        assert_eq!(metrics(ControlSize::Sm).group_padding, 0.0);
        assert_eq!(
            metrics(ControlSize::Sm).height,
            metrics(ControlSize::Sm).action_height
                + metrics(ControlSize::Sm).toolbar_padding_v * 2.0
        );
    }

    #[test]
    fn action_height_matches_control_metrics_across_densities_and_sizes() {
        for density in crate::theme::ThemeDensity::ALL {
            let theme =
                crate::theme::Theme::builder("Toolbar metric test", crate::theme::ThemeMode::Dark)
                    .density(density)
                    .build();

            for size in [
                ControlSize::Xs,
                ControlSize::Sm,
                ControlSize::Md,
                ControlSize::Lg,
            ] {
                assert_eq!(
                    metrics_for_theme(theme, size).action_height,
                    theme.control_metrics(size).height
                );
            }
        }
    }
}
