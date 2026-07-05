use iced::{
    alignment,
    widget::{column, container, mouse_area, stack},
    Length, Padding,
};

pub use nive_core::{ToastPresentation, ToastTone};

use crate::theme::{self, GapRole, ToneRole};
use crate::widgets::{controls::button, IconName, InlineAlert};
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    #[default]
    BottomRight,
}

pub struct ToastHost<'a, Message> {
    content: Element<'a, Message>,
    items: Vec<Element<'a, Message>>,
    position: ToastPosition,
    on_pause: Option<Message>,
    on_resume: Option<Message>,
}

impl<'a, Message> ToastHost<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            items: Vec::new(),
            position: ToastPosition::default(),
            on_pause: None,
            on_resume: None,
        }
    }

    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    pub fn on_hover(mut self, pause: Message, resume: Message) -> Self {
        self.on_pause = Some(pause);
        self.on_resume = Some(resume);
        self
    }

    pub fn toasts<T>(
        mut self,
        toasts: impl IntoIterator<Item = &'a T>,
        on_dismiss: impl Fn(T::Id) -> Message + Copy + 'a,
    ) -> Self
    where
        T: ToastPresentation + 'a,
    {
        self.items = toasts
            .into_iter()
            .map(|toast| toast_view(toast, on_dismiss))
            .collect();
        self
    }

    pub fn into_element(self) -> Element<'a, Message> {
        if self.items.is_empty() {
            return self.content;
        }

        let toast_stack = column(self.items)
            .spacing(theme::gap(GapRole::Related))
            .width(Length::Fixed(360.0));
        let toast_stack: Element<'a, Message> = match (self.on_pause, self.on_resume) {
            (Some(pause), Some(resume)) => mouse_area(toast_stack)
                .on_enter(pause)
                .on_exit(resume)
                .into(),
            _ => toast_stack.into(),
        };
        let (horizontal, vertical) = alignment_for(self.position);
        let overlay = container(toast_stack)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(horizontal)
            .align_y(vertical)
            .padding(Padding::new(theme::gap(GapRole::Section)));

        stack(vec![self.content, overlay.into()])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl<'a, Message> From<ToastHost<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(host: ToastHost<'a, Message>) -> Self {
        host.into_element()
    }
}

fn toast_view<'a, Message, T>(
    toast: &'a T,
    on_dismiss: impl Fn(T::Id) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    T: ToastPresentation + 'a,
{
    let dismiss = button::icon(IconName::Close)
        .xs()
        .tooltip("Dismiss")
        .on_press(on_dismiss(toast.id()));
    let mut alert = InlineAlert::new(toast.title())
        .tone(toast_tone(toast.tone()))
        .action(dismiss);

    if let Some(body) = toast.body() {
        alert = alert.body(body);
    }

    alert.into()
}

fn toast_tone(tone: ToastTone) -> ToneRole {
    match tone {
        ToastTone::Info => ToneRole::Info,
        ToastTone::Success => ToneRole::Success,
        ToastTone::Warning => ToneRole::Warning,
        ToastTone::Danger => ToneRole::Danger,
    }
}

fn alignment_for(position: ToastPosition) -> (alignment::Horizontal, alignment::Vertical) {
    match position {
        ToastPosition::TopLeft => (alignment::Horizontal::Left, alignment::Vertical::Top),
        ToastPosition::TopRight => (alignment::Horizontal::Right, alignment::Vertical::Top),
        ToastPosition::BottomLeft => (alignment::Horizontal::Left, alignment::Vertical::Bottom),
        ToastPosition::BottomRight => (alignment::Horizontal::Right, alignment::Vertical::Bottom),
    }
}

#[cfg(test)]
mod toast_host_tests {
    use super::*;

    #[test]
    fn default_position_is_bottom_right() {
        assert_eq!(ToastPosition::default(), ToastPosition::BottomRight);
    }

    #[test]
    fn position_alignment_matches_corner() {
        assert_eq!(
            alignment_for(ToastPosition::TopLeft),
            (alignment::Horizontal::Left, alignment::Vertical::Top)
        );
        assert_eq!(
            alignment_for(ToastPosition::BottomRight),
            (alignment::Horizontal::Right, alignment::Vertical::Bottom)
        );
    }
}
