use super::*;

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
