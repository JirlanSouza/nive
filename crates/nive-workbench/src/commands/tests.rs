use super::*;

#[test]
fn command_palette_state_tracks_open_query_highlight_and_submit() {
    let mut state = CommandPaletteState::new();
    state.open();
    state.set_query("save");
    state.move_highlight(1, 3);

    let commands = vec![
        WorkbenchCommand::new("open", "Open"),
        WorkbenchCommand::new("save", "Save"),
        WorkbenchCommand::new("close", "Close").disabled(true),
    ];

    assert!(state.open);
    assert_eq!(state.query, "save");
    assert_eq!(state.highlighted, Some(1));
    assert_eq!(
        state.submit(&commands),
        Some(WorkbenchCommandPaletteEvent::Submitted("save"))
    );
}
