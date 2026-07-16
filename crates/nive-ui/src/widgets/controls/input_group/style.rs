use iced::{border::Radius, widget::container, Background, Color, Shadow};

use crate::theme::{self, control_metrics, ControlSize, TextRole};

use crate::advanced::control_style::{alpha_when_disabled, transparent_border_with_radius};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InputGroupMetrics {
    pub height: f32,
    pub radius: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub input_padding_v: f32,
    pub input_padding_h: f32,
    pub slot_padding_h: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputGroupVariant {
    Default,
    Ghost,
}

pub fn metrics(size: ControlSize) -> InputGroupMetrics {
    let control = control_metrics(size);
    let form = theme::form_control_metrics(size);
    let spacing = theme::spacing();

    InputGroupMetrics {
        height: control.height,
        radius: control.radius,
        font_size: form.text_style.size,
        icon_size: control.icon_size,
        input_padding_v: form.content_inset,
        input_padding_h: form.padding.left,
        slot_padding_h: match size {
            ControlSize::Xs => spacing.sm,
            ControlSize::Sm => spacing.md,
            ControlSize::Md => spacing.md + spacing.xxs,
            ControlSize::Lg => spacing.xl,
        },
    }
}

pub(crate) fn slot_style(
    radius: Radius,
    disabled: bool,
    text_role: TextRole,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| container::Style {
        text_color: Some(slot_text_color(theme, disabled, text_role)),
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: transparent_border_with_radius(radius),
        shadow: Shadow::default(),
        ..container::Style::default()
    }
}

fn slot_text_color(theme: &crate::theme::Theme, disabled: bool, role: TextRole) -> Color {
    let color = theme.text(role).color;

    alpha_when_disabled(color, disabled)
}
