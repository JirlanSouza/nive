use std::convert::Infallible;

use iced::Task;
use nive_ui::theme::ThemePreference;

use crate::{RequestCancellation, RequestTask, Toast, WindowCommand};

/// Alias for a window-kind type parameter that is never used.
pub type Never = Infallible;

/// Composes an Iced [`Task`] and ordered runtime commands (toasts, window
/// commands, theme changes, exit) produced by an
/// [`Application`](super::Application) hook.
///
/// Built via direct constructors ([`Effect::task`], [`Effect::window`],
/// [`Effect::toast`], [`Effect::theme`], [`Effect::exit`]) and composed with
/// `with_*` methods. The runtime drains the commands after each update.
#[derive(Debug)]
pub struct Effect<M, K = Never> {
    task: Task<M>,
    requests: Vec<RequestTask<M>>,
    cancellations: Vec<RequestCancellation>,
    runtime: Vec<RuntimeCommand<M, K>>,
}

pub(crate) type EffectParts<M, K> = (
    Task<M>,
    Vec<RequestTask<M>>,
    Vec<RequestCancellation>,
    Vec<RuntimeCommand<M, K>>,
);

/// A runtime-level command emitted by an [`Effect`], drained by the runner.
///
/// Kept internal to the crate: app authors build these indirectly through
/// [`Effect`] constructors and `with_*` methods rather than naming this type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RuntimeCommand<M, K> {
    /// Show a toast.
    Toast(Toast<M>),
    /// Apply a window command (open, focus, close, etc.).
    Window(WindowCommand<K>),
    /// Change the active theme preference.
    Theme(ThemePreference),
    /// Exit the application.
    Exit,
}

impl<M, K> RuntimeCommand<M, K> {
    fn map_message<N>(self, map: impl Fn(M) -> N) -> RuntimeCommand<N, K> {
        match self {
            RuntimeCommand::Toast(toast) => RuntimeCommand::Toast(toast.map_action(map)),
            RuntimeCommand::Window(command) => RuntimeCommand::Window(command),
            RuntimeCommand::Theme(preference) => RuntimeCommand::Theme(preference),
            RuntimeCommand::Exit => RuntimeCommand::Exit,
        }
    }
}

impl<M, K> Effect<M, K> {
    pub fn none() -> Self {
        Self {
            task: Task::none(),
            requests: Vec::new(),
            cancellations: Vec::new(),
            runtime: Vec::new(),
        }
    }

    pub fn task(task: Task<M>) -> Self {
        Self {
            task,
            requests: Vec::new(),
            cancellations: Vec::new(),
            runtime: Vec::new(),
        }
    }

    pub fn request(request: RequestTask<M>) -> Self {
        Self {
            task: Task::none(),
            requests: vec![request],
            cancellations: Vec::new(),
            runtime: Vec::new(),
        }
    }

    pub fn cancel(cancellation: Option<RequestCancellation>) -> Self {
        Self {
            task: Task::none(),
            requests: Vec::new(),
            cancellations: cancellation.into_iter().collect(),
            runtime: Vec::new(),
        }
    }

    pub fn window(command: WindowCommand<K>) -> Self {
        Self {
            task: Task::none(),
            requests: Vec::new(),
            cancellations: Vec::new(),
            runtime: vec![RuntimeCommand::Window(command)],
        }
    }

    pub fn toast(toast: Toast<M>) -> Self {
        Self {
            task: Task::none(),
            requests: Vec::new(),
            cancellations: Vec::new(),
            runtime: vec![RuntimeCommand::Toast(toast)],
        }
    }

    pub fn theme(preference: ThemePreference) -> Self {
        Self {
            task: Task::none(),
            requests: Vec::new(),
            cancellations: Vec::new(),
            runtime: vec![RuntimeCommand::Theme(preference)],
        }
    }

    pub fn exit() -> Self {
        Self {
            task: Task::none(),
            requests: Vec::new(),
            cancellations: Vec::new(),
            runtime: vec![RuntimeCommand::Exit],
        }
    }

