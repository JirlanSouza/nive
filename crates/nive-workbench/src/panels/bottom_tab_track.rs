use iced::{
    advanced::{
        layout::{self, Layout, Node},
        mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Renderer as _, Shell, Widget,
    },
    keyboard, touch,
    widget::{container, text, Row},
    Alignment, Background, Border, Color, Event, Length, Padding, Rectangle, Shadow, Size, Vector,
};
use nive_ui::{
    advanced::focus::FocusState,
    theme::{BorderRole, ControlRole, ControlSize, ControlState, TextRole},
    widgets::{Badge, BadgeContent, ToneDot},
    Element,
};

use super::BottomHeaderTab;

const MAX_TAB_WIDTH: f32 = 220.0;

pub(super) struct BottomPanelTabTrack<'a, Message> {
    items: Vec<TrackItem<'a, Message>>,
    size: ControlSize,
    active_index: usize,
    content: Element<'a, Message>,
}

struct TrackItem<'a, Message> {
    metadata: BottomHeaderTab<'a, ()>,
    message: Option<Message>,
}

#[derive(Debug, Default)]
struct TrackState {
    focus: FocusState,
    focused_index: Option<usize>,
    hovered_index: Option<usize>,
    item_bounds: Vec<Rectangle>,
    offset: f32,
    max_offset: f32,
    viewport_width: f32,
    last_active_index: Option<usize>,
}

impl<'a, Message> BottomPanelTabTrack<'a, Message>
where
    Message: Clone + 'a,
{
    pub(super) fn new(size: ControlSize, active_index: usize) -> Self {
        Self {
            items: Vec::new(),
            size,
            active_index,
            content: iced::widget::Space::new().into(),
        }
    }

    pub(super) fn push<PanelId>(
        mut self,
        tab: BottomHeaderTab<'a, PanelId>,
        message: Option<Message>,
    ) -> Self {
        self.items.push(TrackItem {
            metadata: BottomHeaderTab {
                panel_id: (),
                label: tab.label,
                icon: tab.icon,
                badge: tab.badge,
                status: tab.status,
                disabled: tab.disabled,
                tooltip: tab.tooltip,
            },
            message,
        });
        self.content = nive_ui::widgets::TooltipScope::new(build_content(
            &self.items,
            self.size,
            self.active_index,
        ))
        .into();
        self
    }

    fn metrics(&self) -> TrackMetrics {
        let control = nive_ui::theme::control_metrics(self.size);
        TrackMetrics {
            height: control.height,
            font_size: control.font_size.max(14.0),
            icon_size: control.icon_size,
            gap: control.gap,
            padding_h: nive_ui::theme::spacing().md,
            radius: control.radius,
        }
    }

    fn enabled_indices(&self) -> Vec<usize> {
        self.items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| (!item.metadata.disabled).then_some(index))
            .collect()
    }

    fn reconcile_focus(&self, state: &mut TrackState) {
        let enabled = self.enabled_indices();
        if !state
            .focused_index
            .is_some_and(|index| enabled.contains(&index))
        {
            state.focused_index = enabled
                .contains(&self.active_index)
                .then_some(self.active_index)
                .or_else(|| enabled.first().copied());
        }
    }
}

