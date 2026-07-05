use iced::{
    border::Radius,
    widget::{container, Space},
    Background, Border, Length, Shadow,
};

use crate::theme::ToneRole;
use crate::Element;

pub(crate) fn tone_dot<'a, Message>(tone: ToneRole, size: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    container(Space::new().width(Length::Fixed(size)))
        .style(tone_dot_style(tone, size))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .into()
}

fn tone_dot_style(tone: ToneRole, size: f32) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let tone = theme.tone(tone);

        container::Style {
            text_color: None,
            background: Some(Background::Color(tone.color)),
            border: Border {
                color: tone.border.color,
                width: 0.0,
                radius: Radius::new(size / 2.0),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}
