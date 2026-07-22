use std::borrow::Cow;

use iced::{
    widget::{container, row, text},
    Alignment, Border, Length, Shadow,
};

use super::{FieldHint, FieldLabel, FieldMetrics, FieldRequirement};
use crate::theme::{self, text as theme_text, SpaceStep, TextRole, ToneRole, TypographyRole};
use crate::Element;

pub(super) fn metrics() -> FieldMetrics {
    let support = theme::typography(TypographyRole::BodySmall);
    FieldMetrics {
        label_to_control_gap: theme::space(SpaceStep::Sm),
        control_to_support_gap: theme::space(SpaceStep::Xs),
        requirement_gap: theme::space(SpaceStep::Xs),
        support_line_height: support.size * support.line_height,
    }
}

pub(in crate::widgets::controls) fn normalized_error<'a>(
    error: Option<Cow<'a, str>>,
) -> Option<Cow<'a, str>> {
    error.filter(|value| !value.trim().is_empty())
}

pub(super) fn sanitize_minimum(minimum: f32) -> f32 {
    if minimum.is_finite() && minimum > 0.0 {
        minimum
    } else {
        240.0
    }
}

pub(super) fn field_label<'a, Message>(
    label: Cow<'a, str>,
    requirement: Option<FieldRequirement<'a>>,
    gap: f32,
) -> Element<'a, Message>
where
    Message: 'a,
{
    let mut content = row![FieldLabel::new(label)]
        .spacing(gap)
        .align_y(Alignment::Center)
        .width(Length::Fill);
    if let Some(requirement) = requirement {
        let requirement = match requirement {
            FieldRequirement::Required(text) | FieldRequirement::Optional(text) => text,
        };
        content = content.push(FieldHint::new(requirement));
    }

    content.wrap().into()
}

pub(super) fn style(theme: &crate::theme::Theme) -> container::Style {
    container::Style {
        text_color: Some(theme.text(TextRole::Primary).color),
        background: None,
        border: Border::default(),
        shadow: Shadow::default(),
        ..container::Style::default()
    }
}

pub(super) fn label_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    theme_text::style(TextRole::Primary)
}

pub(super) fn hint_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    theme_text::style(TextRole::Secondary)
}

pub(super) fn error_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    theme_text::tone(ToneRole::Danger)
}
