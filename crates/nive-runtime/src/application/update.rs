use std::convert::Infallible;

use iced::Task;
use nive_ui::theme::ThemePreference;

use crate::{Toast, WindowCommand};

/// Alias for an outcome type that can never occur.
pub type Never = Infallible;
/// The [`Update`] returned by [`Application`](super::Application) hooks, which
/// never produces a non-runtime outcome.
pub type AppUpdate<M, K> = Update<M, Never, K>;

/// Composes an Iced [`Task`], an optional operation outcome, and ordered
/// runtime commands (toasts, window commands, theme changes, exit).
///
/// Built fluently by application hooks via [`Update::task`], [`Update::toast`],
/// [`Update::window`], [`Update::theme`], and [`Update::exit`]. The runtime
/// drains the commands after each update.
#[derive(Debug)]
pub struct Update<M, O = Never, K = Never> {
    task: Task<M>,
    outcome: Option<O>,
    runtime: Vec<RuntimeCommand<K>>,
}

/// A runtime-level command emitted by an [`Update`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeCommand<K> {
    /// Show a toast.
    Toast(Toast),
    /// Apply a window command (open, focus, close, etc.).
    Window(WindowCommand<K>),
    /// Change the active theme preference.
    Theme(ThemePreference),
    /// Exit the application.
    Exit,
}

impl<M, O, K> Update<M, O, K> {
    pub fn none() -> Self {
        Self {
            task: Task::none(),
            outcome: None,
            runtime: Vec::new(),
        }
    }

    pub fn from_task(task: Task<M>) -> Self {
        Self {
            task,
            outcome: None,
            runtime: Vec::new(),
        }
    }

    pub fn from_outcome(outcome: O) -> Self {
        Self {
            task: Task::none(),
            outcome: Some(outcome),
            runtime: Vec::new(),
        }
    }

    pub fn task(mut self, task: Task<M>) -> Self
    where
        M: 'static,
    {
        self.task = Task::batch([self.task, task]);
        self
    }

    pub fn outcome(mut self, outcome: O) -> Self {
        if self.outcome.is_none() {
            self.outcome = Some(outcome);
        } else {
            duplicate_outcome("Update::outcome");
        }
        self
    }

    pub fn toast(mut self, toast: Toast) -> Self {
        self.runtime.push(RuntimeCommand::Toast(toast));
        self
    }

    pub fn window(mut self, command: WindowCommand<K>) -> Self {
        self.runtime.push(RuntimeCommand::Window(command));
        self
    }

    pub fn theme(mut self, preference: ThemePreference) -> Self {
        self.runtime.push(RuntimeCommand::Theme(preference));
        self
    }

    pub fn exit(mut self) -> Self {
        self.runtime.push(RuntimeCommand::Exit);
        self
    }

    pub fn outcome_ref(&self) -> Option<&O> {
        self.outcome.as_ref()
    }

    pub fn take_outcome(&mut self) -> Option<O> {
        self.outcome.take()
    }

    pub fn runtime_commands(&self) -> &[RuntimeCommand<K>] {
        self.runtime.as_slice()
    }

    pub(crate) fn into_parts(self) -> (Task<M>, Option<O>, Vec<RuntimeCommand<K>>) {
        (self.task, self.outcome, self.runtime)
    }

    pub fn map_message<N>(self, map: impl FnMut(M) -> N + Send + 'static) -> Update<N, O, K>
    where
        M: Send + 'static,
        N: Send + 'static,
    {
        Update {
            task: self.task.map(map),
            outcome: self.outcome,
            runtime: self.runtime,
        }
    }

    pub fn map_outcome<P>(self, map: impl FnOnce(O) -> P) -> Update<M, P, K> {
        Update {
            task: self.task,
            outcome: self.outcome.map(map),
            runtime: self.runtime,
        }
    }

    pub fn merge(self, other: Self) -> Self
    where
        M: 'static,
    {
        let mut runtime = self.runtime;
        runtime.extend(other.runtime);

        if self.outcome.is_some() && other.outcome.is_some() {
            duplicate_outcome("Update::merge");
        }

        Self {
            task: Task::batch([self.task, other.task]),
            outcome: self.outcome.or(other.outcome),
            runtime,
        }
    }
}

fn duplicate_outcome(operation: &'static str) {
    #[cfg(debug_assertions)]
    log::warn!(
        target: "nive_runtime::update",
        "duplicate update outcome ignored: operation={operation}"
    );
}

impl<M, O, K> Default for Update<M, O, K> {
    fn default() -> Self {
        Self::none()
    }
}

#[cfg(test)]
mod update_tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Window {
        Main,
    }

    #[test]
    fn merge_preserves_first_outcome_and_command_order() {
        let first = Update::<(), _, Window>::from_outcome("first")
            .window(WindowCommand::Open(Window::Main));
        let second = Update::from_outcome("second").exit();

        let merged = first.merge(second);

        assert_eq!(merged.outcome_ref(), Some(&"first"));
        assert!(matches!(
            merged.runtime_commands(),
            [
                RuntimeCommand::Window(WindowCommand::Open(Window::Main)),
                RuntimeCommand::Exit
            ]
        ));
    }

    #[test]
    fn maps_outcome_without_changing_runtime_commands() {
        let update = Update::<(), _, Window>::from_outcome(2_u8).exit();
        let mapped = update.map_outcome(|value| value.to_string());

        assert_eq!(mapped.outcome_ref(), Some(&String::from("2")));
        assert!(matches!(mapped.runtime_commands(), [RuntimeCommand::Exit]));
    }
}
