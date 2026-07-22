#![allow(refining_impl_trait_internal)]
use std::borrow::Cow;
use std::time::Duration;

use super::shortcuts::{devtools_toggle_from_event, shortcut_message_from_event};
use super::*;
use crate::application::update::RuntimeCommand;
use crate::application::{
    ApplicationConfig, CloseDecision, CommandRejectionReason, Context, Effect, ExitDecision,
    MessageContext, RuntimeEvent, WindowCommand, WindowContext,
};
use crate::{
    Action, ActionMap, DialogDismiss, DialogRequest, NamedShortcutKey, ScreenView, ShortcutBinding,
    ShortcutMap, ShortcutModifiers, ThemePreference, Toast, WindowHandle, WindowSession,
};
use nive_ui::theme::{testing::ThemeTestGuard, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestWindow {
    Main,
    Secondary,
    Multiple,
    Auxiliary,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestMessage {
    Shortcut,
    Action,
    LegacyShortcut,
}

#[derive(Debug, Clone)]
struct TestApp {
    cancel_exit: bool,
    close_requests: usize,
    rejections: usize,
    show_dialog: bool,
    last_message_context: Option<MessageContext<TestWindow>>,
}

#[derive(Debug)]
struct BootstrapTestApp {
    bootstrap: String,
}

#[derive(Debug)]
struct PendingInitTaskApp;

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

fn program() -> Program<TestApp> {
    plain_program::<TestApp>()
        .map(|(program, _)| program)
        .unwrap_or_else(|error| panic!("test program failed: {error}"))
}

#[cfg(feature = "devtools")]
fn plain_program<A>() -> Result<ProgramBoot<A, NoProbe>>
where
    A: Application,
{
    Program::<A>::new(A::config(), None)
}

#[cfg(not(feature = "devtools"))]
fn plain_program<A>() -> Result<ProgramBoot<A, NoProbe>>
where
    A: Application,
{
    Program::<A>::new(A::config())
}

#[cfg(feature = "devtools")]
fn plain_program_with_config<A>(
    config: ApplicationConfig<A::Window, A::Bootstrap>,
) -> Result<ProgramBoot<A, NoProbe>>
where
    A: Application,
{
    Program::<A>::new(config, None)
}

#[cfg(not(feature = "devtools"))]
fn plain_program_with_config<A>(
    config: ApplicationConfig<A::Window, A::Bootstrap>,
) -> Result<ProgramBoot<A, NoProbe>>
where
    A: Application,
{
    Program::<A>::new(config)
}

#[cfg(feature = "devtools")]
fn devtools_program(config: DevtoolsConfig) -> Program<TestApp> {
    Program::new(
        TestApp::config(),
        Some(DevtoolsRuntime::<TestApp>::new(config)),
    )
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
fn runtime_settings_disabled_by_default() {
    let config = TestApp::config();

    assert!(config.configured_settings().is_none());
}

#[test]
fn runtime_session_loads_theme_preference_before_theme_controller() {
    let _guard = ThemeTestGuard::activate(Theme::Dark);
    let directory = tempfile::tempdir().expect("tempdir");
    let settings = SettingsConfig::file(directory.path().join("settings.json"));
    crate::settings::save_session(
        &settings,
        &RuntimeSession::new().with_theme_preference(ThemePreference::Dark),
    )
    .expect("settings should save");
    let config = TestApp::config()
        .theme_preference(ThemePreference::Light)
        .settings(settings);

    let (program, _task) = plain_program_with_config::<TestApp>(config)
        .unwrap_or_else(|error| panic!("test program failed: {error}"));

    assert_eq!(program.core.theme.preference(), ThemePreference::Dark);
}

#[test]
fn missing_session_file_falls_back_to_config_defaults() {
    let _guard = ThemeTestGuard::activate(Theme::Dark);
    let directory = tempfile::tempdir().expect("tempdir");
    let config = TestApp::config()
        .theme_preference(ThemePreference::Light)
        .settings(SettingsConfig::file(directory.path().join("missing.json")));

    let (program, _task) = plain_program_with_config::<TestApp>(config)
        .unwrap_or_else(|error| panic!("test program failed: {error}"));

    assert_eq!(program.core.theme.preference(), ThemePreference::Light);
}

#[test]
fn corrupt_session_file_falls_back_to_config_defaults() {
    let _guard = ThemeTestGuard::activate(Theme::Dark);
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("settings.json");
    std::fs::write(&path, "not-json").expect("write corrupt settings");
    let config = TestApp::config()
        .theme_preference(ThemePreference::Light)
        .settings(SettingsConfig::file(path));

    let (program, _task) = plain_program_with_config::<TestApp>(config)
        .unwrap_or_else(|error| panic!("test program failed: {error}"));

    assert_eq!(program.core.theme.preference(), ThemePreference::Light);
}

#[test]
fn unknown_session_version_falls_back_to_config_defaults() {
    let _guard = ThemeTestGuard::activate(Theme::Dark);
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("settings.json");
    std::fs::write(
        &path,
        r#"{"version":999,"session":{"theme_preference":"dark"}}"#,
    )
    .expect("write unsupported settings");
    let config = TestApp::config()
        .theme_preference(ThemePreference::Light)
        .settings(SettingsConfig::file(path));

    let (program, _task) = plain_program_with_config::<TestApp>(config)
        .unwrap_or_else(|error| panic!("test program failed: {error}"));

    assert_eq!(program.core.theme.preference(), ThemePreference::Light);
}

#[test]
fn theme_preference_change_schedules_session_save() {
    let _guard = ThemeTestGuard::activate(Theme::Dark);
    let directory = tempfile::tempdir().expect("tempdir");
    let config =
        TestApp::config().settings(SettingsConfig::file(directory.path().join("settings.json")));
    let (mut program, _task) = plain_program_with_config::<TestApp>(config)
        .unwrap_or_else(|error| panic!("test program failed: {error}"));

    let _task = program.handle_runtime_command(RuntimeCommand::Theme(ThemePreference::Light), None);

    let preference = program
        .core
        .settings
        .as_ref()
        .and_then(|settings| settings.session.theme_preference());
    assert_eq!(preference, Some(ThemePreference::Light));
}

#[test]
fn window_spec_exposes_session_key() {
    let spec = WindowSpec::app().session_key("workspace");

    assert_eq!(spec.configured_session_key(), Some("workspace"));
}

#[test]
fn windows_without_session_key_do_not_restore_session_state() {
    assert_eq!(WindowSpec::app().configured_session_key(), None);
}

#[test]
fn runtime_session_restores_window_size_and_position() {
    let directory = tempfile::tempdir().expect("tempdir");
    let settings = SettingsConfig::file(directory.path().join("settings.json"));
    crate::settings::save_session(
        &settings,
        &RuntimeSession::new().with_window(
            WindowSession::new("main")
                .with_size(1280.0, 820.0)
                .with_position(120.0, 80.0),
        ),
    )
    .expect("settings should save");
    let config = ApplicationConfig::new("restore-window")
        .window(TestWindow::Main, WindowSpec::app().session_key("main"))
        .initial_window(TestWindow::Main)
        .settings(settings);

    let (program, _task) = plain_program_with_config::<TestApp>(config)
        .unwrap_or_else(|error| panic!("test program failed: {error}"));
    let spec = program
        .core
        .window_spec(TestWindow::Main)
        .expect("window spec should exist");

    assert_eq!(spec.size, Size::new(1280.0, 820.0));
    assert!(matches!(
        spec.position,
        window::Position::Specific(position) if position == Point::new(120.0, 80.0)
    ));
}

#[test]
fn runtime_session_clamps_restored_window_size_to_spec_bounds() {
    let directory = tempfile::tempdir().expect("tempdir");
    let settings = SettingsConfig::file(directory.path().join("settings.json"));
    crate::settings::save_session(
        &settings,
        &RuntimeSession::new().with_window(WindowSession::new("main").with_size(320.0, 240.0)),
    )
    .expect("settings should save");
    let config = ApplicationConfig::new("restore-window")
        .window(
            TestWindow::Main,
            WindowSpec::app().min_size(640.0, 480.0).session_key("main"),
        )
        .initial_window(TestWindow::Main)
        .settings(settings);

    let (program, _task) = plain_program_with_config::<TestApp>(config)
        .unwrap_or_else(|error| panic!("test program failed: {error}"));
    let spec = program
        .core
        .window_spec(TestWindow::Main)
        .expect("window spec should exist");

    assert_eq!(spec.size, Size::new(640.0, 480.0));
}

#[test]
fn runtime_session_ignores_window_state_without_session_key() {
    let directory = tempfile::tempdir().expect("tempdir");
    let settings = SettingsConfig::file(directory.path().join("settings.json"));
    crate::settings::save_session(
        &settings,
        &RuntimeSession::new().with_window(WindowSession::new("main").with_size(1280.0, 820.0)),
    )
    .expect("settings should save");
    let config = ApplicationConfig::new("restore-window")
        .window(TestWindow::Main, WindowSpec::app())
        .initial_window(TestWindow::Main)
        .settings(settings);

    let (program, _task) = plain_program_with_config::<TestApp>(config)
        .unwrap_or_else(|error| panic!("test program failed: {error}"));
    let spec = program
        .core
        .window_spec(TestWindow::Main)
        .expect("window spec should exist");

    assert_eq!(spec.size, Size::new(1024.0, 720.0));
    assert!(matches!(spec.position, window::Position::Centered));
}

#[test]
fn window_resize_updates_runtime_session() {
    let directory = tempfile::tempdir().expect("tempdir");
    let config = ApplicationConfig::new("resize-window")
        .window(TestWindow::Main, WindowSpec::app().session_key("main"))
        .initial_window(TestWindow::Main)
        .settings(SettingsConfig::file(directory.path().join("settings.json")));
    let (mut program, _task) = plain_program_with_config::<TestApp>(config)
        .unwrap_or_else(|error| panic!("test program failed: {error}"));
    let window_id = main_window_id(&program);

    let _task = program.update_core(CoreMessage::WindowResized(
        window_id,
        Size::new(1360.0, 860.0),
    ));

    let size = program
        .core
        .settings
        .as_ref()
        .and_then(|settings| settings.session.window("main"))
        .and_then(WindowSession::size);
    assert_eq!(size, Some(Size::new(1360.0, 860.0)));
}

#[test]
fn window_move_updates_runtime_session() {
    let directory = tempfile::tempdir().expect("tempdir");
    let config = ApplicationConfig::new("move-window")
        .window(TestWindow::Main, WindowSpec::app().session_key("main"))
        .initial_window(TestWindow::Main)
        .settings(SettingsConfig::file(directory.path().join("settings.json")));
    let (mut program, _task) = plain_program_with_config::<TestApp>(config)
        .unwrap_or_else(|error| panic!("test program failed: {error}"));
    let window_id = main_window_id(&program);

    let _task = program.update_core(CoreMessage::WindowMoved(window_id, Point::new(140.0, 96.0)));

    let position = program
        .core
        .settings
        .as_ref()
        .and_then(|settings| settings.session.window("main"))
        .and_then(WindowSession::position);
    assert_eq!(position, Some(Point::new(140.0, 96.0)));
}

#[test]
fn duplicate_window_session_keys_are_reported_without_panicking() {
    let _guard = ThemeTestGuard::activate(Theme::Dark);
    let config = ApplicationConfig::new("duplicate-keys")
        .window(TestWindow::Main, WindowSpec::app().session_key("main"))
        .window(TestWindow::Secondary, WindowSpec::app().session_key("main"))
        .initial_window(TestWindow::Main);

    assert!(plain_program_with_config::<TestApp>(config).is_ok());
}

#[test]
fn initial_windows_open_without_waiting_for_init_task() {
    let (program, _task) = plain_program::<PendingInitTaskApp>()
        .unwrap_or_else(|error| panic!("test program failed: {error}"));

    assert!(program.core.registry.contains(TestWindow::Main));
    assert_eq!(program.core.registry.app_window_count(), 1);
}

#[cfg(feature = "devtools")]
#[test]
fn devtools_runtime_starts_closed_and_ready_for_shortcut() {
    let program = devtools_program(DevtoolsConfig::default());
    let devtools = program
        .devtools
        .as_ref()
        .unwrap_or_else(|| panic!("devtools runtime should be installed"));

    assert!(devtools.host.is_enabled());
    assert_eq!(devtools.window_id, None);
}

#[cfg(feature = "devtools")]
#[test]
fn devtools_open_config_creates_initial_auxiliary_window() {
    let program = devtools_program(DevtoolsConfig::from_env_value(Some("open")));

    assert!(program
        .devtools
        .as_ref()
        .is_some_and(|devtools| devtools.window_id.is_some()));
}

#[cfg(feature = "devtools")]
#[test]
fn devtools_config_applies_initial_panel_tab() {
    let program = devtools_program(
        DevtoolsConfig::default().with_initial_tab(crate::devtools::DevtoolsPanelTab::Operations),
    );
    let active_tab = program
        .devtools
        .as_ref()
        .and_then(|devtools| devtools.host.panel())
        .map(|panel| panel.active_tab);

    assert_eq!(
        active_tab,
        Some(crate::devtools::DevtoolsPanelTab::Operations)
    );
}

#[cfg(feature = "devtools")]
#[test]
fn closing_devtools_allows_shortcut_to_open_it_again() {
    let mut program = devtools_program(DevtoolsConfig::from_env_value(Some("open")));
    let first_window = program
        .devtools
        .as_ref()
        .and_then(|devtools| devtools.window_id)
        .unwrap_or_else(window::Id::unique);

    let _task = program.update_core(CoreMessage::WindowClosed(first_window));
    assert!(program
        .devtools
        .as_ref()
        .is_some_and(|devtools| devtools.window_id.is_none()));

    let _task = program.update_core(CoreMessage::ToggleDevtools);

    assert!(program
        .devtools
        .as_ref()
        .is_some_and(|devtools| devtools.window_id.is_some()));
}

#[cfg(feature = "devtools")]
#[test]
fn platform_shortcut_routes_to_devtools_toggle() {
    use iced::keyboard::key::{Code, Physical};
    use iced::keyboard::{Key, Location, Modifiers};

    let modifiers = if cfg!(target_os = "macos") {
        Modifiers::COMMAND | Modifiers::ALT
    } else {
        Modifiers::CTRL | Modifiers::ALT
    };
    let event = keyboard::Event::KeyPressed {
        key: Key::Character("i".into()),
        modified_key: Key::Character("i".into()),
        physical_key: Physical::Code(Code::KeyI),
        location: Location::Standard,
        modifiers,
        text: Some("i".into()),
        repeat: false,
    };

    assert!(matches!(
        devtools_toggle_from_event::<TestWindow, (), (), NoProbe>(event),
        Some(NiveMessage::Core(CoreMessage::ToggleDevtools))
    ));
}

fn key_pressed(
    key: keyboard::Key,
    modifiers: keyboard::Modifiers,
    repeat: bool,
) -> keyboard::Event {
    use iced::keyboard::key::{Code, Physical};
    use iced::keyboard::Location;

    keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: Physical::Code(Code::KeyK),
        location: Location::Standard,
        modifiers,
        text: None,
        repeat,
    }
}

#[test]
fn product_shortcut_routes_to_unscoped_app_message() {
    let actions = ActionMap::new();
    let shortcuts = ShortcutMap::new().bind(
        ShortcutBinding::character('K', ShortcutModifiers::CONTROL),
        TestMessage::Shortcut,
    );
    let event = key_pressed(
        keyboard::Key::Character("k".into()),
        keyboard::Modifiers::CTRL,
        false,
    );

    assert!(matches!(
        shortcut_message_from_event::<TestApp, NoProbe>(&actions, &shortcuts, event),
        Some(NiveMessage::App {
            window_id: None,
            source: MessageSource::Action,
            message: TestMessage::Shortcut
        })
    ));
}

#[test]
fn repeated_product_shortcut_keypress_is_ignored() {
    let actions = ActionMap::new();
    let shortcuts = ShortcutMap::new().bind(
        ShortcutBinding::character('k', ShortcutModifiers::CONTROL),
        TestMessage::Shortcut,
    );
    let event = key_pressed(
        keyboard::Key::Character("k".into()),
        keyboard::Modifiers::CTRL,
        true,
    );

    assert!(shortcut_message_from_event::<TestApp, NoProbe>(&actions, &shortcuts, event).is_none());
}

#[test]
fn action_shortcut_routes_before_legacy_shortcut() {
    let actions = ActionMap::new().action(
        Action::new("test.action", "Test action", TestMessage::Action)
            .shortcut(ShortcutBinding::character('k', ShortcutModifiers::CONTROL)),
    );
    let shortcuts = ShortcutMap::new().bind(
        ShortcutBinding::character('k', ShortcutModifiers::CONTROL),
        TestMessage::LegacyShortcut,
    );
    let event = key_pressed(
        keyboard::Key::Character("k".into()),
        keyboard::Modifiers::CTRL,
        false,
    );

    assert!(matches!(
        shortcut_message_from_event::<TestApp, NoProbe>(&actions, &shortcuts, event),
        Some(NiveMessage::App {
            window_id: None,
            source: MessageSource::Action,
            message: TestMessage::Action
        })
    ));
}

#[test]
fn disabled_action_shortcut_does_not_dispatch() {
    let actions = ActionMap::new().action(
        Action::new("test.action", "Test action", TestMessage::Action)
            .shortcut(ShortcutBinding::character('k', ShortcutModifiers::CONTROL))
            .disabled(),
    );
    let shortcuts = ShortcutMap::new();
    let event = key_pressed(
        keyboard::Key::Character("k".into()),
        keyboard::Modifiers::CTRL,
        false,
    );

    assert!(shortcut_message_from_event::<TestApp, NoProbe>(&actions, &shortcuts, event).is_none());
}

#[test]
fn framework_shortcut_wins_product_conflict() {
    let actions = ActionMap::new();
    let shortcuts = ShortcutMap::new().bind(
        ShortcutBinding::named(NamedShortcutKey::Tab, ShortcutModifiers::NONE),
        TestMessage::Shortcut,
    );
    let event = key_pressed(
        keyboard::Key::Named(keyboard::key::Named::Tab),
        keyboard::Modifiers::NONE,
        false,
    );

    assert!(matches!(
        shortcut_message_from_event::<TestApp, NoProbe>(&actions, &shortcuts, event),
        Some(NiveMessage::Core(CoreMessage::KeyboardNavigation(
            KeyboardNavigation::FocusNext
        )))
    ));
}

#[test]
fn framework_shortcut_wins_action_conflict() {
    let actions = ActionMap::new().action(
        Action::new("test.action", "Test action", TestMessage::Shortcut).shortcut(
            ShortcutBinding::named(NamedShortcutKey::Tab, ShortcutModifiers::NONE),
        ),
    );
    let shortcuts = ShortcutMap::new();
    let event = key_pressed(
        keyboard::Key::Named(keyboard::key::Named::Tab),
        keyboard::Modifiers::NONE,
        false,
    );

    assert!(matches!(
        shortcut_message_from_event::<TestApp, NoProbe>(&actions, &shortcuts, event),
        Some(NiveMessage::Core(CoreMessage::KeyboardNavigation(
            KeyboardNavigation::FocusNext
        )))
    ));
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
    let main_id = open_main_window(&mut program);
    program.core.registry.set_opened(WindowHandle::new(
        TestWindow::Secondary,
        window::Id::unique(),
    ));

    let _task = program.request_close(main_id);

    assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(1));
    assert!(!program.core.exiting);
}

#[test]
fn last_app_window_uses_exit_hook() {
    let mut program = program();
    let main_id = open_main_window(&mut program);

    let _task = program.request_close(main_id);

    assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(0));
    assert!(program.core.exiting);
}

