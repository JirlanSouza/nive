use std::sync::Mutex;
use std::time::Instant;

use iced::{window, Subscription, Task};

use super::{
    Application, ApplicationConfig, CloseDecision, CommandRejected, CommandRejectionReason,
    Context, CoreEvent, Error, ExitDecision, Result, WindowCommand, WindowContext, WindowQuery,
};
use crate::{
    AppUpdate, RuntimeCommand, ThemeController, ThemeEvent, ToastPosition, ToastState,
    WindowCardinality, WindowHandle, WindowRegistry, WindowRole, WindowSpec,
};

pub(super) fn run<A: Application>() -> Result {
    let config = A::config();
    let fonts = config.fonts.clone();
    let default_font = config.default_font;
    let (program, initial_task) = Program::<A>::new(config)?;
    let boot = Mutex::new(Some((program, initial_task)));

    let mut daemon = iced::daemon(
        move || {
            boot.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take()
                .unwrap_or_else(|| panic!("Nive program booted more than once"))
        },
        Program::<A>::update,
        Program::<A>::view,
    )
    .title(Program::<A>::title)
    .theme(Program::<A>::theme)
    .subscription(Program::<A>::subscription)
    .default_font(default_font);

    for font in fonts {
        daemon = daemon.font(font);
    }

    daemon.run().map_err(Error::from)
}

struct Program<A: Application> {
    core: NiveCore<A::Window>,
    app: A,
}

type RuntimeMessage<A> = NiveMessage<<A as Application>::Window, <A as Application>::Message>;
type ProgramBoot<A> = (Program<A>, Task<RuntimeMessage<A>>);

#[derive(Debug, Clone)]
enum NiveMessage<K, M> {
    Core(CoreMessage<K>),
    App {
        window_id: Option<window::Id>,
        message: M,
    },
}

#[derive(Debug, Clone)]
enum CoreMessage<K> {
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    WindowFocused(window::Id),
    WindowCloseRequested(window::Id),
    Theme(ThemeEvent),
    ConfirmClose(window::Id),
    ConfirmExit,
    Rejected(CommandRejected<K>),
}

struct NiveCore<K> {
    app_id: String,
    app_name: String,
    windows: Vec<(K, WindowSpec)>,
    initial_windows: Vec<K>,
    registry: WindowRegistry<K>,
    theme: ThemeController,
    exiting: bool,
    window_icon: Option<window::Icon>,
    _toast_position: ToastPosition,
    toasts: ToastState,
}

