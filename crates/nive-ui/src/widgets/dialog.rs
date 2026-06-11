mod footer;

use iced::{
    widget::{column, row, text},
    Length,
};

pub use footer::DialogActionFooter;

use super::Panel;
use crate::theme::{self, text as theme_text, SurfaceRole, TextRole, TypographyRole};
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq)]
struct DialogMetrics {
    width: f32,
    padding: f32,
    header_gap: f32,
    title_size: f32,
    description_size: f32,
}

pub struct Dialog<'a, Message> {
    content: Element<'a, Message>,
}

impl<'a, Message> Dialog<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
        }
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = metrics();

        Panel::new(self.content)
            .role(SurfaceRole::Dialog)
            .padding(metrics.padding)
            .width(metrics.width)
            .into()
    }
}

impl<'a, Message> From<Dialog<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(dialog: Dialog<'a, Message>) -> Self {
        dialog.into_element()
    }
}

pub struct DialogHeader<'a> {
    title: &'a str,
    description: Option<&'a str>,
}

impl<'a> DialogHeader<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            description: None,
        }
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    fn into_element<Message>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let metrics = metrics();
        let mut content = column![text(self.title)
            .size(metrics.title_size)
            .style(title_style())]
        .spacing(metrics.header_gap);

        if let Some(description) = self.description {
            content = content.push(
                text(description)
                    .size(metrics.description_size)
                    .style(description_style()),
            );
        }

        content.into()
    }
}

impl<'a, Message> From<DialogHeader<'a>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(header: DialogHeader<'a>) -> Self {
        header.into_element()
    }
}

pub struct DialogFooter<'a, Message> {
    content: Element<'a, Message>,
}

impl<'a, Message> DialogFooter<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
        }
    }

    fn into_element(self) -> Element<'a, Message> {
        row![self.content].width(Length::Fill).into()
    }
}

impl<'a, Message> From<DialogFooter<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(footer: DialogFooter<'a, Message>) -> Self {
        footer.into_element()
    }
}

fn metrics() -> DialogMetrics {
    let spacing = theme::spacing();

    DialogMetrics {
        width: 420.0,
        padding: spacing.xl + spacing.xs,
        header_gap: spacing.xs,
        title_size: theme::typography(TypographyRole::Heading).size,
        description_size: theme::typography(TypographyRole::Body).size,
    }
}

fn title_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    theme_text::style(TextRole::Primary)
}

fn description_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    theme_text::style(TextRole::Muted)
}