#[test]
fn simultaneous_closes_treat_second_app_window_as_exit_request() {
    let mut program = program();
    let main_id = open_main_window(&mut program);
    program.core.registry.set_opened(WindowHandle::new(
        TestWindow::Secondary,
        window::Id::unique(),
    ));
    if let Some(app) = program.app.as_mut() {
        app.cancel_exit = true;
    }
    let secondary_id = program
        .core
        .registry
        .latest(TestWindow::Secondary)
        .map(|handle| handle.id)
        .unwrap_or_else(window::Id::unique);

    let _task = program.request_close(main_id);
    let _task = program.request_close(secondary_id);

    assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(1));
    assert!(program.core.pending_app_closes.contains(&main_id));
    assert!(!program.core.pending_app_closes.contains(&secondary_id));
    assert!(!program.core.exiting);
}

#[test]
fn close_all_kind_respects_cancelled_final_exit() {
    let mut program = program();
    program
        .core
        .registry
        .set_opened(WindowHandle::new(TestWindow::Main, window::Id::unique()));
    if let Some(app) = program.app.as_mut() {
        app.cancel_exit = true;
    }

    let _task = program.handle_window_command(WindowCommand::CloseAllKind(TestWindow::Main));

    assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(1));
    assert_eq!(program.core.effective_app_window_count(), 1);
    assert!(!program.core.exiting);
}

