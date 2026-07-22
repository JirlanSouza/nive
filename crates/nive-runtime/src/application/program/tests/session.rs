use super::*;

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
