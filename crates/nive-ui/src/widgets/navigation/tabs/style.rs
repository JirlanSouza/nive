use iced::{widget::container, Background, Shadow};

use crate::advanced::control_group::{radius_for_position, SlotPosition};
use crate::advanced::control_style::{border_with_radius, transparent_border_with_radius};

use crate::theme::{self, ControlRole, ControlSize, ControlState, SurfaceRole, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TabPart {
    Full,
    Leading,
    Trailing,
}

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

pub fn dirty_indicator_style(size: f32) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| container::Style {
        text_color: None,
        background: Some(Background::Color(insertion_marker_color(theme))),
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

pub(crate) fn slot_position_for_part(part: TabPart) -> SlotPosition {
    match part {
        TabPart::Full => SlotPosition::Single,
        TabPart::Leading => SlotPosition::First,
        TabPart::Trailing => SlotPosition::Last,
    }
}

pub(crate) fn part_radius(part: TabPart, radius: f32) -> iced::border::Radius {
    radius_for_position(slot_position_for_part(part), radius)
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
            }
        }
    }

    #[test]
    fn tab_parts_map_to_group_slot_positions() {
        assert_eq!(
            slot_position_for_part(TabPart::Leading),
            SlotPosition::First
        );
        assert_eq!(
            slot_position_for_part(TabPart::Trailing),
            SlotPosition::Last
        );
        assert_eq!(slot_position_for_part(TabPart::Full), SlotPosition::Single);
    }
}
