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
use crate::widgets::primitives::IconName;

pub use style::ButtonVariant;
pub use style::{button_control_state, focus_ring, ButtonFocusRing};

pub struct Button<'a, Message> {
    content: Content<'a, Message>,
    leading_icon: Option<IconName>,
    trailing_icon: Option<IconName>,
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
    pub(crate) kind: GroupedItemKind,
}

pub fn primary<'a, Message>(label: &'a str) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(Content::Label(label), ButtonVariant::Primary)
}

pub fn outline<'a, Message>(label: &'a str) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(Content::Label(label), ButtonVariant::Outline)
}

pub fn secondary<'a, Message>(label: &'a str) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(Content::Label(label), ButtonVariant::Secondary)
}

pub fn ghost<'a, Message>(label: &'a str) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(Content::Label(label), ButtonVariant::Ghost)
}

pub fn destructive<'a, Message>(label: &'a str) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(Content::Label(label), ButtonVariant::Destructive)
}

pub fn link<'a, Message>(label: &'a str) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(Content::Label(label), ButtonVariant::Link)
}

pub fn icon<'a, Message>(app_icon: IconName) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(Content::Icon(app_icon), ButtonVariant::Ghost)
}

impl<'a, Message> Button<'a, Message>
where
    Message: Clone + 'a,
{
    fn new(content: Content<'a, Message>, variant: ButtonVariant) -> Self {
        Self {
            content,
            leading_icon: None,
            trailing_icon: None,
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

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub(crate) fn custom(content: Element<'a, Message>) -> Self {
        Self::new(Content::Custom(content), ButtonVariant::Ghost)
    }

    pub fn primary(self) -> Self {
        self.variant(ButtonVariant::Primary)
    }

    pub fn secondary(self) -> Self {
        self.variant(ButtonVariant::Secondary)
    }

    pub fn outline(self) -> Self {
        self.variant(ButtonVariant::Outline)
    }

    pub fn ghost(self) -> Self {
        self.variant(ButtonVariant::Ghost)
    }

    pub fn destructive(self) -> Self {
        self.variant(ButtonVariant::Destructive)
    }

    pub fn link(self) -> Self {
        self.variant(ButtonVariant::Link)
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

    pub fn shrink(mut self) -> Self {
        self.width = Some(Length::Shrink);
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

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    pub fn leading_icon(mut self, icon: IconName) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn trailing_icon(mut self, icon: IconName) -> Self {
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
            ButtonVariant::Primary => ButtonFocusRing::OnPrimary,
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
