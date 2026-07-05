mod alpha_percent;
mod controls;
mod event;
mod external_sync;
mod hex_color;
mod hsva_color;
mod state;
mod view;
mod widget;

use iced::Color;

pub use hex_color::RgbHexColor;

use crate::Element;

use self::widget::ColorPickerWidget;

pub struct ColorPicker<'a, Message> {
    value: Color,
    disabled: bool,
    on_change: Option<Box<dyn Fn(Color) -> Message + 'a>>,
}

impl<'a, Message> ColorPicker<'a, Message>
where
    Message: 'a,
{
    pub fn new(value: Color) -> Self {
        Self {
            value,
            disabled: false,
            on_change: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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
        Element::new(ColorPickerWidget {
            value: self.value,
            disabled: self.disabled,
            on_change: self.on_change,
        })
    }
}

impl<'a, Message> From<ColorPicker<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(picker: ColorPicker<'a, Message>) -> Self {
        picker.into_element()
    }
}
