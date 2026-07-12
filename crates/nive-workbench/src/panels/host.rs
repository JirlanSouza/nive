use std::borrow::Cow;
use std::rc::Rc;

use iced::widget::{row, Space};
use iced::{Alignment, Length};
use nive_ui::theme::SurfaceRole;
use nive_ui::widgets::{Panel, SegmentedControl, SegmentedItem};
use nive_ui::{Element, IconRole};

use super::model::PanelRail;
use super::model::{
    BottomHeaderTab, PanelHeaderBar, PanelRailItem, PanelSelectorPlacement, WorkbenchPanel,
    WorkbenchPanelEvent, WorkbenchPanelHostState,
};
use crate::layout::WorkbenchRegion;

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
    panel_host_with_options(state, panels, false, false, mapper)
}

pub(crate) fn panel_host_with_collapsed<'a, PanelId, ActionId, Message>(
    state: WorkbenchPanelHostState<PanelId>,
    panels: impl IntoIterator<Item = WorkbenchPanel<'a, PanelId, ActionId, Message>>,
    collapsed: bool,
    mapper: impl Fn(WorkbenchPanelEvent<PanelId, ActionId>) -> Message + 'a,
) -> Element<'a, Message>
where
    PanelId: Clone + Eq + 'a,
    ActionId: Clone + 'a,
    Message: Clone + 'a,
{
    panel_host_with_options(state, panels, collapsed, false, mapper)
}

pub(crate) fn panel_host_with_restore<'a, PanelId, ActionId, Message>(
    state: WorkbenchPanelHostState<PanelId>,
    panels: impl IntoIterator<Item = WorkbenchPanel<'a, PanelId, ActionId, Message>>,
    mapper: impl Fn(WorkbenchPanelEvent<PanelId, ActionId>) -> Message + 'a,
) -> Element<'a, Message>
where
    PanelId: Clone + Eq + 'a,
    ActionId: Clone + 'a,
    Message: Clone + 'a,
{
    panel_host_with_options(state, panels, false, true, mapper)
}

fn panel_host_with_options<'a, PanelId, ActionId, Message>(
    state: WorkbenchPanelHostState<PanelId>,
    panels: impl IntoIterator<Item = WorkbenchPanel<'a, PanelId, ActionId, Message>>,
    collapsed: bool,
    show_restore: bool,
    mapper: impl Fn(WorkbenchPanelEvent<PanelId, ActionId>) -> Message + 'a,
) -> Element<'a, Message>
where
    PanelId: Clone + Eq + 'a,
    ActionId: Clone + 'a,
    Message: Clone + 'a,
{
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
            let items = rail_items(&panels, &active_id);
            let rail_mapper = {
                let mapper = mapper.clone();
                let state = state.clone();
                move |panel_id| {
                    let event = state.rail_activation_event(panel_id, collapsed);
                    mapper(event)
                }
            };
            let rail = PanelRail::new(state.region, items)
                .on_select(rail_mapper)
                .view();
            if collapsed {
                return rail;
            }
            let panel = render_active_panel(
                state.region,
                panels.remove(active_index),
                show_restore,
                mapper,
            );
            row![rail, panel].height(Length::Fill).into()
        }
        PanelSelectorPlacement::HeaderTabs => {
            render_bottom_host(state.region, panels, active_index, show_restore, mapper)
        }
        PanelSelectorPlacement::Hidden => render_active_panel(
            state.region,
            panels.remove(active_index),
            show_restore,
            mapper,
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
                item = item.badge(badge.clone());
            }
            if let Some(status) = panel.status {
                item = item.status(status);
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
) -> Element<'a, Message>
where
    PanelId: Clone + 'a,
    ActionId: Clone + 'a,
    Message: Clone + 'a,
{
    let WorkbenchPanel {
        id,
        title,
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
        icon,
        badge,
        status,
        actions,
        collapsible,
        restorable: restorable && show_restore,
        maximizable,
        closable,
    }
    .view(mapper);
    Panel::new(content)
        .header(header)
        .role(SurfaceRole::Panel)
        .fill()
        .into()
}

fn render_bottom_host<'a, PanelId, ActionId, Message>(
    region: WorkbenchRegion,
    mut panels: Vec<WorkbenchPanel<'a, PanelId, ActionId, Message>>,
    active_index: usize,
    show_restore: bool,
    mapper: Rc<dyn Fn(WorkbenchPanelEvent<PanelId, ActionId>) -> Message + 'a>,
) -> Element<'a, Message>
where
    PanelId: Clone + Eq + 'a,
    ActionId: Clone + 'a,
    Message: Clone + 'a,
{
    let mut tabs = SegmentedControl::new().flat().sm();

    for (index, panel) in panels.iter().enumerate() {
        let tab = BottomHeaderTab::from(panel);
        let active = index == active_index;
        let mut item = SegmentedItem::new(tab.label)
            .selected(active)
            .disabled(tab.disabled)
            .tooltip_maybe(tab.tooltip.clone());
        if let Some(icon) = tab.icon {
            item = item.icon(icon);
        }
        if let Some(badge) = tab.badge {
            item = item.badge(badge);
        }
        if let Some(status) = tab.status {
            item = item.status(status);
        }
        let event = WorkbenchPanelEvent::Selected {
            region,
            panel_id: tab.panel_id,
        };
        tabs = tabs.push(item.on_press_maybe((!tab.disabled).then(|| mapper(event))));
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
    let controls = PanelHeaderBar {
        region,
        panel_id: id,
        title: Cow::Borrowed(""),
        icon: None,
        badge: None,
        status: None,
        actions,
        collapsible,
        restorable: restorable && show_restore,
        maximizable,
        closable,
    }
    .view(mapper);
    let header = row![Element::from(tabs), controls]
        .spacing(nive_ui::theme::spacing().xs)
        .align_y(Alignment::Center);
    Panel::new(content)
        .header(header)
        .role(SurfaceRole::Panel)
        .fill()
        .into()
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
