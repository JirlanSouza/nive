use iced::{border::Radius, widget::container, widget::text, Background, Border, Shadow};

use crate::theme::{
    self, control_metrics, BorderRole, ControlSize, SurfaceRole, TextRole, ToneRole,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetadataMetrics {
    pub gap: f32,
    pub row_gap: f32,
    pub padding_v: f32,
    pub padding_h: f32,
    pub radius: f32,
    pub label_size: f32,
    pub value_size: f32,
    pub row_height: f32,
    pub tone_size: f32,
}

pub fn metrics(size: ControlSize) -> MetadataMetrics {
    let control = control_metrics(size);
    let spacing = theme::spacing();

    MetadataMetrics {
        gap: spacing.xs,
        row_gap: control.gap,
        padding_v: match size {
            ControlSize::Xs => spacing.xxs,
            ControlSize::Sm => spacing.xs,
            ControlSize::Md => spacing.sm,
            ControlSize::Lg => spacing.md,
        },
        padding_h: match size {
            ControlSize::Xs => spacing.xs,
            ControlSize::Sm => spacing.sm,
            ControlSize::Md => spacing.md,
            ControlSize::Lg => spacing.md + spacing.xxs,
        },
        radius: control.radius,
        label_size: match size {
            ControlSize::Xs | ControlSize::Sm => control_metrics(ControlSize::Xs).font_size,
            ControlSize::Md | ControlSize::Lg => control_metrics(ControlSize::Sm).font_size,
        },
        value_size: control.font_size,
        row_height: control.height,
        tone_size: match size {
            ControlSize::Xs => 5.0,
            ControlSize::Sm => 6.0,
            ControlSize::Md | ControlSize::Lg => 7.0,
        },
    }
}

pub fn item_style(
    role: SurfaceRole,
    radius: f32,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let surface = theme.surface(role);

        container::Style {
            text_color: Some(surface.foreground),
            background: Some(Background::Color(surface.background)),
            border: Border {
                color: surface.border.color,
                width: surface.border.width,
                radius: radius.into(),
            },
            shadow: surface.shadow,
            ..container::Style::default()
        }
    }
}

pub fn row_style(radius: f32) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let theme = *theme;
        let border = theme.border(BorderRole::Subtle);

        container::Style {
            text_color: Some(theme.text(TextRole::Primary).color),
            background: None,
            border: Border {
                color: border.color,
                width: 0.0,
                radius: radius.into(),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

pub fn label_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    move |theme| text::Style {
        color: Some(theme.text(TextRole::Muted).color),
    }
}

pub fn value_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    move |theme| text::Style {
        color: Some(theme.text(TextRole::Primary).color),
    }
}

pub fn secondary_value_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    move |theme| text::Style {
        color: Some(theme.text(TextRole::Secondary).color),
    }
}

pub fn tone_indicator_style(
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
                radius: Radius::new(size / 2.0),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

#[cfg(test)]
mod metadata_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn item_uses_app_requested_surface_role() {
        let theme = Theme::Dark;
        let style = item_style(SurfaceRole::Elevated, 6.0)(&theme);

        assert_eq!(
            background_color(&style),
            theme.surface(SurfaceRole::Elevated).background
        );
    }

    #[test]
    fn label_style_uses_app_muted_text() {
        let theme = Theme::Dark;

        assert_eq!(
            label_style()(&theme).color,
            Some(theme.text(TextRole::Muted).color)
        );
    }

    fn background_color(style: &container::Style) -> iced::Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }
}
