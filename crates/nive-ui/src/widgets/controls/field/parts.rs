use std::borrow::Cow;

use iced::{
    widget::{row, text},
    Alignment, Length,
};

use super::style::{error_style, hint_style, label_style};
use super::{FieldError, FieldHint, FieldLabel};
use crate::theme::{self, SpaceStep, ToneRole, TypographyRole};
use crate::widgets::primitives::{icon, IconRole};
use crate::Element;

impl<'a> FieldLabel<'a> {
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
        }
    }

    pub(in crate::widgets::controls) fn into_element<Message>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        text(self.label)
            .font(theme::typography(TypographyRole::BodyStrong).font)
            .size(theme::typography(TypographyRole::BodyStrong).size)
            .line_height(text::LineHeight::Relative(
                theme::typography(TypographyRole::BodyStrong).line_height,
            ))
            .wrapping(text::Wrapping::WordOrGlyph)
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

impl<'a> FieldHint<'a> {
    pub fn new(hint: impl Into<Cow<'a, str>>) -> Self {
        Self { hint: hint.into() }
    }

    pub(in crate::widgets::controls) fn into_element<Message>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        text(self.hint)
            .font(theme::typography(TypographyRole::BodySmall).font)
            .size(theme::typography(TypographyRole::BodySmall).size)
            .line_height(text::LineHeight::Relative(
                theme::typography(TypographyRole::BodySmall).line_height,
            ))
            .wrapping(text::Wrapping::WordOrGlyph)
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

impl<'a> FieldError<'a> {
    pub fn new(error: impl Into<Cow<'a, str>>) -> Self {
        Self {
            error: error.into(),
        }
    }

    pub(in crate::widgets::controls) fn into_element<Message>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let typography = theme::typography(TypographyRole::BodySmall);
        let message = text(self.error)
            .font(typography.font)
            .size(typography.size)
            .line_height(text::LineHeight::Relative(typography.line_height))
            .wrapping(text::Wrapping::WordOrGlyph)
            .style(error_style())
            .width(Length::Fill);

        row![
            icon::role(IconRole::ValidationError)
                .custom_size(14.0)
                .color(theme::active().tone(ToneRole::Danger).color),
            message
        ]
        .spacing(theme::space(SpaceStep::Xs))
        .align_y(Alignment::Start)
        .width(Length::Fill)
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
