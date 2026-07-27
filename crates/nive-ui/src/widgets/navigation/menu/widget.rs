use iced::{Point, Rectangle};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

use crate::{
    advanced::focus::FocusState,
    theme::{choice::ChoicePersistentState, TypographyRole},
    widgets::overlays::anchored_overlay::{
        scroll::EnsureVisibleHandle, OverlayIdentity, OverlayNodeState,
    },
    Element,
};

use super::MENU_SEPARATOR_MARGIN;

mod branch;
mod helpers;
mod list;
mod state;
mod trailing;

const SEPARATOR_HEIGHT: f32 = 1.0 + MENU_SEPARATOR_MARGIN * 2.0;
const TYPEAHEAD_TIMEOUT: Duration = Duration::from_millis(700);
const SUBMENU_OPEN_DELAY: Duration = Duration::from_millis(200);
const SUBMENU_TRANSFER_GRACE: Duration = Duration::from_millis(300);

#[derive(Debug, Clone)]
pub(super) struct MenuBranchHandle {
    open: Rc<Cell<bool>>,
    pointer_inside: Rc<Cell<bool>>,
    child_bounds: Rc<Cell<Rectangle>>,
    identity: Rc<RefCell<OverlayIdentity>>,
}

/// One row's declared state, resolved once per entry and shared by both things
/// that consume it: the [`MenuSlot`] the list navigates and the row the reader
/// sees.
///
/// It exists so those two cannot disagree. They used to receive the same facts
/// as separate arguments built side by side, which is how a highlight flag once
/// reached a row style in the position meant for `destructive` and rendered a
/// suggestion as if it were dangerous.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MenuRowSpec {
    pub(super) persistent: ChoicePersistentState,
    /// Whether the row can be activated or opened. Drives navigation, the
    /// highlight, and the pointer cursor alike.
    pub(super) eligible: bool,
    /// Explicit `disabled(true)`, which is stronger than mere ineligibility: it
    /// dims the row, while a display-only or already committed row keeps its
    /// ordinary appearance.
    pub(super) disabled: bool,
    pub(super) destructive: bool,
}

impl MenuRowSpec {
    pub(super) fn selected(self) -> bool {
        !matches!(self.persistent, ChoicePersistentState::Unselected)
    }
}

#[derive(Debug, Clone)]
pub(super) struct MenuSlot<Message> {
    pub(super) eligible: bool,
    pub(super) separator: bool,
    activation: Option<Message>,
    label: Option<String>,
    trailing: Option<MenuTrailingMeasure>,
    persistent: ChoicePersistentState,
    disabled: bool,
    logical_focus: Option<Rc<Cell<bool>>>,
    branch: Option<MenuBranchHandle>,
}

#[derive(Debug, Clone)]
pub(super) enum MenuTrailingMeasure {
    Text(String, TypographyRole),
    Icon,
}

pub(super) struct MenuList<'a, Message> {
    content: Element<'a, Message>,
    slots: Vec<MenuSlot<Message>>,
    reserve_choice: bool,
    reserve_icon: bool,
    trailing_width: Rc<Cell<f32>>,
    root: bool,
    shared_focus_visible: Rc<Cell<bool>>,
    level_open: Option<Rc<Cell<bool>>>,
    level_pointer_inside: Option<Rc<Cell<bool>>>,
    ensure_visible: EnsureVisibleHandle,
}

#[derive(Debug, Clone)]
pub(super) struct MenuLevelContext {
    root: bool,
    shared_focus_visible: Rc<Cell<bool>>,
    level_open: Option<Rc<Cell<bool>>>,
    level_pointer_inside: Option<Rc<Cell<bool>>>,
    ensure_visible: EnsureVisibleHandle,
}

/// Why a highlight is being written.
///
/// Decides two things: whether the write counts as this level having
/// established a highlight, and whether the highlighted row is painted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HighlightOrigin {
    /// The reader moved it, by pointer or by key. Clearing counts too: the
    /// pointer moving onto a row that cannot be highlighted leaves the level
    /// deliberately empty.
    Decision,
    /// Entering the level placed it on the first eligible row, so arrows and
    /// Enter work immediately.
    ///
    /// This is a navigation cursor rather than an answer to anything the reader
    /// did, so it is painted only when the level was entered by keyboard. A menu
    /// opened by clicking would otherwise show a highlighted row nowhere near
    /// the pointer.
    Entry,
    /// A rebuild carrying an existing highlight across. Never establishes one,
    /// so it cannot turn a deliberately empty level back into an unvisited one.
    Reconciliation,
}

/// How far this level's highlight has got since it became active.
///
/// One value rather than a pair of flags, because only three of their four
/// combinations mean anything: the highlight cannot come from the reader before
/// the level has been entered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LevelHighlight {
    /// The level has not been entered. `highlight` being `None` here means "not
    /// placed yet", so entry must place it on the first eligible row.
    Unvisited,
    /// Entry parked a navigation cursor so arrows and Enter work immediately.
    /// `None` here means the reader cleared it deliberately, and no rebuild or
    /// focus pass may refill it.
    Parked,
    /// The reader moved it, by pointer or by key, so it is painted.
    Chosen,
}

#[derive(Debug)]
pub(super) struct MenuListState {
    focus: Option<FocusState>,
    pub(super) highlight: Option<usize>,
    highlighted_label: Option<String>,
    level_highlight: LevelHighlight,
    pressed: Option<usize>,
    typeahead: String,
    typeahead_deadline: Option<iced::time::Instant>,
    now: Option<iced::time::Instant>,
    pub(super) open_submenu: Option<usize>,
    open_submenu_label: Option<String>,
    submenu_intent: Option<(usize, iced::time::Instant)>,
    transfer_deadline: Option<iced::time::Instant>,
    overlay_nodes: Vec<OverlayNodeState>,
    last_pointer: Option<Point>,
}

impl<'a, Message> From<MenuList<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(list: MenuList<'a, Message>) -> Self {
        Element::new(list)
    }
}

#[derive(Debug, Clone)]
enum BranchEvent<Message> {
    Content(Message),
    Close,
}

pub(super) struct MenuBranch<'a, Message> {
    anchor: Element<'a, Message>,
    content: Element<'a, BranchEvent<Message>>,
    handle: MenuBranchHandle,
    ensure_visible: EnsureVisibleHandle,
}

impl<'a, Message> From<MenuBranch<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(branch: MenuBranch<'a, Message>) -> Self {
        Element::new(branch)
    }
}

pub(super) struct MenuTrailingTrack<'a, Message> {
    content: Element<'a, Message>,
    width: Rc<Cell<f32>>,
}

impl<'a, Message> From<MenuTrailingTrack<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(track: MenuTrailingTrack<'a, Message>) -> Self {
        Element::new(track)
    }
}