impl<A> Program<A>
where
    A: Application,
{
    fn new(mut config: ApplicationConfig<A::Window, A::Bootstrap>) -> Result<ProgramBoot<A>> {
        let bootstrap = config
            .immediate_bootstrap
            .take()
            .ok_or(Error::BootstrapUnavailable)?;
        let core = NiveCore::new(&config);
        let context = core.context();
        let (app, update) = A::init(context, bootstrap);
        let mut program = Self { core, app };

        let update_task = program.apply_update(update);
        let initial_windows = program.core.initial_windows.clone();
        let initial_task = initial_windows
            .into_iter()
            .fold(Task::none(), |task, kind| {
                task.chain(program.handle_window_command(WindowCommand::Open(kind)))
            });
        let theme_task = program
            .core
            .theme
            .initial_task()
            .map(|event| NiveMessage::Core(CoreMessage::Theme(event)));

        Ok((
            program,
            Task::batch([update_task.chain(initial_task), theme_task]),
        ))
    }

    fn update(
        &mut self,
        message: NiveMessage<A::Window, A::Message>,
    ) -> Task<NiveMessage<A::Window, A::Message>> {
        match message {
            NiveMessage::Core(message) => self.update_core(message),
            NiveMessage::App { window_id, message } => {
                let window = window_id.and_then(|id| self.core.window_context(id));
                let context = self.core.context();
                let update = self.app.update(context, window, message);
                self.apply_update(update)
            }
        }
    }

    fn view(
        &self,
        window_id: window::Id,
    ) -> nive_ui::Element<'_, NiveMessage<A::Window, A::Message>> {
        let Some(window) = self.core.window_context(window_id) else {
            return iced::widget::text("").into();
        };

        self.app
            .view(self.core.context(), window)
            .map(move |message| NiveMessage::App {
                window_id: Some(window_id),
                message,
            })
            .into_element()
    }

    fn title(&self, window_id: window::Id) -> String {
        self.core
            .window_context(window_id)
            .map(|window| {
                self.app
                    .window_title(self.core.context(), window)
                    .into_owned()
            })
            .unwrap_or_else(|| self.core.app_name.clone())
    }

    fn theme(&self, _window_id: window::Id) -> nive_ui::Theme {
        self.core.theme.effective()
    }

    fn subscription(&self) -> Subscription<NiveMessage<A::Window, A::Message>> {
        let window_events = window::events().filter_map(|(window_id, event)| match event {
            window::Event::Closed => Some(NiveMessage::Core(CoreMessage::WindowClosed(window_id))),
            window::Event::Focused => {
                Some(NiveMessage::Core(CoreMessage::WindowFocused(window_id)))
            }
            window::Event::CloseRequested => Some(NiveMessage::Core(
                CoreMessage::WindowCloseRequested(window_id),
            )),
            _ => None,
        });
        let theme = self
            .core
            .theme
            .subscription()
            .map(|event| NiveMessage::Core(CoreMessage::Theme(event)));
        let app = self
            .app
            .subscription(self.core.context())
            .map(|message| NiveMessage::App {
                window_id: None,
                message,
            });

        Subscription::batch([window_events, theme, app])
    }

    fn update_core(
        &mut self,
        message: CoreMessage<A::Window>,
    ) -> Task<NiveMessage<A::Window, A::Message>> {
        match message {
            CoreMessage::WindowOpened(window_id) => {
                let Some(handle) = self.core.registry.mark_opened(window_id) else {
                    return Task::none();
                };

                self.emit_core_event(CoreEvent::WindowOpened(handle.into()))
            }
            CoreMessage::WindowClosed(window_id) => {
                let Some(handle) = self.core.registry.get(window_id) else {
                    return Task::none();
                };
                self.core.registry.set_closed(window_id);

                let closed = self.emit_core_event(CoreEvent::WindowClosed(handle.into()));
                if handle.role == WindowRole::App && self.core.registry.app_window_count() == 0 {
                    let last_closed = self.emit_core_event(CoreEvent::LastAppWindowClosed);
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
                let Some(handle) = self.core.registry.set_focused(window_id) else {
                    return Task::none();
                };

                self.emit_core_event(CoreEvent::WindowFocused(handle.into()))
            }
            CoreMessage::WindowCloseRequested(window_id) => self.request_close(window_id),
            CoreMessage::Theme(event) => {
                if self.core.theme.handle(event) {
                    self.emit_core_event(CoreEvent::ThemeChanged(self.core.theme.effective()))
                } else {
                    Task::none()
                }
            }
            CoreMessage::ConfirmClose(window_id) => window::close(window_id),
            CoreMessage::ConfirmExit => self.accept_exit(),
            CoreMessage::Rejected(rejection) => {
                self.emit_core_event(CoreEvent::CommandRejected(rejection))
            }
        }
    }

    fn apply_update(
        &mut self,
        update: AppUpdate<A::Message, A::Window>,
    ) -> Task<NiveMessage<A::Window, A::Message>> {
        let (task, _, commands) = update.into_parts();
        let app_task = task.map(|message| NiveMessage::App {
            window_id: None,
            message,
        });
        let runtime_task = commands.into_iter().fold(Task::none(), |task, command| {
            task.chain(self.handle_runtime_command(command))
        });

        Task::batch([app_task, runtime_task])
    }

    fn handle_runtime_command(
        &mut self,
        command: RuntimeCommand<A::Window>,
    ) -> Task<NiveMessage<A::Window, A::Message>> {
        match command {
            RuntimeCommand::Toast(toast) => {
                self.core.toasts.push(toast, Instant::now());
                Task::none()
            }
            RuntimeCommand::Window(command) => self.handle_window_command(command),
            RuntimeCommand::Theme(preference) => {
                if self.core.theme.set_preference(preference) {
                    self.emit_core_event(CoreEvent::ThemeChanged(self.core.theme.effective()))
                } else {
                    Task::none()
                }
            }
            RuntimeCommand::Exit => self.request_exit(),
        }
    }

    fn handle_window_command(
        &mut self,
        command: WindowCommand<A::Window>,
    ) -> Task<NiveMessage<A::Window, A::Message>> {
        match command {
            WindowCommand::Open(kind) => self.open_window(kind),
            WindowCommand::Close(window_id) => {
                if self.core.registry.get(window_id).is_some() {
                    self.request_close(window_id)
                } else {
                    self.reject(command, CommandRejectionReason::MissingWindow)
                }
            }
            WindowCommand::CloseKind(kind) => {
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
            WindowCommand::FocusKind(kind) => {
                if let Some(handle) = self.core.registry.first(kind) {
                    self.core.registry.set_focused(handle.id);
                    window::gain_focus(handle.id)
                } else {
                    self.reject(command, CommandRejectionReason::MissingWindow)
                }
            }
        }
    }

    fn open_window(&mut self, kind: A::Window) -> Task<NiveMessage<A::Window, A::Message>> {
        let command = WindowCommand::Open(kind);
        if self.core.exiting {
            return self.reject(command, CommandRejectionReason::Exiting);
        }

        let Some(spec) = self.core.window_spec(kind) else {
            return self.reject(command, CommandRejectionReason::MissingWindowSpec);
        };

        if spec.cardinality == WindowCardinality::Single {
            if let Some(existing) = self.core.registry.first(kind) {
                self.core.registry.set_focused(existing.id);
                return window::gain_focus(existing.id);
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

    fn request_close(&mut self, window_id: window::Id) -> Task<NiveMessage<A::Window, A::Message>> {
        let Some(window) = self.core.window_context(window_id) else {
            return Task::none();
        };

        if window.role == WindowRole::Auxiliary {
            return window::close(window_id);
        }

        if self.core.registry.app_window_count() <= 1 {
            return self.request_exit();
        }

        let context = self.core.context();
        match self.app.on_window_close_requested(context, window) {
            CloseDecision::Close => window::close(window_id),
            CloseDecision::Cancel => Task::none(),
            CloseDecision::Defer(task) => task
                .map(|message| NiveMessage::App {
                    window_id: None,
                    message,
                })
                .chain(Task::done(NiveMessage::Core(CoreMessage::ConfirmClose(
                    window_id,
                )))),
        }
    }

    fn request_exit(&mut self) -> Task<NiveMessage<A::Window, A::Message>> {
        if self.core.exiting {
            return Task::none();
        }

        let context = self.core.context();
        match self.app.on_exit_requested(context) {
            ExitDecision::Accept => self.accept_exit(),
            ExitDecision::Cancel => Task::none(),
            ExitDecision::Defer(task) => {
                self.core.exiting = true;
                task.map(|message| NiveMessage::App {
                    window_id: None,
                    message,
                })
                .chain(Task::done(NiveMessage::Core(CoreMessage::ConfirmExit)))
            }
        }
    }

    fn accept_exit(&mut self) -> Task<NiveMessage<A::Window, A::Message>> {
        self.core.exiting = true;
        let close_auxiliary = self
            .core
            .registry
            .handles()
            .filter(|handle| handle.role == WindowRole::Auxiliary)
            .fold(Task::none(), |task, handle| {
                task.chain(window::close(handle.id))
            });

        close_auxiliary.chain(iced::exit())
    }

    fn reject(
        &self,
        command: WindowCommand<A::Window>,
        reason: CommandRejectionReason,
    ) -> Task<NiveMessage<A::Window, A::Message>> {
        Task::done(NiveMessage::Core(CoreMessage::Rejected(CommandRejected {
            command,
            reason,
        })))
    }

    fn emit_core_event(
        &mut self,
        event: CoreEvent<A::Window>,
    ) -> Task<NiveMessage<A::Window, A::Message>> {
        let context = self.core.context();
        let update = self.app.on_core_event(context, event);
        self.apply_update(update)
    }
}

impl<K> NiveCore<K>
where
    K: Copy + Eq,
{
    fn new<B>(config: &ApplicationConfig<K, B>) -> Self {
        Self {
            app_id: config.app_id.clone(),
            app_name: config.app_name.clone(),
            windows: config
                .windows
                .iter()
                .map(|registration| (registration.kind, registration.spec))
                .collect(),
            initial_windows: config.initial_windows.clone(),
            registry: WindowRegistry::new(),
            theme: ThemeController::new(config.theme_preference),
            exiting: false,
            window_icon: config.window_icon.clone(),
            _toast_position: config.toast_position,
            toasts: ToastState::default(),
        }
    }

    fn context(&self) -> Context<'_, K> {
        Context {
            app_id: self.app_id.as_str(),
            app_name: self.app_name.as_str(),
            theme: self.theme.effective(),
            theme_preference: self.theme.preference(),
            windows: WindowQuery {
                registry: &self.registry,
            },
            exiting: self.exiting,
        }
    }

    fn window_context(&self, window_id: window::Id) -> Option<WindowContext<K>> {
        self.registry.get(window_id).map(Into::into)
    }

    fn window_spec(&self, kind: K) -> Option<WindowSpec> {
        self.windows
            .iter()
            .find(|(registered_kind, _)| *registered_kind == kind)
            .map(|(_, spec)| *spec)
    }
}

impl<K> From<WindowHandle<K>> for WindowContext<K> {
    fn from(handle: WindowHandle<K>) -> Self {
        Self {
            id: handle.id,
            kind: handle.kind,
            role: handle.role,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;
    use crate::ScreenView;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TestWindow {
        Main,
        Secondary,
        Multiple,
        Missing,
    }

    #[derive(Debug, Clone)]
    struct TestApp {
        cancel_exit: bool,
        close_requests: usize,
        rejections: usize,
    }

    impl Application for TestApp {
        type Message = ();
        type Window = TestWindow;
        type Bootstrap = ();

        fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
            ApplicationConfig::new("test")
                .window(TestWindow::Main, WindowSpec::app())
                .window(TestWindow::Secondary, WindowSpec::app())
                .window(TestWindow::Multiple, WindowSpec::app().multiple())
                .initial_window(TestWindow::Main)
        }

        fn init(
            _context: Context<'_, Self::Window>,
            _bootstrap: Self::Bootstrap,
        ) -> (Self, AppUpdate<Self::Message, Self::Window>) {
            (
                Self {
                    cancel_exit: false,
                    close_requests: 0,
                    rejections: 0,
                },
                AppUpdate::none(),
            )
        }

        fn update(
            &mut self,
            _context: Context<'_, Self::Window>,
            _window: Option<WindowContext<Self::Window>>,
            _message: Self::Message,
        ) -> AppUpdate<Self::Message, Self::Window> {
            AppUpdate::none()
        }

        fn view(
            &self,
            _context: Context<'_, Self::Window>,
            _window: WindowContext<Self::Window>,
        ) -> ScreenView<'_, Self::Message> {
            ScreenView::new(iced::widget::text(""))
        }

        fn window_title<'a>(
            &'a self,
            _context: Context<'a, Self::Window>,
            _window: WindowContext<Self::Window>,
        ) -> Cow<'a, str> {
            Cow::Borrowed("Test")
        }

        fn on_core_event(
            &mut self,
            _context: Context<'_, Self::Window>,
            event: CoreEvent<Self::Window>,
        ) -> AppUpdate<Self::Message, Self::Window> {
            if matches!(event, CoreEvent::CommandRejected(_)) {
                self.rejections += 1;
            }

            AppUpdate::none()
        }

        fn on_window_close_requested(
            &mut self,
            _context: Context<'_, Self::Window>,
            _window: WindowContext<Self::Window>,
        ) -> CloseDecision<Self::Message> {
            self.close_requests += 1;
            CloseDecision::Close
        }

        fn on_exit_requested(
            &mut self,
            _context: Context<'_, Self::Window>,
        ) -> ExitDecision<Self::Message> {
            if self.cancel_exit {
                ExitDecision::Cancel
            } else {
                ExitDecision::Accept
            }
        }
    }

    fn program() -> Program<TestApp> {
        Program::new(TestApp::config())
            .map(|(program, _)| program)
            .unwrap_or_else(|error| panic!("test program failed: {error}"))
    }

    #[test]
    fn configured_initial_window_is_registered_as_opening() {
        let program = program();

        assert!(program.core.registry.contains(TestWindow::Main));
        assert_eq!(program.core.registry.app_window_count(), 1);
    }

    #[test]
    fn single_window_open_focuses_existing_instance() {
        let mut program = program();

        let _task = program.handle_window_command(WindowCommand::Open(TestWindow::Main));

        assert_eq!(program.core.registry.all(TestWindow::Main).count(), 1);
    }

    #[test]
    fn multiple_window_spec_preserves_each_instance() {
        let mut program = program();

        let _first = program.handle_window_command(WindowCommand::Open(TestWindow::Multiple));
        let _second = program.handle_window_command(WindowCommand::Open(TestWindow::Multiple));

        assert_eq!(program.core.registry.all(TestWindow::Multiple).count(), 2);
    }

    #[test]
    fn non_final_app_window_uses_close_hook() {
        let mut program = program();
        program.core.registry.set_opened(WindowHandle::new(
            TestWindow::Secondary,
            window::Id::unique(),
        ));
        let main_id = program
            .core
            .registry
            .first(TestWindow::Main)
            .map(|handle| handle.id)
            .unwrap_or_else(window::Id::unique);

        let _task = program.request_close(main_id);

        assert_eq!(program.app.close_requests, 1);
        assert!(!program.core.exiting);
    }

    #[test]
    fn last_app_window_uses_exit_hook() {
        let mut program = program();
        let main_id = program
            .core
            .registry
            .first(TestWindow::Main)
            .map(|handle| handle.id)
            .unwrap_or_else(window::Id::unique);

        let _task = program.request_close(main_id);

        assert_eq!(program.app.close_requests, 0);
        assert!(program.core.exiting);
    }

    #[test]
    fn cancelled_exit_keeps_runtime_active() {
        let mut program = program();
        program.app.cancel_exit = true;

        let _task = program.request_exit();

        assert!(!program.core.exiting);
    }

    #[test]
    fn command_rejection_is_forwarded_as_core_event() {
        let mut program = program();
        let rejection = CommandRejected {
            command: WindowCommand::Open(TestWindow::Missing),
            reason: CommandRejectionReason::MissingWindowSpec,
        };

        let _task = program.update_core(CoreMessage::Rejected(rejection));

        assert_eq!(program.app.rejections, 1);
    }
}
