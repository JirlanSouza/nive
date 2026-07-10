mod chrome;
mod content;
mod style;

use iced::{border::Radius, widget::Id, Length, Padding};

use crate::theme::ControlSize;
use crate::Element;

use self::{
    chrome::ButtonChrome,
    content::{Content, TextAlign},
};
use crate::advanced::pressable::Pressable;
use crate::widgets::overlays::tooltip as tooltip_widget;
use crate::widgets::primitives::IconRole;

pub use style::{button_control_state, focus_ring, ButtonFocusRing};
pub use style::{ButtonIntent, ButtonVariant};

/// Action button with separate intent and visual variant axes.
pub struct Button<'a, Message> {
    content: Content<'a, Message>,
    leading_icon: Option<IconRole>,
    trailing_icon: Option<IconRole>,
    intent: ButtonIntent,
    variant: ButtonVariant,
    size: ControlSize,
    text_align: TextAlign,
    width: Option<Length>,
    height: Option<Length>,
    padding: Option<Padding>,
    loading: bool,
    reserve_loading_indicator: bool,
    disabled: bool,
    focusable: bool,
    id: Option<Id>,
    on_press: Option<Message>,
    tooltip: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GroupedItemKind {
    Embedded,
    Selectable,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GroupedItemSpec {
    pub(crate) size: ControlSize,
    pub(crate) radius: Radius,
    pub(crate) height: f32,
    pub(crate) padding_h: f32,
    pub(crate) selected: bool,
    pub(crate) destructive: bool,
    pub(crate) kind: GroupedItemKind,
}

/// Creates a suggested solid button.
pub fn primary<'a, Message>(label: &'a str) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(
        Content::Label(label),
        ButtonIntent::Suggested,
        ButtonVariant::Solid,
    )
}

/// Creates a neutral outline button.
pub fn outline<'a, Message>(label: &'a str) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(
        Content::Label(label),
        ButtonIntent::Neutral,
        ButtonVariant::Outline,
    )
}

/// Creates a neutral subtle button.
pub fn secondary<'a, Message>(label: &'a str) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(
        Content::Label(label),
        ButtonIntent::Neutral,
        ButtonVariant::Subtle,
    )
}

/// Creates a neutral ghost button.
pub fn ghost<'a, Message>(label: &'a str) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(
        Content::Label(label),
        ButtonIntent::Neutral,
        ButtonVariant::Ghost,
    )
}

/// Creates a destructive solid button.
pub fn destructive<'a, Message>(label: &'a str) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(
        Content::Label(label),
        ButtonIntent::Destructive,
        ButtonVariant::Solid,
    )
}

/// Creates a neutral ghost icon button.
pub fn icon<'a, Message>(app_icon: IconRole) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(
        Content::Icon(app_icon),
        ButtonIntent::Neutral,
        ButtonVariant::Ghost,
    )
}

