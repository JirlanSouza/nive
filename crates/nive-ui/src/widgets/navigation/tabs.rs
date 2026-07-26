//! Controlled document/view tab collection widget.
//!
//! `TabBar` is id-based and controlled: applications own tab order, active id,
//! dirty state, pin state, and close policy. The widget owns only ephemeral UI
//! state such as scroll offset, pointer gestures, the active drag session, and
//! the insertion-preview target.
//!
//! Tab identity is required at construction ([`TabItem::new(id, label)`]) and
//! flows through every interaction payload. Active tab state is app-owned:
//! feeding a new active id back to the widget reveals the active tab inside the
//! scroll viewport without emitting a domain message.
//!
//! Interaction intents are enabled by bar-level callbacks. Presence of a
//! callback enables the corresponding capability; absence disables it. Close
//! affordances render only when [`TabBar::on_close_request`] is present, even
//! if items are marked [`TabItem::closable`]. Reorder and tear-off only fire
//! when their respective callbacks are present.
//!
//! Pinned tabs render as a leading group while preserving relative order
//! within each partition. Reorder hit-testing runs [`crate::interaction::linear_insertion`]
//! per pinned or unpinned segment, so a pinned tab cannot enter the unpinned
//! area and vice versa.
//!
//! Overflow is scroll-based: the strip is wrapped in a horizontal clip and the
//! application of `scroll_offset` shifts the visible tab strip
//! left by that many pixels. Chevrons appear when measured content overflow is
//! detected and disappear again when no overflow remains. An all-tabs trigger
//! opens a canonical anchored Menu listing every `TabItem` (visible or not).
//!
//! Reorder and tear-off drives go through the shared gesture-driven
//! [`DragSession`]; TabBar is its first consumer. The dragged tab is tracked by
//! id, not by position index, so an app-applied reorder between `DragStarted`
//! and `DragReleased` cannot address the wrong tab. Tear-off payloads carry
//! the source-window-relative release position; spawning the new window
//! remains app/runtime work.
//!
//! ```

//! use nive_ui::widgets::{TabBar, TabItem};
//!
//! #[derive(Clone, Debug, PartialEq, Eq)]
//! enum TabId {
//!     Overview,
//!     Details,
//! }
//!
//! #[derive(Clone, Debug)]
//! enum Message {
//!     Select(TabId),
//! }
//!
//! let tabs = [TabId::Overview, TabId::Details];
//! let bar: nive_ui::Element<'_, Message> = TabBar::new(TabId::Overview)
//!     .tabs(tabs.iter().cloned().map(|id| {
//!         TabItem::new(id, "Tab")
//!     }))
//!     .on_select(Message::Select)
//!     .into();
//! ```
//!
//! [`DragSession`]: crate::interaction::dnd

use std::{borrow::Cow, cell::Cell, rc::Rc};

use iced::{Length, Point, Rectangle};

use crate::advanced::focus::FocusState;
use crate::interaction::dnd::DragSession;
use crate::interaction::{
    CollectionTransferPayload, ContextRequest, PointerGestureState, TransferOperation,
};
use crate::theme::{ControlSize, SurfaceRole};
use crate::Element;

use super::overflow::{Overflow, OverflowDirection};
use crate::IconRef;

mod builder;
mod geometry;
mod interaction;
mod render;
mod style;
mod widget;

#[cfg(test)]
mod tabs_tests;
#[cfg(test)]
mod widget_tests;

type SelectCallback<'a, Id, Message> = Rc<dyn Fn(Id) -> Message + 'a>;
type CloseCallback<'a, Id, Message> = Box<dyn Fn(TabCloseRequest<Id>) -> Message + 'a>;
type ContextCallback<'a, Id, Message> = Box<dyn Fn(ContextRequest<Id>) -> Message + 'a>;
type ReorderCallback<'a, Id, Message> = Box<dyn Fn(TabDrop<Id>) -> Message + 'a>;
type TearOffCallback<'a, Id, Message> = Box<dyn Fn(TabTearOff<Id>) -> Message + 'a>;

const TEAR_OFF_HYSTERESIS: f32 = 24.0;
const CHEVRON_SCROLL_STEP_FACTOR: f32 = 0.8;
const INSERTION_MARKER_WIDTH: f32 = 2.0;
const HIDDEN_AFFORDANCE_WIDTH: f32 = 0.1;
/// Controlled tab strip keyed by stable tab IDs.
pub struct TabBar<'a, Id, Message> {
    active: Option<Id>,
    tabs: Vec<TabItem<'a, Id>>,
    size: ControlSize,
    role: SurfaceRole,
    active_role: SurfaceRole,
    width: Option<Length>,
    on_select: Option<SelectCallback<'a, Id, Message>>,
    on_close_request: Option<CloseCallback<'a, Id, Message>>,
    on_context: Option<ContextCallback<'a, Id, Message>>,
    on_reorder: Option<ReorderCallback<'a, Id, Message>>,
    on_tear_off: Option<TearOffCallback<'a, Id, Message>>,
    /// Rebuilt canonical child retained so Iced can borrow its overlay.
    overlay_content: Element<'a, Message>,
}

/// Data for one tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabItem<'a, Id> {
    id: Id,
    label: Cow<'a, str>,
    icon: Option<IconRef>,
    dirty: bool,
    pinned: bool,
    closable: bool,
    disabled: bool,
    tooltip: Option<Cow<'a, str>>,
}

