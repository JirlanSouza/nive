use super::*;

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