#[test]
fn close_all_kind_requests_close_for_every_matching_window() {
    let mut program = program();
    program.core.registry.set_opened(WindowHandle::new(
        TestWindow::Multiple,
        window::Id::unique(),
    ));
    program.core.registry.set_opened(WindowHandle::new(
        TestWindow::Multiple,
        window::Id::unique(),
    ));

    let _task = program.handle_window_command(WindowCommand::CloseAllKind(TestWindow::Multiple));

    assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(2));
}

#[test]
fn close_all_kind_rejects_without_side_effects_when_no_matching_windows() {
    let mut program = program();

    let _task = program.handle_window_command(WindowCommand::CloseAllKind(TestWindow::Multiple));

    assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(0));
    assert_eq!(program.core.registry.all(TestWindow::Multiple).count(), 0);
}

#[test]
fn window_query_latest_returns_most_recently_active_matching_window() {
    let mut program = program();
    program.core.registry.set_opened(WindowHandle::new(
        TestWindow::Multiple,
        window::Id::unique(),
    ));
    let recent_id = window::Id::unique();
    program
        .core
        .registry
        .set_opened(WindowHandle::new(TestWindow::Multiple, recent_id));

    let windows = program.core.context().windows();

    assert_eq!(
        windows
            .latest(TestWindow::Multiple)
            .map(|context| context.id),
        Some(recent_id)
    );
    assert_eq!(windows.latest_id(TestWindow::Multiple), Some(recent_id));
}

