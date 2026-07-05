use iced::{widget::container, Background, Shadow};

use crate::advanced::control_style::border_with_radius;

use crate::theme::{self, control_metrics, BorderRole, ControlRole, ControlSize, ControlState};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentedControlMetrics {
    pub height: f32,
    pub radius: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub gap: f32,
    pub padding_h: f32,
    pub outer_padding: f32,
}

pub fn metrics(size: ControlSize) -> SegmentedControlMetrics {
    let control = control_metrics(size);
    let spacing = theme::spacing();

    SegmentedControlMetrics {
        height: control.height,
        radius: control.radius,
        font_size: control.font_size,
        icon_size: control.icon_size,
        gap: control.gap,
        padding_h: match size {
            ControlSize::Xs => spacing.sm,
            ControlSize::Sm => spacing.md,
            ControlSize::Md => spacing.md + spacing.xxs,
            ControlSize::Lg => spacing.lg,
        },
        outer_padding: spacing.xxs,
    }
}

pub fn container_style(radius: f32) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        let theme = *theme;
        let control = theme.control(ControlRole::Standard, ControlState::ENABLED);
        let border = theme.border(BorderRole::Default);

        container::Style {
            text_color: Some(control.foreground),
            background: Some(Background::Color(control.background)),
            border: border_with_radius(border, radius),
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}
