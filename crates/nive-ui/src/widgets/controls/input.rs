pub(super) mod adapter;
mod builder;
mod style;

#[cfg(test)]
mod text_input_tests;

use std::{borrow::Cow, cell::Cell, rc::Rc};

use iced::widget::Id;
use iced::{border::Radius, Font, Length, Padding};

use crate::theme::ControlSize;
use crate::Element;

pub use crate::theme::FieldValidation;
pub use style::TextInputAppearance;

/// Controlled text input.
///
/// Text changes are reported with [`Input::on_change`]. The older
/// `on_input` spelling is intentionally not part of the public API.
/// Without an `on_change` callback, the input is read-only rather than
/// disabled: focus, selection, navigation, Select All, and Copy remain
/// available while mutation, paste, cut, and IME composition are blocked.
/// `disabled(true)` is stronger and blocks focus and activation.
///
/// Standard appearance owns one form frame; Embedded appearance is reserved
/// for a typed [`super::InputGroup`] that owns the complete frame. Values stay
/// single-line and use the native Iced horizontal caret scrolling behavior.
/// Iced 0.14 does not expose an independent caret-color or native accessible
/// relationship API, so semantic names are retained metadata, not a claim of
/// current AccessKit emission.
pub struct Input<'a, Message> {
    placeholder: Cow<'a, str>,
    value: Cow<'a, str>,
    appearance: TextInputAppearance,
    validation: FieldValidation,
    size: ControlSize,
    width: Length,
    id: Option<Id>,
    generated_field_id: bool,
    semantic_name: Option<Cow<'a, str>>,
    disabled: bool,
    read_only: bool,
    secure: bool,
    on_change: Option<Box<dyn Fn(String) -> Message + 'a>>,
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
    width: Length,
    font: Font,
    font_size: f32,
    line_height: f32,
    padding: Padding,
    radius: Radius,
    appearance: TextInputAppearance,
}

impl<'a, Message> From<Input<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(input: Input<'a, Message>) -> Self {
        input.into_text_input()
    }
}
