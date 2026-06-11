mod blur;
mod focus_tracker;
mod style;

use std::{borrow::Cow, cell::Cell, rc::Rc};

use iced::{border::Radius, widget::text_input, Length};

use crate::theme::ControlSize;
use crate::Element;

use self::style as theme_text_input;
use self::{blur::InputBlur, focus_tracker::InputFocusTracker};

pub use crate::theme::FieldValidation;
pub use style::TextInputAppearance;

pub struct Input<'a, Message> {
    placeholder: Cow<'a, str>,
    value: Cow<'a, str>,
    appearance: TextInputAppearance,
    validation: FieldValidation,
    size: ControlSize,
    fill: bool,
    disabled: bool,
    secure: bool,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_submit: Option<Message>,
    on_blur: Option<Message>,
    focus_tracker: Option<Rc<Cell<bool>>>,
}

pub fn default<'a, Message>(placeholder: &'a str, value: &'a str) -> Input<'a, Message>
where
    Message: Clone + 'a,
{
    Input::new(placeholder, value)
}

pub fn invalid<'a, Message>(placeholder: &'a str, value: &'a str) -> Input<'a, Message>
where
    Message: Clone + 'a,
{
    Input::new(placeholder, value).validation(FieldValidation::Invalid)
}

pub fn default_owned<'a, Message>(
    placeholder: impl Into<Cow<'a, str>>,
    value: impl Into<Cow<'a, str>>,
) -> Input<'a, Message>
where
    Message: Clone + 'a,
{
    Input::new(placeholder, value)
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct InputRender {
    fill: bool,
    font_size: f32,
    padding_v: f32,
    padding_h: f32,
    radius: Radius,
    appearance: TextInputAppearance,
}

impl<'a, Message> Input<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(placeholder: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) -> Self {
        Self {
            placeholder: placeholder.into(),
            value: value.into(),
            appearance: TextInputAppearance::Standard,
            validation: FieldValidation::Valid,
            size: ControlSize::Sm,
            fill: true,
            disabled: false,
            secure: false,
            on_input: None,
            on_submit: None,
            on_blur: None,
            focus_tracker: None,
        }
    }

    pub fn appearance(mut self, appearance: TextInputAppearance) -> Self {
        self.appearance = appearance;
        self
    }

    pub fn standard(self) -> Self {
        self.appearance(TextInputAppearance::Standard)
    }

    pub fn embedded(self) -> Self {
        self.appearance(TextInputAppearance::Embedded)
    }

    pub fn validation(mut self, validation: FieldValidation) -> Self {
        self.validation = validation;
        self
    }

    pub fn invalid(self, invalid: bool) -> Self {
        self.validation(if invalid {
            FieldValidation::Invalid
        } else {
            FieldValidation::Valid
        })
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

    pub fn shrink(mut self) -> Self {
        self.fill = false;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    pub fn on_input(mut self, f: impl Fn(String) -> Message + 'a) -> Self {
        self.on_input = Some(Box::new(f));
        self
    }

    pub fn on_input_maybe(mut self, f: Option<impl Fn(String) -> Message + 'a>) -> Self {
        self.on_input = f.map(|f| Box::new(f) as _);
        self
    }

    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    pub fn on_blur(mut self, message: Message) -> Self {
        self.on_blur = Some(message);
        self
    }

    pub fn on_blur_maybe(mut self, message: Option<Message>) -> Self {
        self.on_blur = message;
        self
    }

    pub fn value(&self) -> &str {
        self.value.as_ref()
    }

    pub fn track_focus(mut self, focused: Rc<Cell<bool>>) -> Self {
        self.focus_tracker = Some(focused);
        self
    }

    pub fn map<NewMessage: Clone + 'a>(
        self,
        f: impl Fn(Message) -> NewMessage + 'a,
    ) -> Input<'a, NewMessage> {
        let f: Rc<dyn Fn(Message) -> NewMessage + 'a> = Rc::new(f);
        let on_input = self.on_input.map({
            let f = Rc::clone(&f);
            move |on_input| {
                Box::new(move |value| f(on_input(value))) as Box<dyn Fn(String) -> NewMessage + 'a>
            }
        });

        Input {
            placeholder: self.placeholder,
            value: self.value,
            appearance: self.appearance,
            validation: self.validation,
            size: self.size,
            fill: self.fill,
            disabled: self.disabled,
            secure: self.secure,
            on_input,
            on_submit: self.on_submit.map(|message| f(message)),
            on_blur: self.on_blur.map(|message| f(message)),
            focus_tracker: self.focus_tracker,
        }
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn field_validation(&self) -> FieldValidation {
        self.validation
    }

    pub fn into_group_element(
        self,
        fill: bool,
        radius: Radius,
        font_size: f32,
        padding_v: f32,
        padding_h: f32,
    ) -> Element<'a, Message> {
        self.into_text_input_with(InputRender {
            fill,
            font_size,
            padding_v,
            padding_h,
            radius,
            appearance: TextInputAppearance::Embedded,
        })
    }

    fn into_text_input(self) -> Element<'a, Message> {
        let metrics = theme_text_input::metrics(self.size);
        let fill = self.fill;
        let appearance = self.appearance;
        self.into_text_input_with(InputRender {
            fill,
            font_size: metrics.font_size,
            padding_v: metrics.padding_v,
            padding_h: metrics.padding_h,
            radius: metrics.radius.into(),
            appearance,
        })
    }

    fn into_text_input_with(self, render: InputRender) -> Element<'a, Message> {
        let mut input = text_input::TextInput::new(self.placeholder.as_ref(), self.value.as_ref())
            .padding([render.padding_v, render.padding_h])
            .size(render.font_size)
            .secure(self.secure);

        input = input.style(theme_text_input::style(
            render.appearance,
            self.validation,
            render.radius,
        ));

        if render.fill {
            input = input.width(Length::Fill);
        }

        input = input.on_input_maybe(if self.disabled { None } else { self.on_input });

        if let Some(msg) = self.on_submit {
            input = input.on_submit(msg);
        }

        let input: Element<'a, Message> = input.into();

        let input = match self.on_blur {
            Some(on_blur) => Element::new(InputBlur {
                content: input,
                on_blur,
            }),
            None => input,
        };

        match self.focus_tracker {
            Some(focused) => Element::new(InputFocusTracker {
                content: input,
                focused,
            }),
            None => input,
        }
    }
}

impl<'a, Message> From<Input<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(input: Input<'a, Message>) -> Self {
        input.into_text_input()
    }
}
