use std::rc::Rc;

use iced::widget::{column, container, row, scrollable, Column, Space};
use iced::{Length, Padding};
use nive_ui::interaction::Orientation;
use nive_ui::theme::{self, ControlSize, SurfaceRole};
use nive_ui::widgets::{Panel, SplitPane, Toolbar};
use nive_ui::Element;

use crate::documents::{DocumentArea, WorkbenchDocument, WorkbenchDocumentEvent};
use crate::layout::{WorkbenchLayoutChange, WorkbenchLayoutState, WorkbenchRegion};
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
            Self::Layout(WorkbenchLayoutChange::SplitRatioChanged { region, ratio }) => {
                state.set_split_ratio(*region, *ratio);
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
                state.restore_region(*region);
                state.set_active_panel(*region, panel_id.clone());
            }
            Self::Panel(WorkbenchPanelEvent::CollapseRequested { region, .. }) => {
                state.collapse_region(*region);
            }
            Self::Panel(WorkbenchPanelEvent::MaximizeRequested { region, panel_id }) => {
                state.maximize_panel(*region, panel_id.clone());
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
            let toolbar = scrollable(toolbar.size(chrome_size))
                .horizontal()
                .height(Length::Shrink)
                .width(Length::Fill);
            root = root.push(layout_probe::probe(
                "toolbar",
                container(toolbar)
                    .padding(Padding::ZERO)
                    .width(Length::Fill)
                    .style(theme::surface::style(SurfaceRole::Chrome)),
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
        if let Some(maximized) = self.state.maximized().cloned() {
            if let Some(panel) = self.take_panel(maximized.region, &maximized.panel_id) {
                let state = WorkbenchPanelHostState::new(maximized.region)
                    .active_panel(maximized.panel_id)
                    .mode(PanelHostMode::Maximized);
                return panel_host_with_size(state, [panel], self.panel_mapper(), self.chrome_size);
            }
        }

        let center = self.center_region();
        let center_with_sides = self.with_side_regions(center);
        let body = self.with_bottom_region(center_with_sides);

        container(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::surface::style(SurfaceRole::App))
            .into()
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

    fn with_side_regions(&mut self, center: Element<'a, Message>) -> Element<'a, Message> {
        let mut content = center;
        if !self.left_panels.is_empty() {
            let panels = std::mem::take(&mut self.left_panels);
            let left_collapsed = self.state.is_collapsed(WorkbenchRegion::Left);
            let left = self.panel_region(WorkbenchRegion::Left, panels, left_collapsed);
            content = if left_collapsed {
                row![left, content].height(Length::Fill).into()
            } else {
                SplitPane::new(
                    layout_probe::probe("left_split_leading", left),
                    layout_probe::probe("left_split_trailing", content),
                )
                .size(self.chrome_size)
                .ratio(self.state.split_ratios().left)
                .on_change(self.layout_ratio_mapper(WorkbenchRegion::Left))
                .into()
            };
        }

        if !self.right_panels.is_empty() {
            let panels = std::mem::take(&mut self.right_panels);
            let right_collapsed = self.state.is_collapsed(WorkbenchRegion::Right);
            let right = self.panel_region(WorkbenchRegion::Right, panels, right_collapsed);
            content = if right_collapsed {
                row![content, right].height(Length::Fill).into()
            } else {
                SplitPane::new(
                    layout_probe::probe("right_split_leading", content),
                    layout_probe::probe("right_split_trailing", right),
                )
                .size(self.chrome_size)
                .ratio(self.state.split_ratios().right)
                .on_change(self.layout_ratio_mapper(WorkbenchRegion::Right))
                .into()
            };
        }

        content
    }

    fn with_bottom_region(&mut self, content: Element<'a, Message>) -> Element<'a, Message> {
        if self.bottom_panels.is_empty() || self.state.is_collapsed(WorkbenchRegion::Bottom) {
            return content;
        }

        let panels = std::mem::take(&mut self.bottom_panels);
        let bottom = self.panel_region(WorkbenchRegion::Bottom, panels, false);
        SplitPane::new(
            layout_probe::probe("bottom_split_leading", content),
            layout_probe::probe("bottom_split_trailing", bottom),
        )
        .size(self.chrome_size)
        .orientation(Orientation::Vertical)
        .ratio(self.state.split_ratios().bottom)
        .on_change(self.layout_ratio_mapper(WorkbenchRegion::Bottom))
        .into()
    }

    fn panel_region(
        &self,
        region: WorkbenchRegion,
        panels: Vec<WorkbenchPanel<'a, PanelId, ActionId, Message>>,
        collapsed: bool,
    ) -> Element<'a, Message> {
        let mut state = WorkbenchPanelHostState::new(region).collapsed(collapsed);
        if let Some(active) = self.state.active_panel(region).cloned() {
            state = state.active_panel(active);
        }

        panel_host_with_size(state, panels, self.panel_mapper(), self.chrome_size)
    }

    fn take_panel(
        &mut self,
        region: WorkbenchRegion,
        panel_id: &PanelId,
    ) -> Option<WorkbenchPanel<'a, PanelId, ActionId, Message>> {
        let panels = match region {
            WorkbenchRegion::Left => &mut self.left_panels,
            WorkbenchRegion::Right => &mut self.right_panels,
            WorkbenchRegion::Bottom => &mut self.bottom_panels,
            WorkbenchRegion::Toolbar | WorkbenchRegion::Center | WorkbenchRegion::Status => {
                return None;
            }
        };
        let index = panels.iter().position(|panel| panel.id() == panel_id)?;
        Some(panels.remove(index))
    }

    fn layout_ratio_mapper(&self, region: WorkbenchRegion) -> impl Fn(f32) -> Message + 'a {
        let mapper = self.on_event.clone();
        move |ratio| {
            mapper(WorkbenchEvent::Layout(
                WorkbenchLayoutChange::SplitRatioChanged {
                    region,
                    ratio: crate::layout::WorkbenchSplitRatios::clamp(ratio),
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
