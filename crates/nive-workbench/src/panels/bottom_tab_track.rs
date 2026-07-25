use std::borrow::Cow;

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
    theme::{BorderRole, ControlRole, ControlSize, ControlState, TextRole, ToneRole},
    widgets::{Badge, BadgeContent, StatusIndicator},
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
            TrackBuild::Actual(&[]),
        ))
        .into();
        self
    }

    fn metrics(&self) -> TrackMetrics {
        track_metrics(self.size)
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
    build: TrackBuild<'_>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let metrics = track_metrics(size);
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
        if let Some(signal) = tab_signal(&item.metadata) {
            content = content.push(signal);
        }

        let mut tab = container(content)
            .style(tab_content_style(active, item.metadata.disabled))
            .padding(Padding::ZERO.horizontal(metrics.padding_h))
            .height(Length::Fixed(metrics.height));
        if matches!(build, TrackBuild::Actual(_)) {
            tab = tab.max_width(MAX_TAB_WIDTH).clip(true);
        }
        let tab: Element<'_, Message> = tab.into();
        let truncated = match build {
            TrackBuild::Actual(truncated) => truncated.get(index).copied().unwrap_or(false),
            TrackBuild::Measure => false,
        };
        let tab = match tab_tooltip(&item.metadata, truncated) {
            Some(tooltip) => nive_ui::widgets::Tooltip::new(tab, tooltip).into(),
            None => tab,
        };
        items = items.push(tab);
    }

    let mut content = container(items).height(Length::Fixed(metrics.height));
    if matches!(build, TrackBuild::Actual(_)) {
        content = content.width(Length::Fill).clip(true);
    }
    content.into()
}

#[derive(Debug, Clone, Copy)]
enum TrackBuild<'a> {
    Actual(&'a [bool]),
    Measure,
}

/// Resolves the single trailing signal a bottom-panel tab may carry.
///
/// A count answers "how many" and a status answers "how is it": two competing
/// signals rather than two styles of one, so they share a single slot instead of
/// stacking. A count that carries meaning wins it, and an app that cares more
/// about state than quantity omits the count. Whatever survives renders as a
/// toned `Badge` rather than a bare dot, so the slot always keeps visible
/// wording instead of leaving color as the sole carrier.
fn tab_signal<'a, PanelId, Message>(
    tab: &BottomHeaderTab<'a, PanelId>,
) -> Option<Element<'a, Message>>
where
    Message: Clone + 'a,
{
    let tone = tab
        .status
        .as_ref()
        .map_or(ToneRole::Neutral, StatusIndicator::tone);

    Some(
        Badge::from_content(tab_signal_content(tab)?)
            .tone(tone)
            .disabled(tab.disabled)
            .into(),
    )
}

/// Picks which of the two competing signals occupies the slot.
///
/// A zero count and a blank status label carry nothing, so neither claims the
/// slot and the status behind them gets its turn.
fn tab_signal_content<'a, PanelId>(tab: &BottomHeaderTab<'a, PanelId>) -> Option<BadgeContent<'a>> {
    match tab.badge.as_ref() {
        None | Some(BadgeContent::Count(0)) => {}
        Some(BadgeContent::Status(label)) if label.trim().is_empty() => {}
        Some(content) => return Some(content.clone()),
    }

    tab.status
        .as_ref()
        .filter(|status| !status.is_empty())
        .map(|status| BadgeContent::Status(Cow::Owned(status.label().to_owned())))
}

/// Whether a count took the slot and pushed the status wording out of the tab.
///
/// Displaced wording has to resurface elsewhere — the tooltip, and the header
/// of the panel once it is active. Wording that still holds the slot must not,
/// or the header would restate what the tab already says.
pub(super) fn status_displaced<PanelId>(tab: &BottomHeaderTab<'_, PanelId>) -> bool {
    tab.status.as_ref().is_some_and(|status| !status.is_empty())
        && matches!(tab_signal_content(tab), Some(BadgeContent::Count(_)))
}

/// Status wording that lost the slot to a count.
fn suppressed_status<'a, PanelId>(tab: &'a BottomHeaderTab<'_, PanelId>) -> Option<&'a str> {
    let status = tab.status.as_ref()?;

    status_displaced(tab).then(|| status.label())
}

fn tab_tooltip<'a, PanelId>(
    tab: &BottomHeaderTab<'a, PanelId>,
    truncated: bool,
) -> Option<Cow<'a, str>> {
    if let Some(tooltip) = tab.tooltip.clone() {
        return Some(tooltip);
    }

    match (suppressed_status(tab), truncated) {
        (Some(status), _) => Some(Cow::Owned(format!("{} — {status}", tab.label))),
        (None, true) => Some(tab.label.clone()),
        (None, false) => None,
    }
}

fn measured_truncation<Message>(
    items: &[TrackItem<'_, Message>],
    size: ControlSize,
    active_index: usize,
    renderer: &nive_ui::Renderer,
) -> Vec<bool>
where
    Message: Clone,
{
    let mut content = build_content(items, size, active_index, TrackBuild::Measure);
    let mut tree = tree::Tree::new(&content);
    let height = nive_ui::theme::control_metrics(size).height;
    let node = content.as_widget_mut().layout(
        &mut tree,
        renderer,
        &layout::Limits::new(Size::ZERO, Size::new(100_000.0, height)),
    );

    node.children()
        .first()
        .map(|row| {
            row.children()
                .iter()
                .map(|item| item.size().width > MAX_TAB_WIDTH)
                .collect()
        })
        .unwrap_or_else(|| vec![false; items.len()])
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

/// Tab geometry, resolved once so layout and measurement cannot drift apart.
///
/// The horizontal padding sits a step above the intra-tab gap so the space
/// between two tabs stays several times the space between one tab's own parts.
/// That ratio is what lets the eye chunk the strip into tabs at a glance.
fn track_metrics(size: ControlSize) -> TrackMetrics {
    let control = nive_ui::theme::control_metrics(size);
    TrackMetrics {
        height: control.height,
        font_size: control.font_size.max(14.0),
        icon_size: control.icon_size,
        gap: control.gap,
        padding_h: nive_ui::theme::spacing().lg,
        radius: control.radius,
    }
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
