use nive::prelude::*;

use super::{AppCommand, Message};

pub(super) fn commands() -> ActionMap<Message> {
    ActionMap::new()
        .action(Action::new(
            "monitor.palette",
            "Open command palette",
            Message::OpenPalette,
        ))
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