#[test]
fn window_query_latest_returns_none_when_kind_absent() {
    let program = program();

    let windows = program.core.context().windows();

    assert_eq!(windows.latest(TestWindow::Multiple), None);
    assert_eq!(windows.latest_id(TestWindow::Multiple), None);
}

#[test]
fn window_query_latest_ignores_opening_windows() {
    let mut program = program();
    program.core.registry.set_opening(WindowHandle::new(
        TestWindow::Multiple,
        window::Id::unique(),
    ));

    let windows = program.core.context().windows();

    assert_eq!(windows.latest(TestWindow::Multiple), None);
    assert_eq!(windows.latest_id(TestWindow::Multiple), None);
}

#[test]
fn replace_rejects_missing_current() {
    let mut program = program();
    let missing_current = window::Id::unique();

    let _task = program.handle_window_command(WindowCommand::Replace {
        current: missing_current,
        next: TestWindow::Secondary,
    });

    assert!(!program.core.registry.contains(TestWindow::Secondary));
}

#[test]
fn replace_rejects_missing_next_spec() {
    let mut program = program();
    let main_id = open_main_window(&mut program);

    let _task = program.handle_window_command(WindowCommand::Replace {
        current: main_id,
        next: TestWindow::Missing,
    });

    assert!(program.core.registry.contains(TestWindow::Main));
    assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(0));
}

