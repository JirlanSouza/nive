mod style;

use std::borrow::Cow;

use iced::{
    border::Radius,
    widget::{container, text, Row},
    Alignment, Length,
};

use crate::theme::{self, ControlSize, ToneRole};
use crate::widgets::display::Badge;
use crate::widgets::primitives::tone_dot::tone_dot;
use crate::Element;

use self::style as theme_segmented_control;

use super::button::{self, GroupedItemKind, GroupedItemSpec};
use crate::advanced::control_group::radius_for_position;
use crate::advanced::control_group::{position_for_index, SlotPosition};
use crate::widgets::primitives::IconRole;

/// A single-selection control rendered as a track with a rounded selected thumb.
///
/// Use [`SegmentedControl::flat`] for the linked button-group variant.
pub struct SegmentedControl<'a, Message> {
    items: Vec<SegmentedItem<'a, Message>>,
    size: ControlSize,
    width: Length,
    variant: SegmentedControlVariant,
}

pub struct SegmentedItem<'a, Message> {
    label: Cow<'a, str>,
    icon: Option<IconRole>,
    badge: Option<Cow<'a, str>>,
    status: Option<ToneRole>,
    tooltip: Option<Cow<'a, str>>,
    selected: bool,
    disabled: bool,
    on_press: Option<Message>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SegmentedControlVariant {
    Default,
    Flat,
}

impl<'a, Message> SegmentedControl<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            size: ControlSize::Sm,
            width: Length::Shrink,
            variant: SegmentedControlVariant::Default,
        }
    }

    pub fn push(mut self, item: SegmentedItem<'a, Message>) -> Self {
        self.items.push(item);
        self
    }

    pub fn item(self, item: SegmentedItem<'a, Message>) -> Self {
        self.push(item)
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

    crate::impl_layout_builders!(width_direct, fill_width_direct, shrink_width_direct);

    /// Renders the control as linked items sharing one outer border.
    ///
    /// The default variant uses an inset rounded thumb for the selected item.
    /// The flat variant removes that inset and rounds only the outer corners.
    pub fn flat(mut self) -> Self {
        self.variant = SegmentedControlVariant::Flat;
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = theme_segmented_control::metrics(self.size);
        let item_count = self.items.len();
        let variant = self.variant;
        let size = self.size;
        let fill = matches!(self.width, Length::Fill);
        let outer_padding = outer_padding_for_variant(variant, metrics);
        let item_height = item_height_for_variant(variant, metrics);
        let items = self.items.into_iter().enumerate().map(|(index, item)| {
            item.into_element(
                size,
                metrics,
                position_for_index(index, item_count),
                fill,
                variant,
                item_height,
            )
        });
        let mut content = Row::new()
            .spacing(0)
            .align_y(Alignment::Center)
            .height(Length::Fixed(item_height));

        for item in items {
            content = content.push(item);
        }

        let mut segmented = container(content)
            .style(theme_segmented_control::container_style(metrics.radius))
            .padding(outer_padding)
            .height(Length::Fixed(metrics.height));

        segmented = segmented.width(self.width);

        segmented.into()
    }
}

impl<'a, Message> Default for SegmentedControl<'a, Message>
where
    Message: Clone + 'a,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message> From<SegmentedControl<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(control: SegmentedControl<'a, Message>) -> Self {
        control.into_element()
    }
}

impl<'a, Message: Clone + 'a> SegmentedItem<'a, Message> {
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            badge: None,
            status: None,
            tooltip: None,
            selected: false,
            disabled: false,
            on_press: None,
        }
    }

    pub fn icon(mut self, icon: IconRole) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn badge(mut self, badge: impl Into<Cow<'a, str>>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    pub fn status(mut self, status: ToneRole) -> Self {
        self.status = Some(status);
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn tooltip_maybe<T>(mut self, tooltip: Option<T>) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.tooltip = tooltip.map(Into::into);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    fn into_element(
        self,
        size: ControlSize,
        metrics: theme_segmented_control::SegmentedControlMetrics,
        position: SlotPosition,
        fill: bool,
        variant: SegmentedControlVariant,
        item_height: f32,
    ) -> Element<'a, Message> {
        let spacing = theme::spacing();
        let mut content = Row::new().spacing(spacing.xxs).align_y(Alignment::Center);

        if let Some(icon) = self.icon {
            content = content.push(crate::widgets::primitives::icon::role(icon));
        }
        content = content.push(text(self.label));
        if let Some(status) = self.status {
            content = content.push(tone_dot(status, 6.0));
        }
        if let Some(badge) = self.badge {
            content = content.push(Badge::new(badge).tone(status_tone(self.status)).xs());
        }

        let mut item = button::Button::custom(content.into()).secondary();
        if self.disabled {
            item = item.disabled(true);
        }

        if fill {
            item = item.width(Length::Fill);
        }

        let (kind, radius) = match variant {
            SegmentedControlVariant::Default => {
                (GroupedItemKind::Selectable, Radius::new(metrics.radius))
            }
            SegmentedControlVariant::Flat => (
                GroupedItemKind::Embedded,
                radius_for_position(position, metrics.radius),
            ),
        };
        item.tooltip_maybe(self.tooltip)
            .on_press_maybe(self.on_press)
            .into_grouped_item(GroupedItemSpec {
                size,
                radius,
                height: item_height,
                padding_h: metrics.padding_h,
                selected: self.selected,
                destructive: false,
                kind,
            })
    }
}

fn status_tone(status: Option<ToneRole>) -> ToneRole {
    status.unwrap_or(ToneRole::Neutral)
}

fn outer_padding_for_variant(
    variant: SegmentedControlVariant,
    metrics: theme_segmented_control::SegmentedControlMetrics,
) -> f32 {
    match variant {
        SegmentedControlVariant::Default => metrics.outer_padding,
        SegmentedControlVariant::Flat => 0.0,
    }
}

fn item_height_for_variant(
    variant: SegmentedControlVariant,
    metrics: theme_segmented_control::SegmentedControlMetrics,
) -> f32 {
    (metrics.height - outer_padding_for_variant(variant, metrics) * 2.0).max(0.0)
}

#[cfg(test)]
mod segmented_control_tests {
    use super::*;

    #[test]
    fn default_item_height_consumes_track_inset_inside_control_height() {
        let metrics = theme_segmented_control::metrics(ControlSize::Sm);

        assert_eq!(
            item_height_for_variant(SegmentedControlVariant::Default, metrics),
            metrics.height - metrics.outer_padding * 2.0
        );
    }

    #[test]
    fn flat_item_height_matches_control_height() {
        let metrics = theme_segmented_control::metrics(ControlSize::Sm);

        assert_eq!(
            item_height_for_variant(SegmentedControlVariant::Flat, metrics),
            metrics.height
        );
    }
}
