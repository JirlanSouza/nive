use std::{borrow::Cow, rc::Rc};

use iced::{
    widget::{container, row, Space},
    Alignment, Length,
};
use nive_ui::{
    theme::{self, ControlSize, SurfaceRole},
    widgets::{Panel, RailSide, VerticalRailBadge},
    Element, IconRole,
};

use super::bottom_tab_track::BottomPanelTabTrack;
use super::header::{trailing_controls, TrailingControlFlags};
use super::model::{
    BottomHeaderTab, PanelAction, PanelHeaderBar, PanelHostMode, PanelRail, PanelRailItem,
    PanelSelectorPlacement, WorkbenchPanel, WorkbenchPanelEvent, WorkbenchPanelHostState,
};
use crate::layout::WorkbenchRegion;
use crate::layout_probe;

/// Renders a generic panel host.
pub fn panel_host<'a, PanelId, ActionId, Message>(
    state: WorkbenchPanelHostState<PanelId>,
    panels: impl IntoIterator<Item = WorkbenchPanel<'a, PanelId, ActionId, Message>>,
    mapper: impl Fn(WorkbenchPanelEvent<PanelId, ActionId>) -> Message + 'a,
) -> Element<'a, Message>
where
    PanelId: Clone + Eq + 'a,
    ActionId: Clone + 'a,
    Message: Clone + 'a,
{
    panel_host_with_size(state, panels, mapper, ControlSize::Sm)
}

pub(crate) fn panel_host_with_size<'a, PanelId, ActionId, Message>(
    state: WorkbenchPanelHostState<PanelId>,
    panels: impl IntoIterator<Item = WorkbenchPanel<'a, PanelId, ActionId, Message>>,
    mapper: impl Fn(WorkbenchPanelEvent<PanelId, ActionId>) -> Message + 'a,
    size: ControlSize,
) -> Element<'a, Message>
where
    PanelId: Clone + Eq + 'a,
    ActionId: Clone + 'a,
    Message: Clone + 'a,
{
    let collapsed = matches!(state.mode, PanelHostMode::Collapsed);
    let show_restore = matches!(state.mode, PanelHostMode::Maximized);
    let mapper: Rc<dyn Fn(WorkbenchPanelEvent<PanelId, ActionId>) -> Message + 'a> =
        Rc::new(mapper);
    let mut panels: Vec<_> = panels.into_iter().filter(|panel| panel.visible).collect();
    if panels.is_empty() {
        return Space::new().into();
    }

    let active_index = state
        .active_panel
        .as_ref()
        .and_then(|active| panels.iter().position(|panel| &panel.id == active))
        .unwrap_or(0);
    let active_id = panels[active_index].id.clone();

    match state.selector {
        PanelSelectorPlacement::SideRail => {
            let Some(side) = state.region.rail_side() else {
                return render_active_panel(
                    state.region,
                    panels.remove(active_index),
                    show_restore,
                    mapper,
                    size,
                );
            };
            let items = rail_items(&panels, &active_id);
            let rail_mapper = {
                let mapper = mapper.clone();
                let state = state.clone();
                move |panel_id| {
                    let event = state.rail_activation_event(panel_id);
                    mapper(event)
                }
            };
            let rail = PanelRail::new(side, items)
                .on_select(rail_mapper)
                .view_with_size(size);
            let rail = layout_probe::probe(panel_rail_probe(state.region), rail);
            if collapsed {
                return rail;
            }
            let panel = render_active_panel(
                state.region,
                panels.remove(active_index),
                show_restore,
                mapper,
                size,
            );

            match side {
                RailSide::Left => row![rail, panel].height(Length::Fill).into(),
                RailSide::Right => row![panel, rail].height(Length::Fill).into(),
            }
        }
        PanelSelectorPlacement::HeaderTabs => render_bottom_host(
            state.region,
            panels,
            active_index,
            show_restore,
            mapper,
            size,
        ),
        PanelSelectorPlacement::Hidden => render_active_panel(
            state.region,
            panels.remove(active_index),
            show_restore,
            mapper,
            size,
        ),
    }
}

