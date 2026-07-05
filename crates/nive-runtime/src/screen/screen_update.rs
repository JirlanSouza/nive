use iced::Task;

/// The return value of a screen or component action: an Iced task, an
/// optional semantic output for the parent, and any toasts to surface.
///
/// Generic over the toast payload so screens can use their own toast type;
/// defaults to [`Toast`](crate::Toast). `ScreenEffect` stays below runtime
/// lifecycle authority: it cannot carry window commands, theme changes, or
/// application exit directly. A screen that needs its parent to act on those
/// asks for it through a semantic `Output` value.
#[derive(Debug)]
pub struct ScreenEffect<Message, Output = (), Toast = crate::Toast> {
    /// The Iced task to run after the action.
    pub task: Task<Message>,
    /// The action's typed output for the parent, if any.
    pub output: Option<Output>,
    /// Toasts to surface from this action, in order.
    pub toasts: Vec<Toast>,
}

impl<Message, Output, Toast> ScreenEffect<Message, Output, Toast> {
    pub fn none() -> Self {
        Self {
            task: Task::none(),
            output: None,
            toasts: Vec::new(),
        }
    }

    pub fn task(task: Task<Message>) -> Self {
        Self {
            task,
            output: None,
            toasts: Vec::new(),
        }
    }

    pub fn output(output: Output) -> Self {
        Self {
            task: Task::none(),
            output: Some(output),
            toasts: Vec::new(),
        }
    }

    pub fn toast(toast: Toast) -> Self {
        Self {
            task: Task::none(),
            output: None,
            toasts: vec![toast],
        }
    }

    pub fn with_task(mut self, task: Task<Message>) -> Self
    where
        Message: 'static,
    {
        self.task = Task::batch([self.task, task]);
        self
    }

    pub fn with_output(mut self, output: Output) -> Self {
        if self.output.is_none() {
            self.output = Some(output);
        }
        self
    }

    pub fn with_toast(mut self, toast: Toast) -> Self {
        self.toasts.push(toast);
        self
    }

    pub fn map_message<N>(
        self,
        map: impl FnMut(Message) -> N + Send + 'static,
    ) -> ScreenEffect<N, Output, Toast>
    where
        Message: Send + 'static,
        N: Send + 'static,
    {
        ScreenEffect {
            task: self.task.map(map),
            output: self.output,
            toasts: self.toasts,
        }
    }

    pub fn map_output<P>(self, map: impl FnOnce(Output) -> P) -> ScreenEffect<Message, P, Toast> {
        ScreenEffect {
            task: self.task,
            output: self.output.map(map),
            toasts: self.toasts,
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
            output: self.output.or(other.output),
            toasts,
        }
    }
}

impl<Message, Output, Toast> Default for ScreenEffect<Message, Output, Toast> {
    fn default() -> Self {
        Self::none()
    }
}

#[cfg(test)]
mod screen_effect_tests {
    use super::*;

    #[test]
    fn merge_preserves_toasts_in_order() {
        let first = ScreenEffect::<(), (), &str>::toast("Saved");
        let second = ScreenEffect::<(), (), &str>::toast("Queued");

        let merged = first.merge(second);

        assert_eq!(merged.toasts, vec!["Saved", "Queued"]);
    }

    #[test]
    fn merge_preserves_first_output() {
        let first = ScreenEffect::<(), _, &str>::output("first");
        let second = ScreenEffect::<(), _, &str>::output("second");

        let merged = first.merge(second);

        assert_eq!(merged.output, Some("first"));
    }

    #[test]
    fn map_output_transforms_output_without_changing_toasts() {
        let effect = ScreenEffect::<(), _, &str>::output(2_u8).with_toast("Saved");

        let mapped = effect.map_output(|value| value.to_string());

        assert_eq!(mapped.output, Some(String::from("2")));
        assert_eq!(mapped.toasts, vec!["Saved"]);
    }
}
