use iced::Task;

/// The return value of a screen action: an Iced task, an optional outcome,
/// and any toasts to surface.
///
/// Generic over the toast payload so screens can use their own toast type;
/// defaults to [`Toast`](crate::Toast).
#[derive(Debug)]
pub struct ScreenUpdate<Message, Outcome, Toast = crate::Toast> {
    /// The Iced task to run after the action.
    pub task: Task<Message>,
    /// The action's typed outcome, if any.
    pub outcome: Option<Outcome>,
    /// Toasts to surface from this action, in order.
    pub toasts: Vec<Toast>,
}

impl<Message, Outcome, Toast> ScreenUpdate<Message, Outcome, Toast> {
    pub fn none() -> Self {
        Self {
            task: Task::none(),
            outcome: None,
            toasts: Vec::new(),
        }
    }

    pub fn task(task: Task<Message>) -> Self {
        Self {
            task,
            outcome: None,
            toasts: Vec::new(),
        }
    }

    pub fn outcome(outcome: Outcome) -> Self {
        Self {
            task: Task::none(),
            outcome: Some(outcome),
            toasts: Vec::new(),
        }
    }

    pub fn toast(toast: Toast) -> Self {
        Self {
            task: Task::none(),
            outcome: None,
            toasts: vec![toast],
        }
    }

    pub fn merge(self, other: Self) -> Self
    where
        Message: 'static,
    {
        let mut toasts = self.toasts;
        toasts.extend(other.toasts);

        Self {
            task: Task::batch([self.task, other.task]),
            outcome: self.outcome.or(other.outcome),
            toasts,
        }
    }
}

#[cfg(test)]
mod screen_update_tests {
    use super::*;

    #[test]
    fn merge_preserves_toasts_in_order() {
        let first = ScreenUpdate::<(), (), &str>::toast("Saved");
        let second = ScreenUpdate::<(), (), &str>::toast("Queued");

        let merged = first.merge(second);

        assert_eq!(merged.toasts, vec!["Saved", "Queued"]);
    }
}
