use std::rc::Rc;

use iced::widget::{column, container, row, Column, Row, Space};
use iced::{Length, Padding};
use nive_ui::theme::{self, ControlSize, SurfaceRole};
use nive_ui::widgets::{Panel, SplitCollapse, SplitResize, SplitStack, SplitStackPane, Toolbar};
use nive_ui::Element;

use crate::documents::{DocumentArea, WorkbenchDocument, WorkbenchDocumentEvent};
use crate::layout::{
    MaximizedPanel, WorkbenchLayoutChange, WorkbenchLayoutState, WorkbenchPaneSizes,
    WorkbenchRegion,
};
use crate::layout_probe;
use crate::panels::{
    panel_host_with_size, PanelHostMode, WorkbenchPanel, WorkbenchPanelEvent,
    WorkbenchPanelHostState,
};
use crate::status::StatusBar;

type EventMapper<'a, DocumentId, PanelId, ActionId, Message> =
    Rc<dyn Fn(WorkbenchEvent<DocumentId, PanelId, ActionId>) -> Message + 'a>;

/// Top-level workbench event.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum WorkbenchEvent<DocumentId, PanelId, ActionId> {
    /// Layout state event.
    Layout(WorkbenchLayoutChange<DocumentId, PanelId>),
    /// Document-area event.
    Document(WorkbenchDocumentEvent<DocumentId>),
    /// Panel event.
    Panel(WorkbenchPanelEvent<PanelId, ActionId>),
}

/// Non-persisted minimum sizes for expanded workbench regions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorkbenchPaneConstraints {
    left: f32,
    center: f32,
    right: f32,
    upper: f32,
    bottom: f32,
}

impl WorkbenchPaneConstraints {
    /// Creates normalized logical-pixel minima for every shell split.
    pub fn new(left: f32, center: f32, right: f32, upper: f32, bottom: f32) -> Self {
        Self {
            left: pane_minimum(left),
            center: pane_minimum(center),
            right: pane_minimum(right),
            upper: pane_minimum(upper),
            bottom: pane_minimum(bottom),
        }
    }

    pub const fn left(self) -> f32 {
        self.left
    }
    pub const fn center(self) -> f32 {
        self.center
    }
    pub const fn right(self) -> f32 {
        self.right
    }
    pub const fn upper(self) -> f32 {
        self.upper
    }
    pub const fn bottom(self) -> f32 {
        self.bottom
    }

    pub fn left_min(mut self, minimum: f32) -> Self {
        self.left = pane_minimum(minimum);
        self
    }
    pub fn center_min(mut self, minimum: f32) -> Self {
        self.center = pane_minimum(minimum);
        self
    }
    pub fn right_min(mut self, minimum: f32) -> Self {
        self.right = pane_minimum(minimum);
        self
    }
    pub fn upper_min(mut self, minimum: f32) -> Self {
        self.upper = pane_minimum(minimum);
        self
    }
    pub fn bottom_min(mut self, minimum: f32) -> Self {
        self.bottom = pane_minimum(minimum);
        self
    }
}

impl Default for WorkbenchPaneConstraints {
    fn default() -> Self {
        Self::new(160.0, 240.0, 160.0, 160.0, 96.0)
    }
}