#[test]
fn replace_rejects_self_target() {
    let mut program = program();
    let main_id = open_main_window(&mut program);

    let _task = program.handle_window_command(WindowCommand::Replace {
        current: main_id,
        next: TestWindow::Main,
    });

    assert_eq!(program.core.registry.all(TestWindow::Main).count(), 1);
    assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(0));
}

#[test]
fn replace_rejects_opening_current() {
    let mut program = program();
    let opening_current = window::Id::unique();
    program
        .core
        .registry
        .set_opening(WindowHandle::new(TestWindow::Multiple, opening_current));

    let _task = program.handle_window_command(WindowCommand::Replace {
        current: opening_current,
        next: TestWindow::Secondary,
    });

    assert!(!program.core.registry.contains(TestWindow::Secondary));
    assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(0));
}

#[test]
fn replace_rejects_auxiliary_current() {
    let mut program = program();
    let auxiliary_id = window::Id::unique();
    program
        .core
        .registry
        .set_opened(WindowHandle::auxiliary(TestWindow::Auxiliary, auxiliary_id));

    let _task = program.handle_window_command(WindowCommand::Replace {
        current: auxiliary_id,
        next: TestWindow::Secondary,
    });

    assert!(!program.core.registry.contains(TestWindow::Secondary));
    assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(0));
}

