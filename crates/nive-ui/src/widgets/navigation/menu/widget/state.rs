use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use iced::Rectangle;

use super::{
    MenuBranchHandle, MenuLevelContext, MenuListState, MenuSlot, MenuTrailingMeasure,
    SEPARATOR_HEIGHT,
};
use crate::widgets::navigation::menu::MENU_ROW_HEIGHT;
use crate::{
    advanced::focus::{FocusState, FocusVisibility},
    theme::choice::ChoicePersistentState,
    widgets::overlays::anchored_overlay::{scroll::EnsureVisibleHandle, OverlayIdentity},
};

impl MenuBranchHandle {
    pub(in crate::widgets::navigation::menu) fn new() -> Self {
        Self {
            open: Rc::new(Cell::new(false)),
            pointer_inside: Rc::new(Cell::new(false)),
            child_bounds: Rc::new(Cell::new(Rectangle::default())),
            identity: Rc::new(RefCell::new(OverlayIdentity::root())),
        }
    }

    pub(in crate::widgets::navigation::menu) fn open(&self) -> Rc<Cell<bool>> {
        Rc::clone(&self.open)
    }

    pub(in crate::widgets::navigation::menu) fn pointer_inside(&self) -> Rc<Cell<bool>> {
        Rc::clone(&self.pointer_inside)
    }
}

impl<Message> MenuSlot<Message> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::widgets::navigation::menu) fn row(
        eligible: bool,
        activation: Option<Message>,
        label: impl Into<String>,
        trailing: Option<MenuTrailingMeasure>,
        persistent: ChoicePersistentState,
        disabled: bool,
        logical_focus: Rc<Cell<bool>>,
        branch: Option<MenuBranchHandle>,
    ) -> Self {
        Self {
            eligible,
            separator: false,
            activation,
            label: Some(label.into()),
            trailing,
            persistent,
            disabled,
            logical_focus: Some(logical_focus),
            branch,
        }
    }

    pub(in crate::widgets::navigation::menu) fn separator() -> Self {
        Self {
            eligible: false,
            separator: true,
            activation: None,
            label: None,
            trailing: None,
            persistent: ChoicePersistentState::Unselected,
            disabled: false,
            logical_focus: None,
            branch: None,
        }
    }

    pub(in crate::widgets::navigation::menu) fn height(&self) -> f32 {
        if self.separator {
            SEPARATOR_HEIGHT
        } else {
            MENU_ROW_HEIGHT
        }
    }
}

impl MenuLevelContext {
    pub(in crate::widgets::navigation::menu) fn root() -> Self {
        Self {
            root: true,
            shared_focus_visible: Rc::new(Cell::new(false)),
            level_open: None,
            level_pointer_inside: None,
            ensure_visible: EnsureVisibleHandle::new(),
        }
    }

    pub(in crate::widgets::navigation::menu) fn child(&self, branch: &MenuBranchHandle) -> Self {
        Self {
            root: false,
            shared_focus_visible: Rc::clone(&self.shared_focus_visible),
            level_open: Some(branch.open()),
            level_pointer_inside: Some(branch.pointer_inside()),
            ensure_visible: EnsureVisibleHandle::new(),
        }
    }

    pub(in crate::widgets::navigation::menu) fn ensure_visible(&self) -> EnsureVisibleHandle {
        self.ensure_visible.clone()
    }
}

impl Default for MenuListState {
    fn default() -> Self {
        Self::new(true)
    }
}

impl MenuListState {
    pub(in crate::widgets::navigation::menu) fn new(root: bool) -> Self {
        Self {
            focus: root.then(|| FocusState::new(FocusVisibility::Auto)),
            highlight: None,
            highlighted_label: None,
            pressed: None,
            typeahead: String::new(),
            typeahead_deadline: None,
            now: None,
            open_submenu: None,
            open_submenu_label: None,
            submenu_intent: None,
            transfer_deadline: None,
            overlay_nodes: Vec::new(),
            last_pointer: None,
        }
    }
}
