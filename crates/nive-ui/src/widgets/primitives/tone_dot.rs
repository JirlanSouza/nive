use iced::{
    border::Radius,
    widget::{container, Space},
    Background, Border, Length, Shadow,
};

use crate::theme::{ControlSize, ToneRole};
use crate::Element;

/// Compact filled dot using a semantic status tone.
///
/// `ToneDot` is intended for dense status metadata in rows, headers, rails,
/// and status bars. Use the size builders to keep dot sizing aligned with the
/// shared control-size vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToneDot {
    tone: ToneRole,
    size: ControlSize,
}

impl ToneDot {
    /// Creates a tone dot using the small control size.
    pub fn new(tone: ToneRole) -> Self {
        Self {
            tone,
            size: ControlSize::Sm,
        }
    }

    /// Sets the control size used to choose the dot diameter.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Uses the extra-small dot size.
    pub fn xs(self) -> Self {
        self.size(ControlSize::Xs)
    }

    /// Uses the small dot size.
    pub fn sm(self) -> Self {
        self.size(ControlSize::Sm)
    }

    /// Uses the medium dot size.
    pub fn md(self) -> Self {
        self.size(ControlSize::Md)
    }

    /// Uses the large dot size.
    pub fn lg(self) -> Self {
        self.size(ControlSize::Lg)
    }

    fn into_element<'a, Message>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        tone_dot(self.tone, dot_size(self.size))
    }
}

impl<'a, Message> From<ToneDot> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(dot: ToneDot) -> Self {
        dot.into_element()
    }
}

pub(crate) fn tone_dot<'a, Message>(tone: ToneRole, size: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    container(Space::new().width(Length::Fixed(size)))
        .style(tone_dot_style(tone, size))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .into()
}

fn tone_dot_style(tone: ToneRole, size: f32) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let tone = theme.tone(tone);

        container::Style {
            text_color: None,
            background: Some(Background::Color(tone.color)),
            border: Border {
                color: tone.border.color,
                width: 0.0,
                radius: Radius::new(size / 2.0),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

fn dot_size(size: ControlSize) -> f32 {
    match size {
        ControlSize::Xs => 4.0,
        ControlSize::Sm => 6.0,
        ControlSize::Md => 8.0,
        ControlSize::Lg => 10.0,
    }
}