fn pane_minimum(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

impl<DocumentId, PanelId, ActionId> WorkbenchEvent<DocumentId, PanelId, ActionId>
where
    DocumentId: Clone,
    PanelId: Clone + PartialEq,
{
    /// Applies built-in shell view-state transitions to app-owned layout state.
    ///
    /// Domain side effects remain app-owned. Panel app actions, close requests,
    /// document close requests, context requests, and tear-off requests are
    /// intentionally not handled by this helper.
    pub fn apply_to(&self, state: &mut WorkbenchLayoutState<DocumentId, PanelId>) {
        match self {
            Self::Layout(WorkbenchLayoutChange::RegionResized { region, size }) => {
                state.set_region_size(*region, *size);
            }
            Self::Layout(WorkbenchLayoutChange::RegionCollapsed {
                region,
                restore_size,
            }) => {
                if let Some(size) = restore_size {
                    state.set_region_size(*region, *size);
                }
                state.collapse_region(*region);
            }
            Self::Layout(_) => {}
            Self::Document(WorkbenchDocumentEvent::Select(document_id)) => {
                state.set_active_document(Some(document_id.clone()));
            }
            Self::Document(_) => {}
            Self::Panel(WorkbenchPanelEvent::Selected { region, panel_id }) => {
                state.set_active_panel(*region, panel_id.clone());
            }
            Self::Panel(WorkbenchPanelEvent::RestoreRequested { region, panel_id }) => {
                // Reaching for another region's rail leaves the maximized view;
                // without this the rail would render but do nothing.
                state.restore_maximized();
                state.restore_region(*region);
                state.set_active_panel(*region, panel_id.clone());
            }
            Self::Panel(WorkbenchPanelEvent::CollapseRequested { region, .. }) => {
                state.collapse_region(*region);
            }
            Self::Panel(WorkbenchPanelEvent::MaximizeRequested { region, panel_id }) => {
                // Snapshot first, then make the maximized panel the active one so
                // the region shows it and its rail can swap to a sibling.
                state.maximize_panel(*region, panel_id.clone());
                state.set_active_panel(*region, panel_id.clone());
            }
            Self::Panel(WorkbenchPanelEvent::PanelRestoreRequested { .. }) => {
                state.restore_maximized();
            }
            Self::Panel(WorkbenchPanelEvent::Action { .. })
            | Self::Panel(WorkbenchPanelEvent::CloseRequested { .. }) => {}
        }
    }
}

/// Root workbench builder.
pub struct WorkbenchShell<'a, DocumentId, PanelId, ActionId, Message> {
    toolbar: Option<Toolbar<'a, Message>>,
    left_panels: Vec<WorkbenchPanel<'a, PanelId, ActionId, Message>>,
    documents: Vec<WorkbenchDocument<'a, DocumentId>>,
    document_content: Option<Element<'a, Message>>,
    right_panels: Vec<WorkbenchPanel<'a, PanelId, ActionId, Message>>,
    bottom_panels: Vec<WorkbenchPanel<'a, PanelId, ActionId, Message>>,
    status_bar: Option<StatusBar<'a>>,
    chrome_size: ControlSize,
    pane_constraints: WorkbenchPaneConstraints,
    state: WorkbenchLayoutState<DocumentId, PanelId>,
    on_event: EventMapper<'a, DocumentId, PanelId, ActionId, Message>,
}

/// Alias for the root workbench builder.
pub type Workbench<'a, DocumentId, PanelId, ActionId, Message> =
    WorkbenchShell<'a, DocumentId, PanelId, ActionId, Message>;

impl<'a, DocumentId, PanelId, ActionId, Message>
    WorkbenchShell<'a, DocumentId, PanelId, ActionId, Message>
{
    /// Builds an empty workbench shell with the required event mapper.
    pub fn new(
        state: WorkbenchLayoutState<DocumentId, PanelId>,
        on_event: impl Fn(WorkbenchEvent<DocumentId, PanelId, ActionId>) -> Message + 'a,
    ) -> Self {
        Self {
            toolbar: None,
            left_panels: Vec::new(),
            documents: Vec::new(),
            document_content: None,
            right_panels: Vec::new(),
            bottom_panels: Vec::new(),
            status_bar: None,
            chrome_size: ControlSize::Sm,
            pane_constraints: WorkbenchPaneConstraints::default(),
            state,
            on_event: Rc::new(on_event),
        }
    }

    /// Sets the shared control size for framework-managed workbench chrome.
    ///
    /// The default is [`ControlSize::Sm`]. The selected size is applied at
    /// render time to the typed toolbar, status bar, document tabs, side rails,
    /// panel headers, bottom selector, and split panes. It is independent from
    /// the global [`nive_ui::theme::ThemeDensity`] setting and is not persisted
    /// in layout state.
    pub fn chrome_size(mut self, size: ControlSize) -> Self {
        self.chrome_size = size;
        self
    }

    /// Sets non-persisted minimum sizes for expanded side and bottom regions.
    pub fn pane_constraints(mut self, constraints: WorkbenchPaneConstraints) -> Self {
        self.pane_constraints = constraints;
        self
    }

    /// Sets typed toolbar content.
    ///
    /// The shell retains the [`Toolbar`] until rendering, then applies its
    /// final [`Self::chrome_size`] value. That value takes precedence over a
    /// size previously set on `toolbar`, regardless of builder order.
    pub fn toolbar(mut self, toolbar: Toolbar<'a, Message>) -> Self {
        self.toolbar = Some(toolbar);
        self
    }

    /// Sets left panels.
    pub fn left_panels(
        mut self,
        panels: impl IntoIterator<Item = WorkbenchPanel<'a, PanelId, ActionId, Message>>,
    ) -> Self {
        self.left_panels = panels.into_iter().collect();
        self
    }

    /// Sets documents.
    pub fn documents(
        mut self,
        documents: impl IntoIterator<Item = WorkbenchDocument<'a, DocumentId>>,
    ) -> Self {
        self.documents = documents.into_iter().collect();
        self
    }

    /// Sets document content below the document tabs.
    pub fn document_content(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.document_content = Some(content.into());
        self
    }

    /// Sets right panels.
    pub fn right_panels(
        mut self,
        panels: impl IntoIterator<Item = WorkbenchPanel<'a, PanelId, ActionId, Message>>,
    ) -> Self {
        self.right_panels = panels.into_iter().collect();
        self
    }

    /// Sets bottom panels.
    pub fn bottom_panels(
        mut self,
        panels: impl IntoIterator<Item = WorkbenchPanel<'a, PanelId, ActionId, Message>>,
    ) -> Self {
        self.bottom_panels = panels.into_iter().collect();
        self
    }

    /// Sets a typed workbench status bar.
    ///
    /// The shell retains the [`StatusBar`] until rendering so it can apply the
    /// final shared chrome size. Arbitrary status elements are intentionally
    /// not accepted by the shell API.
    pub fn status(mut self, status: StatusBar<'a>) -> Self {
        self.status_bar = Some(status);
        self
    }
}

