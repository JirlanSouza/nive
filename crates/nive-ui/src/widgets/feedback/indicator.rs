use iced::{
    widget::{container, Space},
    Length,
};

use crate::theme::ToneRole;
use crate::Element;

use super::style as theme_feedback;

pub(super) fn indicator<'a, Message>(tone: ToneRole, size: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    container(Space::new().width(Length::Fixed(size)))
        .style(theme_feedback::indicator_style(tone, size))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .into()
}
