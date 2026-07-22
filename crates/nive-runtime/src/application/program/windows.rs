use iced::{window, Task};

use crate::application::program::{
    CoreMessage, NiveMessage, ProbeCatalogEntry, Program, RuntimeMessage,
};
use crate::application::{
    Application, CloseDecision, CommandRejected, CommandRejectionReason, Effect, ExitDecision,
    MessageSource, RuntimeEvent, WindowCommand,
};
use crate::lifecycle::WindowLifecycle;
use crate::{WindowCardinality, WindowHandle, WindowRole};

impl<A, P> Program<A, P>
where
    A: Application,
    P: ProbeCatalogEntry,
{
    pub(super) fn open_initial_windows(&mut self) -> Task<RuntimeMessage<A, P>> {
        self.core
            .initial_windows
            .clone()
            .into_iter()
            .fold(Task::none(), |task, kind| {
                task.chain(self.handle_window_command(WindowCommand::Open(kind)))
            })
    }

    pub(super) fn handle_window_command(
        &mut self,
        command: WindowCommand<A::Window>,
    ) -> Task<RuntimeMessage<A, P>> {
        match command {
            WindowCommand::Open(kind) => self.open_window(kind),
            WindowCommand::Close(window_id) => {
                if self.core.registry.get(window_id).is_some() {
                    self.request_close(window_id)
                } else {
                    self.reject(command, CommandRejectionReason::MissingWindow)
                }
            }
            WindowCommand::CloseAllKind(kind) => {
                let window_ids = self
                    .core
                    .registry
                    .all(kind)
                    .map(|handle| handle.id)
                    .collect::<Vec<_>>();

                if window_ids.is_empty() {
                    self.reject(command, CommandRejectionReason::MissingWindow)
                } else {
                    window_ids
                        .into_iter()
                        .fold(Task::none(), |task, window_id| {
                            task.chain(self.request_close(window_id))
                        })
                }
            }
            WindowCommand::Focus(window_id) => {
                if self.core.registry.set_focused(window_id).is_some() {
                    window::gain_focus(window_id)
                } else {
                    self.reject(command, CommandRejectionReason::MissingWindow)
                }
            }
            WindowCommand::Replace { current, next } => self.replace_window(current, next),
        }
    }

    pub(super) fn open_window(&mut self, kind: A::Window) -> Task<RuntimeMessage<A, P>> {
        let command = WindowCommand::Open(kind);
        if self.core.exiting {
            return self.reject(command, CommandRejectionReason::Exiting);
        }

        let Some(spec) = self.core.window_spec(kind) else {
            return self.reject(command, CommandRejectionReason::MissingWindowSpec);
        };

        if spec.cardinality == WindowCardinality::Single {
            if let Some(existing) = self.core.registry.latest(kind) {
                self.core.registry.set_focused(existing.id);
                return window::gain_focus(existing.id);
            }
            if self.core.registry.contains(kind) {
                return Task::none();
            }
        }

        let (window_id, task) = window::open(spec.settings(self.core.window_icon.clone()));
        self.core.registry.set_opening(WindowHandle {
            kind,
            id: window_id,
            role: spec.role,
        });

        task.map(|window_id| NiveMessage::Core(CoreMessage::WindowOpened(window_id)))
    }

    /// Opens `next` and closes `current` only once `next` has actually
    /// become available (opened, or an existing single-cardinality instance
    /// focused). `current` stays open and registered for the entire duration
    /// of a pending open, so a rejected or still-opening `next` never leaves
    /// the app without a window.
    pub(super) fn replace_window(
        &mut self,
        current: window::Id,
        next: A::Window,
    ) -> Task<RuntimeMessage<A, P>> {
        let command = WindowCommand::Replace { current, next };

        if self.core.exiting {
            return self.reject(command, CommandRejectionReason::Exiting);
        }
        let Some(current_handle) = self.core.registry.get(current) else {
            return self.reject(command, CommandRejectionReason::MissingWindow);
        };
        if current_handle.role != WindowRole::App
            || self.core.registry.lifecycle(current) != Some(WindowLifecycle::Open)
        {
            return self.reject(command, CommandRejectionReason::InvalidState);
        }
        let Some(spec) = self.core.window_spec(next) else {
            return self.reject(command, CommandRejectionReason::MissingWindowSpec);
        };
        if spec.role != WindowRole::App {
            return self.reject(command, CommandRejectionReason::InvalidState);
        }

        if spec.cardinality == WindowCardinality::Single {
            if let Some(existing) = self.core.registry.latest(next) {
                if existing.id == current {
                    return self.reject(command, CommandRejectionReason::InvalidState);
                }
                self.core.registry.set_focused(existing.id);
                return window::gain_focus(existing.id).chain(self.request_close(current));
            }
            if let Some(opening) = self.core.registry.opening(next) {
                self.core.pending_replacements.insert(opening.id, current);
                return Task::none();
            }
        }

        let (next_id, task) = window::open(spec.settings(self.core.window_icon.clone()));
        self.core.registry.set_opening(WindowHandle {
            kind: next,
            id: next_id,
            role: spec.role,
        });
        self.core.pending_replacements.insert(next_id, current);

        task.map(|window_id| NiveMessage::Core(CoreMessage::WindowOpened(window_id)))
    }

    pub(super) fn request_close(&mut self, window_id: window::Id) -> Task<RuntimeMessage<A, P>> {
        let Some(window) = self.core.window_context(window_id) else {
            return Task::none();
        };

        if window.role == WindowRole::Auxiliary {
            return window::close(window_id);
        }

        if self.core.pending_app_closes.contains(&window_id) {
            return Task::none();
        }

        if self.core.effective_app_window_count() <= 1 {
            return self.request_exit();
        }

        let context = self.core.context();
        let Some(app) = self.app.as_mut() else {
            return Task::none();
        };
        match app.on_window_close_requested(context, window) {
            CloseDecision::Close => {
                self.core.pending_app_closes.insert(window_id);
                window::close(window_id)
            }
            CloseDecision::Cancel => Task::none(),
            CloseDecision::Defer(task) => {
                self.core.pending_app_closes.insert(window_id);
                task.map(|message| NiveMessage::App {
                    window_id: None,
                    source: MessageSource::Task,
                    message,
                })
                .chain(Task::done(NiveMessage::Core(CoreMessage::ConfirmClose(
                    window_id,
                ))))
            }
        }
    }

    pub(super) fn request_exit(&mut self) -> Task<RuntimeMessage<A, P>> {
        if self.core.exiting {
            return Task::none();
        }

        let Some(app) = self.app.as_mut() else {
            self.core.exiting = true;
            if let Some(bootstrap) = self.bootstrap.as_mut() {
                bootstrap.controller.cancel();
            }
            return iced::exit();
        };
        let context = self.core.context();
        match app.on_exit_requested(context) {
            ExitDecision::Exit => self.accept_exit(),
            ExitDecision::Cancel => Task::none(),
            ExitDecision::Defer(task) => {
                self.core.exiting = true;
                task.map(|message| NiveMessage::App {
                    window_id: None,
                    source: MessageSource::Task,
                    message,
                })
                .chain(Task::done(NiveMessage::Core(CoreMessage::ConfirmExit)))
            }
        }
    }

    pub(super) fn accept_exit(&mut self) -> Task<RuntimeMessage<A, P>> {
        self.core.exiting = true;
        let close_auxiliary = self
            .core
            .registry
            .handles()
            .filter(|handle| handle.role == WindowRole::Auxiliary)
            .fold(Task::none(), |task, handle| {
                task.chain(window::close(handle.id))
            });
        #[cfg(feature = "devtools")]
        let close_devtools = self
            .devtools
            .as_ref()
            .and_then(|devtools| devtools.window_id)
            .map(window::close)
            .unwrap_or(Task::none());
        #[cfg(not(feature = "devtools"))]
        let close_devtools = Task::none();

        close_auxiliary.chain(close_devtools).chain(iced::exit())
    }

    pub(super) fn reject(
        &self,
        command: WindowCommand<A::Window>,
        reason: CommandRejectionReason,
    ) -> Task<RuntimeMessage<A, P>> {
        Task::done(NiveMessage::Core(CoreMessage::Rejected(CommandRejected {
            command,
            reason,
        })))
    }

    pub(super) fn emit_runtime_event(
        &mut self,
        event: RuntimeEvent<A::Window>,
    ) -> Task<RuntimeMessage<A, P>> {
        let update: Effect<A::Message, A::Window> = {
            let Some(app) = self.app.as_mut() else {
                return Task::none();
            };
            let context = self.core.context();
            app.on_runtime_event(context, event).into()
        };
        self.apply_update(update)
    }

    pub(super) fn is_bootstrap_window(&self, window_id: window::Id) -> bool {
        self.bootstrap
            .as_ref()
            .is_some_and(|bootstrap| bootstrap.window_id == window_id)
    }
}
