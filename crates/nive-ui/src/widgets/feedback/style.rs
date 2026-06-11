use iced::{widget::container, widget::progress_bar, widget::text, Background, Border, Shadow};

use crate::theme::{self, control_metrics, ControlSize, ShapeRole, TextRole, ToneRole};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InlineAlertMetrics {
    pub padding: f32,
    pub gap: f32,
    pub radius: f32,
    pub indicator_size: f32,
    pub title_size: f32,
    pub body_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgressMetrics {
    pub height: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoadingMetrics {
    pub indicator_size: f32,
    pub label_size: f32,
    pub gap: f32,
}

pub fn inline_alert_metrics(size: ControlSize) -> InlineAlertMetrics {
    let control = control_metrics(size);
    let spacing = theme::spacing();

    InlineAlertMetrics {
        padding: match size {
            ControlSize::Xs => spacing.md,
            ControlSize::Sm => spacing.md,
            ControlSize::Md => spacing.lg,
            ControlSize::Lg => spacing.xl,
        },
        gap: spacing.md.max(control.gap),
        radius: theme::active().shape(ShapeRole::Large).radius_value(),
        indicator_size: match size {
            ControlSize::Xs => 6.0,
            ControlSize::Sm => 7.0,
            ControlSize::Md | ControlSize::Lg => 8.0,
        },
        title_size: control.font_size,
        body_size: match size {
            ControlSize::Xs | ControlSize::Sm => control_metrics(ControlSize::Xs).font_size,
            ControlSize::Md | ControlSize::Lg => control_metrics(ControlSize::Sm).font_size,
        },
    }
}

pub fn progress_metrics(size: ControlSize) -> ProgressMetrics {
    let control = control_metrics(size);

    ProgressMetrics {
        height: match size {
            ControlSize::Xs => 3.0,
            ControlSize::Sm => 4.0,
            ControlSize::Md => 6.0,
            ControlSize::Lg => 8.0,
        },
        radius: control.radius,
    }
}

pub fn loading_metrics(size: ControlSize) -> LoadingMetrics {
    let control = control_metrics(size);

    LoadingMetrics {
        indicator_size: match size {
            ControlSize::Xs => 6.0,
            ControlSize::Sm => 7.0,
            ControlSize::Md => 8.0,
            ControlSize::Lg => 10.0,
        },
        label_size: control.font_size,
        gap: control.gap,
    }
}

pub fn inline_alert_style(
    tone: ToneRole,
    radius: f32,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
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

pub fn title_style(tone: ToneRole) -> impl Fn(&crate::theme::Theme) -> text::Style {
    move |theme| text::Style {
        color: Some(theme.tone(tone).color),
    }
}

pub fn body_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    move |theme| text::Style {
        color: Some(theme.text(TextRole::Secondary).color),
    }
}

pub fn progress_style(
    tone: ToneRole,
    radius: f32,
) -> impl Fn(&crate::theme::Theme) -> progress_bar::Style {
    move |theme| {
        let tone = theme.tone(tone);

        progress_bar::Style {
            background: Background::Color(tone.container),
            bar: Background::Color(tone.color),
            border: Border {
                color: tone.border.color,
                width: 0.0,
                radius: radius.into(),
            },
        }
    }
}

pub fn indicator_style(
    tone: ToneRole,
    size: f32,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let tone = theme.tone(tone);

        container::Style {
            text_color: None,
            background: Some(Background::Color(tone.color)),
            border: Border {
                color: tone.border.color,
                width: 0.0,
                radius: (size / 2.0).into(),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

pub fn loading_label_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    move |theme| text::Style {
        color: Some(theme.text(TextRole::Secondary).color),
    }
}

#[cfg(test)]
mod feedback_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn inline_alert_uses_app_tone_role() {
        let theme = Theme::Dark;
        let style = inline_alert_style(ToneRole::Warning, 6.0)(&theme);
        let tone = theme.tone(ToneRole::Warning);

        assert_eq!(style.text_color, Some(tone.color));
        assert_eq!(style.border.color, tone.border.color);
    }

    #[test]
    fn progress_uses_app_tone_role() {
        let theme = Theme::Dark;
        let style = progress_style(ToneRole::Primary, 6.0)(&theme);
        let tone = theme.tone(ToneRole::Primary);

        assert_eq!(background_color(style.bar), tone.color);
    }

    fn background_color(background: Background) -> iced::Color {
        match background {
            Background::Color(color) => color,
            _ => panic!("Expected color background"),
        }
    }
}
