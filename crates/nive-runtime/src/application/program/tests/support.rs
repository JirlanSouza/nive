use std::borrow::Cow;
use std::time::Duration;

use iced::Task;

use crate::application::{
    Application, ApplicationConfig, CloseDecision, Context, Effect, ExitDecision, MessageContext,
    RuntimeEvent, WindowContext,
};
use crate::{
    BootstrapSpec, DialogDismiss, DialogRequest, NamedShortcutKey, ScreenView, ShortcutBinding,
    ShortcutMap, ShortcutModifiers, WindowSpec,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum TestWindow {
    Main,
    Secondary,
    Multiple,
    Auxiliary,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TestMessage {
    Shortcut,
    Action,
    LegacyShortcut,
}

#[derive(Debug, Clone)]

pub(super) struct TestApp {
    pub(super) cancel_exit: bool,
    pub(super) close_requests: usize,
    pub(super) rejections: usize,
    pub(super) show_dialog: bool,
    pub(super) last_message_context: Option<MessageContext<TestWindow>>,
}

#[derive(Debug)]
pub(super) struct BootstrapTestApp {
    pub(super) bootstrap: String,
}

#[derive(Debug)]
pub(super) struct PendingInitTaskApp;

impl Application for TestApp {
    type Message = TestMessage;
    type Window = TestWindow;
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("test")
            .window(TestWindow::Main, WindowSpec::app())
            .window(TestWindow::Secondary, WindowSpec::app())
            .window(TestWindow::Multiple, WindowSpec::app().multiple())
            .window(TestWindow::Auxiliary, WindowSpec::auxiliary())
            .initial_window(TestWindow::Main)
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, Effect<Self::Message, Self::Window>) {
        (
            Self {
                cancel_exit: false,
                close_requests: 0,
                rejections: 0,
                show_dialog: false,
                last_message_context: None,
            },
            Effect::none(),
        )
    }

    fn update(
        &mut self,
        _context: Context<'_, Self::Window>,
        message_context: MessageContext<Self::Window>,
        _message: Self::Message,
    ) -> Effect<Self::Message, Self::Window> {
        self.last_message_context = Some(message_context);
        Effect::none()
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        let base = iced::widget::container(iced::widget::Column::with_children(vec![
            nive_ui::widgets::button::primary("First")
                .id(iced::widget::Id::new("runtime-focus-first"))
                .on_press(TestMessage::Action)
                .into(),
            nive_ui::widgets::Input::new("Value", "")
                .id(iced::widget::Id::new("runtime-focus-input"))
                .on_change(|_| TestMessage::Action)
                .into(),
            nive_ui::widgets::button::primary("Second")
                .id(iced::widget::Id::new("runtime-focus-second"))
                .on_press(TestMessage::Action)
                .into(),
        ]))
        .width(iced::Length::Fill)
        .height(iced::Length::Fill);
        if self.show_dialog {
            ScreenView::new(base).dialog(
                DialogRequest::new(iced::widget::text("dialog"))
                    .dismiss(DialogDismiss::escape(TestMessage::Shortcut)),
            )
        } else {
            ScreenView::new(base)
        }
    }

    fn window_title<'a>(
        &'a self,
        _context: Context<'a, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> Cow<'a, str> {
        Cow::Borrowed("Test")
    }

    fn shortcuts(&self, _context: Context<'_, Self::Window>) -> ShortcutMap<Self::Message> {
        ShortcutMap::new()
            .bind(
                ShortcutBinding::character('k', ShortcutModifiers::CONTROL),
                TestMessage::Shortcut,
            )
            .bind(
                ShortcutBinding::named(NamedShortcutKey::Tab, ShortcutModifiers::NONE),
                TestMessage::Shortcut,
            )
    }

    fn on_runtime_event(
        &mut self,
        _context: Context<'_, Self::Window>,
        event: RuntimeEvent<Self::Window>,
    ) -> Effect<Self::Message, Self::Window> {
        if matches!(event, RuntimeEvent::CommandRejected(_)) {
            self.rejections += 1;
        }

        Effect::none()
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
            ExitDecision::Exit
        }
    }
}

impl Application for BootstrapTestApp {
    type Message = TestMessage;
    type Window = TestWindow;
    type Bootstrap = String;

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("bootstrap-test")
            .window(TestWindow::Main, WindowSpec::app())
            .initial_window(TestWindow::Main)
            .bootstrap(
                BootstrapSpec::new(|| Task::done(Ok(String::from("services"))))
                    .minimum_duration(Duration::ZERO),
            )
    }

    fn init(
        _context: Context<'_, Self::Window>,
        bootstrap: Self::Bootstrap,
    ) -> (Self, Effect<Self::Message, Self::Window>) {
        (Self { bootstrap }, Effect::none())
    }

    fn update(
        &mut self,
        _context: Context<'_, Self::Window>,
        _message_context: MessageContext<Self::Window>,
        _message: Self::Message,
    ) -> Effect<Self::Message, Self::Window> {
        Effect::none()
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        ScreenView::new(iced::widget::text(""))
    }
}

impl Application for PendingInitTaskApp {
    type Message = TestMessage;
    type Window = TestWindow;
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("pending-init-test")
            .window(TestWindow::Main, WindowSpec::app())
            .initial_window(TestWindow::Main)
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, Effect<Self::Message, Self::Window>) {
        (
            Self,
            Effect::task(Task::perform(std::future::pending::<()>(), |_| {
                TestMessage::Shortcut
            })),
        )
    }

    fn update(
        &mut self,
        _context: Context<'_, Self::Window>,
        _message_context: MessageContext<Self::Window>,
        _message: Self::Message,
    ) -> Effect<Self::Message, Self::Window> {
        Effect::none()
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        ScreenView::new(iced::widget::text(""))
    }
}

#[cfg(feature = "devtools")]
impl crate::inspect::Inspect for TestApp {
    fn inspect(
        &mut self,
        _: &mut crate::inspect::InspectPath,
        _: &mut dyn crate::inspect::InspectSink,
    ) {
    }
}

#[cfg(feature = "devtools")]
impl crate::devtools::DevtoolsApp for TestApp {
    type State = Self;

    fn devtool_state_mut(&mut self) -> &mut Self {
        self
    }
}
