use std::borrow::Cow;

use iced::Point;
use nive_ui::interaction::{ContextRequest, ContextTarget};
use nive_ui::theme::ControlSize;
use nive_ui::widgets::{
    TabBar, TabCloseRequest, TabCloseTrigger, TabDrop, TabDropTarget, TabItem, TabTearOff,
};
use nive_ui::{Element, IconRole};

use crate::layout_probe;

/// Metadata for one controlled workbench document tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkbenchDocument<'a, Id> {
    id: Id,
    label: Cow<'a, str>,
    icon: Option<IconRole>,
    dirty: bool,
    pinned: bool,
    closable: bool,
    disabled: bool,
    tooltip: Option<Cow<'a, str>>,
}

/// Controlled document-tab area.
///
/// If no mapper is configured with [`DocumentArea::on_event`], the document
/// tabs render from metadata but do not emit interactive workbench messages.
pub struct DocumentArea<'a, Id, Message> {
    active: Option<Id>,
    documents: Vec<WorkbenchDocument<'a, Id>>,
    on_event: Option<Box<dyn Fn(WorkbenchDocumentEvent<Id>) -> Message + 'a>>,
}

/// Alias for the controlled document collection wrapper.
pub type WorkbenchDocuments<'a, Id, Message> = DocumentArea<'a, Id, Message>;

/// Semantic document-area event emitted by the workbench.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum WorkbenchDocumentEvent<Id> {
    /// A document should become active.
    Select(Id),
    /// A document close was requested.
    CloseRequested {
        /// Affected document.
        id: Id,
        /// Close request trigger.
        trigger: TabCloseTrigger,
    },
    /// A document was dropped for reorder.
    Reorder {
        /// Dragged document ids in visible order.
        dragged: Vec<Id>,
        /// Requested drop target.
        target: WorkbenchDocumentDropTarget<Id>,
    },
    /// A context menu was requested.
    ContextRequested {
        /// Target document, or `None` for empty area.
        id: Option<Id>,
        /// Pointer or focus position from the underlying tab widget.
        position: WorkbenchDocumentContextPosition,
    },
    /// A document tear-off was requested.
    TearOffRequested {
        /// Dragged document ids in visible order.
        dragged: Vec<Id>,
        /// Source-window-relative release position.
        position: Point,
    },
}

/// Document reorder target.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum WorkbenchDocumentDropTarget<Id> {
    /// Insert before the target document.
    Before(Id),
    /// Insert after the target document.
    After(Id),
    /// Unknown future target from the underlying tab widget.
    Unknown,
}

/// Context request position represented without exposing app-domain state.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum WorkbenchDocumentContextPosition {
    /// Pointer position in widget coordinates.
    Pointer(Point),
    /// Focused document position.
    FocusedItem,
}

impl<'a, Id> WorkbenchDocument<'a, Id> {
    /// Creates document metadata with required id and label.
    pub fn new(id: Id, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id,
            label: label.into(),
            icon: None,
            dirty: false,
            pinned: false,
            closable: true,
            disabled: false,
            tooltip: None,
        }
    }

    /// Returns the document id.
    pub fn id(&self) -> &Id {
        &self.id
    }

    /// Returns the visible document label.
    pub fn label(&self) -> &str {
        self.label.as_ref()
    }

    /// Returns configured icon role.
    pub const fn icon_role(&self) -> Option<IconRole> {
        self.icon
    }

    /// Returns whether the document is dirty.
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Returns whether the document is pinned.
    pub const fn is_pinned(&self) -> bool {
        self.pinned
    }

    /// Returns whether close affordances are allowed.
    pub const fn is_closable(&self) -> bool {
        self.closable
    }

    /// Returns whether the document tab is disabled.
    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Returns the optional tooltip.
    pub fn tooltip_text(&self) -> Option<&str> {
        self.tooltip.as_deref()
    }

    /// Sets the document icon.
    pub fn icon(mut self, icon: IconRole) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets dirty state.
    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    /// Sets pinned state.
    pub fn pinned(mut self, pinned: bool) -> Self {
        self.pinned = pinned;
        self
    }

    /// Sets close policy.
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Sets disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the document tooltip.
    pub fn tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Converts workbench metadata into a `nive-ui` tab item.
    pub fn into_tab_item(self) -> TabItem<'a, Id> {
        let mut item = TabItem::new(self.id, self.label)
            .dirty(self.dirty)
            .pinned(self.pinned)
            .closable(self.closable)
            .disabled(self.disabled);

        if let Some(icon) = self.icon {
            item = item.icon(icon);
        }

        if let Some(tooltip) = self.tooltip {
            item = item.tooltip(tooltip);
        }

        item
    }
}

