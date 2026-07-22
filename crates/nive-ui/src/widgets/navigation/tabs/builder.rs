use std::{borrow::Cow, rc::Rc};

use iced::{widget::Space, Length};

use super::{
    CloseCallback, ContextCallback, ReorderCallback, SelectCallback, TabBar, TabCloseRequest,
    TabDrop, TabItem, TabTearOff, TearOffCallback,
};
use crate::interaction::ContextRequest;
use crate::theme::{ControlSize, SurfaceRole};
use crate::widgets::primitives::IconRole;

impl<'a, Id, Message> TabBar<'a, Id, Message>
where
    Id: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    /// Builds a controlled tab bar with the active tab id.
    pub fn new(active: impl Into<Option<Id>>) -> Self {
        Self {
            active: active.into(),
            tabs: Vec::new(),
            size: ControlSize::Sm,
            role: SurfaceRole::Chrome,
            active_role: SurfaceRole::Canvas,
            width: None,
            on_select: None,
            on_close_request: None,
            on_context: None,
            on_reorder: None,
            on_tear_off: None,
            overlay_content: Space::new().into(),
        }
    }

    /// Replaces the app-owned active tab id.
    pub fn active(mut self, active: impl Into<Option<Id>>) -> Self {
        self.active = active.into();
        self
    }

    /// Replaces all tabs from an iterator.
    pub fn tabs(mut self, tabs: impl IntoIterator<Item = TabItem<'a, Id>>) -> Self {
        self.tabs = tabs.into_iter().collect();
        self
    }

    /// Adds one tab as a small-builder convenience.
    pub fn push(mut self, tab: TabItem<'a, Id>) -> Self {
        self.tabs.push(tab);
        self
    }

    /// Adds one tab as a small-builder convenience.
    pub fn tab(self, tab: TabItem<'a, Id>) -> Self {
        self.push(tab)
    }

    /// Maps tab selection into app messages.
    pub fn on_select(mut self, f: impl Fn(Id) -> Message + 'a) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }

    /// Conditionally maps tab selection into app messages.
    pub fn on_select_maybe(mut self, f: Option<impl Fn(Id) -> Message + 'a>) -> Self {
        self.on_select = f.map(|f| Rc::new(f) as SelectCallback<'a, Id, Message>);
        self
    }

    /// Maps close requests into app messages.
    ///
    /// Close affordances and middle-click close are disabled when this callback
    /// is absent, even if items are marked [`TabItem::closable`].
    pub fn on_close_request(mut self, f: impl Fn(TabCloseRequest<Id>) -> Message + 'a) -> Self {
        self.on_close_request = Some(Box::new(f));
        self
    }

    /// Conditionally maps close requests into app messages.
    pub fn on_close_request_maybe(
        mut self,
        f: Option<impl Fn(TabCloseRequest<Id>) -> Message + 'a>,
    ) -> Self {
        self.on_close_request = f.map(|f| Box::new(f) as CloseCallback<'a, Id, Message>);
        self
    }

    /// Maps context requests into app messages.
    pub fn on_context(mut self, f: impl Fn(ContextRequest<Id>) -> Message + 'a) -> Self {
        self.on_context = Some(Box::new(f));
        self
    }

    /// Conditionally maps context requests into app messages.
    pub fn on_context_maybe(
        mut self,
        f: Option<impl Fn(ContextRequest<Id>) -> Message + 'a>,
    ) -> Self {
        self.on_context = f.map(|f| Box::new(f) as ContextCallback<'a, Id, Message>);
        self
    }

    /// Maps legal reorder drops into app messages.
    pub fn on_reorder(mut self, f: impl Fn(TabDrop<Id>) -> Message + 'a) -> Self {
        self.on_reorder = Some(Box::new(f));
        self
    }

    /// Conditionally maps legal reorder drops into app messages.
    pub fn on_reorder_maybe(mut self, f: Option<impl Fn(TabDrop<Id>) -> Message + 'a>) -> Self {
        self.on_reorder = f.map(|f| Box::new(f) as ReorderCallback<'a, Id, Message>);
        self
    }

    /// Maps tear-off requests into app messages.
    ///
    /// The position is source-window relative. Spawning or positioning a new
    /// window remains app/runtime work.
    pub fn on_tear_off(mut self, f: impl Fn(TabTearOff<Id>) -> Message + 'a) -> Self {
        self.on_tear_off = Some(Box::new(f));
        self
    }

    /// Conditionally maps tear-off requests into app messages.
    pub fn on_tear_off_maybe(mut self, f: Option<impl Fn(TabTearOff<Id>) -> Message + 'a>) -> Self {
        self.on_tear_off = f.map(|f| Box::new(f) as TearOffCallback<'a, Id, Message>);
        self
    }

    /// Sets the control size.
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

    /// Sets the surface role.
    pub fn role(mut self, role: SurfaceRole) -> Self {
        self.role = role;
        self
    }

    /// Sets the adjacent-content surface painted inside the active tab.
    ///
    /// The default is [`SurfaceRole::Canvas`]. The paint remains contained by
    /// the tab and does not create another host surface or structural seam.
    pub fn active_role(mut self, role: SurfaceRole) -> Self {
        self.active_role = role;
        self
    }

    crate::impl_layout_builders!(width_opt, fill_width_opt, shrink_width_opt);
}

impl<'a, Id> TabItem<'a, Id> {
    /// Creates tab data with required identity and label.
    pub fn new(id: Id, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id,
            label: label.into(),
            icon: None,
            dirty: false,
            pinned: false,
            closable: false,
            disabled: false,
            tooltip: None,
        }
    }

    /// Returns the tab id.
    pub fn id(&self) -> &Id {
        &self.id
    }

    /// Returns the tab label.
    pub fn label(&self) -> &str {
        self.label.as_ref()
    }

    pub fn icon(mut self, icon: IconRole) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    pub fn pinned(mut self, pinned: bool) -> Self {
        self.pinned = pinned;
        self
    }

    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
}
