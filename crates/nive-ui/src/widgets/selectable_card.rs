use iced::{widget::button, Length, Padding};

use crate::theme::SurfaceRole;
use crate::Element;

use super::action_card::style as theme_action_card;
use super::button::ButtonFocusRing;
use super::pressable::Pressable;

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

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = Some(height.into());
        self
    }

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
