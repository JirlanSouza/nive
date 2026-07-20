use std::collections::VecDeque;
use std::convert::Infallible;
use std::time::{Duration, Instant};

use iced::window;
use nive_core::ToastPresentation;
pub use nive_core::ToastTone;
pub use nive_ui::widgets::overlays::{ToastInsets, ToastPosition};

use crate::UserFacingError;

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

impl<Message> Toast<Message> {
    pub fn success(title: impl Into<String>) -> Self {
        Self::new(title, ToastTone::Success)
    }

    pub fn warning(title: impl Into<String>) -> Self {
        Self::new(title, ToastTone::Warning)
    }

    pub fn danger(title: impl Into<String>) -> Self {
        Self::new(title, ToastTone::Danger)
    }

    pub fn info(title: impl Into<String>) -> Self {
        Self::new(title, ToastTone::Info)
    }

    /// Shows only the safe summary as the title; the diagnostic detail
    /// (if any) never enters the toast body. Complete diagnostics remain
    /// available only through `ErrorDetailsDialog`.
    pub fn error(error: UserFacingError) -> Self {
        Self::danger(error.summary()).long()
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn long(mut self) -> Self {
        self.duration = ToastDuration::Long;
        self
    }

    /// Marks the toast persistent: it never auto-expires and stays until
    /// acted on or dismissed.
    pub fn persistent(mut self) -> Self {
        self.duration = ToastDuration::Persistent;
        self
    }

    /// Marks the toast explicitly global: it renders once in the active
    /// window rather than being scoped to the window that emitted it.
    pub fn global(mut self) -> Self {
        self.global = true;
        self
    }

    /// Attaches a single secondary action. Its presence makes the toast
    /// persistent until the user acts on it or dismisses it.
    pub fn with_action(mut self, label: impl Into<String>, message: Message) -> Self {
        self.action = Some(ToastAction {
            label: label.into(),
            message,
        });
        self.duration = ToastDuration::Persistent;
        self
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    pub fn tone(&self) -> ToastTone {
        self.tone
    }

    pub fn is_global(&self) -> bool {
        self.global
    }

    pub fn action(&self) -> Option<(&str, &Message)> {
        self.action
            .as_ref()
            .map(|action| (action.label.as_str(), &action.message))
    }

    /// Maps the action's message type, leaving everything else unchanged.
    pub fn map_action<N>(self, map: impl FnOnce(Message) -> N) -> Toast<N> {
        Toast {
            title: self.title,
            body: self.body,
            tone: self.tone,
            duration: self.duration,
            global: self.global,
            action: self.action.map(|action| ToastAction {
                label: action.label,
                message: map(action.message),
            }),
        }
    }

    fn new(title: impl Into<String>, tone: ToastTone) -> Self {
        let duration = match tone {
            ToastTone::Info | ToastTone::Success => ToastDuration::Short,
            ToastTone::Warning => ToastDuration::Medium,
            ToastTone::Danger => ToastDuration::Long,
        };
        Self {
            title: title.into(),
            body: None,
            tone,
            duration,
            global: false,
            action: None,
        }
    }

    /// Coalescing identity: toasts with matching title/body/tone are
    /// considered duplicates of one another. A toast carrying an action is
    /// never coalesced away.
    fn coalesce_key(&self) -> Option<(&str, Option<&str>, ToastTone)> {
        if self.action.is_some() {
            return None;
        }
        Some((self.title.as_str(), self.body.as_deref(), self.tone))
    }
}

impl ToastDuration {
    /// Resolves to the auto-expire duration, or `None` when persistent.
    pub fn as_duration(self) -> Option<Duration> {
        match self {
            ToastDuration::Short => Some(Duration::from_secs(4)),
            ToastDuration::Medium => Some(Duration::from_secs(6)),
            ToastDuration::Long => Some(Duration::from_secs(8)),
            ToastDuration::Persistent => None,
        }
    }
}

impl<Message> ToastItem<Message> {
    pub fn id(&self) -> ToastId {
        self.id
    }

    pub fn request(&self) -> &Toast<Message> {
        &self.request
    }

    /// The window that originated this toast, if it was scoped to one.
    pub fn origin(&self) -> Option<window::Id> {
        self.origin
    }
}

impl<Message> ToastPresentation for ToastItem<Message> {
    type Id = ToastId;

    fn id(&self) -> ToastId {
        ToastItem::id(self)
    }

    fn title(&self) -> &str {
        self.request().title()
    }

    fn body(&self) -> Option<&str> {
        self.request().body()
    }

    fn tone(&self) -> ToastTone {
        self.request().tone()
    }
}

impl ToastId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl<Message> ToastState<Message> {
    /// Pushes a toast, coalescing an exact duplicate already visible or
    /// queued (returning its existing id) rather than adding a new entry.
    /// `origin` is the window that emitted it (`None` when no window
    /// context is available, treated like an explicitly global toast for
    /// rendering so the toast is never lost).
    pub fn push(
        &mut self,
        toast: Toast<Message>,
        now: Instant,
        origin: Option<window::Id>,
    ) -> ToastId {
        if let Some(existing) = self.find_duplicate(&toast) {
            return existing;
        }

        let id = ToastId(self.next_id);
        self.next_id += 1;

        self.visible.push_front(ToastItem {
            id,
            expires_at: resolve_expiry(&toast, now),
            request: toast,
            origin,
        });

        while self.visible.len() > MAX_VISIBLE_TOASTS {
            if let Some(item) = self.visible.pop_back() {
                self.queued.push_back(QueuedToast {
                    id: item.id,
                    request: item.request,
                    origin: item.origin,
                });
            }
        }

        self.evict_stale_if_over_capacity();

        id
    }

    pub fn dismiss(&mut self, id: ToastId, now: Instant) {
        self.visible.retain(|item| item.id != id);
        self.queued.retain(|item| item.id != id);
        self.promote_queued(now);
    }

    pub fn expire(&mut self, now: Instant) {
        self.visible
            .retain(|item| item.expires_at.is_none_or(|expires_at| expires_at > now));
        self.promote_queued(now);
    }

    /// Sets the hover-pause input (pauses the complete visible stack while
    /// any toast is hovered).
    pub fn set_hover(&mut self, hovered: bool, now: Instant) {
        let was_paused = self.is_paused();
        self.hover = hovered;
        self.sync_pause(was_paused, now);
    }

    /// Sets the focus-within pause input (pauses while focus is inside any
    /// toast).
    pub fn set_focus_within(&mut self, focused: bool, now: Instant) {
        let was_paused = self.is_paused();
        self.focus_within = focused;
        self.sync_pause(was_paused, now);
    }

    /// Sets whether the originating window is visible and focused. Iced
    /// exposes `Unfocused`, not a separate "hidden" signal, so this input
    /// covers both the hidden and inactive cases the spec names.
    pub fn set_window_active(&mut self, active: bool, now: Instant) {
        let was_paused = self.is_paused();
        self.window_active = active;
        self.sync_pause(was_paused, now);
    }

    /// Sets whether a modal is currently open (pauses and makes the stack
    /// inert while true).
    pub fn set_modal_active(&mut self, active: bool, now: Instant) {
        let was_paused = self.is_paused();
        self.modal_active = active;
        self.sync_pause(was_paused, now);
    }

    pub fn is_paused(&self) -> bool {
        self.hover || self.focus_within || !self.window_active || self.modal_active
    }

    pub fn update(&mut self, message: ToastMessage) {
        match message {
            ToastMessage::Dismiss { id, now } => self.dismiss(id, now),
            ToastMessage::Tick(now) => self.tick(now),
        }
    }

    pub fn should_subscribe(&self) -> bool {
        self.has_visible()
    }

    /// Periodic subscription tick: expires elapsed toasts unless the stack
    /// is currently paused.
    pub fn tick(&mut self, now: Instant) {
        if !self.is_paused() {
            self.expire(now);
        }
    }

    pub fn has_visible(&self) -> bool {
        !self.visible.is_empty()
    }

    pub fn visible(&self) -> impl Iterator<Item = &ToastItem<Message>> {
        self.visible.iter()
    }

    /// The subset of the visible stack that should render in `window_id`:
    /// window-scoped toasts originating there, plus any explicitly global
    /// (or origin-less) toast only when `window_id` is the active window.
    pub fn visible_for(
        &self,
        window_id: window::Id,
        active_window: Option<window::Id>,
    ) -> impl Iterator<Item = &ToastItem<Message>> {
        self.visible.iter().filter(move |item| {
            if item.request.is_global() || item.origin.is_none() {
                Some(window_id) == active_window
            } else {
                item.origin == Some(window_id)
            }
        })
    }

    fn find_duplicate(&self, toast: &Toast<Message>) -> Option<ToastId> {
        let key = toast.coalesce_key()?;
        self.visible
            .iter()
            .find(|item| item.request.coalesce_key().as_ref() == Some(&key))
            .map(|item| item.id)
            .or_else(|| {
                self.queued
                    .iter()
                    .find(|item| item.request.coalesce_key().as_ref() == Some(&key))
                    .map(|item| item.id)
            })
    }

    fn sync_pause(&mut self, was_paused: bool, now: Instant) {
        let now_paused = self.is_paused();
        if now_paused && !was_paused {
            self.paused_since = Some(now);
        } else if !now_paused && was_paused {
            if let Some(since) = self.paused_since.take() {
                let elapsed = now.saturating_duration_since(since);
                for item in &mut self.visible {
                    if let Some(expires_at) = item.expires_at.as_mut() {
                        *expires_at += elapsed;
                    }
                }
            }
        }
    }

    fn promote_queued(&mut self, now: Instant) {
        while self.visible.len() < MAX_VISIBLE_TOASTS {
            let Some(queued) = self.queued.pop_front() else {
                return;
            };
            self.visible.push_back(ToastItem {
                id: queued.id,
                expires_at: resolve_expiry(&queued.request, now),
                request: queued.request,
                origin: queued.origin,
            });
        }
    }

    /// Evicts the oldest queued toast without an action once the queue
    /// exceeds its bound, falling back to the true oldest only once every
    /// queued toast carries an action — distinct actionable toasts are
    /// never silently dropped ahead of that fallback.
    fn evict_stale_if_over_capacity(&mut self) {
        while self.queued.len() > MAX_QUEUED_TOASTS {
            let evict_index = self
                .queued
                .iter()
                .position(|item| item.request.action().is_none())
                .unwrap_or(0);
            self.queued.remove(evict_index);
        }
    }
}

fn resolve_expiry<Message>(toast: &Toast<Message>, now: Instant) -> Option<Instant> {
    toast.duration.as_duration().map(|duration| now + duration)
}

impl<Message> Default for ToastState<Message> {
    fn default() -> Self {
        Self {
            next_id: 0,
            visible: VecDeque::new(),
            queued: VecDeque::new(),
            hover: false,
            focus_within: false,
            window_active: true,
            modal_active: false,
            paused_since: None,
        }
    }
}

#[cfg(test)]
mod toast_tests {
    use super::*;

    fn push(state: &mut ToastState, toast: Toast, now: Instant) -> ToastId {
        state.push(toast, now, None)
    }

    #[test]
    fn push_keeps_newest_three_toasts() {
        let now = Instant::now();
        let mut state = ToastState::default();

        for index in 0..4 {
            push(&mut state, Toast::info(format!("Toast {index}")), now);
        }

        let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
        assert_eq!(titles, vec!["Toast 3", "Toast 2", "Toast 1"]);
    }

    #[test]
    fn dismiss_removes_matching_toast() {
        let now = Instant::now();
        let mut state = ToastState::default();
        let first = push(&mut state, Toast::info("First"), now);
        push(&mut state, Toast::info("Second"), now);

        state.dismiss(first, now);

        let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
        assert_eq!(titles, vec!["Second"]);
    }

    #[test]
    fn overflow_toasts_are_queued_and_promoted_after_dismiss() {
        let now = Instant::now();
        let mut state = ToastState::default();

        for index in 0..4 {
            push(&mut state, Toast::info(format!("Toast {index}")), now);
        }

        let newest = state
            .visible()
            .next()
            .map(|item| item.id())
            .expect("newest toast is visible");
        state.dismiss(newest, now + Duration::from_secs(1));

        let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
        assert_eq!(titles, vec!["Toast 2", "Toast 1", "Toast 0"]);
    }

    #[test]
    fn overflow_toasts_are_queued_and_promoted_after_expiration() {
        let now = Instant::now();
        let mut state = ToastState::default();

        for index in 0..4 {
            push(&mut state, Toast::info(format!("Toast {index}")), now);
        }

        state.expire(now + Duration::from_secs(5));

        let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
        assert_eq!(titles, vec!["Toast 0"]);

        state.expire(now + Duration::from_secs(8));

        let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
        assert_eq!(titles, vec!["Toast 0"]);
    }

    #[test]
    fn default_duration_depends_on_tone() {
        assert_eq!(
            Toast::<()>::info("Info").duration.as_duration(),
            Some(Duration::from_secs(4))
        );
        assert_eq!(
            Toast::<()>::success("Success").duration.as_duration(),
            Some(Duration::from_secs(4))
        );
        assert_eq!(
            Toast::<()>::warning("Warning").duration.as_duration(),
            Some(Duration::from_secs(6))
        );
        assert_eq!(
            Toast::<()>::danger("Danger").duration.as_duration(),
            Some(Duration::from_secs(8))
        );
    }

    #[test]
    fn persistent_toast_never_auto_expires() {
        let now = Instant::now();
        assert_eq!(
            Toast::<()>::info("Pinned")
                .persistent()
                .duration
                .as_duration(),
            None
        );

        let mut state = ToastState::default();
        push(&mut state, Toast::info("Pinned").persistent(), now);

        state.expire(now + Duration::from_secs(60 * 60));

        let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
        assert_eq!(titles, vec!["Pinned"]);
    }

    #[test]
    fn action_forces_persistent_duration() {
        let toast = Toast::info("Undo?").with_action("Undo", 42_u8);

        assert_eq!(toast.duration.as_duration(), None);
        assert_eq!(toast.action(), Some(("Undo", &42_u8)));
    }

    #[test]
    fn actionable_toasts_are_never_coalesced() {
        let now = Instant::now();
        let mut state: ToastState<u8> = ToastState::default();

        let first = state.push(Toast::info("Undo?").with_action("Undo", 1), now, None);
        let second = state.push(Toast::info("Undo?").with_action("Undo", 2), now, None);

        assert_ne!(first, second);
    }

    #[test]
    fn duplicate_toasts_are_coalesced() {
        let now = Instant::now();
        let mut state = ToastState::default();

        let first = push(&mut state, Toast::info("Saved"), now);
        let second = push(&mut state, Toast::info("Saved"), now);

        assert_eq!(first, second);
        assert_eq!(state.visible().count(), 1);
    }

    #[test]
    fn distinct_toasts_with_the_same_title_but_different_bodies_are_not_coalesced() {
        let now = Instant::now();
        let mut state = ToastState::default();

        push(&mut state, Toast::info("Saved").with_body("Item A"), now);
        push(&mut state, Toast::info("Saved").with_body("Item B"), now);

        assert_eq!(state.visible().count(), 2);
    }

    #[test]
    fn queue_is_bounded_and_evicts_the_oldest_stale_entry() {
        let now = Instant::now();
        let mut state = ToastState::default();

        for index in 0..(MAX_VISIBLE_TOASTS + MAX_QUEUED_TOASTS + 5) {
            push(&mut state, Toast::info(format!("Toast {index}")), now);
        }

        assert!(state.queued.len() <= MAX_QUEUED_TOASTS);
    }

    #[test]
    fn queue_eviction_prefers_non_actionable_entries() {
        let now = Instant::now();
        let mut state: ToastState<u8> = ToastState::default();

        for index in 0..MAX_VISIBLE_TOASTS {
            state.push(Toast::info(format!("filler {index}")), now, None);
        }
        // The first three actionable pushes bump the fillers into the
        // queue; the rest bump earlier actionable toasts, filling the
        // queue to exactly its bound with fillers first, actionable after.
        for index in 0..MAX_QUEUED_TOASTS {
            state.push(
                Toast::info(format!("Actionable {index}")).with_action("Undo", index as u8),
                now,
                None,
            );
        }
        assert_eq!(state.queued.len(), MAX_QUEUED_TOASTS);
        assert!(state
            .queued
            .iter()
            .take(MAX_VISIBLE_TOASTS)
            .all(|item| item.request.action().is_none()));

        // One more push exceeds the bound; the oldest non-actionable queued
        // entry is evicted before any actionable one.
        state.push(Toast::info("final"), now, None);

        assert_eq!(state.queued.len(), MAX_QUEUED_TOASTS);
        assert_eq!(
            state
                .queued
                .iter()
                .filter(|item| item.request.action().is_none())
                .count(),
            MAX_VISIBLE_TOASTS - 1
        );
    }

    #[test]
    fn expire_removes_elapsed_toasts() {
        let now = Instant::now();
        let mut state = ToastState::default();
        push(&mut state, Toast::info("Short"), now);
        push(&mut state, Toast::info("Long").long(), now);

        state.expire(now + Duration::from_secs(5));

        let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
        assert_eq!(titles, vec!["Long"]);
    }

    #[test]
    fn hover_pause_and_resume_preserves_remaining_toast_duration() {
        let now = Instant::now();
        let mut state = ToastState::default();
        push(&mut state, Toast::info("Short"), now);

        state.set_hover(true, now + Duration::from_secs(1));
        state.set_hover(false, now + Duration::from_secs(3));
        state.expire(now + Duration::from_secs(5));

        let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
        assert_eq!(titles, vec!["Short"]);

        state.expire(now + Duration::from_secs(7));

        assert!(state.visible().next().is_none());
    }

    #[test]
    fn focus_within_pauses_the_stack() {
        let now = Instant::now();
        let mut state = ToastState::default();
        push(&mut state, Toast::info("Short"), now);

        state.set_focus_within(true, now + Duration::from_secs(1));
        state.tick(now + Duration::from_secs(6));

        assert!(state.has_visible());
    }

    #[test]
    fn inactive_window_pauses_the_stack() {
        let now = Instant::now();
        let mut state = ToastState::default();
        push(&mut state, Toast::info("Short"), now);

        state.set_window_active(false, now + Duration::from_secs(2));
        state.tick(now + Duration::from_secs(6));

        let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
        assert_eq!(titles, vec!["Short"]);

        state.set_window_active(true, now + Duration::from_secs(9));
        state.tick(now + Duration::from_secs(12));

        assert!(state.visible().next().is_none());
    }

    #[test]
    fn modal_active_pauses_the_stack() {
        let now = Instant::now();
        let mut state = ToastState::default();
        push(&mut state, Toast::info("Short"), now);

        state.set_modal_active(true, now + Duration::from_secs(1));
        state.tick(now + Duration::from_secs(6));

        assert!(state.has_visible());
    }

    #[test]
    fn dismiss_promotes_the_next_queued_toast_immediately() {
        let now = Instant::now();
        let mut state = ToastState::default();
        for index in 0..4 {
            push(&mut state, Toast::info(format!("Toast {index}")), now);
        }
        assert_eq!(state.visible().count(), MAX_VISIBLE_TOASTS);

        let newest = state.visible().next().map(|item| item.id()).unwrap();
        state.dismiss(newest, now);

        assert_eq!(state.visible().count(), MAX_VISIBLE_TOASTS);
    }

    #[test]
    fn should_subscribe_when_any_toast_is_visible() {
        let now = Instant::now();
        let mut state = ToastState::default();

        assert!(!state.should_subscribe());

        push(&mut state, Toast::info("Toast"), now);

        assert!(state.should_subscribe());
    }

    #[test]
    fn tick_expires_toasts_when_not_paused() {
        let now = Instant::now();
        let mut state = ToastState::default();
        push(&mut state, Toast::info("Toast"), now);

        state.tick(now + Duration::from_secs(5));

        assert!(state.visible().next().is_none());
    }

    #[test]
    fn error_toast_shows_only_the_safe_summary() {
        let error = UserFacingError::custom("catalog", "Record not found (record_id: r1)");

        let toast: Toast = Toast::error(error);

        assert_eq!(toast.title(), "Record not found");
        assert_eq!(toast.body(), None);
        assert_eq!(toast.tone(), ToastTone::Danger);
    }

    #[test]
    fn toast_item_implements_presentation_contract() {
        let now = Instant::now();
        let mut state = ToastState::default();
        let id = push(
            &mut state,
            Toast::success("Project created").with_body("details"),
            now,
        );

        let item = state.visible().next().expect("toast is visible");

        assert_eq!(ToastPresentation::id(item), id);
        assert_eq!(item.title(), "Project created");
        assert_eq!(item.body(), Some("details"));
        assert_eq!(item.tone(), ToastTone::Success);
    }

    #[test]
    fn window_scoped_toast_renders_only_in_its_origin_window() {
        let now = Instant::now();
        let mut state = ToastState::<()>::default();
        let (a, b) = (window::Id::unique(), window::Id::unique());
        state.push(Toast::info("A only"), now, Some(a));

        assert_eq!(state.visible_for(a, Some(a)).count(), 1);
        assert_eq!(state.visible_for(b, Some(a)).count(), 0);
    }

    #[test]
    fn global_toast_renders_once_in_the_active_window_only() {
        let now = Instant::now();
        let mut state = ToastState::<()>::default();
        let (a, b) = (window::Id::unique(), window::Id::unique());
        // Originates in `a`, but only the *active* window should show it.
        state.push(Toast::info("Everywhere").global(), now, Some(a));

        assert_eq!(state.visible_for(a, Some(b)).count(), 0);
        assert_eq!(state.visible_for(b, Some(b)).count(), 1);
    }

    #[test]
    fn origin_less_toast_falls_back_to_global_rendering() {
        let now = Instant::now();
        let mut state = ToastState::<()>::default();
        let (a, b) = (window::Id::unique(), window::Id::unique());
        state.push(Toast::info("No context"), now, None);

        assert_eq!(state.visible_for(a, Some(b)).count(), 0);
        assert_eq!(state.visible_for(b, Some(b)).count(), 1);
    }
}
