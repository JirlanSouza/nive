use iced::{widget::container, Background, Border, Color, Shadow};

use crate::advanced::control_style::transparent_border_with_radius;

use crate::theme::{self, ControlRole, ControlSize, ControlState, SurfaceRole, Theme};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TabBarMetrics {
    pub size: ControlSize,
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
    pub min_tab_width: f32,
    pub max_tab_width: f32,
    pub status_side: f32,
    pub seam_width: f32,
    pub indicator_width: f32,
}

pub fn metrics(size: ControlSize) -> TabBarMetrics {
    metrics_for_theme(theme::active(), size)
}

fn metrics_for_theme(theme: Theme, size: ControlSize) -> TabBarMetrics {
    let control = theme.control_metrics(size);
    let spacing = theme.spacing();
    let bar_padding_v = 0.0;

    TabBarMetrics {
        size,
        height: control.height,
        tab_height: control.height,
        radius: control.radius,
        font_size: control.font_size.max(14.0),
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
        min_tab_width: (control.height * 3.0).max(96.0),
        max_tab_width: 240.0,
        status_side: spacing.sm,
        seam_width: 1.0,
        indicator_width: 2.0,
    }
}

pub fn bar_style(role: SurfaceRole) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        let surface = theme.surface(role);

        container::Style {
            text_color: Some(surface.foreground),
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border::default(),
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

pub(super) fn tab_content_style(
    selected: bool,
    disabled: bool,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let state = if disabled {
            ControlState::DISABLED
        } else if selected {
            ControlState::SELECTED
        } else {
            ControlState::ENABLED
        };
        let foreground = if disabled {
            theme.control(ControlRole::Selectable, state).foreground
        } else if selected {
            theme.text(crate::theme::TextRole::Primary).color
        } else {
            theme.text(crate::theme::TextRole::Secondary).color
        };

        container::Style {
            text_color: Some(foreground),
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border::default(),
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

pub(super) fn tab_background(
    theme: &Theme,
    active_role: SurfaceRole,
    selected: bool,
    hovered: bool,
    pressed: bool,
    disabled: bool,
) -> Color {
    if disabled {
        return if selected {
            theme
                .control(ControlRole::Selectable, ControlState::DISABLED.selected())
                .background
        } else {
            Color::TRANSPARENT
        };
    }

    if pressed {
        let state = if selected {
            ControlState::PRESSED.selected()
        } else {
            ControlState::PRESSED
        };
        return theme.control(ControlRole::Selectable, state).background;
    }
    if hovered {
        let state = if selected {
            ControlState::HOVERED.selected()
        } else {
            ControlState::HOVERED
        };
        return theme.control(ControlRole::Selectable, state).background;
    }
    if selected {
        theme.surface(active_role).background
    } else {
        Color::TRANSPARENT
    }
}

pub(super) fn strip_background(theme: &Theme, role: SurfaceRole) -> Color {
    theme.surface(role).background
}

pub(super) fn strip_divider(theme: &Theme, role: SurfaceRole) -> Color {
    theme.surface(role).border.color
}

pub(super) fn active_indicator(theme: &Theme) -> Color {
    theme
        .control(ControlRole::Selectable, ControlState::SELECTED)
        .foreground
}

pub(super) fn status_indicator_style(
    size: f32,
    visible: bool,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| container::Style {
        text_color: None,
        background: Some(Background::Color(if visible {
            insertion_marker_color(theme)
        } else {
            Color::TRANSPARENT
        })),
        border: transparent_border_with_radius(size / 2.0),
        shadow: Shadow::default(),
        ..container::Style::default()
    }
}

pub(super) fn insertion_marker_color(theme: &crate::theme::Theme) -> iced::Color {
    theme
        .control(ControlRole::Selectable, ControlState::SELECTED)
        .foreground
}

#[cfg(test)]
mod tabs_tests {
    use super::*;
    #[test]
    fn metrics_follow_control_size() {
        assert_eq!(
            metrics(ControlSize::Sm).height,
            theme::control_metrics(ControlSize::Sm).height
        );
        assert_eq!(
            metrics(ControlSize::Sm).tab_height,
            theme::control_metrics(ControlSize::Sm).height
        );
        assert_eq!(metrics(ControlSize::Sm).bar_padding_v, 0.0);
        assert!(metrics(ControlSize::Xs).height < metrics(ControlSize::Lg).height);
    }

    #[test]
    fn outer_height_matches_control_metrics_across_densities_and_sizes() {
        for density in crate::theme::ThemeDensity::ALL {
            let theme =
                crate::theme::Theme::builder("TabBar metric test", crate::theme::ThemeMode::Dark)
                    .density(density)
                    .build();

            for size in [
                ControlSize::Xs,
                ControlSize::Sm,
                ControlSize::Md,
                ControlSize::Lg,
            ] {
                assert_eq!(
                    metrics_for_theme(theme, size).height,
                    theme.control_metrics(size).height
                );
                assert_eq!(
                    metrics_for_theme(theme, size).tab_height,
                    theme.control_metrics(size).height
                );
                assert_eq!(metrics_for_theme(theme, size).bar_padding_v, 0.0);
                assert!(metrics_for_theme(theme, size).font_size >= 14.0);
                assert_eq!(metrics_for_theme(theme, size).max_tab_width, 240.0);
                assert_eq!(
                    metrics_for_theme(theme, size).min_tab_width,
                    (theme.control_metrics(size).height * 3.0).max(96.0)
                );
            }
        }
    }

    #[test]
    fn bar_style_is_square_transparent_and_shadowless_for_custom_draw_order() {
        let style = bar_style(SurfaceRole::Chrome)(&Theme::Dark);

        assert_eq!(
            style.background,
            Some(Background::Color(Color::TRANSPARENT))
        );
        assert_eq!(style.border, Border::default());
        assert_eq!(style.shadow, Shadow::default());
    }
}
