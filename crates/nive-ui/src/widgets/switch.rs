use iced::{
    widget::{toggler, Toggler},
    Background, Color, Length,
};

use crate::theme::{
    self, control_metrics, BorderRole, ControlRole, ControlSize, ControlState, SpaceStep, TextRole,
    ToneRole,
};
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq)]
struct SwitchMetrics {
    size: u16,
    spacing: f32,
    text_size: f32,
}

pub struct Switch<'a, Message> {
    checked: bool,
    label: Option<&'a str>,
    size: ControlSize,
    disabled: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
}

impl<'a, Message> Switch<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(checked: bool) -> Self {
        Self {
            checked,
            label: None,
            size: ControlSize::Sm,
            disabled: false,
            on_toggle: None,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
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

    fn into_toggler(self) -> Toggler<'a, Message, crate::theme::Theme> {
        let metrics = metrics(self.size);
        let mut switch = toggler(self.checked)
            .size(u32::from(metrics.size))
            .width(Length::Shrink)
            .spacing(metrics.spacing)
            .text_size(metrics.text_size)
            .style(style);

        if let Some(label) = self.label {
            switch = switch.label(label);
        }

        switch.on_toggle_maybe(if self.disabled { None } else { self.on_toggle })
    }
}

impl<'a, Message> From<Switch<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(switch: Switch<'a, Message>) -> Self {
        switch.into_toggler().into()
    }
}

fn metrics(size: ControlSize) -> SwitchMetrics {
    let control = control_metrics(size);

    SwitchMetrics {
        size: match size {
            ControlSize::Xs => 18,
            ControlSize::Sm => 20,
            ControlSize::Md => 22,
            ControlSize::Lg => 24,
        },
        spacing: theme::space(SpaceStep::Md).max(control.gap),
        text_size: control.font_size,
    }
}

fn style(theme: &crate::theme::Theme, status: toggler::Status) -> toggler::Style {
    let theme = *theme;
    let is_toggled = match status {
        toggler::Status::Active { is_toggled }
        | toggler::Status::Hovered { is_toggled }
        | toggler::Status::Disabled { is_toggled } => is_toggled,
    };
    let is_hovered = matches!(status, toggler::Status::Hovered { .. });
    let is_disabled = matches!(status, toggler::Status::Disabled { .. });
    let state = if is_disabled {
        ControlState::DISABLED
    } else if is_hovered {
        ControlState::HOVERED
    } else {
        ControlState::ENABLED
    };
    let control = theme.control(ControlRole::Standard, state);
    let primary = theme.tone(ToneRole::Primary);
    let background = if is_toggled {
        primary.color
    } else {
        control.background
    };
    let foreground = if is_toggled {
        theme.tone(ToneRole::Primary).on_color
    } else {
        theme.text(TextRole::Secondary).color
    };
    let alpha = if is_disabled { 0.5 } else { 1.0 };
    let border = theme.border(BorderRole::Default);

    toggler::Style {
        background: Background::Color(background.scale_alpha(alpha)),
        foreground: Background::Color(foreground.scale_alpha(alpha)),
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
        background_border_width: border.width,
        background_border_color: border.color.scale_alpha(alpha),
        text_color: None,
        border_radius: None,
        padding_ratio: 0.1,
    }
}

#[cfg(test)]
mod switch_tests {
    use super::*;
    use crate::theme::{Theme, ToneRole};

    #[test]
    fn toggled_switch_uses_app_primary_background() {
        let theme = Theme::Dark;
        let style = style(&theme, toggler::Status::Active { is_toggled: true });

        assert_eq!(
            background_color(style.background),
            theme.tone(ToneRole::Primary).color
        );
    }

    fn background_color(background: Background) -> Color {
        match background {
            Background::Color(color) => color,
            _ => panic!("Expected color background"),
        }
    }
}
