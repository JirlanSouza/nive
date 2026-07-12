use nive::prelude::*;

use super::AppCommand;

pub(super) fn commands() -> Vec<WorkbenchCommand<'static, AppCommand>> {
    vec![
        WorkbenchCommand::new(AppCommand::OpenApi, "Open API Gateway")
            .description("Open the API service document"),
        WorkbenchCommand::new(AppCommand::OpenBilling, "Open Billing Worker")
            .description("Open the billing service document"),
        WorkbenchCommand::new(AppCommand::AcknowledgeFirstAlert, "Acknowledge first alert")
            .description("Clear the oldest active alert"),
        WorkbenchCommand::new(AppCommand::RunHealthCheck, "Run health check").shortcut_label("⌘H"),
        WorkbenchCommand::new(AppCommand::SwitchEnvironment, "Switch environment"),
        WorkbenchCommand::new(AppCommand::ToggleLeft, "Toggle left panel"),
        WorkbenchCommand::new(AppCommand::ToggleBottom, "Toggle bottom panel"),
        WorkbenchCommand::new(AppCommand::ToggleTheme, "Toggle light/dark theme"),
    ]
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
