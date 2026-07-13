use iced::widget::text;

use crate::theme::{self, ControlSize, SpaceStep, Theme, TypographyRole};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SectionHeaderMetrics {
    pub height: f32,
    pub title_size: f32,
    pub title_line_height: text::LineHeight,
    pub status_size: f32,
    pub status_line_height: text::LineHeight,
    pub status_height: f32,
    pub icon_button_side: f32,
    pub icon_size: f32,
    pub gap: f32,
    pub status_gap: f32,
    pub action_gap: f32,
}

pub fn metrics(size: ControlSize) -> SectionHeaderMetrics {
    metrics_for_theme(theme::active(), size)
}

fn metrics_for_theme(theme: Theme, size: ControlSize) -> SectionHeaderMetrics {
    let spacing = theme.spacing();
    let control = theme.control_metrics(size);
    let title = theme.typography(TypographyRole::SectionLabel);
    let status = theme.typography(TypographyRole::Caption);

    SectionHeaderMetrics {
        height: control.height,
        title_size: title.size,
        title_line_height: text::LineHeight::Relative(1.0),
        status_size: status.size,
        status_line_height: text::LineHeight::Relative(1.0),
        status_height: status_height(size, control.height),
        icon_button_side: icon_button_side(size, control.height),
        icon_size: control.icon_size,
        gap: spacing.gap(theme::GapRole::Tight),
        status_gap: spacing.step(SpaceStep::Xs),
        action_gap: spacing.step(SpaceStep::Xxs),
    }
}

fn status_height(size: ControlSize, control_height: f32) -> f32 {
    match size {
        ControlSize::Xs => control_height - 8.0,
        ControlSize::Sm => control_height - 10.0,
        ControlSize::Md => control_height - 12.0,
        ControlSize::Lg => control_height - 14.0,
    }
}

fn icon_button_side(size: ControlSize, control_height: f32) -> f32 {
    match size {
        ControlSize::Xs => control_height - 4.0,
        ControlSize::Sm => control_height - 6.0,
        ControlSize::Md => control_height - 8.0,
        ControlSize::Lg => control_height - 8.0,
    }
}

#[cfg(test)]
mod section_header_style_tests {
    use super::*;

    #[test]
    fn compact_header_actions_are_smaller_than_standard_controls() {
        let metrics = metrics(ControlSize::Xs);

        assert!(metrics.icon_button_side < theme::control_metrics(ControlSize::Xs).height);
        assert_eq!(
            metrics.height,
            theme::control_metrics(ControlSize::Xs).height
        );
    }

    #[test]
    fn height_matches_control_metrics_across_densities_and_sizes() {
        for density in crate::theme::ThemeDensity::ALL {
            let theme = crate::theme::Theme::builder(
                "SectionHeader metric test",
                crate::theme::ThemeMode::Dark,
            )
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
            }
        }
    }
}
