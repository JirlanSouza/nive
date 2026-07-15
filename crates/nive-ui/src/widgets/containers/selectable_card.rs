use iced::{
    widget::{button, container, row, Space},
    Alignment, Length, Padding,
};

use crate::theme::{self, GapRole, ShapeSize};
use crate::Element;

use crate::advanced::pressable::Pressable;
use crate::widgets::controls::button::ButtonFocusRing;
use crate::widgets::primitives::{Icon, IconRole, IconSize};

use super::{
    card_frame::{self, CardVariant},
    min_height::MinHeight,
};

const MIN_TARGET_HEIGHT: f32 = 48.0;

/// App-controlled persistent selection over one complete card target.
///
/// Activation requests an app decision and never mutates `selected` locally.
/// The target is at least 48 logical pixels high. Do not nest independent
/// interaction targets inside it; use a passive [`Card`](super::Card) when
/// content needs sibling actions.
pub struct SelectableCard<'a, Message> {
    content: Element<'a, Message>,
    selected: bool,
    selection_indicator: bool,
    variant: CardVariant,
    radius: f32,
    padding: Padding,
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
            selection_indicator: false,
            variant: CardVariant::Filled,
            radius: card_frame::metrics(theme::active()).radius,
            padding: card_frame::metrics(theme::active()).padding,
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

    /// Reserves a stable display-only trailing check slot.
    pub fn selection_indicator(mut self, visible: bool) -> Self {
        self.selection_indicator = visible;
        self
    }

    /// Sets the card-owned visual variant without changing controlled selection.
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
        let content = if self.selection_indicator {
            let indicator: Element<'a, Message> = if self.selected {
                Icon::role(IconRole::ActionConfirm)
                    .size(IconSize::Sm)
                    .into()
            } else {
                Space::new()
                    .width(Length::Fixed(IconSize::Sm.pixels()))
                    .height(Length::Fixed(IconSize::Sm.pixels()))
                    .into()
            };
            row![self.content, indicator]
                .spacing(theme::gap(GapRole::Related))
                .align_y(Alignment::Center)
                .into()
        } else {
            self.content
        };
        let mut card = button::Button::new(content)
            .style(card_frame::interaction_style(
                self.selected,
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
    use crate::widgets::containers::card_test_support::{CardHarness, Message};
    use iced::Size;

    #[test]
    fn shape_builders_resolve_selectable_card_radius() {
        let default = SelectableCard::<()>::new(iced::widget::Space::new());
        let square = SelectableCard::<()>::new(iced::widget::Space::new()).square();
        let none = SelectableCard::<()>::new(iced::widget::Space::new()).shape(ShapeSize::None);
        let full = SelectableCard::<()>::new(iced::widget::Space::new()).shape(ShapeSize::Full);

        assert_eq!(default.radius, token_radius::MD);
        assert_eq!(square.radius, none.radius);
        assert_eq!(full.radius, token_radius::FULL);
    }

    #[test]
    fn selection_indicator_is_opt_in_and_does_not_own_selection() {
        let card = SelectableCard::<()>::new(iced::widget::Space::new())
            .selected(true)
            .selection_indicator(true);

        assert!(card.selected);
        assert!(card.selection_indicator);
        assert!(card.on_press.is_none());
    }

    #[test]
    fn controlled_activation_does_not_mutate_selection() {
        let mut harness = CardHarness::new(
            SelectableCard::new(iced::widget::Space::new())
                .selected(true)
                .on_press(Message::Activated)
                .into(),
            Size::new(240.0, 80.0),
        );

        assert_eq!(harness.click_center(), vec![Message::Activated]);
    }

    #[test]
    fn indicator_reserves_identical_selected_and_unselected_bounds() {
        let selected = CardHarness::new(
            SelectableCard::new(iced::widget::Space::new())
                .selected(true)
                .selection_indicator(true)
                .into(),
            Size::new(240.0, 80.0),
        );
        let unselected = CardHarness::new(
            SelectableCard::new(iced::widget::Space::new())
                .selected(false)
                .selection_indicator(true)
                .into(),
            Size::new(240.0, 80.0),
        );

        assert_eq!(selected.size(), unselected.size());
        assert!(selected.size().height >= MIN_TARGET_HEIGHT);
    }

    #[test]
    fn absent_callback_and_disabled_selection_preserve_inert_controlled_state() {
        let mut absent = CardHarness::new(
            SelectableCard::new(iced::widget::Space::new())
                .selected(true)
                .into(),
            Size::new(240.0, 80.0),
        );
        let mut disabled = CardHarness::new(
            SelectableCard::new(iced::widget::Space::new())
                .selected(true)
                .on_press(Message::Activated)
                .disabled(true)
                .into(),
            Size::new(240.0, 80.0),
        );

        assert!(absent.click_center().is_empty());
        assert!(disabled.click_center().is_empty());
        assert_eq!(absent.size().height, MIN_TARGET_HEIGHT);
        assert_eq!(disabled.size().height, MIN_TARGET_HEIGHT);
    }
}
