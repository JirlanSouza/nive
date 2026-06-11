use iced::{
    widget::{container, Space},
    Length,
};

use crate::Element;

use super::style as theme_toolbar;

pub(super) fn separator<'a, Message>(metrics: theme_toolbar::ToolbarMetrics) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    container(Space::new().width(Length::Fixed(metrics.separator_width)))
        .style(theme_toolbar::separator_style())
        .width(Length::Fixed(metrics.separator_width))
        .height(Length::Fixed(metrics.separator_height))
        .into()
}
