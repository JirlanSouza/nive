use iced::{widget::container, Length, Padding};

use crate::theme::{self, ShapeSize};
use crate::Element;

use super::card_frame::{self, CardVariant};

/// Passive content surface with a card-owned variant and frame.
///
/// Structural surface roles are not valid card variants:
///
/// ```compile_fail
/// use nive_ui::{theme::SurfaceRole, widgets::Card};
///
/// let _ = Card::<()>::new(iced::widget::Space::new()).role(SurfaceRole::App);
/// ```
pub struct Card<'a, Message> {
    content: Element<'a, Message>,
    variant: CardVariant,
    radius: f32,
    padding: Padding,
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
            variant: CardVariant::Filled,
            radius: card_frame::metrics(theme::active()).radius,
            padding: card_frame::metrics(theme::active()).padding,
            width: None,
            height: None,
        }
    }

    /// Sets the card-owned visual variant.
    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Uses the default Panel-filled frame.
    pub fn filled(self) -> Self {
        self.variant(CardVariant::Filled)
    }

    /// Uses a transparent frame with one semantic perimeter.
    pub fn outlined(self) -> Self {
        self.variant(CardVariant::Outlined)
    }

    /// Uses the semantic elevated fill and shadow.
    pub fn elevated(self) -> Self {
        self.variant(CardVariant::Elevated)
    }

    /// Uses a transparent frame without perimeter or shadow.
    pub fn ghost(self) -> Self {
        self.variant(CardVariant::Ghost)
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

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
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
        let mut card = container(self.content)
            .style(card_frame::base_style(self.variant, self.radius))
            .padding(self.padding);

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
    use crate::widgets::containers::card_test_support::{CardHarness, Message};
    use iced::Size;

    #[test]
    fn shape_builders_resolve_card_radius() {
        let default = Card::<()>::new(iced::widget::Space::new());
        let square = Card::<()>::new(iced::widget::Space::new()).square();
        let none = Card::<()>::new(iced::widget::Space::new()).shape(ShapeSize::None);
        let full = Card::<()>::new(iced::widget::Space::new()).shape(ShapeSize::Full);

        assert_eq!(default.radius, token_radius::MD);
        assert_eq!(square.radius, none.radius);
        assert_eq!(full.radius, token_radius::FULL);
        assert_eq!(
            Card::<()>::new(iced::widget::Space::new())
                .shape_xs()
                .radius,
            token_radius::XS
        );
        assert_eq!(
            Card::<()>::new(iced::widget::Space::new())
                .shape_sm()
                .radius,
            token_radius::SM
        );
        assert_eq!(
            Card::<()>::new(iced::widget::Space::new())
                .shape_md()
                .radius,
            token_radius::MD
        );
        assert_eq!(
            Card::<()>::new(iced::widget::Space::new())
                .shape_lg()
                .radius,
            token_radius::LG
        );
        assert_eq!(
            Card::<()>::new(iced::widget::Space::new())
                .shape_xl()
                .radius,
            token_radius::XL
        );
        assert_eq!(
            Card::<()>::new(iced::widget::Space::new())
                .shape_xxl()
                .radius,
            token_radius::XXL
        );
        assert_eq!(
            Card::<()>::new(iced::widget::Space::new())
                .radius(7.5)
                .radius,
            7.5
        );
        assert_eq!(
            Card::<()>::new(iced::widget::Space::new())
                .padding(3)
                .padding,
            Padding::new(3.0)
        );
    }

    #[test]
    fn passive_card_shrinks_without_an_interactive_minimum_and_fill_is_explicit() {
        let shrink = CardHarness::new(
            Card::<Message>::new(
                iced::widget::Space::new()
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(10.0)),
            )
            .into(),
            Size::new(240.0, 100.0),
        );
        let fill = CardHarness::new(
            Card::<Message>::new(iced::widget::Space::new())
                .fill_width()
                .into(),
            Size::new(240.0, 100.0),
        );
        let fill_both = CardHarness::new(
            Card::<Message>::new(iced::widget::Space::new())
                .fill()
                .into(),
            Size::new(240.0, 100.0),
        );
        let flush = CardHarness::new(
            Card::<Message>::new(iced::widget::Space::new().height(Length::Fixed(10.0)))
                .padding(0)
                .into(),
            Size::new(240.0, 100.0),
        );

        assert!(shrink.size().height < 48.0);
        assert_eq!(fill.size().width, 240.0);
        assert_eq!(fill_both.size(), Size::new(240.0, 100.0));
        assert_eq!(flush.size().height, 10.0);
    }
}
