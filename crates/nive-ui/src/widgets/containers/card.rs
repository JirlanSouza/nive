use iced::{widget::container, Length, Padding};

use crate::theme::{self, surface as theme_surface, ShapeSize, SurfaceRole};
use crate::Element;

/// Content surface with configurable shape.
pub struct Card<'a, Message> {
    content: Element<'a, Message>,
    role: SurfaceRole,
    radius: f32,
    padding: Option<Padding>,
    width: Option<Length>,
    height: Option<Length>,
}

impl<'a, Message> Card<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            role: SurfaceRole::Panel,
            radius: theme::active().shape(ShapeSize::Xl).radius_value(),
            padding: None,
            width: None,
            height: None,
        }
    }

    /// Sets the card shape from the theme scale.
    pub fn shape(mut self, shape: ShapeSize) -> Self {
        self.radius = theme::active().shape(shape).radius_value();
        self
    }

    pub fn shape_xs(self) -> Self {
        self.shape(ShapeSize::Xs)
    }

    pub fn shape_sm(self) -> Self {
        self.shape(ShapeSize::Sm)
    }

    pub fn shape_md(self) -> Self {
        self.shape(ShapeSize::Md)
    }

    pub fn shape_lg(self) -> Self {
        self.shape(ShapeSize::Lg)
    }

    pub fn shape_xl(self) -> Self {
        self.shape(ShapeSize::Xl)
    }

    pub fn shape_xxl(self) -> Self {
        self.shape(ShapeSize::Xxl)
    }

    /// Sets square corners, equivalent to `shape(ShapeSize::None)`.
    pub fn square(self) -> Self {
        self.shape(ShapeSize::None)
    }

    /// Sets a raw radius in pixels.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
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

    crate::impl_layout_builders!(
        width_opt,
        height_opt,
        fill_width_opt,
        fill_height_opt,
        fill_opt,
        shrink_width_opt
    );

    fn into_container(self) -> container::Container<'a, Message, crate::theme::Theme> {
        let mut card = container(self.content).style(theme_surface::style_with_radius(
            self.role,
            self.radius.into(),
        ));

        if let Some(padding) = self.padding {
            card = card.padding(padding);
        }

        if let Some(width) = self.width {
            card = card.width(width);
        }

        if let Some(height) = self.height {
            card = card.height(height);
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

#[cfg(test)]
mod card_tests {
    use super::*;
    use crate::tokens::radius as token_radius;

    #[test]
    fn shape_builders_resolve_card_radius() {
        let default = Card::<()>::new(iced::widget::Space::new());
        let square = Card::<()>::new(iced::widget::Space::new()).square();
        let none = Card::<()>::new(iced::widget::Space::new()).shape(ShapeSize::None);
        let full = Card::<()>::new(iced::widget::Space::new()).shape(ShapeSize::Full);

        assert_eq!(default.radius, token_radius::XL);
        assert_eq!(square.radius, none.radius);
        assert_eq!(full.radius, token_radius::FULL);
    }
}