fn rail_items<'a, PanelId, ActionId, Message>(
    panels: &[WorkbenchPanel<'a, PanelId, ActionId, Message>],
    active_id: &PanelId,
) -> Vec<PanelRailItem<'a, PanelId>>
where
    PanelId: Clone + Eq,
{
    panels
        .iter()
        .filter_map(|panel| {
            let icon = panel.icon?;
            let mut item = PanelRailItem::new(panel.id.clone(), icon, panel.title.clone())
                .selected(&panel.id == active_id)
                .disabled(panel.disabled);
            if let Some(badge) = &panel.badge {
                let mut rail_badge = VerticalRailBadge::new(badge.clone());
                if let Some(status) = panel.status {
                    rail_badge = rail_badge.tone(status);
                }
                item = item.badge(rail_badge);
            }
            Some(item)
        })
        .collect()
}

fn render_active_panel<'a, PanelId, ActionId, Message>(
    region: WorkbenchRegion,
    panel: WorkbenchPanel<'a, PanelId, ActionId, Message>,
    show_restore: bool,
    mapper: Rc<dyn Fn(WorkbenchPanelEvent<PanelId, ActionId>) -> Message + 'a>,
    size: ControlSize,
) -> Element<'a, Message>
where
    PanelId: Clone + 'a,
    ActionId: Clone + 'a,
    Message: Clone + 'a,
{
    let WorkbenchPanel {
        id,
        title,
        tooltip,
        icon,
        badge,
        status,
        content,
        actions,
        collapsible,
        restorable,
        maximizable,
        closable,
        ..
    } = panel;
    let header = PanelHeaderBar {
        region,
        panel_id: id,
        title,
        tooltip,
        icon,
        badge,
        status,
        actions,
        collapsible,
        restorable: restorable && show_restore,
        maximizable,
        closable,
    }
    .view_with_size(mapper, size);
    let header = layout_probe::probe(panel_header_probe(region), header);
    let content = layout_probe::probe(panel_content_probe(region), content);
    Panel::new(content)
        .header(header)
        .role(SurfaceRole::Sidebar)
        .fill()
        .into()
}

fn render_bottom_host<'a, PanelId, ActionId, Message>(
    region: WorkbenchRegion,
    mut panels: Vec<WorkbenchPanel<'a, PanelId, ActionId, Message>>,
    active_index: usize,
    show_restore: bool,
    mapper: Rc<dyn Fn(WorkbenchPanelEvent<PanelId, ActionId>) -> Message + 'a>,
    size: ControlSize,
) -> Element<'a, Message>
where
    PanelId: Clone + Eq + 'a,
    ActionId: Clone + 'a,
    Message: Clone + 'a,
{
    let mut tabs = BottomPanelTabTrack::new(size, active_index);

    for panel in &panels {
        let tab = BottomHeaderTab::from(panel);
        let disabled = tab.disabled;
        let event = WorkbenchPanelEvent::Selected {
            region,
            panel_id: tab.panel_id.clone(),
        };
        tabs = tabs.push(tab, (!disabled).then(|| mapper(event)));
    }

    let active = panels.remove(active_index);
    let WorkbenchPanel {
        id,
        content,
        actions,
        collapsible,
        restorable,
        maximizable,
        closable,
        ..
    } = active;
    let restorable = restorable && show_restore;
    let controls_width = bottom_controls_width(
        &actions,
        collapsible,
        restorable,
        maximizable,
        closable,
        size,
    );
    let controls = trailing_controls(
        region,
        id,
        actions,
        TrailingControlFlags {
            restorable,
            maximizable,
            collapsible,
            closable,
        },
        mapper,
        size,
    );
    let tabs = layout_probe::probe("bottom_tab_track", Element::from(tabs));
    let tabs = container(tabs).width(Length::Fill).clip(true);
    let tabs = layout_probe::probe("bottom_selector", tabs);
    let controls = container(controls)
        .width(Length::Shrink)
        .max_width(controls_width)
        .clip(true);
    let controls = layout_probe::probe("bottom_controls", controls);
    let header = row![tabs, controls]
        .spacing(nive_ui::theme::spacing().xs)
        .align_y(Alignment::Center)
        .width(Length::Fill);
    let header = layout_probe::probe("bottom_header", header);
    let content = layout_probe::probe("bottom_content", content);
    Panel::new(content)
        .header(header)
        .role(SurfaceRole::Sidebar)
        .fill()
        .into()
}

