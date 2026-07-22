use std::collections::VecDeque;
use std::time::Instant;

use iced::window;

use super::{
    QueuedToast, Toast, ToastId, ToastItem, ToastMessage, ToastState, MAX_QUEUED_TOASTS,
    MAX_VISIBLE_TOASTS,
};

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

        // A dismiss always destroys whichever widget currently holds focus
        // (pressing a button re-focuses it first), so any `true` here is
        // necessarily stale: the host's focus-within tracker only re-scans on
        // the next real input event and can't detect that its previously
        // focused widget was just removed from the tree.
        if self.focus_within {
            self.set_focus_within(false, now);
        }

        // Dismissing the last visible toast unmounts the host's `mouse_area`
        // entirely (an empty stack renders no wrapper at all), so a `true`
        // hover here would otherwise survive to wrongly pause a future toast
        // with no widget left to ever report the pointer leaving.
        if self.visible.is_empty() && self.hover {
            self.set_hover(false, now);
        }
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
