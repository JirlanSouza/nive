use iced::{
    widget::{container, Space},
    Length,
};

use crate::theme::ToneRole;
use crate::Element;

use super::style as theme_tree_item;

pub(super) fn tone_dot<'a, Message>(tone: ToneRole, size: f32) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    container(Space::new().width(Length::Fixed(size)))
        .style(theme_tree_item::tone_indicator_style(tone, size))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .into()
}