    pub fn with_task(mut self, task: Task<M>) -> Self
    where
        M: 'static,
    {
        self.task = Task::batch([self.task, task]);
        self
    }

    pub fn with_window(mut self, command: WindowCommand<K>) -> Self {
        self.runtime.push(RuntimeCommand::Window(command));
        self
    }

    pub fn with_request(mut self, request: RequestTask<M>) -> Self {
        self.requests.push(request);
        self
    }

    pub fn with_cancellation(mut self, cancellation: RequestCancellation) -> Self {
        self.cancellations.push(cancellation);
        self
    }

    pub fn with_toast(mut self, toast: Toast<M>) -> Self {
        self.runtime.push(RuntimeCommand::Toast(toast));
        self
    }

    pub fn with_theme(mut self, preference: ThemePreference) -> Self {
        self.runtime.push(RuntimeCommand::Theme(preference));
        self
    }

    pub fn with_exit(mut self) -> Self {
        self.runtime.push(RuntimeCommand::Exit);
        self
    }

    pub(crate) fn into_parts(self) -> EffectParts<M, K> {
        (self.task, self.requests, self.cancellations, self.runtime)
    }

    pub fn map_message<N>(self, map: impl Fn(M) -> N + Send + Sync + 'static) -> Effect<N, K>
    where
        M: Send + 'static,
        N: Send + 'static,
    {
        let map = std::sync::Arc::new(map);
        let task_map = std::sync::Arc::clone(&map);
        Effect {
            task: self.task.map(move |value| task_map(value)),
            requests: self
                .requests
                .into_iter()
                .map(|request| {
                    request.map({
                        let map = std::sync::Arc::clone(&map);
                        move |message| map(message)
                    })
                })
                .collect(),
            cancellations: self.cancellations,
            runtime: self
                .runtime
                .into_iter()
                .map(|command| command.map_message(map.as_ref()))
                .collect(),
        }
    }

    pub fn merge(self, other: Self) -> Self
    where
        M: 'static,
    {
        let mut runtime = self.runtime;
        runtime.extend(other.runtime);
        let mut requests = self.requests;
        requests.extend(other.requests);
        let mut cancellations = self.cancellations;
        cancellations.extend(other.cancellations);

        Self {
            task: Task::batch([self.task, other.task]),
            requests,
            cancellations,
            runtime,
        }
    }
}

impl<M, K> Default for Effect<M, K> {
    fn default() -> Self {
        Self::none()
    }
}

/// Converts `()` into [`Effect::none()`] so that `Application::init` and
/// `Application::update` can return `()` when they have no side effects.
impl<M, K> From<()> for Effect<M, K> {
    fn from(_: ()) -> Self {
        Self::none()
    }
}

impl<M, K> From<RequestTask<M>> for Effect<M, K> {
    fn from(request: RequestTask<M>) -> Self {
        Self::request(request)
    }
}

#[cfg(test)]
mod effect_tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Window {
        Main,
    }

    #[test]
    fn merge_preserves_command_order() {
        let first = Effect::<(), Window>::window(WindowCommand::Open(Window::Main));
        let second = Effect::exit();

        let merged = first.merge(second);

        assert!(matches!(
            merged.into_parts().3.as_slice(),
            [
                RuntimeCommand::Window(WindowCommand::Open(Window::Main)),
                RuntimeCommand::Exit
            ]
        ));
    }

    #[test]
    fn with_combinators_compose_in_declared_order() {
        let effect = Effect::<(), Window>::task(Task::none())
            .with_toast(Toast::info("hi"))
            .with_window(WindowCommand::Open(Window::Main))
            .with_theme(ThemePreference::Dark)
            .with_exit();

        assert!(matches!(
            effect.into_parts().3.as_slice(),
            [
                RuntimeCommand::Toast(_),
                RuntimeCommand::Window(WindowCommand::Open(Window::Main)),
                RuntimeCommand::Theme(ThemePreference::Dark),
                RuntimeCommand::Exit,
            ]
        ));
    }

    #[test]
    fn unit_converts_to_none() {
        let effect: Effect<(), Window> = ().into();

        assert!(effect.into_parts().3.is_empty());
    }
}
