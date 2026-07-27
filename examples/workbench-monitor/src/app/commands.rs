use nive::prelude::*;

use super::{AppCommand, Message};

pub(super) fn commands() -> ActionMap<Message> {
    ActionMap::new()
        .action(
            Action::new(
                "monitor.palette",
                "Open command palette",
                Message::OpenPalette,
            )
            .shortcut(ShortcutBinding::primary_character('k')),
        )
        .action(
            Action::new(
                "monitor.open_api",
                "Open API Gateway",
                Message::Command(AppCommand::OpenApi),
            )
            .description("Open the API service document"),
        )
        .action(
            Action::new(
                "monitor.open_billing",
                "Open Billing Worker",
                Message::Command(AppCommand::OpenBilling),
            )
            .description("Open the billing service document"),
        )
        .action(
            Action::new(
                "monitor.ack_first_alert",
                "Acknowledge first alert",
                Message::Command(AppCommand::AcknowledgeFirstAlert),
            )
            .description("Clear the oldest active alert"),
        )
        .action(Action::new(
            "monitor.health",
            "Run health check",
            Message::Command(AppCommand::RunHealthCheck),
        ))
        .action(Action::new(
            "monitor.switch_environment",
            "Switch environment",
            Message::Command(AppCommand::SwitchEnvironment),
        ))
        .action(Action::new(
            "monitor.toggle_left",
            "Toggle left panel",
            Message::Command(AppCommand::ToggleLeft),
        ))
        .action(Action::new(
            "monitor.toggle_bottom",
            "Toggle bottom panel",
            Message::Command(AppCommand::ToggleBottom),
        ))
        .action(Action::new(
            "monitor.theme",
            "Toggle light/dark theme",
            Message::ToggleTheme,
        ))
        .action(
            Action::new(
                "monitor.demo_sync_failure",
                "Demo: simulate sync failure",
                Message::SimulateSyncFailure,
            )
            .description("Show the monitor's recoverable sync error flow"),
        )
}

pub(super) fn workbench_theme_catalog() -> ThemeCatalog {
    ThemeCatalog::new(
        Theme::builder("Workbench Monitor Light", ThemeMode::Light)
            .density(ThemeDensity::Compact)
            .build(),
        Theme::builder("Workbench Monitor Dark", ThemeMode::Dark)
            .density(ThemeDensity::Compact)
            .build(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_palette_action_uses_the_platform_primary_shortcut() {
        let commands = commands();
        let command = commands
            .iter()
            .find(|action| action.id().as_str() == "monitor.palette")
            .expect("open palette command");

        assert_eq!(
            command.shortcut_binding(),
            Some(&ShortcutBinding::primary_character('k'))
        );
    }

    #[test]
    fn demo_sync_failure_is_available_in_the_shared_action_map() {
        let commands = commands();
        let command = commands
            .iter()
            .find(|action| action.id().as_str() == "monitor.demo_sync_failure")
            .expect("sync failure demo command");

        assert_eq!(command.label(), "Demo: simulate sync failure");
        assert!(matches!(command.message(), Message::SimulateSyncFailure));
        assert!(command.is_enabled());
    }
}
