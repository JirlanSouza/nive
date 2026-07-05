use iced::widget::text;

use crate::theme::{
    self, control_metrics, spacing, typography, ControlSize, SpaceStep, TypographyRole,
};

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
    let spacing = spacing();
    let title = typography(TypographyRole::SectionLabel);
    let status = typography(TypographyRole::Caption);

    SectionHeaderMetrics {
        height: control_metrics(size).height,
        title_size: title.size,
        title_line_height: text::LineHeight::Relative(1.0),
        status_size: status.size,
        status_line_height: text::LineHeight::Relative(1.0),
        status_height: status_height(size),
        icon_button_side: icon_button_side(size),
        icon_size: icon_size(size),
        gap: spacing.gap(theme::GapRole::Tight),
        status_gap: spacing.step(SpaceStep::Xs),
        action_gap: spacing.step(SpaceStep::Xxs),
    }
}

fn status_height(size: ControlSize) -> f32 {
    match size {
        ControlSize::Xs => 16.0,
        ControlSize::Sm => 18.0,
        ControlSize::Md => 20.0,
        ControlSize::Lg => 22.0,
    }
}

fn icon_button_side(size: ControlSize) -> f32 {
    match size {
        ControlSize::Xs => 20.0,
        ControlSize::Sm => 22.0,
        ControlSize::Md => 24.0,
        ControlSize::Lg => 28.0,
    }
}

fn icon_size(size: ControlSize) -> f32 {
    match size {
        ControlSize::Xs => 12.0,
        ControlSize::Sm => 14.0,
        ControlSize::Md => 14.0,
        ControlSize::Lg => 16.0,
    }
}

#[cfg(test)]
mod section_header_style_tests {
    use super::*;

    #[test]
    fn compact_header_actions_are_smaller_than_standard_controls() {
        let metrics = metrics(ControlSize::Xs);

        assert!(metrics.icon_button_side < control_metrics(ControlSize::Xs).height);
        assert_eq!(metrics.height, control_metrics(ControlSize::Xs).height);
    }
}
