use iced::advanced::Shell;

pub fn propagate_to_parent<LocalMessage, Message>(
    local_shell: &mut Shell<'_, LocalMessage>,
    shell: &mut Shell<'_, Message>,
) {
    if local_shell.is_event_captured() {
        shell.capture_event();
    }

    local_shell.revalidate_layout(|| shell.invalidate_layout());
    if local_shell.are_widgets_invalid() {
        shell.invalidate_widgets();
    }

    shell.request_redraw_at(local_shell.redraw_request());
    shell.request_input_method(local_shell.input_method());
}
