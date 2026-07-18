mod scope;
mod widget;

use std::{borrow::Cow, time::Duration};

use iced::{
    border::Radius,
    widget::{container, text},
    Background, Border, Length, Padding,
};

use crate::{
    theme::{self, BorderRole, SurfaceRole, TypographyRole},
    Element,
};

use self::widget::TooltipWidget;
pub use scope::TooltipScope;

const COLD_DELAY: Duration = Duration::from_millis(500);
const TOOLTIP_MAX_WIDTH: f32 = 280.0;

/// Preferred physical side for passive Tooltip disclosure.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TooltipPlacement {
    Top,
    Right,
    #[default]
    Bottom,
    Left,
}

/// Passive text disclosure anchored to one widget.
///
/// Tooltip text supplements rather than replaces the anchor's semantic name.
/// It reveals after 500ms when isolated, uses scoped neighboring timing inside
/// [`TooltipScope`], and emits no native accessibility node.
pub struct Tooltip<'a, Message> {
    anchor: Element<'a, Message>,
    label: Cow<'a, str>,
    placement: TooltipPlacement,
    delay: Duration,
    now_override: Option<iced::time::Instant>,
    intent_override: Option<(bool, bool)>,
}

impl<'a, Message> Tooltip<'a, Message>
where
    Message: 'a,
{
    pub fn new(anchor: impl Into<Element<'a, Message>>, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            anchor: anchor.into(),
            label: label.into(),
            placement: TooltipPlacement::default(),
            delay: COLD_DELAY,
            now_override: None,
            intent_override: None,
        }
    }

    pub fn placement(mut self, placement: TooltipPlacement) -> Self {
        self.placement = placement;
        self
    }

    #[cfg(test)]
    fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    #[cfg(test)]
    fn at(mut self, now: iced::time::Instant) -> Self {
        self.now_override = Some(now);
        self
    }

    #[cfg(test)]
    fn intent(mut self, hovered: bool, focused: bool) -> Self {
        self.intent_override = Some((hovered, focused));
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let label = surface(self.label);
        Element::new(TooltipWidget::new(
            self.anchor,
            label,
            self.placement,
            self.delay,
            self.now_override,
            self.intent_override,
        ))
    }
}

impl<'a, Message> From<Tooltip<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(tooltip: Tooltip<'a, Message>) -> Self {
        tooltip.into_element()
    }
}

#[cfg(test)]
pub(crate) fn immediate_for_test<'a, Message>(
    anchor: impl Into<Element<'a, Message>>,
    label: impl Into<Cow<'a, str>>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    Tooltip::new(anchor, label)
        .placement(TooltipPlacement::Bottom)
        .delay(Duration::ZERO)
        .into()
}

fn surface<'a, Message>(label: Cow<'a, str>) -> Element<'a, Message>
where
    Message: 'a,
{
    let typography = theme::typography(TypographyRole::BodySmall);
    let label = text(label)
        .size(typography.size)
        .line_height(typography.line_height)
        .shaping(text::Shaping::Auto)
        .wrapping(text::Wrapping::WordOrGlyph)
        .width(Length::Shrink);

    container(label)
        .padding(Padding {
            top: 4.0,
            right: 8.0,
            bottom: 4.0,
            left: 8.0,
        })
        .max_width(TOOLTIP_MAX_WIDTH)
        .style(surface_style)
        .into()
}

fn surface_style(theme: &crate::theme::Theme) -> container::Style {
    let theme = *theme;
    let surface = theme.surface(SurfaceRole::Popover);
    let perimeter = theme.border(BorderRole::Subtle);

    container::Style {
        text_color: Some(surface.foreground),
        background: Some(Background::Color(surface.background)),
        border: Border {
            color: perimeter.color,
            width: 1.0,
            radius: Radius::new(4.0),
        },
        shadow: surface.shadow,
        ..container::Style::default()
    }
}

#[cfg(test)]
#[path = "tooltip/lifecycle_tests.rs"]
mod lifecycle_tests;

#[cfg(test)]
#[path = "tooltip/scope_tests.rs"]
mod scope_tests;

#[cfg(test)]
#[path = "tooltip/geometry_tests.rs"]
mod geometry_tests;