impl<'a, DocumentId, PanelId, ActionId, Message>
    WorkbenchShell<'a, DocumentId, PanelId, ActionId, Message>
where
    DocumentId: Clone + Eq + 'static,
    PanelId: Clone + Eq + 'a,
    ActionId: Clone + 'a,
    Message: Clone + 'a,
{
    /// Renders the fixed-region shell.
    pub fn view(mut self) -> Element<'a, Message> {
        let chrome_size = self.chrome_size;
        let mut root = Column::new()
            .spacing(0.0)
            .width(Length::Fill)
            .height(Length::Fill);

        let toolbar = self.toolbar.take();
        let status_bar = self.status_bar.take();
        if let Some(toolbar) = toolbar {
            // `Toolbar` owns its Chrome surface, inset, bottom hairline, and
            // horizontal overflow. The shell only supplies final size/width.
            let toolbar = toolbar.size(chrome_size).fill_width();
            root = root.push(layout_probe::probe(
                "toolbar",
                container(toolbar)
                    .padding(Padding::ZERO)
                    .width(Length::Fill),
            ));
        }

        let body = layout_probe::probe("body", self.body());
        root = root.push(body);

        if let Some(status_bar) = status_bar {
            root = root.push(layout_probe::probe(
                "status",
                status_bar.view_with_size(chrome_size),
            ));
        }

        root.into()
    }

    fn body(&mut self) -> Element<'a, Message> {
        let body = match self.state.maximized().cloned() {
            Some(maximized) if self.hosts_panel(&maximized) => self.maximized_body(maximized),
            _ => {
                let center = self.center_region();
                let center_with_sides = self.with_side_regions(center);
                self.with_bottom_region(center_with_sides)
            }
        };

        container(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::surface::style(SurfaceRole::App))
            .into()
    }

    /// Gives the maximized region the whole content area.
    ///
    /// Every other region still contributes its rail, so the panel list of both
    /// sides stays reachable instead of the shell collapsing to one panel.
    fn maximized_body(&mut self, maximized: MaximizedPanel<PanelId>) -> Element<'a, Message> {
        let left = self.maximized_region(WorkbenchRegion::Left, &maximized);
        let right = self.maximized_region(WorkbenchRegion::Right, &maximized);
        let bottom = self.maximized_region(WorkbenchRegion::Bottom, &maximized);

        [left, bottom, right]
            .into_iter()
            .flatten()
            .fold(Row::new().height(Length::Fill), Row::push)
            .into()
    }

    /// Renders one region while `maximized` holds the content area.
    ///
    /// The maximized region fills; the others shrink to their rail. The bottom
    /// region has no rail, so it only appears when it is the maximized one.
    fn maximized_region(
        &mut self,
        region: WorkbenchRegion,
        maximized: &MaximizedPanel<PanelId>,
    ) -> Option<Element<'a, Message>> {
        let panels = match region {
            WorkbenchRegion::Left => std::mem::take(&mut self.left_panels),
            WorkbenchRegion::Right => std::mem::take(&mut self.right_panels),
            _ => std::mem::take(&mut self.bottom_panels),
        };
        let is_maximized = maximized.region == region;
        if panels.is_empty() || (!is_maximized && region == WorkbenchRegion::Bottom) {
            return None;
        }

        let mut state = WorkbenchPanelHostState::new(region)
            // Collapsing out of a maximized region has no meaningful target, so
            // only the other regions keep the rail toggle.
            .collapse_on_active_click(!is_maximized)
            .mode(if is_maximized {
                PanelHostMode::Maximized
            } else {
                PanelHostMode::Collapsed
            });
        if let Some(active) = self.state.active_panel(region).cloned() {
            state = state.active_panel(active);
        }

        let host = panel_host_with_size(state, panels, self.panel_mapper(), self.chrome_size);
        let probe = if is_maximized {
            layout_probe::probe("maximized_pane", host)
        } else {
            host
        };

        Some(probe)
    }

    fn hosts_panel(&self, maximized: &MaximizedPanel<PanelId>) -> bool {
        let panels = match maximized.region {
            WorkbenchRegion::Left => &self.left_panels,
            WorkbenchRegion::Right => &self.right_panels,
            WorkbenchRegion::Bottom => &self.bottom_panels,
            _ => return false,
        };

        panels.iter().any(|panel| panel.id() == &maximized.panel_id)
    }

    fn center_region(&mut self) -> Element<'a, Message> {
        let active = self.state.active_document().cloned();
        let tabs = if self.documents.is_empty() {
            Space::new().height(Length::Shrink).into()
        } else {
            DocumentArea::new(active, std::mem::take(&mut self.documents))
                .on_event(self.document_mapper())
                .view_with_size(self.chrome_size)
        };

        let content = self
            .document_content
            .take()
            .unwrap_or_else(|| Space::new().width(Length::Fill).height(Length::Fill).into());
        let content = layout_probe::probe("document_content", content);

        Panel::new(column![tabs, content].height(Length::Fill))
            .role(SurfaceRole::Canvas)
            .fill()
            .into()
    }

    /// Lays the left, center, and right regions out on one horizontal axis.
    ///
    /// One container holds all three, so a region divider is never able to reach
    /// past its two neighbours. Collapsed regions leave the stack entirely and
    /// render as a rail beside it, which stays the semantic zero-size path.
    fn with_side_regions(&mut self, center: Element<'a, Message>) -> Element<'a, Message> {
        let sizes = self.state.pane_sizes();
        let left_split =
            !self.left_panels.is_empty() && !self.state.is_collapsed(WorkbenchRegion::Left);
        let right_split =
            !self.right_panels.is_empty() && !self.state.is_collapsed(WorkbenchRegion::Right);

        let mut regions = Vec::with_capacity(3);
        let mut stack = SplitStack::horizontal().size(self.chrome_size);

        let left_rail = (!self.left_panels.is_empty() && !left_split).then(|| {
            let panels = std::mem::take(&mut self.left_panels);
            self.panel_region(WorkbenchRegion::Left, panels, true)
        });

        if left_split {
            let panels = std::mem::take(&mut self.left_panels);
            let left = self.panel_region(WorkbenchRegion::Left, panels, false);
            regions.push(WorkbenchRegion::Left);
            stack = stack.pane(
                SplitStackPane::fixed(layout_probe::probe("left_pane", left), sizes.left)
                    .minimum(self.pane_constraints.left)
                    .collapsible(true),
            );
        }

        regions.push(WorkbenchRegion::Center);
        stack = stack.pane(
            SplitStackPane::fill(layout_probe::probe("center_pane", center))
                .minimum(self.pane_constraints.center),
        );

        let right_rail = (!self.right_panels.is_empty() && !right_split).then(|| {
            let panels = std::mem::take(&mut self.right_panels);
            self.panel_region(WorkbenchRegion::Right, panels, true)
        });

        if right_split {
            let panels = std::mem::take(&mut self.right_panels);
            let right = self.panel_region(WorkbenchRegion::Right, panels, false);
            regions.push(WorkbenchRegion::Right);
            stack = stack.pane(
                SplitStackPane::fixed(layout_probe::probe("right_pane", right), sizes.right)
                    .minimum(self.pane_constraints.right)
                    .collapsible(true),
            );
        }

        let mut content: Element<'a, Message> = stack
            .on_resize(self.region_resize_mapper(regions.clone()))
            .on_collapse(self.region_collapse_mapper(regions))
            .into();

        if let Some(left_rail) = left_rail {
            content = row![left_rail, content].height(Length::Fill).into();
        }
        if let Some(right_rail) = right_rail {
            content = row![content, right_rail].height(Length::Fill).into();
        }

        content
    }

    /// Splits the bottom region off on the vertical axis.
    ///
    /// A separate axis carries no coupling with the horizontal regions, which is
    /// the alternating arrangement reference workbench grids use.
    fn with_bottom_region(&mut self, content: Element<'a, Message>) -> Element<'a, Message> {
        if self.bottom_panels.is_empty() || self.state.is_collapsed(WorkbenchRegion::Bottom) {
            return content;
        }

        let panels = std::mem::take(&mut self.bottom_panels);
        let bottom = self.panel_region(WorkbenchRegion::Bottom, panels, false);

        SplitStack::vertical()
            .size(self.chrome_size)
            .pane(
                SplitStackPane::fill(layout_probe::probe("upper_pane", content))
                    .minimum(self.pane_constraints.upper),
            )
            .pane(
                SplitStackPane::fixed(
                    layout_probe::probe("bottom_pane", bottom),
                    self.state.pane_sizes().bottom,
                )
                .minimum(self.pane_constraints.bottom),
            )
            .on_resize(
                self.region_resize_mapper(vec![WorkbenchRegion::Center, WorkbenchRegion::Bottom]),
            )
            .into()
    }

    fn panel_region(
        &self,
        region: WorkbenchRegion,
        panels: Vec<WorkbenchPanel<'a, PanelId, ActionId, Message>>,
        collapsed: bool,
    ) -> Element<'a, Message> {
        // Clicking a rail item toggles its region: the active one folds it away,
        // any other selects that panel, and a collapsed region comes back.
        let mut state = WorkbenchPanelHostState::new(region)
            .collapsed(collapsed)
            .collapse_on_active_click(true);
        if let Some(active) = self.state.active_panel(region).cloned() {
            state = state.active_panel(active);
        }

        panel_host_with_size(state, panels, self.panel_mapper(), self.chrome_size)
    }

    /// Maps a stack resize onto the region whose size the app stores.
    ///
    /// `regions` names the pane at each index, so the divider index resolves to
    /// the sized region on either side of it. The centre region has no stored
    /// size and is skipped.
    fn region_resize_mapper(
        &self,
        regions: Vec<WorkbenchRegion>,
    ) -> impl Fn(SplitResize) -> Message + 'a {
        let mapper = self.on_event.clone();
        move |resize| {
            let (region, size) = match regions.get(resize.divider) {
                Some(WorkbenchRegion::Center) | None => {
                    (regions.get(resize.divider + 1).copied(), resize.trailing)
                }
                Some(region) => (Some(*region), resize.leading),
            };

            mapper(WorkbenchEvent::Layout(
                WorkbenchLayoutChange::RegionResized {
                    region: region.unwrap_or(WorkbenchRegion::Center),
                    size: WorkbenchPaneSizes::normalize(size),
                },
            ))
        }
    }

    /// Maps a stack collapse onto the region that should fold away.
    ///
    /// The reported restore length is the region's width before the drag, so
    /// restoring it later returns it to that width rather than the minimum the
    /// same drag squeezed it to.
    fn region_collapse_mapper(
        &self,
        regions: Vec<WorkbenchRegion>,
    ) -> impl Fn(SplitCollapse) -> Message + 'a {
        let mapper = self.on_event.clone();
        move |collapse| {
            let region = regions
                .get(collapse.pane)
                .copied()
                .unwrap_or(WorkbenchRegion::Center);

            mapper(WorkbenchEvent::Layout(
                WorkbenchLayoutChange::RegionCollapsed {
                    region,
                    restore_size: Some(WorkbenchPaneSizes::normalize(collapse.restore)),
                },
            ))
        }
    }

    fn document_mapper(&self) -> impl Fn(WorkbenchDocumentEvent<DocumentId>) -> Message + 'a {
        let mapper = self.on_event.clone();
        move |event| mapper(WorkbenchEvent::Document(event))
    }

    fn panel_mapper(&self) -> impl Fn(WorkbenchPanelEvent<PanelId, ActionId>) -> Message + 'a {
        let mapper = self.on_event.clone();
        move |event| mapper(WorkbenchEvent::Panel(event))
    }
}

#[cfg(test)]
mod tests;