/// Close request emitted by `TabBar`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TabCloseRequest<Id> {
    /// Tab being requested for close.
    pub id: Id,
    /// Input that caused the request.
    pub trigger: TabCloseTrigger,
}

/// Input source for a close request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TabCloseTrigger {
    /// Visible close affordance.
    CloseButton,
    /// Middle pointer click.
    MiddleClick,
}

/// Reorder drop request emitted by `TabBar`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TabDrop<Id> {
    /// Singleton dragged-tab payload.
    pub payload: CollectionTransferPayload<Id>,
    /// Legal insertion target.
    pub target: TabDropTarget<Id>,
    /// Effective transfer operation.
    pub operation: TransferOperation,
}

/// Tab reorder insertion target.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TabDropTarget<Id> {
    /// Insert before the target tab.
    Before(Id),
    /// Insert after the target tab.
    After(Id),
}

/// Tear-off request emitted by `TabBar`.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct TabTearOff<Id> {
    /// Singleton dragged-tab payload.
    pub payload: CollectionTransferPayload<Id>,
    /// Release position in source-window coordinates.
    pub position: Point,
}

#[derive(Debug)]
struct TabBarState<Id: Clone + Eq + 'static> {
    overflow: Overflow,
    scroll_offset: f32,
    max_scroll: f32,
    content_width: f32,
    strip_width: f32,
    has_overflow: bool,
    last_active_id: Option<Id>,
    dragged_id: Option<Id>,
    insertion_target: Option<TabDropTarget<Id>>,
    tab_bounds: Vec<(Id, Rectangle, bool)>,
    close_bounds: Vec<(Id, Rectangle)>,
    hovered_id: Option<Id>,
    focused_id: Option<Id>,
    previous_focus_order: Vec<Id>,
    focus: FocusState,
    pressed_id: Option<Id>,
    invalid_target: bool,
    edge_scroll: Option<OverflowDirection>,
    last_redraw: Option<iced::time::Instant>,
    strip_bounds: Option<Rectangle>,
    left_chevron: Option<Rectangle>,
    right_chevron: Option<Rectangle>,
    all_tabs_button: Option<Rectangle>,
    menu_open: Rc<Cell<bool>>,
    gestures: PointerGestureState<TabRegion>,
    drag_session: DragSession<CollectionTransferPayload<Id>, (), TabDropTarget<Id>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TabRegion {
    Tab(usize),
    Close(usize),
    ChevronLeft,
    ChevronRight,
    AllTabsButton,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusMovement {
    Previous,
    Next,
    First,
    Last,
}

#[derive(Debug, Clone)]
struct DisplayedTab<'a, 'b, Id> {
    #[allow(dead_code)]
    original_index: usize,
    item: &'a TabItem<'b, Id>,
}

#[cfg(test)]
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct DisplayTab<Id> {
    original_index: usize,
    id: Id,
    pinned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AllTabsMenuEntry<'a, Id> {
    id: Id,
    label: Cow<'a, str>,
    icon: Option<IconRef>,
    active: bool,
    dirty: bool,
    pinned: bool,
    disabled: bool,
}

/// Internal messages produced by the canonical all-tabs Menu.
#[derive(Debug, Clone)]
enum MenuMessage<Id> {
    Open,
    Select(Id),
    Dismiss,
}

#[derive(Debug, Clone)]
struct HitGeometry<Id> {
    tab_bounds: Vec<(Id, Rectangle, bool)>,
    close_bounds: Vec<(Id, Rectangle)>,
    left_chevron: Option<Rectangle>,
    right_chevron: Option<Rectangle>,
    all_tabs_button: Option<Rectangle>,
    strip_bounds: Option<Rectangle>,
}

impl<Id> Default for HitGeometry<Id> {
    fn default() -> Self {
        Self {
            tab_bounds: Vec::new(),
            close_bounds: Vec::new(),
            left_chevron: None,
            right_chevron: None,
            all_tabs_button: None,
            strip_bounds: None,
        }
    }
}

impl<'a, Id, Message> Default for TabBar<'a, Id, Message>
where
    Id: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    fn default() -> Self {
        Self::new(None)
    }
}

impl<Id: Clone + Eq + 'static> Default for TabBarState<Id> {
    fn default() -> Self {
        Self {
            overflow: Overflow::default(),
            scroll_offset: 0.0,
            max_scroll: 0.0,
            content_width: 0.0,
            strip_width: 0.0,
            has_overflow: false,
            last_active_id: None,
            dragged_id: None,
            insertion_target: None,
            tab_bounds: Vec::new(),
            close_bounds: Vec::new(),
            hovered_id: None,
            focused_id: None,
            previous_focus_order: Vec::new(),
            focus: FocusState::default(),
            pressed_id: None,
            invalid_target: false,
            edge_scroll: None,
            last_redraw: None,
            strip_bounds: None,
            left_chevron: None,
            right_chevron: None,
            all_tabs_button: None,
            menu_open: Rc::new(Cell::new(false)),
            gestures: PointerGestureState::default(),
            drag_session: DragSession::default(),
        }
    }
}

struct TabBarFocus<'a, Id> {
    focus: &'a mut FocusState,
    pressed_id: &'a mut Option<Id>,
}

impl<'a, Id, Message> From<TabBar<'a, Id, Message>> for Element<'a, Message>
where
    Id: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    fn from(tab_bar: TabBar<'a, Id, Message>) -> Self {
        Element::new(tab_bar)
    }
}
