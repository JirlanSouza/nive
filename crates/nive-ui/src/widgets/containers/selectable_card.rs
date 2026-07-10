use iced::{widget::button, Length, Padding};

use crate::theme::{self, ShapeSize, SurfaceRole};
use crate::Element;

use super::action_card::style as theme_action_card;
use crate::advanced::pressable::Pressable;
use crate::widgets::controls::button::ButtonFocusRing;

/// Selectable pressable content card with configurable shape.
pub struct SelectableCard<'a, Message> {
    content: Element<'a, Message>,
    selected: bool,
    role: SurfaceRole,
    radius: f32,
    padding: Option<Padding>,
    width: Option<Length>,
    height: Option<Length>,
    disabled: bool,
    on_press: Option<Message>,
}

impl<'a, Message> SelectableCard<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            selected: false,
            role: SurfaceRole::Panel,
            radius: theme_action_card::metrics().radius,
            padding: None,
            width: None,
            height: None,
            disabled: false,
            on_press: None,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn role(mut self, role: SurfaceRole) -> Self {
        self.role = role;
        self
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let activation = if self.disabled {
            None
        } else {
            self.on_press.clone()
        };
        let radius = self.radius;
        let mut card = button::Button::new(self.content)
            .style(theme_action_card::style(
                self.role,
                self.selected,
                self.radius,
            ))
            .padding(
                self.padding
                    .unwrap_or_else(|| Padding::new(theme_action_card::metrics().padding)),
            );

        if let Some(width) = self.width {
            card = card.width(width);
        }

        if let Some(height) = self.height {
            card = card.height(height);
        }

        let card = card.on_press_maybe(activation.clone());

        Pressable::maybe(card, activation, radius.into(), ButtonFocusRing::Default)
    }
}

impl<'a, Message> From<SelectableCard<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(card: SelectableCard<'a, Message>) -> Self {
        card.into_element()
    }
}

#[cfg(test)]
mod selectable_card_tests {
    use super::*;
    use crate::tokens::radius as token_radius;

    #[test]
    fn shape_builders_resolve_selectable_card_radius() {
        let default = SelectableCard::<()>::new(iced::widget::Space::new());
        let square = SelectableCard::<()>::new(iced::widget::Space::new()).square();
        let none = SelectableCard::<()>::new(iced::widget::Space::new()).shape(ShapeSize::None);
        let full = SelectableCard::<()>::new(iced::widget::Space::new()).shape(ShapeSize::Full);

        assert_eq!(default.radius, token_radius::LG);
        assert_eq!(square.radius, none.radius);
        assert_eq!(full.radius, token_radius::FULL);
    }
}
