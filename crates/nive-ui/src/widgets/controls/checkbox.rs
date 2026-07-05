use iced::{
    border::Radius,
    widget::checkbox::Status,
    widget::{checkbox as iced_checkbox, text, Checkbox as IcedCheckbox},
    Background, Border, Color, Length,
};

use crate::theme::{
    self, control_metrics, ControlRole, ControlSize, ControlState, SpaceStep, TextRole, ToneRole,
};
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq)]
struct CheckboxMetrics {
    size: f32,
    radius: f32,
    spacing: f32,
    font_size: f32,
}

pub struct Checkbox<'a, Message> {
    label: &'a str,
    checked: bool,
    size: ControlSize,
    width: Option<Length>,
    disabled: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
}

impl<'a, Message> Checkbox<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(label: &'a str, checked: bool) -> Self {
        Self {
            label,
            checked,
            size: ControlSize::Sm,
            width: None,
            disabled: false,
            on_toggle: None,
        }
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn xs(self) -> Self {
        self.size(ControlSize::Xs)
    }

    pub fn sm(self) -> Self {
        self.size(ControlSize::Sm)
    }

    pub fn md(self) -> Self {
        self.size(ControlSize::Md)
    }

    pub fn lg(self) -> Self {
        self.size(ControlSize::Lg)
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn fill(self) -> Self {
        self.width(Length::Fill)
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_toggle(mut self, on_toggle: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(on_toggle));
        self
    }

    pub fn on_toggle_maybe(mut self, on_toggle: Option<impl Fn(bool) -> Message + 'a>) -> Self {
        self.on_toggle = on_toggle.map(|on_toggle| Box::new(on_toggle) as _);
        self
    }

    fn into_checkbox(self) -> IcedCheckbox<'a, Message, crate::theme::Theme> {
        let metrics = metrics(self.size);
        let mut checkbox = iced_checkbox(self.checked)
            .label(self.label)
            .size(metrics.size)
            .spacing(metrics.spacing)
            .text_size(metrics.font_size)
            .text_shaping(text::Shaping::Auto)
            .style(style(metrics.radius.into()));

        if let Some(width) = self.width {
            checkbox = checkbox.width(width);
        }

        checkbox.on_toggle_maybe(if self.disabled { None } else { self.on_toggle })
    }
}

impl<'a, Message> From<Checkbox<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(checkbox: Checkbox<'a, Message>) -> Self {
        checkbox.into_checkbox().into()
    }
}

fn metrics(size: ControlSize) -> CheckboxMetrics {
    let control = control_metrics(size);

    CheckboxMetrics {
        size: match size {
            ControlSize::Xs => 14.0,
            ControlSize::Sm => 16.0,
            ControlSize::Md => 18.0,
            ControlSize::Lg => 20.0,
        },
        radius: control.radius.min(5.0),
        spacing: theme::space(SpaceStep::Md).max(control.gap),
        font_size: control.font_size,
    }
}

fn style(radius: Radius) -> impl Fn(&crate::theme::Theme, Status) -> iced_checkbox::Style {
    move |theme: &crate::theme::Theme, status: Status| {
        let theme = *theme;
        let is_checked = match status {
            Status::Active { is_checked }
            | Status::Hovered { is_checked }
            | Status::Disabled { is_checked } => is_checked,
        };
        let state = match status {
            Status::Active { .. } => ControlState::ENABLED,
            Status::Hovered { .. } => ControlState::HOVERED,
            Status::Disabled { .. } => ControlState::DISABLED,
        };
        let control = theme.control(ControlRole::Standard, state);
        let primary = theme.tone(ToneRole::Primary);
        let disabled = matches!(status, Status::Disabled { .. });
        let alpha = if disabled { 0.55 } else { 1.0 };

        iced_checkbox::Style {
            background: Background::Color(if is_checked {
                primary.color.scale_alpha(alpha)
            } else {
                control.background
            }),
            icon_color: if is_checked {
                theme.tone(ToneRole::Primary).on_color.scale_alpha(alpha)
            } else {
                Color::TRANSPARENT
            },
            border: Border {
                color: if is_checked {
                    primary.border.color.scale_alpha(alpha)
                } else {
                    control.border.color
                },
                width: if is_checked {
                    primary.border.width
                } else {
                    control.border.width
                },
                radius,
            },
            text_color: Some(if disabled {
                theme.text(TextRole::Muted).color.scale_alpha(0.65)
            } else {
                theme.text(TextRole::Secondary).color
            }),
        }
    }
}

#[cfg(test)]
mod checkbox_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn checked_checkbox_uses_app_primary_background() {
        let theme = Theme::Dark;
        let checkbox = style(Radius::new(4.0))(&theme, Status::Active { is_checked: true });

        assert_eq!(
            background_color(checkbox.background),
            theme.tone(ToneRole::Primary).color
        );
    }

    #[test]
    fn unchecked_checkbox_uses_app_active_control_background() {
        let theme = Theme::Dark;
        let checkbox = style(Radius::new(4.0))(&theme, Status::Active { is_checked: false });

        assert_eq!(
            background_color(checkbox.background),
            theme
                .control(ControlRole::Standard, ControlState::ENABLED)
                .background
        );
    }

    fn background_color(background: Background) -> Color {
        match background {
            Background::Color(color) => color,
            _ => panic!("Expected color background"),
        }
    }
}
