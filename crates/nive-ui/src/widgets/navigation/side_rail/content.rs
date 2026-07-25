use std::borrow::Cow;

use iced::{
    widget::{canvas, container, rule, stack, Column},
    Alignment, Length, Padding,
};

use crate::theme::SurfaceRole;
use crate::widgets::controls::button::{self, GroupedItemKind, GroupedItemSpec, SelectionChrome};
use crate::widgets::navigation::overflow::ClipViewport;
use crate::widgets::overlays::{Tooltip, TooltipPlacement, TooltipScope};
use crate::widgets::primitives::{icon as icon_widget, IconRole};
use crate::Element;

use super::item::SideRailItem;
use super::label::RailLabelCanvas;
use super::layout::{
    ellipsize_label, item_layout, metrics, RailMetrics, HIDDEN_AFFORDANCE_HEIGHT,
    SELECTED_ACCENT_WIDTH,
};
use super::style::{rail_container_style, selected_accent_style};
use super::widget::{SideRail, SideRailState};
use super::RailSide;

impl<'a, Id, Message> SideRail<'a, Id, Message>
where
    Id: Clone + 'a,
    Message: Clone + 'a,
{
    pub(super) fn content_element(&self, state: &SideRailState) -> Element<'_, Message> {
        let metrics = metrics(self.size);
        let mut items = Column::new()
            .spacing(metrics.gap)
            .align_x(Alignment::Center)
            .width(Length::Fixed(metrics.width))
            .height(Length::Shrink);

        for item in &self.items {
            items = items.push(self.item_element(item));
        }

        let strip = ClipViewport::vertical(items)
            .width(Length::Fixed(metrics.width))
            .height(Length::Fill);
        let rail = Column::new()
            .spacing(0.0)
            .align_x(Alignment::Center)
            .width(Length::Fixed(metrics.width))
            .height(self.height.unwrap_or(Length::Fill))
            .push(self.chevron_button(
                IconRole::NiveDisclosureUp,
                state.overflow.show_start_chevron(),
            ))
            .push(strip)
            .push(self.chevron_button(
                IconRole::NiveDisclosureDown,
                state.overflow.show_end_chevron(),
            ));

        TooltipScope::new(
            container(rail)
                .style(rail_container_style(SurfaceRole::Chrome))
                .width(Length::Fixed(metrics.width))
                .height(self.height.unwrap_or(Length::Fill)),
        )
        .into()
    }

    pub(super) fn item_activation(&self, item: &SideRailItem<'a, Id>) -> Option<Message> {
        if item.disabled {
            return None;
        }
        self.on_select
            .as_ref()
            .map(|mapper| mapper(item.id.clone()))
    }

    fn chevron_button(&self, role: IconRole, visible: bool) -> Element<'_, Message> {
        let metrics = metrics(self.size);
        button::icon(
            role,
            match role {
                IconRole::NiveDisclosureUp => "Scroll rail up",
                IconRole::NiveDisclosureDown => "Scroll rail down",
                _ => "Scroll rail",
            },
        )
        .width(Length::Fixed(metrics.width))
        .into_grouped_item(GroupedItemSpec {
            size: metrics.size,
            radius: metrics.radius.into(),
            height: if visible {
                metrics.width
            } else {
                HIDDEN_AFFORDANCE_HEIGHT
            },
            padding_h: 0.0,
            selected: false,
            selection: SelectionChrome::Outlined,
            destructive: false,
            kind: GroupedItemKind::Embedded,
        })
    }

    fn item_element<'b>(&'b self, item: &'b SideRailItem<'a, Id>) -> Element<'b, Message> {
        let metrics = metrics(self.size);
        let layout = item_layout(item, metrics);
        let (visible_label, truncated) = ellipsize_label(&item.label, layout.label_track, metrics);
        let tooltip = item_tooltip(item, truncated);
        let mut content = Column::new()
            .spacing(metrics.gap)
            .align_x(Alignment::Center)
            .width(Length::Fixed(metrics.width))
            .height(Length::Fixed(layout.height))
            .padding(Padding::ZERO.vertical(metrics.item_padding_v));

        if let Some(icon) = item.icon {
            content = content.push(
                icon_widget::role(icon)
                    .custom_size(metrics.icon_size)
                    .color_maybe(None),
            );
        }
        content = content.push(
            canvas::Canvas::new(RailLabelCanvas {
                text: visible_label,
                side: self.side,
                font_size: metrics.font_size,
                line_height: metrics.line_height,
                selected: item.selected,
                disabled: item.disabled,
            })
            .width(Length::Fixed(metrics.width))
            .height(Length::Fixed(layout.label_track)),
        );

        let button = button::Button::custom(content.into())
            .disabled(item.disabled)
            .on_press_maybe(self.item_activation(item));

        let button = button.into_grouped_item_inset(GroupedItemSpec {
            size: metrics.size,
            radius: 0.0.into(),
            height: layout.height,
            padding_h: 0.0,
            selected: item.selected,
            // Selection is edge-anchored here: the accent bar carries it, so the
            // shared outline would double up on the same state.
            selection: SelectionChrome::Flat,
            destructive: false,
            kind: GroupedItemKind::Embedded,
        });

        let button = if item.selected {
            stack![button, selected_accent(self.side, metrics, layout.height)]
                .width(Length::Fixed(metrics.width))
                .height(Length::Fixed(layout.height))
                .into()
        } else {
            button
        };

        match tooltip {
            Some(label) => Tooltip::new(button, label)
                .placement(match self.side {
                    super::RailSide::Left => TooltipPlacement::Right,
                    super::RailSide::Right => TooltipPlacement::Left,
                })
                .into(),
            None => button,
        }
    }
}

/// Selection marker on the item's window-facing edge, spanning its full height.
///
/// The marker sits opposite the rail's structural seam so the two never share an
/// edge: the seam stays continuous as the rail-to-panel boundary, and selection
/// reads as a solid bar rather than a stub interrupting that boundary.
fn selected_accent<'a, Message>(
    side: RailSide,
    metrics: RailMetrics,
    item_height: f32,
) -> Element<'a, Message>
where
    Message: 'a,
{
    let accent = rule::vertical(SELECTED_ACCENT_WIDTH).style(selected_accent_style());

    container(accent)
        .width(Length::Fixed(metrics.width))
        .height(Length::Fixed(item_height))
        .align_x(match side {
            RailSide::Left => Alignment::Start,
            RailSide::Right => Alignment::End,
        })
        .into()
}

pub(super) fn item_tooltip<'a, Id>(
    item: &SideRailItem<'a, Id>,
    truncated: bool,
) -> Option<Cow<'a, str>> {
    if let Some(tooltip) = item.tooltip.clone() {
        return Some(tooltip);
    }

    truncated.then(|| Cow::Owned(item.label.to_string()))
}
