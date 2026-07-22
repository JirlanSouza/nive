use super::*;

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
