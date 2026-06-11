use iced::{widget::container, Padding};

use crate::theme::surface as theme_surface;
use crate::theme::SurfaceRole;
use crate::Element;

pub struct Panel<'a, Message> {
    content: Element<'a, Message>,
    role: SurfaceRole,
    padding: Option<Padding>,
    width: Option<iced::Length>,
    height: Option<iced::Length>,
    center: Option<iced::Length>,
}

impl<'a, Message> Panel<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            role: SurfaceRole::Panel,
            padding: None,
            width: None,
            height: None,
            center: None,
        }
    }

    pub fn role(mut self, role: SurfaceRole) -> Self {
        self.role = role;
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    pub fn width(mut self, width: impl Into<iced::Length>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn height(mut self, height: impl Into<iced::Length>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn center(mut self, length: impl Into<iced::Length>) -> Self {
        self.center = Some(length.into());
        self
    }

    fn into_container(self) -> container::Container<'a, Message, crate::theme::Theme> {
        let mut panel = container(self.content).style(theme_surface::style(self.role));

        if let Some(padding) = self.padding {
            panel = panel.padding(padding);
        }

        if let Some(width) = self.width {
            panel = panel.width(width);
        }

        if let Some(height) = self.height {
            panel = panel.height(height);
        }

        if let Some(length) = self.center {
            panel = panel.center(length);
        }

        panel
    }
}

impl<'a, Message> From<Panel<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(panel: Panel<'a, Message>) -> Self {
        panel.into_container().into()
    }
}
