use std::borrow::Cow;

use nive_runtime::prelude::*;
use nive_ui::prelude::text;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestWindow {
    Welcome,
    Workspace,
}

#[derive(Debug, Clone)]
struct TestMessage;

struct TestApp;

impl Application for TestApp {
    type Message = TestMessage;
    type Window = TestWindow;
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("test-app")
            .window(TestWindow::Welcome, WindowSpec::app().size(900.0, 640.0))
            .window(TestWindow::Workspace, WindowSpec::app().size(1280.0, 820.0))
            .initial_window(TestWindow::Welcome)
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, AppUpdate<Self::Message, Self::Window>) {
        (Self, AppUpdate::none())
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
        ScreenView::new(text("test"))
    }

    fn window_title<'a>(
        &'a self,
        _context: Context<'a, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> Cow<'a, str> {
        Cow::Borrowed("Test")
    }
}

#[test]
fn application_contract_is_implementable_on_stable_rust() {
    fn assert_application<A: Application>() {}

    assert_application::<TestApp>();
    let config = TestApp::config();

    assert_eq!(config.app_id(), "test-app");
    assert_eq!(config.app_name(), "test-app");
    assert_eq!(config.initial_windows(), &[TestWindow::Welcome]);
    assert_eq!(config.windows().len(), 2);
}

#[test]
fn update_composes_outcome_and_runtime_commands_in_order() {
    let update = Update::<TestMessage, &'static str, TestWindow>::none()
        .toast(Toast::success("Saved"))
        .window(WindowCommand::Open(TestWindow::Workspace))
        .theme(ThemePreference::Dark)
        .exit()
        .outcome("completed");

    assert_eq!(update.outcome_ref(), Some(&"completed"));
    assert!(matches!(
        update.runtime_commands(),
        [
            RuntimeCommand::Toast(_),
            RuntimeCommand::Window(WindowCommand::Open(TestWindow::Workspace)),
            RuntimeCommand::Theme(ThemePreference::Dark),
            RuntimeCommand::Exit,
        ]
    ));
}

#[test]
fn window_specs_expose_approved_defaults() {
    let app = WindowSpec::app();
    let auxiliary = WindowSpec::auxiliary();

    assert_eq!(app.role(), WindowRole::App);
    assert_eq!(app.cardinality(), WindowCardinality::Single);
    assert_eq!(app.size, Size::new(1024.0, 720.0));
    assert_eq!(app.min_size, Some(Size::new(640.0, 480.0)));

    assert_eq!(auxiliary.role(), WindowRole::Auxiliary);
    assert_eq!(auxiliary.cardinality(), WindowCardinality::Single);
    assert_eq!(auxiliary.size, Size::new(900.0, 640.0));
}

#[test]
fn request_ids_start_at_one_and_skip_zero() {
    let mut counter = RequestCounter::default();

    assert_eq!(counter.next().get(), 1);
    assert_eq!(counter.next().get(), 2);
}

#[cfg(feature = "devtools")]
#[test]
fn runtime_reexports_root_devtools_derive() {
    #[derive(nive_runtime::Devtools)]
    struct DerivedApp;

    fn assert_devtools<T: nive_runtime::devtools::Devtools>() {}

    assert_devtools::<DerivedApp>();
}
