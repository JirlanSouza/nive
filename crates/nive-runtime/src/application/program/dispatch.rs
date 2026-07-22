use std::time::Instant;

use iced::{window, Task};

use crate::application::program::{
    CoreMessage, NiveMessage, ProbeCatalogEntry, Program, RuntimeMessage, RuntimeTask,
};
use crate::application::update::RuntimeCommand;
use crate::application::{Application, Effect, MessageSource, RuntimeEvent};
use crate::WindowRole;

impl<A, P> Program<A, P>
where
    A: Application,
    P: ProbeCatalogEntry,
{
    pub(super) fn update_core(
        &mut self,
        message: CoreMessage<A::Window>,
    ) -> Task<RuntimeMessage<A, P>> {
        match message {
            CoreMessage::WindowOpened(window_id) => {
                if self.is_bootstrap_window(window_id) {
                    return Task::none();
                }
                if self.is_devtools_window(window_id) {
                    return Task::none();
                }
                let Some(handle) = self.core.registry.mark_opened(window_id) else {
                    return Task::none();
                };

                let opened = self.emit_runtime_event(RuntimeEvent::WindowOpened(handle.into()));
                let Some(current_id) = self.core.pending_replacements.remove(&window_id) else {
                    return opened;
                };

                opened.chain(self.request_close(current_id))
            }
            CoreMessage::WindowClosed(window_id) => {
                self.core.pending_app_closes.remove(&window_id);
                // Clears a pending `Replace` handoff if its target closes
                // before ever finishing opening.
                self.core.pending_replacements.remove(&window_id);
                if self.is_bootstrap_window(window_id) {
                    if let Some(mut bootstrap) = self.bootstrap.take() {
                        bootstrap.controller.cancel();
                    }
                    self.core.exiting = true;
                    return iced::exit();
                }
                #[cfg(feature = "devtools")]
                {
                    if self.is_devtools_window(window_id) {
                        if let Some(devtools) = self.devtools.as_mut() {
                            devtools.window_id = None;
                        }
                        return Task::none();
                    }
                }
                let Some(handle) = self.core.registry.get(window_id) else {
                    return Task::none();
                };
                self.core.registry.set_closed(window_id);

                let closed = self.emit_runtime_event(RuntimeEvent::WindowClosed(handle.into()));
                if handle.role == WindowRole::App && self.core.registry.app_window_count() == 0 {
                    let last_closed = self.emit_runtime_event(RuntimeEvent::LastAppWindowClosed);
                    let exit = if self.core.exiting {
                        Task::none()
                    } else {
                        self.request_exit()
                    };

                    Task::batch([closed, last_closed, exit])
                } else {
                    closed
                }
            }
            CoreMessage::WindowFocused(window_id) => {
                if self.is_bootstrap_window(window_id) {
                    return Task::none();
                }
                if self.is_devtools_window(window_id) {
                    return Task::none();
                }
                let Some(handle) = self.core.registry.set_focused(window_id) else {
                    return Task::none();
                };
                self.core.toasts.set_window_active(true, Instant::now());

                self.emit_runtime_event(RuntimeEvent::WindowFocused(handle.into()))
            }
            CoreMessage::WindowUnfocused(window_id) => {
                if self.is_bootstrap_window(window_id) || self.is_devtools_window(window_id) {
                    return Task::none();
                }
                self.core.toasts.set_window_active(false, Instant::now());
                Task::none()
            }
            CoreMessage::WindowMoved(window_id, position) => {
                self.save_window_position(window_id, position)
            }
            CoreMessage::WindowResized(window_id, size) => self.save_window_size(window_id, size),
            CoreMessage::WindowCloseRequested(window_id) => {
                if self.is_bootstrap_window(window_id) {
                    self.core.exiting = true;
                    if let Some(bootstrap) = self.bootstrap.as_mut() {
                        bootstrap.controller.cancel();
                    }
                    iced::exit()
                } else {
                    #[cfg(feature = "devtools")]
                    if self.is_devtools_window(window_id) {
                        return window::close(window_id);
                    }
                    self.request_close(window_id)
                }
            }
            CoreMessage::Theme(event) => {
                let system_changed = self.core.theme.handle(event);
                if self.app.is_some() {
                    let resolved = self.resolve_theme_preference(None);
                    let effective_changed = self.core.theme.set_preference(resolved);
                    let event_task = if effective_changed {
                        self.emit_runtime_event(RuntimeEvent::ThemeChanged(
                            self.core.theme.effective(),
                        ))
                    } else {
                        Task::none()
                    };
                    return event_task;
                }
                if system_changed {
                    self.emit_runtime_event(RuntimeEvent::ThemeChanged(self.core.theme.effective()))
                } else {
                    Task::none()
                }
            }
            CoreMessage::ConfirmClose(window_id) => window::close(window_id),
            CoreMessage::ConfirmExit => self.accept_exit(),
            CoreMessage::Rejected(rejection) => {
                self.emit_runtime_event(RuntimeEvent::CommandRejected(rejection))
            }
            CoreMessage::ToastDismiss(id) => {
                self.core.toasts.dismiss(id, Instant::now());
                Task::none()
            }
            CoreMessage::ToastHoverEntered => {
                self.core.toasts.set_hover(true, Instant::now());
                Task::none()
            }
            CoreMessage::ToastHoverLeft => {
                self.core.toasts.set_hover(false, Instant::now());
                Task::none()
            }
            CoreMessage::ToastFocusWithinEntered => {
                self.core.toasts.set_focus_within(true, Instant::now());
                Task::none()
            }
            CoreMessage::ToastFocusWithinLeft => {
                self.core.toasts.set_focus_within(false, Instant::now());
                Task::none()
            }
            CoreMessage::ToastTick(now) => {
                self.core.toasts.tick(now);
                Task::none()
            }
            CoreMessage::ModalActive(active) => {
                self.core.toasts.set_modal_active(active, Instant::now());
                Task::none()
            }
            CoreMessage::KeyboardNavigation(navigation) => navigation.task(),
            CoreMessage::KeyboardEvent(event) => self.handle_keyboard_event(event),
            CoreMessage::SettingsSaved(result) => {
                if let Err(error) = result {
                    log::warn!(
                        target: "nive_runtime::settings",
                        "settings.save_failed path={} error={}",
                        error.path().display(),
                        error.detail()
                    );
                }
                Task::none()
            }
            #[cfg(feature = "devtools")]
            CoreMessage::ToggleDevtools => {
                let Some(devtools) = self.devtools.as_ref() else {
                    return Task::none();
                };
                if let Some(window_id) = devtools.window_id {
                    window::gain_focus(window_id)
                } else {
                    self.open_devtools_window()
                }
            }
        }
    }

    pub(super) fn apply_initial_update(
        &mut self,
        update: impl Into<Effect<A::Message, A::Window>>,
    ) -> (RuntimeTask<A, P>, RuntimeTask<A, P>) {
        let (task, commands) = update.into().into_parts();
        let app_task = task.map(|message| NiveMessage::App {
            window_id: None,
            source: MessageSource::Task,
            message,
        });
        let runtime_task = commands.into_iter().fold(Task::none(), |task, command| {
            task.chain(self.handle_runtime_command(command, None))
        });

        (app_task, runtime_task)
    }

    pub(super) fn apply_update(
        &mut self,
        update: impl Into<Effect<A::Message, A::Window>>,
    ) -> Task<RuntimeMessage<A, P>> {
        self.apply_update_from(update, None)
    }

    pub(super) fn apply_update_from(
        &mut self,
        update: impl Into<Effect<A::Message, A::Window>>,
        origin_window: Option<window::Id>,
    ) -> Task<RuntimeMessage<A, P>> {
        let (task, commands) = update.into().into_parts();
        let app_task = task.map(|message| NiveMessage::App {
            window_id: None,
            source: MessageSource::Task,
            message,
        });
        let runtime_task = commands.into_iter().fold(Task::none(), |task, command| {
            task.chain(self.handle_runtime_command(command, origin_window))
        });

        Task::batch([app_task, runtime_task])
    }

    pub(super) fn handle_runtime_command(
        &mut self,
        command: RuntimeCommand<A::Message, A::Window>,
        origin_window: Option<window::Id>,
    ) -> Task<RuntimeMessage<A, P>> {
        match command {
            RuntimeCommand::Toast(toast) => {
                self.core.toasts.push(toast, Instant::now(), origin_window);
                Task::none()
            }
            RuntimeCommand::Window(command) => self.handle_window_command(command),
            RuntimeCommand::Theme(preference) => {
                let preference_changed = self.core.theme.preference() != preference;
                // Persist the emitted preference so that future restarts and
                // `Application::theme` calls returning `System` re-resolve to it.
                let save_task = if preference_changed {
                    self.save_theme_preference(preference)
                } else {
                    Task::none()
                };
                // Consult `Application::theme` to apply the tie-break rule.
                let resolved = self.resolve_theme_preference(Some(preference));
                let effective_changed = self.core.theme.set_preference(resolved);
                let event_task = if effective_changed {
                    self.emit_runtime_event(RuntimeEvent::ThemeChanged(self.core.theme.effective()))
                } else {
                    Task::none()
                };

                Task::batch([event_task, save_task])
            }
            RuntimeCommand::Exit => self.request_exit(),
        }
    }
}
