use std::collections::VecDeque;
use std::convert::Infallible;
use std::time::Instant;

use iced::window;
pub use nive_core::ToastTone;
pub use nive_ui::widgets::overlays::{ToastInsets, ToastPosition};

const MAX_VISIBLE_TOASTS: usize = 3;
/// Hard cap on queued (not-yet-visible) toasts. A burst beyond this evicts
/// the oldest queued toast that carries no action first, falling back to the
/// oldest overall only once every queued toast is actionable.
const MAX_QUEUED_TOASTS: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ToastId(pub(crate) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastDuration {
    Short,
    Medium,
    Long,
    /// Never auto-expires; stays until acted on or dismissed.
    Persistent,
}

/// An optional secondary action carrying the application `Message` to
/// dispatch when pressed. Lives on the runtime-owned typed [`Toast`], never
/// on the neutral `nive_core::ToastPresentation` contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastAction<Message> {
    label: String,
    message: Message,
}

/// A queued or visible notification: a title, optional body, [`ToastTone`],
/// [`ToastDuration`], an optional secondary action, and whether it renders
/// once in the active window (`global()`) rather than scoped to whichever
/// window pushed it (the default). Push it with `Effect::toast`/
/// `with_toast`; `ToastState` owns queueing, coalescing, timing, and pause
/// from there — see its rustdoc and `ToastHost`'s.
///
/// A toast is not generic in practice unless it carries an action; the
/// `Infallible` default keeps `Toast::info("x")` and friends ergonomic at
/// call sites that never construct an action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast<Message = Infallible> {
    title: String,
    body: Option<String>,
    tone: ToastTone,
    duration: ToastDuration,
    /// `true` renders once in the active window instead of the originating
    /// window.
    global: bool,
    action: Option<ToastAction<Message>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastItem<Message = Infallible> {
    id: ToastId,
    request: Toast<Message>,
    /// `None` while persistent; never auto-expires.
    expires_at: Option<Instant>,
    origin: Option<window::Id>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToastMessage {
    Dismiss { id: ToastId, now: Instant },
    Tick(Instant),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueuedToast<Message = Infallible> {
    id: ToastId,
    request: Toast<Message>,
    origin: Option<window::Id>,
}

/// Runtime-owned queue, timing, pause, and window-scoping state for toasts.
///
/// Only the methods below (`push`, `dismiss`, `tick`, `set_hover`, ...) are
/// public; the queue/timing/pause fields that back them are not:
///
/// ```compile_fail
/// use nive_runtime::ToastState;
///
/// let state = ToastState::<()>::default();
/// let _ = state.visible;
/// ```
#[derive(Debug)]
pub struct ToastState<Message = Infallible> {
    next_id: u64,
    visible: VecDeque<ToastItem<Message>>,
    queued: VecDeque<QueuedToast<Message>>,
    hover: bool,
    focus_within: bool,
    window_active: bool,
    modal_active: bool,
    paused_since: Option<Instant>,
}

mod builder;
mod item;
mod state;

#[cfg(test)]
mod tests;
