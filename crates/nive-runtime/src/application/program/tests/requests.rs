use super::*;

use crate::state::RequestEvent;
use crate::{Effect, Resource};

#[test]
fn tracked_terminal_event_cleans_registry_before_app_update() {
    let mut program = program();
    let mut resource = Resource::<()>::idle();
    let request = resource.request(program.core.app_scope.id());
    let request_id = request.control.id();
    let tracked = request.perform(|(), _cancel| async { Ok(()) }, |_| TestMessage::Action);

    let _task = program.apply_update(Effect::request(tracked));
    assert!(program.core.running_requests.contains_key(&request_id));

    let _task = program.update(NiveMessage::Request {
        window_id: None,
        event: RequestEvent {
            request: request_id,
            message: Some(TestMessage::Action),
        },
    });

    assert!(!program.core.running_requests.contains_key(&request_id));
    assert_eq!(
        program
            .app
            .as_ref()
            .and_then(|app| app.last_message_context.as_ref())
            .map(|context| context.source),
        Some(MessageSource::Task)
    );
}

#[test]
fn closing_window_cancels_its_scope() {
    let mut program = program();
    let window_id = main_window_id(&program);
    let scope = program
        .core
        .window_context(window_id)
        .expect("main window context")
        .task_scope();

    let _task = program.update_core(CoreMessage::WindowClosed(window_id));

    assert!(scope.is_closed());
}

#[test]
fn application_exit_cancels_root_scope_and_clears_registry() {
    let mut program = program();
    let scope = program.core.app_scope.id();
    let mut resource = Resource::<()>::idle();
    let request = resource.request(scope.clone());
    let tracked = request.perform(|(), _cancel| async { Ok(()) }, |_| TestMessage::Action);
    let _task = program.apply_update(Effect::request(tracked));

    let _task = program.accept_exit();

    assert!(scope.is_closed());
    assert!(program.core.running_requests.is_empty());
}
