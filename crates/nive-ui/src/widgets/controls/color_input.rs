mod event;
mod state;
mod view;
mod widget;

use iced::Color;

use crate::Element;

use self::widget::ColorInputWidget;

pub struct ColorInput<'a, Message> {
    value: Color,
    disabled: bool,
    tooltip: &'a str,
    on_change: Option<Box<dyn Fn(Color) -> Message + 'a>>,
}

impl<'a, Message> ColorInput<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(value: Color) -> Self {
        Self {
            value,
            disabled: false,
            tooltip: "Color",
            on_change: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn tooltip(mut self, tooltip: &'a str) -> Self {
        self.tooltip = tooltip;
        self
    }

    pub fn on_change(mut self, f: impl Fn(Color) -> Message + 'a) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn on_change_maybe(mut self, f: Option<impl Fn(Color) -> Message + 'a>) -> Self {
        self.on_change = f.map(|f| Box::new(f) as _);
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        Element::new(ColorInputWidget::new(
            self.value,
            self.disabled,
            self.tooltip,
            self.on_change,
        ))
    }
}

impl<'a, Message> From<ColorInput<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(input: ColorInput<'a, Message>) -> Self {
        input.into_element()
    }
}
