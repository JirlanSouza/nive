use iced::{
    widget::{container, text},
    Background, Border, Shadow,
};

use crate::theme::{self, control_metrics, ControlSize, ToneRole};
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq)]
struct BadgeMetrics {
    font_size: f32,
    padding_v: f32,
    padding_h: f32,
    radius: f32,
}

pub struct Badge<'a, Message> {
    label: &'a str,
    tone: ToneRole,
    size: ControlSize,
    _marker: std::marker::PhantomData<Message>,
}

impl<'a, Message> Badge<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            tone: ToneRole::Neutral,
            size: ControlSize::Sm,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn tone(mut self, tone: ToneRole) -> Self {
        self.tone = tone;
        self
    }

    pub fn neutral(label: &'a str) -> Self {
        Self::new(label).tone(ToneRole::Neutral)
    }

    pub fn info(label: &'a str) -> Self {
        Self::new(label).tone(ToneRole::Info)
    }

    pub fn success(label: &'a str) -> Self {
        Self::new(label).tone(ToneRole::Success)
    }

    pub fn warning(label: &'a str) -> Self {
        Self::new(label).tone(ToneRole::Warning)
    }

    pub fn danger(label: &'a str) -> Self {
        Self::new(label).tone(ToneRole::Danger)
    }

    pub fn accent(label: &'a str) -> Self {
        Self::new(label).tone(ToneRole::Primary)
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

    fn into_element(self) -> Element<'a, Message> {
        let metrics = metrics(self.size);

        container(text(self.label).size(metrics.font_size))
            .style(style(self.tone, metrics.radius))
            .padding([metrics.padding_v, metrics.padding_h])
            .into()
    }
}

impl<'a, Message> From<Badge<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(badge: Badge<'a, Message>) -> Self {
        badge.into_element()
    }
}

fn metrics(size: ControlSize) -> BadgeMetrics {
    let control = control_metrics(size);
    let spacing = theme::spacing();

    BadgeMetrics {
        font_size: match size {
            ControlSize::Xs | ControlSize::Sm => control_metrics(ControlSize::Xs).font_size,
            ControlSize::Md | ControlSize::Lg => control_metrics(ControlSize::Sm).font_size,
        },
        padding_v: match size {
            ControlSize::Xs => spacing.xxs,
            ControlSize::Sm | ControlSize::Md | ControlSize::Lg => spacing.xs,
        },
        padding_h: match size {
            ControlSize::Xs => spacing.xs,
            ControlSize::Sm => spacing.sm,
            ControlSize::Md | ControlSize::Lg => spacing.md,
        },
        radius: match size {
            ControlSize::Xs | ControlSize::Sm => control_metrics(ControlSize::Xs).radius,
            ControlSize::Md | ControlSize::Lg => control.radius,
        },
    }
}

fn style(tone: ToneRole, radius: f32) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let tone = theme.tone(tone);

        container::Style {
            text_color: Some(tone.color),
            background: Some(Background::Color(tone.container)),
            border: Border {
                color: tone.border.color,
                width: tone.border.width,
                radius: radius.into(),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

#[cfg(test)]
mod badge_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn style_uses_app_tone_role() {
        let theme = Theme::Dark;
        let style = style(ToneRole::Success, 4.0)(&theme);
        let tone = theme.tone(ToneRole::Success);

        assert_eq!(style.text_color, Some(tone.color));
        assert_eq!(style.border.color, tone.border.color);
    }
}
