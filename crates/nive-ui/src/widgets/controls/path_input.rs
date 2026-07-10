use iced::Length;

use crate::theme::ControlSize;
use crate::widgets::controls::{button, input};
use crate::widgets::{IconRole, InputGroup};
use crate::Element;

/// Controlled path text input with a browse affordance.
///
/// Path text uses `on_change`; the browse button uses `on_browse` because the
/// whole field is not itself an action button.
pub struct PathInput<'a, Message> {
    placeholder: &'a str,
    value: &'a str,
    browse_label: &'a str,
    leading_icon: Option<IconRole>,
    size: ControlSize,
    width: Length,
    disabled: bool,
    on_change: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_browse: Option<Message>,
}

impl<'a, Message> PathInput<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(placeholder: &'a str, value: &'a str) -> Self {
        Self {
            placeholder,
            value,
            browse_label: "Browse",
            leading_icon: None,
            size: ControlSize::Sm,
            width: Length::Fill,
            disabled: false,
            on_change: None,
            on_browse: None,
        }
    }

    pub fn browse_label(mut self, label: &'a str) -> Self {
        self.browse_label = label;
        self
    }

    pub fn leading_icon(mut self, icon: IconRole) -> Self {
        self.leading_icon = Some(icon);
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

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    crate::impl_layout_builders!(width_direct, fill_width_direct, shrink_width_direct);

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Maps edited path text values into app messages.
    pub fn on_change(mut self, on_change: impl Fn(String) -> Message + 'a) -> Self {
        self.on_change = Some(Box::new(on_change));
        self
    }

    /// Conditionally maps edited path text values into app messages.
    pub fn on_change_maybe(mut self, on_change: Option<impl Fn(String) -> Message + 'a>) -> Self {
        self.on_change = on_change.map(|on_change| Box::new(on_change) as _);
        self
    }

    /// Emits a message from the browse affordance.
    pub fn on_browse(mut self, message: Message) -> Self {
        self.on_browse = Some(message);
        self
    }

    /// Conditionally emits a message from the browse affordance.
    pub fn on_browse_maybe(mut self, message: Option<Message>) -> Self {
        self.on_browse = message;
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let input = input::default(self.placeholder, self.value)
            .disabled(self.disabled)
            .on_change_maybe(self.on_change);

        let browse = button::icon(self.leading_icon.unwrap_or(IconRole::Folder))
            .disabled(self.disabled)
            .on_press_maybe(self.on_browse)
            .tooltip(self.browse_label);

        InputGroup::new(input)
            .trailing_action(browse)
            .width(self.width)
            .size(self.size)
            .into()
    }
}

impl<'a, Message> From<PathInput<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(input: PathInput<'a, Message>) -> Self {
        input.into_element()
    }
}
