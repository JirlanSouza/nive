use std::borrow::Cow;

use nive_core::{ActionId, ShortcutBinding};

mod builders;
mod items;
pub(crate) mod relay;
mod render;
pub(crate) mod style;
mod widget;

#[cfg(test)]
mod tests;

use crate::widgets::controls::CheckboxState;
use crate::widgets::overlays::{PopoverCollision, PopoverPlacement};
use crate::widgets::primitives::IconRole;
use crate::Element;

const MENU_MAX_WIDTH: f32 = 320.0;
pub(crate) const MENU_LIST_INSET: f32 = 4.0;
pub(crate) const MENU_ROW_HEIGHT: f32 = 28.0;
pub(crate) const MENU_ROW_PADDING_H: f32 = 8.0;
pub(crate) const MENU_ROW_RADIUS: f32 = 4.0;
pub(crate) const MENU_ICON_SIZE: f32 = 16.0;
const MENU_SEPARATOR_MARGIN: f32 = 4.0;
pub(crate) const MENU_COLUMN_GAP: f32 = 8.0;

/// Whether activating a Menu leaf requests closure of the complete chain.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuDismissPolicy {
    /// Publish the leaf action, then request root dismissal when supported.
    #[default]
    DismissAll,
    /// Publish the leaf action and retain the complete open Menu chain.
    KeepOpen,
}

/// A canonical anchored Menu with fluent, category-specific entries.
///
/// Menu internally owns one edge-to-edge, FocusFirst Popover and paints no
/// second floating surface. It keeps one real focus target across the complete
/// submenu chain; rows use transient logical highlight for bounded Up/Down,
/// Home/End, 700ms prefix typeahead, Enter/Space, and scrolling. Submenus use
/// 200ms pointer intent, 300ms transfer grace, and physical-LTR Right/Left
/// navigation.
///
/// Command, checkbox, and radio values remain application-controlled. Missing
/// leaf callbacks make only that leaf display-only and do not request disabled
/// styling. Explicit `disabled(true)` wins over configured capability and
/// suppresses interaction while preserving content, durable marks, and row
/// geometry. All highlight and open-state visuals update immediately.
///
/// [`MenuDismissPolicy::DismissAll`] publishes the leaf message first and then
/// one dismissal message when the Menu has dismissal capability; without that
/// capability it publishes only the leaf and stays open. `KeepOpen` never
/// requests dismissal. Programmatic rebuilds are silent. Labels and logical
/// state are retained for future accessibility integration, but Menu does not
/// currently emit native menu roles, active-descendant relations, or
/// announcements.
///
/// The former static menu models are removed:
///
/// ```compile_fail
/// use nive_ui::widgets::{DropdownMenu, DropdownMenuItem};
/// ```
///
/// Menu owns its fixed desktop metrics and internal Tree/row adapters. Generic
/// `Length` and `ControlSize` builders are deliberately unsupported:
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let menu: Menu<'_, ()> = Menu::new(text("Menu"));
/// let _ = menu.width(Length::Fill);
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let menu: Menu<'_, ()> = Menu::new(text("Menu"));
/// let _ = menu.size(theme::ControlSize::Sm);
/// ```
///
/// ```compile_fail
/// use nive_ui::widgets::navigation::menu::widget::{
///     MenuListState, MenuSlot, MenuTrailingTrack,
/// };
/// ```
///
/// ```compile_fail
/// use nive_ui::widgets::navigation::menu::relay::MessageRelay;
/// ```
pub struct Menu<'a, Message> {
    trigger: Element<'a, Message>,
    entries: Vec<MenuEntry<'a, Message>>,
    open: bool,
    on_dismiss: Option<Message>,
    placement: PopoverPlacement,
    collision: PopoverCollision,
    match_anchor_width: bool,
}

/// A leaf command in a [`Menu`].
///
/// [`MenuCommand::from_action`] is the canonical projection from a shared
/// [`Action`](nive_core::Action). Menu-specific icon, destructive intent, and dismissal policy are
/// presentation decoration; action identity, label, shortcut, enabled state,
/// and activation remain sourced from the immutable action.
pub struct MenuCommand<'a, Message> {
    id: Option<ActionId>,
    label: Cow<'a, str>,
    icon: Option<IconRole>,
    shortcut: Option<ShortcutBinding>,
    destructive: bool,
    disabled: bool,
    source_disabled: bool,
    on_press: Option<Message>,
    dismiss_policy: MenuDismissPolicy,
}

/// A controlled tri-state checkbox leaf in a [`Menu`].
///
/// Activation publishes the requested next [`CheckboxState`]; the application
/// supplies the durable state on the next view.
pub struct MenuCheckbox<'a, Message> {
    label: Cow<'a, str>,
    state: CheckboxState,
    shortcut: Option<ShortcutBinding>,
    disabled: bool,
    on_toggle: Option<Box<dyn Fn(CheckboxState) -> Message + 'a>>,
    dismiss_policy: MenuDismissPolicy,
}

/// One application-valued option in a [`MenuRadioGroup`].
///
/// The value is durable identity. Its optional icon and annotation occupy
/// separate tracks from the persistent radio mark.
pub struct MenuRadioOption<'a, T> {
    value: T,
    label: Cow<'a, str>,
    icon: Option<IconRole>,
    annotation: Option<Cow<'a, str>>,
    disabled: bool,
}

/// A controlled application-valued radio group in a [`Menu`].
///
/// Values must be unique. Activation publishes a `T` and never mutates the
/// supplied selection or exposes a visual row index as application state.
pub struct MenuRadioGroup<'a, T, Message> {
    selected: Option<T>,
    options: Vec<MenuRadioOption<'a, T>>,
    on_select: Option<Box<dyn Fn(T) -> Message + 'a>>,
    dismiss_policy: MenuDismissPolicy,
}

/// A branch entry whose child is another canonical Menu.
///
/// A submenu is navigation rather than a leaf action, so it has no destructive
/// or leaf-dismissal policy.
pub struct MenuSubmenu<'a, Message> {
    label: Cow<'a, str>,
    icon: Option<IconRole>,
    disabled: bool,
    child: Box<Menu<'a, Message>>,
}

enum MenuEntry<'a, Message> {
    Command(MenuCommand<'a, Message>),
    Checkbox(MenuCheckbox<'a, Message>),
    Radio(MenuRadioRow<'a, Message>),
    Submenu(MenuSubmenu<'a, Message>),
    Separator,
}

#[derive(Clone)]
enum MenuEvent<Message> {
    Activate(Message, MenuDismissPolicy),
}

struct MenuRadioRow<'a, Message> {
    label: Cow<'a, str>,
    icon: Option<IconRole>,
    annotation: Option<Cow<'a, str>>,
    selected: bool,
    disabled: bool,
    on_press: Option<Message>,
    dismiss_policy: MenuDismissPolicy,
}

enum MenuTrailing<'a> {
    Shortcut(Cow<'a, str>),
    Annotation(Cow<'a, str>),
    Submenu,
}

impl<'a, Message: Clone + 'a> From<Menu<'a, Message>> for Element<'a, Message> {
    fn from(menu: Menu<'a, Message>) -> Self {
        menu.into_element()
    }
}
