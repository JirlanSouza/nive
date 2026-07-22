use std::time::Duration;

use nive_core::ToastTone;

use super::{Toast, ToastAction, ToastDuration};
use crate::UserFacingError;

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
    pub(super) fn coalesce_key(&self) -> Option<(&str, Option<&str>, ToastTone)> {
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
