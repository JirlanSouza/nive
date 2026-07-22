mod focus_within;
mod host;
mod toast_stack;

#[cfg(test)]
mod toast_host_tests;

use iced::Length;

pub use nive_core::{AnnouncementPoliteness, ToastPresentation, ToastTone};

use crate::Element;

/// Card maximum width; shrinks to fill minus clearance on narrow viewports.
const CARD_MAX_WIDTH: f32 = 360.0;
/// Clearance kept from each viewport edge on narrow layouts.
const NARROW_CLEARANCE: f32 = 16.0;
const CARD_PADDING: f32 = 12.0;
const CARD_GAP: f32 = 8.0;
const STACK_GAP: f32 = 8.0;
const ICON_SIZE: f32 = 16.0;
const TITLE_SIZE: f32 = 14.0;
const BODY_SIZE: f32 = 14.0;
const BODY_LINE_HEIGHT: f32 = 1.4;
const MAX_BODY_LINES: f32 = 3.0;
/// At most three toasts are ever rendered by a single host; the runtime is
/// responsible for keeping its visible set within this cap.
const MAX_VISIBLE: usize = 3;

/// Logical stacking corner. `Start`/`End` stay direction-ready ahead of a
/// full RTL resolver; rendering remains physical-LTR (`Start` = left,
/// `End` = right) until one exists.
///
/// The physical `TopLeft`/`TopRight`/`BottomLeft`/`BottomRight` variants are
/// removed with no facade:
///
/// ```compile_fail
/// use nive_ui::widgets::overlays::ToastPosition;
///
/// let _ = ToastPosition::TopLeft;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastPosition {
    TopStart,
    TopEnd,
    BottomStart,
    #[default]
    BottomEnd,
}

/// Safe-area clearance the host must keep the stack clear of (viewport edges,
/// window chrome such as a status bar), supplied by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ToastInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

/// Overlays a toast stack on top of `content`.
///
/// ## Accessibility contract (preparatory)
///
/// No native AccessKit live-region emission exists in this Iced version, so
/// the points below are settled semantics for whoever wires that surface,
/// not yet-enforced runtime behavior:
///
/// - Announcement politeness follows [`ToastTone::announcement_politeness`].
/// - Only the newest toast is ever meant to announce — never the full stack
///   after a reorder. `toasts()` already receives items newest-first, so
///   the first item is that toast.
/// - The tone icon is decorative; the card owns the accessible name.
///
/// One point *is* enforced today: `ToastHost` never moves keyboard focus
/// into a toast on its own — a toast's dismiss/action buttons are reachable
/// by `Tab` like any other control, but appearing never steals focus.
pub struct ToastHost<'a, Message> {
    content: Element<'a, Message>,
    items: Vec<(u64, Element<'a, Message>)>,
    position: ToastPosition,
    insets: ToastInsets,
    on_pause: Option<Message>,
    on_resume: Option<Message>,
    on_focus_enter: Option<Message>,
    on_focus_exit: Option<Message>,
}

impl<'a, Message> From<ToastHost<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(host: ToastHost<'a, Message>) -> Self {
        host.into_element()
    }
}

/// interaction) the previous occupant of that slot left behind.
/// Vertical stack of toast cards keyed by toast id.
///
/// A near-clone of `iced::widget::keyed_column`'s internals (same flex
/// layout, same per-child delegation), except its `diff()` actually compares
/// every index's key, not just the first and last. Upstream's version uses
/// that boundary check purely to decide *whether the child count changed* —
/// when a dismiss is immediately backfilled by promotion (the common case,
/// since the visible count stays the same), the count never changes, so its
/// diff silently falls through to a plain positional zip and a different
/// toast inherits whatever live widget state (an open Tooltip, mid-press
struct ToastStack<'a, Message> {
    keys: Vec<u64>,
    children: Vec<Element<'a, Message>>,
    spacing: f32,
    width: Length,
    max_width: f32,
}

#[derive(Default)]
struct ToastStackState {
    keys: Vec<u64>,
}

impl<'a, Message> From<ToastStack<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(stack: ToastStack<'a, Message>) -> Self {
        Element::new(stack)
    }
}

/// previous pass so the message fires only on a genuine transition.
/// Transparent wrapper that publishes `on_enter`/`on_exit` when the standard
/// Iced keyboard-focus state of any widget in `content` (tracked per-widget
/// via `operation::Focusable`, the same mechanism `Tab` navigation and
/// `nive-ui`'s `Pressable`-based buttons already use) starts or stops being
/// true — the keyboard-focus counterpart to `mouse_area`'s hover events.
///
/// Every method delegates straight through to `content` except `update`,
/// which re-checks focus state after delegating and diffs it against the
struct FocusWithinArea<'a, Message> {
    content: Element<'a, Message>,
    on_enter: Option<Message>,
    on_exit: Option<Message>,
}

#[derive(Debug, Default)]
struct FocusWithinState {
    focused: bool,
}

impl<'a, Message> From<FocusWithinArea<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(area: FocusWithinArea<'a, Message>) -> Self {
        Element::new(area)
    }
}