#[test]
fn replace_rejects_auxiliary_next() {
    let mut program = program();
    let main_id = open_main_window(&mut program);

    let _task = program.handle_window_command(WindowCommand::Replace {
        current: main_id,
        next: TestWindow::Auxiliary,
    });

    assert!(!program.core.registry.contains(TestWindow::Auxiliary));
    assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(0));
}

#[test]
fn replace_attaches_to_existing_opening_single_cardinality_target() {
    let mut program = program();
    let main_id = open_main_window(&mut program);
    let opening_secondary = window::Id::unique();
    program
        .core
        .registry
        .set_opening(WindowHandle::new(TestWindow::Secondary, opening_secondary));

    let _task = program.handle_window_command(WindowCommand::Replace {
        current: main_id,
        next: TestWindow::Secondary,
    });

    assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(0));
    assert_eq!(program.core.registry.all(TestWindow::Secondary).count(), 1);
    assert_eq!(program.core.pending_replacements.len(), 1);
    assert_eq!(
        program.core.pending_replacements.get(&opening_secondary),
        Some(&main_id)
    );
}

#[test]
fn replace_closes_current_after_existing_opening_single_target_opens() {
    let mut program = program();
    let main_id = open_main_window(&mut program);
    let opening_secondary = window::Id::unique();
    program
        .core
        .registry
        .set_opening(WindowHandle::new(TestWindow::Secondary, opening_secondary));

    let _task = program.handle_window_command(WindowCommand::Replace {
        current: main_id,
        next: TestWindow::Secondary,
    });
    let _task = program.update_core(CoreMessage::WindowOpened(opening_secondary));

    assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(1));
    assert!(program.core.pending_app_closes.contains(&main_id));
    assert_eq!(program.core.registry.all(TestWindow::Secondary).count(), 1);
}

#[test]
fn replace_keeps_current_open_while_next_is_opening() {
    let mut program = program();
    let main_id = open_main_window(&mut program);

    let _task = program.handle_window_command(WindowCommand::Replace {
        current: main_id,
        next: TestWindow::Secondary,
    });

    assert!(program.core.registry.contains(TestWindow::Main));
    assert!(program.core.registry.contains(TestWindow::Secondary));
    assert_eq!(program.core.effective_app_window_count(), 2);
    assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(0));
}

#[test]
fn replace_does_not_exit_during_last_app_window_handoff() {
    let mut program = program();
    let main_id = open_main_window(&mut program);

    let _task = program.handle_window_command(WindowCommand::Replace {
        current: main_id,
        next: TestWindow::Secondary,
    });

    assert!(!program.core.exiting);
}

#[test]
fn replace_closes_current_after_next_opens() {
    let mut program = program();
    let main_id = open_main_window(&mut program);

    let _task = program.handle_window_command(WindowCommand::Replace {
        current: main_id,
        next: TestWindow::Secondary,
    });
    let next_id = program
        .core
        .registry
        .all(TestWindow::Secondary)
        .find(|handle| handle.id != main_id)
        .map(|handle| handle.id)
        .expect("next window registered as opening");

    let _task = program.update_core(CoreMessage::WindowOpened(next_id));

    assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(1));
    assert!(program.core.pending_app_closes.contains(&main_id));
}

