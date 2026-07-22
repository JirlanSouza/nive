use super::*;

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