fn panel_rail_probe(region: WorkbenchRegion) -> &'static str {
    match region {
        WorkbenchRegion::Left => "left_rail",
        WorkbenchRegion::Right => "right_rail",
        WorkbenchRegion::Bottom
        | WorkbenchRegion::Toolbar
        | WorkbenchRegion::Center
        | WorkbenchRegion::Status => "panel_rail",
    }
}

fn panel_header_probe(region: WorkbenchRegion) -> &'static str {
    match region {
        WorkbenchRegion::Left => "left_panel_header",
        WorkbenchRegion::Right => "right_panel_header",
        WorkbenchRegion::Bottom => "bottom_panel_header",
        WorkbenchRegion::Toolbar | WorkbenchRegion::Center | WorkbenchRegion::Status => {
            "panel_header"
        }
    }
}

fn panel_content_probe(region: WorkbenchRegion) -> &'static str {
    match region {
        WorkbenchRegion::Left => "left_panel_content",
        WorkbenchRegion::Right => "right_panel_content",
        WorkbenchRegion::Bottom => "bottom_panel_content",
        WorkbenchRegion::Toolbar | WorkbenchRegion::Center | WorkbenchRegion::Status => {
            "panel_content"
        }
    }
}

fn bottom_controls_width<ActionId>(
    actions: &[PanelAction<'_, ActionId>],
    collapsible: bool,
    restorable: bool,
    maximizable: bool,
    closable: bool,
    size: ControlSize,
) -> f32 {
    let control = theme::control_metrics(size);
    let action_count = actions.len()
        + usize::from(collapsible)
        + usize::from(restorable)
        + usize::from(maximizable)
        + usize::from(closable);
    let app_action_width = actions
        .iter()
        .map(|action| {
            action.label.as_ref().map_or(control.height, |label| {
                (label.chars().count() as f32 * control.font_size * 0.56 + control.height)
                    .max(control.height)
            })
        })
        .sum::<f32>();
    let automatic_count = action_count.saturating_sub(actions.len());
    let spacing = theme::spacing().xxs * action_count.saturating_sub(1) as f32;

    app_action_width + automatic_count as f32 * control.height + spacing
}

/// Creates a generic output bottom-panel slot.
pub fn output_panel_slot<'a, PanelId, ActionId, Message>(
    id: PanelId,
    content: impl Into<Element<'a, Message>>,
) -> WorkbenchPanel<'a, PanelId, ActionId, Message> {
    WorkbenchPanel::new(id, "Output", content).icon(IconRole::OpenMenu)
}

/// Creates a generic logs bottom-panel slot.
pub fn logs_panel_slot<'a, PanelId, ActionId, Message>(
    id: PanelId,
    content: impl Into<Element<'a, Message>>,
) -> WorkbenchPanel<'a, PanelId, ActionId, Message> {
    WorkbenchPanel::new(id, "Logs", content).icon(IconRole::MailInbox)
}

/// Creates a generic operations bottom-panel slot.
pub fn operations_panel_slot<'a, PanelId, ActionId, Message>(
    id: PanelId,
    content: impl Into<Element<'a, Message>>,
) -> WorkbenchPanel<'a, PanelId, ActionId, Message> {
    WorkbenchPanel::new(id, "Operations", content).icon(IconRole::ViewRefresh)
}

/// Creates a generic named bottom-panel slot.
pub fn bottom_panel_slot<'a, PanelId, ActionId, Message>(
    id: PanelId,
    title: impl Into<Cow<'a, str>>,
    content: impl Into<Element<'a, Message>>,
) -> WorkbenchPanel<'a, PanelId, ActionId, Message> {
    WorkbenchPanel::new(id, title, content)
}

impl<'a, PanelId, ActionId, Message> From<&WorkbenchPanel<'a, PanelId, ActionId, Message>>
    for BottomHeaderTab<'a, PanelId>
where
    PanelId: Clone,
{
    fn from(panel: &WorkbenchPanel<'a, PanelId, ActionId, Message>) -> Self {
        Self {
            panel_id: panel.id.clone(),
            label: panel.title.clone(),
            icon: panel.icon,
            badge: panel.badge.clone(),
            status: panel.status,
            disabled: panel.disabled,
            tooltip: panel.tooltip.clone(),
        }
    }
}