#[test]
fn replace_uses_existing_single_cardinality_target() {
    let mut program = program();
    let main_id = open_main_window(&mut program);
    let secondary_id = window::Id::unique();
    program
        .core
        .registry
        .set_opened(WindowHandle::new(TestWindow::Secondary, secondary_id));

    let _task = program.handle_window_command(WindowCommand::Replace {
        current: main_id,
        next: TestWindow::Secondary,
    });

    assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(1));
    assert!(program.core.pending_app_closes.contains(&main_id));
}

#[test]
fn view_message_carries_view_source_and_window() {
    let mut program = program();
    let main_id = main_window_id(&program);

    let _task = program.update(NiveMessage::App {
        window_id: Some(main_id),
        source: MessageSource::View,
        message: TestMessage::Shortcut,
    });

    let context = program
        .app
        .as_ref()
        .and_then(|app| app.last_message_context)
        .expect("message context recorded");
    assert_eq!(context.source, MessageSource::View);
    assert_eq!(context.window.map(|window| window.id), Some(main_id));
}

#[test]
fn task_message_carries_task_source_without_window() {
    let mut program = program();

    let _task = program.update(NiveMessage::App {
        window_id: None,
        source: MessageSource::Task,
        message: TestMessage::Shortcut,
    });

    let context = program
        .app
        .as_ref()
        .and_then(|app| app.last_message_context)
        .expect("message context recorded");
    assert_eq!(context.source, MessageSource::Task);
    assert!(context.window.is_none());
}

#[test]
fn subscription_message_carries_subscription_source() {
    let mut program = program();

    let _task = program.update(NiveMessage::App {
        window_id: None,
        source: MessageSource::Subscription,
        message: TestMessage::Shortcut,
    });

    let context = program
        .app
        .as_ref()
        .and_then(|app| app.last_message_context)
        .expect("message context recorded");
    assert_eq!(context.source, MessageSource::Subscription);
}

#[test]
fn cancelled_exit_keeps_runtime_active() {
    let mut program = program();
    if let Some(app) = program.app.as_mut() {
        app.cancel_exit = true;
    }

    let _task = program.request_exit();

    assert!(!program.core.exiting);
}

#[test]
fn command_rejection_is_forwarded_as_runtime_event() {
    let mut program = program();
    let rejection = CommandRejected {
        command: WindowCommand::Open(TestWindow::Missing),
        reason: CommandRejectionReason::MissingWindowSpec,
    };

    let _task = program.update_core(CoreMessage::Rejected(rejection));

    assert_eq!(program.app.as_ref().map(|app| app.rejections), Some(1));
}

#[test]
fn configured_bootstrap_delays_app_init_and_initial_windows() {
    let (program, _task) = plain_program::<BootstrapTestApp>()
        .unwrap_or_else(|error| panic!("test program failed: {error}"));

    assert!(program.app.is_none());
    assert!(program.bootstrap.is_some());
    assert!(!program.core.registry.contains(TestWindow::Main));
}

#[test]
fn successful_bootstrap_transfers_result_into_app_init() {
    let (mut program, _task) = plain_program::<BootstrapTestApp>()
        .unwrap_or_else(|error| panic!("test program failed: {error}"));
    let result = Arc::new(Mutex::new(Some(Ok(String::from("services")))));

    let _task = program.update_bootstrap(BootstrapMessage::Finished { attempt: 1, result });

    assert_eq!(
        program.app.as_ref().map(|app| app.bootstrap.as_str()),
        Some("services")
    );
    assert!(program.bootstrap.is_none());
    assert!(program.core.registry.contains(TestWindow::Main));
}

#[test]
fn closing_splash_cancels_bootstrap_without_creating_app() {
    let (mut program, _task) = plain_program::<BootstrapTestApp>()
        .unwrap_or_else(|error| panic!("test program failed: {error}"));
    let splash_window = program
        .bootstrap
        .as_ref()
        .map(|bootstrap| bootstrap.window_id)
        .unwrap_or_else(window::Id::unique);

    let _task = program.update_core(CoreMessage::WindowCloseRequested(splash_window));

    assert!(program.core.exiting);
    assert!(program.app.is_none());
}

fn main_window_id(program: &Program<TestApp>) -> window::Id {
    program
        .core
        .registry
        .all(TestWindow::Main)
        .map(|handle| handle.id)
        .next()
        .unwrap_or_else(window::Id::unique)
}

fn open_main_window(program: &mut Program<TestApp>) -> window::Id {
    let main_id = main_window_id(program);
    let _handle = program.core.registry.mark_opened(main_id);
    main_id
}

mod focus;

