use crate::theme::ControlSize;
use crate::widgets::{button, input, AppIcon, InputGroup};
use crate::Element;

pub struct PathInput<'a, Message> {
    placeholder: &'a str,
    value: &'a str,
    browse_label: &'a str,
    leading_icon: Option<AppIcon>,
    size: ControlSize,
    disabled: bool,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
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
            disabled: false,
            on_input: None,
            on_browse: None,
        }
    }

    pub fn browse_label(mut self, label: &'a str) -> Self {
        self.browse_label = label;
        self
    }

    pub fn leading_icon(mut self, icon: AppIcon) -> Self {
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_input(mut self, on_input: impl Fn(String) -> Message + 'a) -> Self {
        self.on_input = Some(Box::new(on_input));
        self
    }

    pub fn on_browse(mut self, message: Message) -> Self {
        self.on_browse = Some(message);
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let input = input::default(self.placeholder, self.value)
            .disabled(self.disabled)
            .on_input_maybe(self.on_input);

        let browse = button::icon(self.leading_icon.unwrap_or(AppIcon::Folder))
            .disabled(self.disabled)
            .on_press_maybe(self.on_browse)
            .tooltip(self.browse_label);

        InputGroup::new(input)
            .trailing_action(browse)
            .fill()
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
