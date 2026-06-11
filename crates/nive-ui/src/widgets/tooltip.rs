use std::time::Duration;

use iced::{
    widget::{container, text, tooltip},
    Background, Border,
};

use crate::theme::{self, BorderRole, ShapeRole, SpaceStep, SurfaceRole, TypographyRole};
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq)]
struct TooltipMetrics {
    font_size: f32,
    gap: f32,
    padding: f32,
    delay: Duration,
}

pub fn bottom<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
) -> Element<'a, Message>
where
    Message: 'a,
{
    let metrics = metrics();

    tooltip(
        content,
        text(label)
            .size(metrics.font_size)
            .shaping(text::Shaping::Auto),
        tooltip::Position::Bottom,
    )
    .gap(metrics.gap)
    .padding(metrics.padding)
    .delay(metrics.delay)
    .style(style())
    .into()
}

fn metrics() -> TooltipMetrics {
    TooltipMetrics {
        font_size: theme::typography(TypographyRole::BodySmall).size,
        gap: theme::space(SpaceStep::Xs),
        padding: theme::space(SpaceStep::Sm),
        delay: Duration::from_millis(450),
    }
}

fn style() -> impl Fn(&crate::theme::Theme) -> container::Style {
    |theme: &crate::theme::Theme| {
        let theme = *theme;
        let surface = theme.surface(SurfaceRole::Popover);
        let border = theme.border(BorderRole::Default);

        container::Style {
            text_color: Some(surface.foreground),
            background: Some(Background::Color(surface.background)),
            border: Border {
                color: border.color,
                width: border.width,
                radius: theme.shape(ShapeRole::Medium).radius(),
            },
            shadow: surface.shadow,
            ..container::Style::default()
        }
    }
}

#[cfg(test)]
mod tooltip_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn style_uses_app_popover_surface() {
        let theme = Theme::Dark;
        let style = style()(&theme);

        assert_eq!(
            background_color(&style),
            theme.surface(SurfaceRole::Popover).background
        );
    }

    fn background_color(style: &container::Style) -> iced::Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }
}