#[test]
fn toast_runtime_command_enqueues_visible_toast() {
    let mut program = program();

    let _task = program.handle_runtime_command(RuntimeCommand::Toast(Toast::info("Saved")), None);

    assert!(program.core.toasts.has_visible());
    assert!(program.core.toasts.should_subscribe());
}

#[test]
fn toast_tick_expires_visible_toast() {
    let now = Instant::now();
    let mut program = program();
    let _task = program.handle_runtime_command(RuntimeCommand::Toast(Toast::info("Saved")), None);

    let _task = program.update_core(CoreMessage::ToastTick(now + Duration::from_secs(5)));

    assert!(!program.core.toasts.has_visible());
}

#[test]
fn toast_expiry_pauses_while_a_modal_is_open() {
    let now = Instant::now();
    let mut program = program();
    let _id = program.core.toasts.push(Toast::info("Saved"), now, None);

    let _task = program.update_core(CoreMessage::ModalActive(true));
    let _task = program.update_core(CoreMessage::ToastTick(now + Duration::from_secs(5)));

    assert!(
        program.core.toasts.has_visible(),
        "toast stays visible while a modal is open"
    );

    let _task = program.update_core(CoreMessage::ModalActive(false));
    let _task = program.update_core(CoreMessage::ToastTick(now + Duration::from_secs(9)));

    assert!(
        !program.core.toasts.has_visible(),
        "toast expires once the modal closes"
    );
}

#[test]
fn toast_dismiss_message_removes_toast() {
    let now = Instant::now();
    let mut program = program();
    let id = program.core.toasts.push(Toast::info("Saved"), now, None);

    let _task = program.update_core(CoreMessage::ToastDismiss(id));

    assert!(!program.core.toasts.has_visible());
}

#[test]
fn toast_hover_pauses_expiry_and_resume_lets_it_expire() {
    let now = Instant::now();
    let mut program = program();
    let _id = program.core.toasts.push(Toast::info("Saved"), now, None);

    let _task = program.update_core(CoreMessage::ToastHoverEntered);
    let _task = program.update_core(CoreMessage::ToastTick(now + Duration::from_secs(5)));

    assert!(
        program.core.toasts.has_visible(),
        "toast stays visible while hovered"
    );

    let _task = program.update_core(CoreMessage::ToastHoverLeft);
    let _task = program.update_core(CoreMessage::ToastTick(now + Duration::from_secs(9)));

    assert!(
        !program.core.toasts.has_visible(),
        "toast expires after hover ends"
    );
}

#[test]
fn toast_focus_within_pauses_expiry_and_leaving_lets_it_expire() {
    let now = Instant::now();
    let mut program = program();
    let _id = program.core.toasts.push(Toast::info("Saved"), now, None);

    let _task = program.update_core(CoreMessage::ToastFocusWithinEntered);
    let _task = program.update_core(CoreMessage::ToastTick(now + Duration::from_secs(5)));

    assert!(
        program.core.toasts.has_visible(),
        "toast stays visible while a keyboard focus is inside it"
    );

    let _task = program.update_core(CoreMessage::ToastFocusWithinLeft);
    let _task = program.update_core(CoreMessage::ToastTick(now + Duration::from_secs(9)));

    assert!(
        !program.core.toasts.has_visible(),
        "toast expires once focus leaves"
    );
}

#[test]
fn toast_host_decorates_app_view_when_toast_visible() {
    let now = Instant::now();
    let mut program = program();
    let _id = program.core.toasts.push(Toast::info("Saved"), now, None);
    let main_id = main_window_id(&program);

    let _element: nive_ui::Element<'_, RuntimeMessage<TestApp>> = program.view(main_id);

    assert!(program.core.toasts.has_visible());
}

#[test]
fn toast_and_dialog_coexist_in_app_view() {
    let now = Instant::now();
    let mut program = program();
    if let Some(app) = program.app.as_mut() {
        app.show_dialog = true;
    }
    let _id = program.core.toasts.push(Toast::info("Saved"), now, None);
    let main_id = main_window_id(&program);

    let _element: nive_ui::Element<'_, RuntimeMessage<TestApp>> = program.view(main_id);

    assert!(program.core.toasts.has_visible());
}

#[test]
fn auxiliary_window_view_skips_toast_decoration() {
    let now = Instant::now();
    let mut program = program();
    program.core.registry.set_opened(WindowHandle::auxiliary(
        TestWindow::Secondary,
        window::Id::unique(),
    ));
    let _id = program.core.toasts.push(Toast::info("Saved"), now, None);
    let auxiliary_id = program
        .core
        .registry
        .latest(TestWindow::Secondary)
        .map(|handle| handle.id)
        .unwrap_or_else(window::Id::unique);

    let _element: nive_ui::Element<'_, RuntimeMessage<TestApp>> = program.view(auxiliary_id);

    assert!(program.core.toasts.has_visible());
}