impl<'a, Message> Button<'a, Message>
where
    Message: Clone + 'a,
{
    fn new(content: Content<'a, Message>, intent: ButtonIntent, variant: ButtonVariant) -> Self {
        Self {
            content,
            leading_icon: None,
            trailing_icon: None,
            intent,
            variant,
            size: ControlSize::Sm,
            text_align: TextAlign::Center,
            width: None,
            height: None,
            padding: None,
            loading: false,
            reserve_loading_indicator: false,
            disabled: false,
            focusable: true,
            id: None,
            on_press: None,
            tooltip: None,
        }
    }

    /// Sets the visual variant axis.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the semantic intent axis.
    pub fn intent(mut self, intent: ButtonIntent) -> Self {
        self.intent = intent;
        self
    }

    pub(crate) fn custom(content: Element<'a, Message>) -> Self {
        Self::new(
            Content::Custom(content),
            ButtonIntent::Neutral,
            ButtonVariant::Ghost,
        )
    }

    /// Sets this button to `Suggested + Solid`.
    pub fn primary(self) -> Self {
        self.intent(ButtonIntent::Suggested)
            .variant(ButtonVariant::Solid)
    }

    /// Sets this button to `Neutral + Subtle`.
    pub fn secondary(self) -> Self {
        self.intent(ButtonIntent::Neutral)
            .variant(ButtonVariant::Subtle)
    }

    /// Sets this button to `Neutral + Outline`.
    pub fn outline(self) -> Self {
        self.intent(ButtonIntent::Neutral)
            .variant(ButtonVariant::Outline)
    }

    /// Sets this button to `Neutral + Ghost`.
    pub fn ghost(self) -> Self {
        self.intent(ButtonIntent::Neutral)
            .variant(ButtonVariant::Ghost)
    }

    /// Sets this button to `Destructive + Solid`.
    ///
    /// Use `destructive` for actions. Status widgets use `danger` for the
    /// corresponding tone language.
    pub fn destructive(self) -> Self {
        self.intent(ButtonIntent::Destructive)
            .variant(ButtonVariant::Solid)
    }

    /// Sets the intent axis to neutral.
    pub fn neutral(self) -> Self {
        self.intent(ButtonIntent::Neutral)
    }

    /// Sets the intent axis to suggested/default action.
    pub fn suggested(self) -> Self {
        self.intent(ButtonIntent::Suggested)
    }

    /// Sets the variant axis to solid.
    pub fn solid(self) -> Self {
        self.variant(ButtonVariant::Solid)
    }

    /// Sets the variant axis to subtle.
    pub fn subtle(self) -> Self {
        self.variant(ButtonVariant::Subtle)
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn xs(mut self) -> Self {
        self.size = ControlSize::Xs;
        self
    }

    pub fn sm(mut self) -> Self {
        self.size = ControlSize::Sm;
        self
    }

    pub fn md(mut self) -> Self {
        self.size = ControlSize::Md;
        self
    }

    pub fn lg(mut self) -> Self {
        self.size = ControlSize::Lg;
        self
    }

    pub fn align_start(mut self) -> Self {
        self.text_align = TextAlign::Start;
        self
    }

    pub fn align_center(mut self) -> Self {
        self.text_align = TextAlign::Center;
        self
    }

    crate::impl_layout_builders!(width_opt, fill_width_opt, shrink_width_opt);

    pub(crate) fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    pub fn leading_icon(mut self, icon: IconRole) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn trailing_icon(mut self, icon: IconRole) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self.reserve_loading_indicator = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn tooltip(mut self, tooltip: &'a str) -> Self {
        self.tooltip = Some(tooltip);
        self
    }

    pub fn tooltip_maybe(mut self, tooltip: Option<&'a str>) -> Self {
        self.tooltip = tooltip;
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

    pub(crate) fn into_grouped_item(mut self, spec: GroupedItemSpec) -> Element<'a, Message> {
        self.size = spec.size;
        self.height = Some(Length::Fixed(spec.height));
        if self.width.is_none() {
            self.width = Some(match &self.content {
                Content::Icon(_) => Length::Fixed(spec.height),
                Content::Label(_) | Content::Custom(_) => Length::Shrink,
            });
        }
        if self.padding.is_none() {
            self.padding = Some(match &self.content {
                Content::Icon(_) => Padding::ZERO,
                Content::Label(_) | Content::Custom(_) => Padding::ZERO.horizontal(spec.padding_h),
            });
        }

        self.into_element_chrome(ButtonChrome::Grouped(spec))
    }

    fn into_element_chrome(self, chrome: ButtonChrome) -> Element<'a, Message> {
        let radius = chrome.radius(self.size);
        let activation = self.keyboard_activation();
        let id = self.id.clone();
        let ring = self.focus_ring();
        let tooltip_label = self.tooltip;
        let button = chrome::into_app_button(self, chrome);
        let button: Element<'a, Message> = match activation {
            Some(message) => Pressable::new(button, message, id, radius, ring).into(),
            None => button.into(),
        };

        match tooltip_label {
            Some(label) => tooltip_widget::bottom(button, label),
            None => button,
        }
    }

    fn keyboard_activation(&self) -> Option<Message> {
        if self.focusable && !self.disabled && !self.loading {
            self.on_press.clone()
        } else {
            None
        }
    }

    fn focus_ring(&self) -> ButtonFocusRing {
        match self.variant {
            ButtonVariant::Solid if self.intent == ButtonIntent::Suggested => {
                ButtonFocusRing::OnPrimary
            }
            _ => ButtonFocusRing::Default,
        }
    }
}

impl<'a, Message> From<Button<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(button: Button<'a, Message>) -> Self {
        button.into_element_chrome(ButtonChrome::Standalone)
    }
}

#[cfg(test)]
mod button_tests {
    use super::*;

    #[test]
    fn shrink_width_sets_button_width_to_shrink() {
        let button = secondary::<()>("Open").fill_width().shrink_width();

        assert_eq!(button.width, Some(Length::Shrink));
    }

    #[test]
    fn high_level_shortcuts_map_to_intent_and_variant_pairs() {
        assert_button(
            primary::<()>("Create"),
            ButtonIntent::Suggested,
            ButtonVariant::Solid,
        );
        assert_button(
            secondary::<()>("Copy"),
            ButtonIntent::Neutral,
            ButtonVariant::Subtle,
        );
        assert_button(
            outline::<()>("Inspect"),
            ButtonIntent::Neutral,
            ButtonVariant::Outline,
        );
        assert_button(
            ghost::<()>("Preview"),
            ButtonIntent::Neutral,
            ButtonVariant::Ghost,
        );
        assert_button(
            destructive::<()>("Delete"),
            ButtonIntent::Destructive,
            ButtonVariant::Solid,
        );
        assert_button(
            icon::<()>(IconRole::WindowClose),
            ButtonIntent::Neutral,
            ButtonVariant::Ghost,
        );
    }

    #[test]
    fn axis_shortcuts_change_their_button_axis() {
        let button = secondary::<()>("Save").suggested().solid();

        assert_button(button, ButtonIntent::Suggested, ButtonVariant::Solid);
        assert_button(
            ghost::<()>("Neutral").solid(),
            ButtonIntent::Neutral,
            ButtonVariant::Solid,
        );
        assert_button(
            primary::<()>("Subtle").neutral().subtle(),
            ButtonIntent::Neutral,
            ButtonVariant::Subtle,
        );
    }

    fn assert_button(button: Button<'_, ()>, intent: ButtonIntent, variant: ButtonVariant) {
        assert_eq!(button.intent, intent);
        assert_eq!(button.variant, variant);
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Message {
        Pressed,
    }

    #[test]
    fn disabled_button_ignores_present_callback() {
        let button = primary("Save").on_press(Message::Pressed).disabled(true);

        assert_eq!(button.keyboard_activation(), None);
    }
}
