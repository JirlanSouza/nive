use iced::{
    widget::{column, container, text},
    Border, Length, Shadow,
};

use crate::theme::{self, text as theme_text, SpaceStep, TextRole, ToneRole, TypographyRole};
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq)]
struct FieldMetrics {
    gap: f32,
    label_size: f32,
    hint_size: f32,
    error_size: f32,
}

pub struct Field<'a, Message> {
    content: Element<'a, Message>,
    label: Option<&'a str>,
    hint: Option<&'a str>,
    error: Option<&'a str>,
}

impl<'a, Message> Field<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            label: None,
            hint: None,
            error: None,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = Some(hint);
        self
    }

    pub fn error(mut self, error: &'a str) -> Self {
        self.error = Some(error);
        self
    }

    fn into_container(self) -> container::Container<'a, Message, crate::theme::Theme> {
        let metrics = metrics();
        let mut content = column![].spacing(metrics.gap);

        if let Some(label) = self.label {
            content = content.push(FieldLabel::new(label));
        }

        content = content.push(self.content);

        if let Some(error) = self.error {
            content = content.push(FieldError::new(error));
        } else if let Some(hint) = self.hint {
            content = content.push(FieldHint::new(hint));
        }

        container(content).style(style).width(Length::Fill)
    }
}

impl<'a, Message> From<Field<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(field: Field<'a, Message>) -> Self {
        field.into_container().into()
    }
}

pub struct FieldGroup<'a, Message> {
    content: Element<'a, Message>,
    fill: bool,
}

impl<'a, Message> FieldGroup<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            fill: true,
        }
    }

    pub fn shrink(mut self) -> Self {
        self.fill = false;
        self
    }

    fn into_container(self) -> container::Container<'a, Message, crate::theme::Theme> {
        let mut group = container(self.content).style(style);

        if self.fill {
            group = group.width(Length::Fill);
        }

        group
    }
}

impl<'a, Message> From<FieldGroup<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(group: FieldGroup<'a, Message>) -> Self {
        group.into_container().into()
    }
}

pub struct FieldLabel<'a> {
    label: &'a str,
}

impl<'a> FieldLabel<'a> {
    pub fn new(label: &'a str) -> Self {
        Self { label }
    }

    fn into_element<Message>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let metrics = metrics();
        text(self.label)
            .size(metrics.label_size)
            .style(label_style())
            .into()
    }
}

impl<'a, Message> From<FieldLabel<'a>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(label: FieldLabel<'a>) -> Self {
        label.into_element()
    }
}

pub struct FieldHint<'a> {
    hint: &'a str,
}

impl<'a> FieldHint<'a> {
    pub fn new(hint: &'a str) -> Self {
        Self { hint }
    }

    fn into_element<Message>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let metrics = metrics();
        text(self.hint)
            .size(metrics.hint_size)
            .style(hint_style())
            .into()
    }
}

impl<'a, Message> From<FieldHint<'a>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(hint: FieldHint<'a>) -> Self {
        hint.into_element()
    }
}

pub struct FieldError<'a> {
    error: &'a str,
}

impl<'a> FieldError<'a> {
    pub fn new(error: &'a str) -> Self {
        Self { error }
    }

    fn into_element<Message>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let metrics = metrics();
        text(self.error)
            .size(metrics.error_size)
            .style(error_style())
            .into()
    }
}

impl<'a, Message> From<FieldError<'a>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(error: FieldError<'a>) -> Self {
        error.into_element()
    }
}

fn metrics() -> FieldMetrics {
    FieldMetrics {
        gap: theme::space(SpaceStep::Xs),
        label_size: theme::typography(TypographyRole::Body).size,
        hint_size: theme::typography(TypographyRole::BodySmall).size,
        error_size: theme::typography(TypographyRole::BodySmall).size,
    }
}

fn style(theme: &crate::theme::Theme) -> container::Style {
    container::Style {
        text_color: Some(theme.text(TextRole::Primary).color),
        background: None,
        border: Border::default(),
        shadow: Shadow::default(),
        ..container::Style::default()
    }
}

fn label_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    theme_text::style(TextRole::Primary)
}

fn hint_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    theme_text::style(TextRole::Muted)
}

fn error_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    theme_text::tone(ToneRole::Danger)
}

#[cfg(test)]
mod field_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn style_uses_primary_text_color() {
        let theme = Theme::Dark;
        let style = style(&theme);

        assert_eq!(style.text_color, Some(theme.text(TextRole::Primary).color));
    }

    #[test]
    fn hint_and_error_styles_use_app_theme() {
        let theme = Theme::Dark;

        assert_eq!(
            hint_style()(&theme).color,
            Some(theme.text(TextRole::Muted).color)
        );
        assert_eq!(
            error_style()(&theme).color,
            Some(theme.tone(ToneRole::Danger).color)
        );
    }

    #[test]
    fn label_style_uses_app_theme() {
        let theme = Theme::Dark;

        assert_eq!(
            label_style()(&theme).color,
            Some(theme.text(TextRole::Primary).color)
        );
    }
}
