use std::collections::VecDeque;
use std::time::{Duration, Instant};

use nive_ui::ToastPresentation;

use crate::UserFacingError;

const MAX_VISIBLE_TOASTS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    #[default]
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ToastId(pub(crate) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastTone {
    Info,
    Success,
    Warning,
    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastDuration {
    Short,
    Medium,
    Long,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastRequest {
    title: String,
    body: Option<String>,
    tone: ToastTone,
    duration: ToastDuration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastItem {
    id: ToastId,
    request: ToastRequest,
    expires_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToastMessage {
    Dismiss { id: ToastId, now: Instant },
    Tick(Instant),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueuedToast {
    id: ToastId,
    request: ToastRequest,
}

#[derive(Debug, Default)]
pub struct ToastState {
    next_id: u64,
    visible: VecDeque<ToastItem>,
    queued: VecDeque<QueuedToast>,
    hidden_since: Option<Instant>,
}

impl ToastRequest {
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

    pub fn error(error: UserFacingError) -> Self {
        let mut toast = Self::danger(error.summary());
        if error.has_diagnostic_detail() {
            toast.body = Some(error.detail().to_owned());
        }
        toast.long()
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn long(mut self) -> Self {
        self.duration = ToastDuration::Long;
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
        }
    }
}

impl ToastDuration {
    pub fn as_duration(self) -> Duration {
        match self {
            ToastDuration::Short => Duration::from_secs(4),
            ToastDuration::Medium => Duration::from_secs(6),
            ToastDuration::Long => Duration::from_secs(8),
        }
    }
}

impl ToastItem {
    pub fn id(&self) -> ToastId {
        self.id
    }

    pub fn request(&self) -> &ToastRequest {
        &self.request
    }
}

impl ToastPresentation for ToastItem {
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

    fn tone(&self) -> nive_ui::ToastTone {
        self.request().tone().into()
    }
}

impl From<ToastTone> for nive_ui::ToastTone {
    fn from(tone: ToastTone) -> Self {
        match tone {
            ToastTone::Info => Self::Info,
            ToastTone::Success => Self::Success,
            ToastTone::Warning => Self::Warning,
            ToastTone::Danger => Self::Danger,
        }
    }
}

impl From<ToastPosition> for nive_ui::ToastPosition {
    fn from(position: ToastPosition) -> Self {
        match position {
            ToastPosition::TopLeft => Self::TopLeft,
            ToastPosition::TopRight => Self::TopRight,
            ToastPosition::BottomLeft => Self::BottomLeft,
            ToastPosition::BottomRight => Self::BottomRight,
        }
    }
}

impl ToastId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl ToastState {
    pub fn push(&mut self, request: ToastRequest, now: Instant) -> ToastId {
        let id = ToastId(self.next_id);
        self.next_id += 1;

        self.visible.push_front(ToastItem {
            id,
            expires_at: now + request.duration.as_duration(),
            request,
        });

        while self.visible.len() > MAX_VISIBLE_TOASTS {
            if let Some(item) = self.visible.pop_back() {
                self.queued.push_back(QueuedToast {
                    id: item.id,
                    request: item.request,
                });
            }
        }

        id
    }

    pub fn dismiss(&mut self, id: ToastId, now: Instant) {
        self.visible.retain(|item| item.id != id);
        self.queued.retain(|item| item.id != id);
        self.promote_queued(now);
    }

    pub fn expire(&mut self, now: Instant) {
        self.visible.retain(|item| item.expires_at > now);
        self.promote_queued(now);
    }

    pub fn pause_expiration(&mut self, now: Instant) {
        if self.hidden_since.is_none() {
            self.hidden_since = Some(now);
        }
    }

    pub fn resume_expiration(&mut self, now: Instant) {
        let Some(hidden_since) = self.hidden_since.take() else {
            return;
        };

        let hidden_duration = now.saturating_duration_since(hidden_since);
        for item in &mut self.visible {
            item.expires_at += hidden_duration;
        }
    }

    pub fn update(&mut self, message: ToastMessage) {
        match message {
            ToastMessage::Dismiss { id, now } => self.dismiss(id, now),
            ToastMessage::Tick(now) => self.expire(now),
        }
    }

    pub fn should_subscribe(&self) -> bool {
        self.has_visible()
    }

    pub fn handle_tick(&mut self, now: Instant, host_visible: bool) {
        if host_visible {
            self.resume_expiration(now);
            self.update(ToastMessage::Tick(now));
        } else {
            self.pause_expiration(now);
        }
    }

    pub fn has_visible(&self) -> bool {
        !self.visible.is_empty()
    }

    pub fn visible(&self) -> impl Iterator<Item = &ToastItem> {
        self.visible.iter()
    }

    fn promote_queued(&mut self, now: Instant) {
        while self.visible.len() < MAX_VISIBLE_TOASTS {
            let Some(queued) = self.queued.pop_front() else {
                return;
            };
            self.visible.push_back(ToastItem {
                id: queued.id,
                expires_at: now + queued.request.duration.as_duration(),
                request: queued.request,
            });
        }
    }
}

#[cfg(test)]
mod toast_tests {
    use super::*;

    #[test]
    fn push_keeps_newest_three_toasts() {
        let now = Instant::now();
        let mut state = ToastState::default();

        for index in 0..4 {
            state.push(ToastRequest::info(format!("Toast {index}")), now);
        }

        let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
        assert_eq!(titles, vec!["Toast 3", "Toast 2", "Toast 1"]);
    }

    #[test]
    fn dismiss_removes_matching_toast() {
        let now = Instant::now();
        let mut state = ToastState::default();
        let first = state.push(ToastRequest::info("First"), now);
        state.push(ToastRequest::info("Second"), now);

        state.dismiss(first, now);

        let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
        assert_eq!(titles, vec!["Second"]);
    }

    #[test]
    fn overflow_toasts_are_queued_and_promoted_after_dismiss() {
        let now = Instant::now();
        let mut state = ToastState::default();

        for index in 0..4 {
            state.push(ToastRequest::info(format!("Toast {index}")), now);
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
            state.push(ToastRequest::info(format!("Toast {index}")), now);
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
            ToastRequest::info("Info").duration.as_duration(),
            Duration::from_secs(4)
        );
        assert_eq!(
            ToastRequest::success("Success").duration.as_duration(),
            Duration::from_secs(4)
        );
        assert_eq!(
            ToastRequest::warning("Warning").duration.as_duration(),
            Duration::from_secs(6)
        );
        assert_eq!(
            ToastRequest::danger("Danger").duration.as_duration(),
            Duration::from_secs(8)
        );
    }

    #[test]
    fn expire_removes_elapsed_toasts() {
        let now = Instant::now();
        let mut state = ToastState::default();
        state.push(ToastRequest::info("Short"), now);
        state.push(ToastRequest::info("Long").long(), now);

        state.expire(now + Duration::from_secs(5));

        let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
        assert_eq!(titles, vec!["Long"]);
    }

    #[test]
    fn pause_and_resume_preserves_remaining_toast_duration() {
        let now = Instant::now();
        let mut state = ToastState::default();
        state.push(ToastRequest::info("Short"), now);

        state.pause_expiration(now + Duration::from_secs(1));
        state.resume_expiration(now + Duration::from_secs(3));
        state.expire(now + Duration::from_secs(5));

        let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
        assert_eq!(titles, vec!["Short"]);

        state.expire(now + Duration::from_secs(7));

        assert!(state.visible().next().is_none());
    }

    #[test]
    fn should_subscribe_when_any_toast_is_visible() {
        let now = Instant::now();
        let mut state = ToastState::default();

        assert!(!state.should_subscribe());

        state.push(ToastRequest::info("Toast"), now);

        assert!(state.should_subscribe());
    }

    #[test]
    fn handle_tick_expires_toasts_when_host_is_visible() {
        let now = Instant::now();
        let mut state = ToastState::default();
        state.push(ToastRequest::info("Toast"), now);

        state.handle_tick(now + Duration::from_secs(5), true);

        assert!(state.visible().next().is_none());
    }

    #[test]
    fn handle_tick_pauses_expiration_when_host_is_hidden() {
        let now = Instant::now();
        let mut state = ToastState::default();
        state.push(ToastRequest::info("Toast"), now);

        state.handle_tick(now + Duration::from_secs(2), false);
        state.handle_tick(now + Duration::from_secs(6), true);

        let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
        assert_eq!(titles, vec!["Toast"]);

        state.handle_tick(now + Duration::from_secs(9), true);

        assert!(state.visible().next().is_none());
    }

    #[test]
    fn toast_item_implements_presentation_contract() {
        let now = Instant::now();
        let mut state = ToastState::default();
        let id = state.push(
            ToastRequest::success("Project created").with_body("details"),
            now,
        );

        let item = state.visible().next().expect("toast is visible");

        assert_eq!(ToastPresentation::id(item), id);
        assert_eq!(item.title(), "Project created");
        assert_eq!(item.body(), Some("details"));
        assert_eq!(item.tone(), nive_ui::ToastTone::Success);
    }
}
