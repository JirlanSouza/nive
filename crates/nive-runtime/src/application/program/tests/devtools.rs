use super::*;

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