impl<'a, Id, Message> DocumentArea<'a, Id, Message>
where
    Id: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    /// Builds a document area from active state and document metadata.
    pub fn new(
        active: impl Into<Option<Id>>,
        documents: impl IntoIterator<Item = WorkbenchDocument<'a, Id>>,
    ) -> Self {
        Self {
            active: active.into(),
            documents: documents.into_iter().collect(),
            on_event: None,
        }
    }

    /// Maps document events into app messages.
    pub fn on_event(mut self, mapper: impl Fn(WorkbenchDocumentEvent<Id>) -> Message + 'a) -> Self {
        self.on_event = Some(Box::new(mapper));
        self
    }

    /// Renders the document tab area.
    pub fn view(self) -> Element<'a, Message> {
        self.view_with_size(ControlSize::Sm)
    }

    pub(crate) fn view_with_size(self, size: ControlSize) -> Element<'a, Message> {
        let mut bar = TabBar::new(self.active).size(size).tabs(
            self.documents
                .into_iter()
                .map(WorkbenchDocument::into_tab_item),
        );

        if let Some(mapper) = self.on_event {
            let mapper = std::rc::Rc::new(mapper);
            let select_mapper = mapper.clone();
            bar = bar.on_select(move |id| select_mapper(WorkbenchDocumentEvent::Select(id)));

            let close_mapper = mapper.clone();
            bar = bar.on_close_request(move |request| close_mapper(map_close_request(request)));

            let reorder_mapper = mapper.clone();
            bar = bar.on_reorder(move |drop| reorder_mapper(map_drop(drop)));

            let context_mapper = mapper.clone();
            bar = bar.on_context(move |request| context_mapper(map_context_request(request)));

            let tear_off_mapper = mapper;
            bar = bar.on_tear_off(move |tear_off| tear_off_mapper(map_tear_off(tear_off)));
        }

        layout_probe::probe("document_tabs", Element::from(bar))
    }
}

fn map_close_request<Id>(request: TabCloseRequest<Id>) -> WorkbenchDocumentEvent<Id> {
    WorkbenchDocumentEvent::CloseRequested {
        id: request.id,
        trigger: request.trigger,
    }
}

fn map_drop<Id>(drop: TabDrop<Id>) -> WorkbenchDocumentEvent<Id> {
    let target = match drop.target {
        TabDropTarget::Before(id) => WorkbenchDocumentDropTarget::Before(id),
        TabDropTarget::After(id) => WorkbenchDocumentDropTarget::After(id),
        _ => WorkbenchDocumentDropTarget::Unknown,
    };

    WorkbenchDocumentEvent::Reorder {
        dragged: drop.payload.ids,
        target,
    }
}

fn map_context_request<Id>(request: ContextRequest<Id>) -> WorkbenchDocumentEvent<Id> {
    let id = match request.target {
        ContextTarget::Item(id) => Some(id),
        ContextTarget::Empty => None,
        _ => None,
    };
    let position = match request.position {
        nive_ui::interaction::ContextPosition::Pointer(point) => {
            WorkbenchDocumentContextPosition::Pointer(point)
        }
        nive_ui::interaction::ContextPosition::FocusedItem => {
            WorkbenchDocumentContextPosition::FocusedItem
        }
        _ => WorkbenchDocumentContextPosition::FocusedItem,
    };

    WorkbenchDocumentEvent::ContextRequested { id, position }
}

fn map_tear_off<Id>(tear_off: TabTearOff<Id>) -> WorkbenchDocumentEvent<Id> {
    WorkbenchDocumentEvent::TearOffRequested {
        dragged: tear_off.payload.ids,
        position: tear_off.position,
    }
}

#[cfg(test)]
mod tests;
