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

#[derive(Debug)]
pub(super) struct MenuListState {
    focus: Option<FocusState>,
    pub(super) highlight: Option<usize>,
    highlighted_label: Option<String>,
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
