use std::borrow::Cow;

use nive_runtime::prelude::ui::*;
use nive_runtime::SimpleApplication;
#[cfg(feature = "file-picker")]
use nive_runtime::{
    pick_file, pick_files, pick_folder, save_file, FileFilter, PickFileParams, SaveFileParams,
};
use nive_ui::prelude::text;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestWindow {
    Welcome,
    Workspace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    ) -> (Self, impl Into<Effect<Self::Message, Self::Window>>) {
        (Self, Effect::none())
    }

    fn update(
        &mut self,
        _context: Context<'_, Self::Window>,
        _message_context: MessageContext<Self::Window>,
        _message: Self::Message,
    ) -> impl Into<Effect<Self::Message, Self::Window>> {
        Effect::none()
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
    ) -> impl Into<Cow<'a, str>> + 'a {
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
fn prelude_exposes_app_facing_runtime_contracts() {
    let _: ActionId = ActionId::new("test.action");
    let _: ActionMap<TestMessage> =
        ActionMap::new().action(Action::new("test.action", "Test action", TestMessage));
    let _: ShortcutBinding = ShortcutBinding::named(
        NamedShortcutKey::Escape,
        ShortcutModifiers::CONTROL | ShortcutModifiers::SHIFT,
    );
    let _: SettingsConfig = SettingsConfig::file("settings.json");
    let _: RuntimeSession = RuntimeSession::new().with_theme_preference(ThemePreference::Dark);
    let _: WindowSession = WindowSession::new("workspace")
        .with_size(1280.0, 820.0)
        .with_position(120.0, 80.0);
    let _: WindowSessionSize = WindowSessionSize::new(1280.0, 820.0);
    let _: WindowSessionPosition = WindowSessionPosition::new(120.0, 80.0);
    let _: Point = Point::new(120.0, 80.0);
    let _: Effect<TestMessage, TestWindow> = Effect::none();
    let _: WindowCommand<TestWindow> = WindowCommand::Open(TestWindow::Workspace);
    let _: CloseDecision<TestMessage> = CloseDecision::Cancel;
    let _: ExitDecision<TestMessage> = ExitDecision::Exit;
    let _: ThemePreference = ThemePreference::System;
    let _: Toast = Toast::info("Ready");
    let _: WindowSpec = WindowSpec::app().session_key("welcome");
    let _: Size = Size::new(320.0, 240.0);
    let _: OperationId = OperationId::new("test.op");
    let _: OperationDescriptor = OperationDescriptor::new("test.op", "Test")
        .progress(OperationProgress::fraction(1, 4))
        .cancellable(true);
    let _: OperationRegistry = OperationRegistry::new();
    let _: DiagnosticEventLog = DiagnosticEventLog::new();
    let _: DiagnosticEvent = DiagnosticEvent::info("test", "ok");
    let _: DiagnosticEventKind = DiagnosticEventKind::Info;
    let snapshot: DiagnosticSnapshot = DiagnosticSnapshot::default();
    let _: std::result::Result<String, serde_json::Error> = snapshot.to_json();
}

#[test]
fn core_actions_interoperate_with_runtime_and_ui_without_conversion() {
    let core_action = nive_core::Action::new("test.action", "Test action", TestMessage)
        .description("Shared command")
        .shortcut(nive_core::ShortcutBinding::primary_character('k'));
    let runtime_action: &Action<TestMessage> = &core_action;

    let item = nive_ui::widgets::CommandPaletteItem::from_action(runtime_action);
    let toolbar = nive_ui::widgets::ToolbarAction::from_action(runtime_action);

    assert_eq!(item.id, "test.action");
    assert_eq!(item.activated(), Some(&TestMessage));
    let _: nive_ui::widgets::ToolbarAction<'_, TestMessage> = toolbar;
}

#[test]
fn prelude_exposes_message_context_and_runtime_event() {
    let _: MessageSource = MessageSource::View;
    let context: MessageContext<TestWindow> = MessageContext {
        window: None,
        source: MessageSource::Task,
    };
    assert_eq!(context.source, MessageSource::Task);
    assert!(context.window.is_none());

    let event: RuntimeEvent<TestWindow> = RuntimeEvent::LastAppWindowClosed;
    assert!(matches!(event, RuntimeEvent::LastAppWindowClosed));
}

#[test]
fn window_command_exposes_close_all_kind_and_replace() {
    let _: WindowCommand<TestWindow> = WindowCommand::CloseAllKind(TestWindow::Workspace);
    let _: WindowCommand<TestWindow> = WindowCommand::Replace {
        current: window::Id::unique(),
        next: TestWindow::Workspace,
    };
}

#[test]
fn screen_effect_carries_output_without_naming_window_command() {
    let effect: ScreenEffect<TestMessage, &'static str> =
        ScreenEffect::output("done").with_toast(Toast::info("ok"));

    assert_eq!(effect.output, Some("done"));
    assert_eq!(effect.toasts.len(), 1);
}

#[test]
fn runtime_area_modules_expose_consolidated_contracts() {
    let config = TestApp::config();
    let _: &[nive_runtime::application::WindowRegistration<TestWindow>] = config.windows();
    let _: nive_runtime::application::Effect<TestMessage, TestWindow> =
        nive_runtime::application::Effect::none();
    let _: nive_core::ActionId = nive_core::ActionId::new("test.action");
    let _: nive_runtime::input::ShortcutMap<TestMessage> = nive_runtime::input::ShortcutMap::new();
    let _: nive_runtime::lifecycle::WindowCommand<TestWindow> =
        nive_runtime::lifecycle::WindowCommand::Open(TestWindow::Workspace);
    let _: nive_runtime::state::Resource<()> = nive_runtime::state::Resource::idle();
    let _: nive_runtime::state::Operation<()> = nive_runtime::state::Operation::idle();
    let _: nive_runtime::feedback::Toast = nive_runtime::feedback::Toast::info("Ready");
    let _: nive_runtime::feedback::UserFacingError =
        nive_runtime::feedback::UserFacingError::custom("test", "Failed");
    let _: nive_runtime::screen::ScreenEffect<TestMessage, (), nive_runtime::feedback::Toast> =
        nive_runtime::screen::ScreenEffect::none();
    let _: nive_runtime::settings::RuntimeSession = nive_runtime::settings::RuntimeSession::new();
}

#[test]
fn effect_composes_runtime_commands_via_with_methods() {
    // Compiles without naming `RuntimeCommand`: the runtime's internal
    // ordered-command representation is not part of the app-author contract.
    let _effect = Effect::<TestMessage, TestWindow>::none()
        .with_toast(Toast::success("Saved"))
        .with_window(WindowCommand::Open(TestWindow::Workspace))
        .with_theme(ThemePreference::Dark)
        .with_exit();
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
    let mut r: Resource<()> = Resource::idle();
    let id1 = r.begin();
    let id2 = r.begin();
    assert_eq!(id1.get(), 1);
    assert_eq!(id2.get(), 2);
}

#[test]
fn toast_tone_and_position_are_unified_with_nive_core_and_nive_ui() {
    use nive_runtime::{ToastItem, ToastPosition, ToastState, ToastTone};

    // Only compiles if `nive_runtime::ToastTone` is the exact same type as
    // `nive_core::ToastTone`, not a duplicate local enum.
    let tone: ToastTone = nive_core::ToastTone::Success;
    assert_eq!(tone, ToastTone::Success);

    // Only compiles if `nive_runtime::ToastPosition` is the exact same type
    // as `nive_ui::widgets::overlays::ToastPosition`.
    let position: ToastPosition = nive_ui::widgets::overlays::ToastPosition::TopLeft;
    assert_eq!(position, ToastPosition::TopLeft);

    fn assert_toast_presentation<T: nive_core::ToastPresentation>() {}
    assert_toast_presentation::<ToastItem>();

    let mut state = ToastState::default();
    state.push(Toast::success("Ready"), std::time::Instant::now());
    let item = state.visible().next().expect("toast is visible");
    assert_eq!(nive_core::ToastPresentation::tone(item), ToastTone::Success);
}

#[cfg(feature = "devtools")]
#[test]
fn runtime_reexports_root_devtools_derive() {
    #[derive(nive_runtime::Inspect)]
    struct DerivedApp;

    let _ = DerivedApp;
}

#[cfg(feature = "file-picker")]
#[test]
fn file_picker_params_constructible_only_with_feature() {
    let pick: PickFileParams = PickFileParams {
        filters: Vec::new(),
        start_dir: None,
    };
    let save: SaveFileParams = SaveFileParams {
        filters: Vec::new(),
        start_dir: None,
        default_name: None,
    };
    let _: FileFilter = FileFilter {
        name: "Markdown",
        extensions: &["md"],
    };

    assert!(pick.filters.is_empty());
    assert!(pick.start_dir.is_none());
    assert!(save.filters.is_empty());
    assert!(save.start_dir.is_none());
    assert!(save.default_name.is_none());
}

#[cfg(feature = "file-picker")]
#[test]
fn file_picker_task_signatures_compile_with_feature() {
    use std::path::PathBuf;

    fn assert_task_type() -> Task<Option<PathBuf>> {
        pick_file(PickFileParams {
            filters: Vec::new(),
            start_dir: None,
        })
    }
    fn assert_files_task_type() -> Task<Option<Vec<PathBuf>>> {
        pick_files(PickFileParams {
            filters: Vec::new(),
            start_dir: None,
        })
    }
    fn assert_folder_task_type() -> Task<Option<PathBuf>> {
        pick_folder(None)
    }
    fn assert_save_task_type() -> Task<Option<PathBuf>> {
        save_file(SaveFileParams {
            filters: Vec::new(),
            start_dir: None,
            default_name: Some("untitled.md".to_string()),
        })
    }

    let _ = assert_task_type();
    let _ = assert_files_task_type();
    let _ = assert_folder_task_type();
    let _ = assert_save_task_type();
}

// Section 1 (Application trait defaults & return-type ergonomics) contract tests.

struct SingleWindowApp {
    counter: u32,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum SingleWindowMessage {
    Increment,
    Reset,
}

impl Application for SingleWindowApp {
    type Message = SingleWindowMessage;
    // Omitting `type Window` and `type Bootstrap` is impossible on stable
    // Rust (associated-type defaults require nightly). Apps instead set
    // both to `()` explicitly; `SimpleApplication` is implied via the
    // blanket impl, enabling the runtime's auto-registration path.
    type Window = ();
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        // Single-window, no-splash apps can build a config without calling
        // `.window(...)` or `.initial_window(...)`; the runtime detects
        // `A::Window = ()` and auto-registers one `WindowSpec::app()`.
        ApplicationConfig::new("single-window-app")
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, impl Into<Effect<Self::Message, Self::Window>>) {
        (Self { counter: 0 }, Effect::none())
    }

    fn update(
        &mut self,
        _context: Context<'_, Self::Window>,
        _message_context: MessageContext<Self::Window>,
        message: Self::Message,
    ) -> impl Into<Effect<Self::Message, Self::Window>> {
        match message {
            SingleWindowMessage::Increment => self.counter += 1,
            SingleWindowMessage::Reset => self.counter = 0,
        }
        // Returning `()` exercises `impl From<()> for Effect`.
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        ScreenView::new(text("single"))
    }

    fn window_title<'a>(
        &'a self,
        _context: Context<'a, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> impl Into<Cow<'a, str>> + 'a {
        // Returning a `&'static str` exercises `impl Into<Cow<'a, str>>`
        // without importing `Cow`.
        "Single Window App"
    }

    fn theme(
        &self,
        _context: Context<'_, Self::Window>,
        _window: Option<WindowContext<Self::Window>>,
    ) -> ThemePreference {
        ThemePreference::Dark
    }

    fn on_runtime_event(
        &mut self,
        _context: Context<'_, Self::Window>,
        _event: RuntimeEvent<Self::Window>,
    ) -> impl Into<Effect<Self::Message, Self::Window>> {
    }
}

#[test]
fn single_window_app_satisfies_simple_application_marker() {
    fn assert_simple<A: SimpleApplication>() {}

    assert_simple::<SingleWindowApp>();
}

#[test]
fn single_window_app_config_has_zero_explicit_windows_until_auto_registration() {
    let config = SingleWindowApp::config();

    assert_eq!(config.app_id(), "single-window-app");
    assert!(config.windows().is_empty());
    assert!(config.initial_windows().is_empty());
}

#[test]
fn single_window_app_window_title_signature_returns_into_cow() {
    // Compile-time assertion: `SingleWindowApp::window_title` returns an
    // `impl Into<Cow<'_, str>> + 'a` (the literal "Single Window App" the
    // impl produces satisfies this bound). The contract that matters is
    // that the impl type-checks against the new signature — verified just
    // by compiling this test (`SingleWindowApp` impl block above).
    fn _assert_application<A: Application>() {}
    _assert_application::<SingleWindowApp>();
}

#[test]
fn unit_return_compiles_as_effect_via_from_impl() {
    fn returns_unit() -> impl Into<Effect<SingleWindowMessage, ()>> {}

    let _: Effect<SingleWindowMessage, ()> = returns_unit().into();
}
