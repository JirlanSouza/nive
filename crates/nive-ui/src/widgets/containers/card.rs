use iced::{widget::container, Padding};

use crate::theme::{self, surface as theme_surface, ShapeRole, SurfaceRole};
use crate::Element;

pub struct Card<'a, Message> {
    content: Element<'a, Message>,
    role: SurfaceRole,
    radius: f32,
    padding: Option<Padding>,
}

impl<'a, Message> Card<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            role: SurfaceRole::Panel,
            radius: theme::active().shape(ShapeRole::ExtraLarge).radius_value(),
            padding: None,
        }
    }

    pub fn sm(mut self) -> Self {
        self.radius = theme::active().shape(ShapeRole::Medium).radius_value();
        self
    }

    pub fn role(mut self, role: SurfaceRole) -> Self {
        self.role = role;
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    fn into_container(self) -> container::Container<'a, Message, crate::theme::Theme> {
        let mut card = container(self.content).style(theme_surface::style_with_radius(
            self.role,
            self.radius.into(),
        ));

        if let Some(padding) = self.padding {
            card = card.padding(padding);
        }

        card
    }
}

impl<'a, Message> From<Card<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(card: Card<'a, Message>) -> Self {
        card.into_container().into()
    }
}