fn build_content<'a, Message>(
    items_data: &[TrackItem<'a, Message>],
    size: ControlSize,
    active_index: usize,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let control = nive_ui::theme::control_metrics(size);
    let metrics = TrackMetrics {
        height: control.height,
        font_size: control.font_size.max(14.0),
        icon_size: control.icon_size,
        gap: control.gap,
        padding_h: nive_ui::theme::spacing().md,
        radius: control.radius,
    };
    let mut items = Row::new()
        .spacing(0.0)
        .align_y(Alignment::Center)
        .height(Length::Fixed(metrics.height))
        .width(Length::Shrink);

    for (index, item) in items_data.iter().enumerate() {
        let active = index == active_index;
        let mut content = Row::new()
            .spacing(metrics.gap)
            .align_y(Alignment::Center)
            .height(Length::Fill)
            .width(Length::Shrink);
        if let Some(icon) = item.metadata.icon {
            content =
                content.push(nive_ui::widgets::Icon::role(icon).custom_size(metrics.icon_size));
        }
        content = content.push(
            text(item.metadata.label.clone())
                .size(metrics.font_size)
                .wrapping(text::Wrapping::None),
        );
        if let Some(badge) = &item.metadata.badge {
            content =
                content.push(Badge::from_content(badge.clone()).disabled(item.metadata.disabled));
        }
        let status_badge_present = item.metadata.badge.as_ref().is_some_and(
            |badge| matches!(badge, BadgeContent::Status(label) if !label.trim().is_empty()),
        );
        if let Some(status) = item
            .metadata
            .status
            .as_ref()
            .filter(|status| !status.is_empty() && !status_badge_present)
        {
            content = content
                .push(
                    ToneDot::new(status.tone())
                        .size(size)
                        .disabled(item.metadata.disabled),
                )
                .push(text(status.label().to_owned()));
        }

        let tab: Element<'_, Message> = container(content)
            .style(tab_content_style(active, item.metadata.disabled))
            .padding(Padding::ZERO.horizontal(metrics.padding_h))
            .height(Length::Fixed(metrics.height))
            .max_width(MAX_TAB_WIDTH)
            .clip(true)
            .into();
        let tooltip = item
            .metadata
            .tooltip
            .clone()
            .unwrap_or_else(|| item.metadata.label.clone());
        items = items.push(nive_ui::widgets::Tooltip::new(tab, tooltip));
    }

    container(items)
        .width(Length::Fill)
        .height(Length::Fixed(metrics.height))
        .clip(true)
        .into()
}

#[derive(Debug, Clone, Copy)]
struct TrackMetrics {
    height: f32,
    font_size: f32,
    icon_size: f32,
    gap: f32,
    padding_h: f32,
    radius: f32,
}

mod widget;

impl<'a, Message> From<BottomPanelTabTrack<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(track: BottomPanelTabTrack<'a, Message>) -> Self {
        Element::new(track)
    }
}

fn tab_content_style(active: bool, disabled: bool) -> impl Fn(&nive_ui::Theme) -> container::Style {
    move |theme| container::Style {
        text_color: Some(if disabled {
            theme
                .control(
                    ControlRole::Selectable,
                    ControlState::DISABLED.selected_if(active),
                )
                .foreground
        } else if active {
            theme.text(TextRole::Primary).color
        } else {
            theme.text(TextRole::Secondary).color
        }),
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border::default(),
        shadow: Shadow::default(),
        ..container::Style::default()
    }
}

fn horizontal_wheel(delta: mouse::ScrollDelta) -> f32 {
    match delta {
        mouse::ScrollDelta::Lines { x, y } => (if x.abs() > f32::EPSILON { x } else { y }) * 24.0,
        mouse::ScrollDelta::Pixels { x, y } => {
            if x.abs() > f32::EPSILON {
                x
            } else {
                y
            }
        }
    }
}

fn reveal_index(state: &mut TrackState, index: usize) {
    let Some(bounds) = state.item_bounds.get(index) else {
        return;
    };
    if bounds.x < 0.0 {
        state.offset = (state.offset + bounds.x).max(0.0);
    } else if bounds.x + bounds.width > state.viewport_width {
        state.offset =
            (state.offset + bounds.x + bounds.width - state.viewport_width).min(state.max_offset);
    }
}

fn translate_track(node: Node, offset: f32) -> (f32, f32, Vec<Rectangle>, Node) {
    let Some(row) = node.children().first() else {
        return (0.0, node.size().width, Vec::new(), node);
    };
    let viewport_width = node.size().width;
    let translation = Vector::new(-offset, 0.0);
    let mut children = Vec::with_capacity(row.children().len());
    let mut bounds = Vec::with_capacity(row.children().len());
    for child in row.children() {
        let mut child = child.clone();
        child.translate_mut(translation);
        bounds.push(child.bounds());
        children.push(child);
    }
    let content_width = row
        .children()
        .last()
        .map(|child| child.bounds().x + child.bounds().width - row.bounds().x)
        .unwrap_or(0.0);
    let translated_row =
        Node::with_children(row.size(), children).move_to(row.bounds().position() + translation);
    let translated = Node::with_children(node.size(), vec![translated_row]);
    (content_width, viewport_width, bounds, translated)
}

fn current_item_bounds(layout: Layout<'_>) -> Vec<Rectangle> {
    layout
        .children()
        .next()
        .map(|row| row.children().map(|item| item.bounds()).collect())
        .unwrap_or_default()
}

trait SelectedIf {
    fn selected_if(self, selected: bool) -> Self;
}

impl SelectedIf for ControlState {
    fn selected_if(self, selected: bool) -> Self {
        if selected {
            self.selected()
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests;
