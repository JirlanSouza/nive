use iced::{
    widget::{button, container},
    Length, Padding,
};

use crate::theme::{self, ShapeSize};
use crate::Element;

use crate::advanced::pressable::Pressable;
use crate::widgets::controls::button::ButtonFocusRing;

use super::{
    card_frame::{self, CardVariant},
    min_height::MinHeight,
};

const MIN_TARGET_HEIGHT: f32 = 48.0;

/// One immediate action whose complete card surface is the target.
///
/// The target is at least 48 logical pixels high and supports pointer, Enter,
/// and Space activation when a callback exists. Callback absence is capability
/// absence, not disabled presentation. Do not place buttons, links, menus,
/// inputs, or other independent targets inside an `ActionCard`; use a passive
/// [`Card`](super::Card) with sibling actions instead.
pub struct ActionCard<'a, Message> {
    content: Element<'a, Message>,
    variant: CardVariant,
    radius: f32,
    padding: Padding,
    width: Option<Length>,
    height: Option<Length>,
    disabled: bool,
    on_press: Option<Message>,
}

impl<'a, Message> ActionCard<'a, Message>
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
            disabled: false,
            on_press: None,
        }
    }

    /// Sets the card-owned visual variant without changing interaction semantics.
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
        let capable = self.on_press.is_some();
        let activation = if self.disabled {
            None
        } else {
            self.on_press.clone()
        };
        let radius = self.radius;
        let mut card = button::Button::new(self.content)
            .style(card_frame::interaction_style(
                false,
                self.disabled,
                capable,
                self.radius,
            ))
            .padding(self.padding);

        if let Some(width) = self.width {
            card = card.width(width);
        }

        if let Some(height) = self.height {
            card = card.height(height);
        }

        let card = card.on_press_maybe(activation.clone());
        let card =
            Pressable::maybe_card_inset(card, activation, radius.into(), ButtonFocusRing::Default);
        let card = MinHeight::new(card, MIN_TARGET_HEIGHT);

        container(card)
            .style(card_frame::base_style(self.variant, radius))
            .into()
    }
}

impl<'a, Message> From<ActionCard<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(card: ActionCard<'a, Message>) -> Self {
        card.into_element()
    }
}

#[cfg(test)]
mod action_card_tests {
    use super::*;
    use crate::tokens::radius as token_radius;
    use crate::widgets::containers::card_test_support::{CardHarness, Message};
    use iced::{keyboard::key::Named, mouse, Size};

    #[test]
    fn shape_builders_resolve_action_card_radius() {
        let default = ActionCard::<()>::new(iced::widget::Space::new());
        let square = ActionCard::<()>::new(iced::widget::Space::new()).square();
        let none = ActionCard::<()>::new(iced::widget::Space::new()).shape(ShapeSize::None);
        let full = ActionCard::<()>::new(iced::widget::Space::new()).shape(ShapeSize::Full);

        assert_eq!(default.radius, token_radius::MD);
        assert_eq!(square.radius, none.radius);
        assert_eq!(full.radius, token_radius::FULL);
    }

    #[test]
    fn variants_and_capability_are_independent_from_explicit_disabled_state() {
        let card = ActionCard::<()>::new(iced::widget::Space::new())
            .elevated()
            .on_press_maybe(None)
            .disabled(false);

        assert_eq!(card.variant, CardVariant::Elevated);
        assert!(!card.disabled);
        assert!(card.on_press.is_none());
    }

    #[test]
    fn complete_target_clamps_short_content_and_grows_for_tall_content() {
        let short = CardHarness::new(
            ActionCard::new(iced::widget::Space::new())
                .height(24)
                .on_press(Message::Activated)
                .into(),
            Size::new(240.0, 200.0),
        );
        let tall = CardHarness::new(
            ActionCard::new(iced::widget::Space::new().height(Length::Fixed(80.0)))
                .on_press(Message::Activated)
                .into(),
            Size::new(240.0, 200.0),
        );
        let fill = CardHarness::new(
            ActionCard::new(iced::widget::Space::new())
                .fill_width()
                .on_press(Message::Activated)
                .into(),
            Size::new(240.0, 200.0),
        );

        assert_eq!(short.size().height, MIN_TARGET_HEIGHT);
        assert!(tall.size().height > MIN_TARGET_HEIGHT);
        assert_eq!(fill.size().width, 240.0);
        assert_eq!(fill.size().height, MIN_TARGET_HEIGHT);
    }

    #[test]
    fn pointer_and_keyboard_activation_emit_exactly_once_and_ignore_repeat() {
        let mut pointer = CardHarness::new(
            ActionCard::new(iced::widget::Space::new())
                .on_press(Message::Activated)
                .into(),
            Size::new(240.0, 80.0),
        );
        assert_eq!(pointer.click_center(), vec![Message::Activated]);

        let mut keyboard = CardHarness::new(
            ActionCard::new(iced::widget::Space::new())
                .on_press(Message::Activated)
                .into(),
            Size::new(240.0, 80.0),
        );
        keyboard.focus_next();
        assert_eq!(
            keyboard.activate_key(Named::Enter, false),
            vec![Message::Activated]
        );
        assert_eq!(
            keyboard.activate_key(Named::Space, false),
            vec![Message::Activated]
        );
        assert!(keyboard.activate_key(Named::Enter, true).is_empty());
    }

    #[test]
    fn absent_callback_and_explicit_disabled_state_are_inert() {
        let mut absent = CardHarness::new(
            ActionCard::new(iced::widget::Space::new()).into(),
            Size::new(240.0, 80.0),
        );
        assert_ne!(absent.mouse_interaction(), mouse::Interaction::Pointer);
        assert!(absent.click_center().is_empty());

        let mut disabled = CardHarness::new(
            ActionCard::new(iced::widget::Space::new())
                .on_press(Message::Activated)
                .disabled(true)
                .into(),
            Size::new(240.0, 80.0),
        );
        assert_ne!(disabled.mouse_interaction(), mouse::Interaction::Pointer);
        assert!(disabled.click_center().is_empty());
    }
}
