use nive::prelude::*;

use super::tone::tone_label;
use super::{AppCommand, DocumentId, Message, PanelActionId, WorkbenchMonitor};
use crate::icons::IconSymbol;

impl WorkbenchMonitor {
    pub(super) fn toolbar(&self) -> Toolbar<'_, Message> {
        let latest_alert = self
            .model
            .active_alerts()
            .next()
            .map(|alert| Message::ShowAlert(alert.id));
        let theme_tooltip = if matches!(self.theme, ThemePreference::Dark) {
            "Switch to light theme"
        } else {
            "Switch to dark theme"
        };

        Toolbar::new()
            .group(
                ToolbarGroup::new()
                    .action(
                        ToolbarAction::icon_label(IconRole::ViewActivity, "Run health check")
                            .loading(self.model.running_jobs() > 0)
                            .on_press(Message::Command(AppCommand::RunHealthCheck)),
                    )
                    .action(
                        ToolbarAction::icon(IconRole::EditFind)
                            .tooltip("Open command palette")
                            .on_press(Message::OpenPalette),
                    ),
            )
            .spacer()
            .group(
                ToolbarGroup::new()
                    .action(
                        ToolbarAction::icon_label(
                            IconRole::PreferencesSystem,
                            self.model.environment_label(),
                        )
                        .tooltip("Switch environment")
                        .on_press(Message::Command(AppCommand::SwitchEnvironment)),
                    )
                    .action(
                        ToolbarAction::icon(IconRole::NotificationAlert)
                            .tooltip("Show latest alert")
                            .disabled(latest_alert.is_none())
                            .on_press_maybe(latest_alert),
                    )
                    .action(
                        ToolbarAction::icon(IconRole::ViewTheme)
                            .tooltip(theme_tooltip)
                            .on_press(Message::ToggleTheme),
                    ),
            )
    }

    pub(super) fn left_panels(
        &self,
    ) -> Vec<WorkbenchPanel<'_, &'static str, PanelActionId, Message>> {
        vec![
            WorkbenchPanel::new("services", "Services", self.services_view())
                .icon(IconSymbol::Server)
                .count_badge(self.model.services.len() as u64)
                .status_text(self.overall_tone(), tone_label(self.overall_tone()))
                .action(PanelAction::icon(
                    PanelActionId::RunHealth,
                    IconRole::ViewActivity,
                    "Run health check",
                ))
                .action(PanelAction::icon(
                    PanelActionId::Refresh,
                    IconRole::ViewRefresh,
                    "Refresh panel",
                )),
            WorkbenchPanel::new("hosts", "Hosts", self.hosts_view())
                .icon(IconSymbol::Server)
                .count_badge(self.model.hosts.len() as u64)
                .status_text(self.host_tone(), tone_label(self.host_tone())),
            WorkbenchPanel::new("alerts", "Alerts", self.alerts_left_view())
                .icon(IconRole::DialogWarning)
                .count_badge(self.active_alert_count() as u64)
                .status_text(self.alert_tone(), tone_label(self.alert_tone())),
            WorkbenchPanel::new("dashboards", "Dashboards", self.dashboards_view())
                .icon(IconSymbol::Dashboard)
                .status_text(ToneRole::Accent, "Active"),
            WorkbenchPanel::new("explorer", "Explorer", self.explorer_view())
                .icon(IconRole::Folder)
                .count_badge(self.model.hosts.len() as u64),
            WorkbenchPanel::new("settings", "Settings", self.settings_view())
                .icon(IconRole::PreferencesSystem)
                .disabled(true),
        ]
    }

    pub(super) fn right_panels(
        &self,
    ) -> Vec<WorkbenchPanel<'_, &'static str, PanelActionId, Message>> {
        let state = if self
            .inspector_loading_until
            .is_some_and(|tick| self.model.tick < tick)
        {
            InspectorState::Loading {
                label: "Loading selected entity…".into(),
            }
        } else {
            match self.inspector_content() {
                Some(content) => InspectorState::Content(content),
                None => InspectorState::NoSelection,
            }
        };

        let mut panel = inspector_panel("inspector", state);
        if !matches!(self.selected, super::Selection::None) {
            panel = panel.action(PanelAction::icon(
                PanelActionId::ClearSelection,
                IconRole::WindowClose,
                "Clear selection",
            ));
        }

        vec![panel]
    }

    pub(super) fn bottom_panels(
        &self,
    ) -> Vec<WorkbenchPanel<'_, &'static str, PanelActionId, Message>> {
        vec![
            ProblemsPanel::new(self.problems()).into_panel("alerts"),
            // A log count climbs on its own and nobody acts on it, so the
            // stream state earns the tab's single signal slot instead.
            logs_panel_slot("logs", self.logs_view()).status_text(ToneRole::Accent, "Live"),
            bottom_panel_slot("events", "Events", self.events_view())
                .icon(IconSymbol::Event)
                .count_badge(self.model.events.len() as u64)
                .action(PanelAction::icon(
                    PanelActionId::Clear,
                    IconRole::EditDelete,
                    "Clear events",
                )),
            operations_panel_slot("jobs", self.jobs_view())
                .count_badge(self.model.running_jobs() as u64)
                .status_text(
                    if self.model.running_jobs() == 0 {
                        ToneRole::Success
                    } else {
                        ToneRole::Accent
                    },
                    if self.model.running_jobs() == 0 {
                        "Idle"
                    } else {
                        "Running"
                    },
                ),
        ]
    }
    pub(super) fn document_tabs(&self) -> Vec<WorkbenchDocument<'static, DocumentId>> {
        self.documents
            .iter()
            .copied()
            .map(|id| {
                let mut tab = WorkbenchDocument::new(id, self.document_label(id))
                    .icon(self.document_icon(id))
                    .closable(matches!(id, DocumentId::Service(_)))
                    .disabled(matches!(id, DocumentId::Service("search")));
                if matches!(id, DocumentId::Dashboard(_)) {
                    tab = tab.pinned(true).dirty(self.dirty_filter);
                }
                tab
            })
            .collect()
    }

    pub(super) fn document_content(&self) -> Element<'_, Message> {
        match self.layout.active_document().copied() {
            Some(DocumentId::Service(id)) => self.service_document(id),
            Some(DocumentId::Dashboard(_)) | None => self.dashboard_document(),
        }
    }

    pub(super) fn status_bar(&self) -> StatusBar<'static> {
        StatusBar::new()
            .leading(StatusItem::text("Workbench monitor"))
            .leading(StatusItem::context(format!(
                "env: {}",
                self.model.environment_label()
            )))
            .leading(StatusItem::severity(self.overall_tone(), "fleet health"))
            .leading(StatusItem::severity(
                self.alert_tone(),
                format!("{} active alerts", self.active_alert_count()),
            ))
            .trailing(StatusItem::operation_summary(
                self.model.running_jobs(),
                "jobs",
            ))
            .trailing(StatusItem::severity(self.connection_tone(), "connected"))
            .trailing(StatusItem::text(format!("mode: {}", self.mode.label())))
    }

    fn document_label(&self, id: DocumentId) -> String {
        match id {
            DocumentId::Dashboard("fleet") => "Fleet overview".into(),
            DocumentId::Dashboard(label) => label.into(),
            DocumentId::Service(service_id) => self
                .model
                .service(service_id)
                .map(|service| service.name.to_string())
                .unwrap_or_else(|| service_id.into()),
        }
    }

    fn document_icon(&self, id: DocumentId) -> IconRef {
        match id {
            DocumentId::Dashboard(_) => IconSymbol::Dashboard.into(),
            DocumentId::Service(_) => IconSymbol::Server.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_panel_models_keep_typed_counts_and_labelled_status() {
        let app = WorkbenchMonitor::seeded();
        let panels = app.left_panels();
        let services = &panels[0];

        assert!(matches!(
            services.badge_content_value(),
            Some(BadgeContent::Count(3))
        ));
        let status = services
            .status_indicator_value()
            .expect("services status indicator");
        assert!(!status.label().trim().is_empty());
        assert_eq!(status.tone(), app.overall_tone());
    }

    #[test]
    fn inspector_exposes_clear_selection_only_while_an_entity_is_selected() {
        let mut app = WorkbenchMonitor::seeded();

        let has_clear_action = app.right_panels().first().is_some_and(|panel| {
            panel
                .panel_actions()
                .iter()
                .any(|action| action.id() == &PanelActionId::ClearSelection)
        });
        assert!(has_clear_action);

        app.clear_selection();

        let has_clear_action = app.right_panels().first().is_some_and(|panel| {
            panel
                .panel_actions()
                .iter()
                .any(|action| action.id() == &PanelActionId::ClearSelection)
        });
        assert!(!has_clear_action);
    }

    #[test]
    fn status_bar_states_the_active_simulation_mode() {
        let mut app = WorkbenchMonitor::seeded();

        app.mode = crate::sim::SimulationMode::Live;
        let live = app.status_bar();
        assert!(live
            .trailing_items()
            .iter()
            .any(|item| matches!(item, StatusItem::Text(label) if label.as_ref() == "mode: live")));

        app.mode = crate::sim::SimulationMode::Frozen;
        let frozen = app.status_bar();
        assert!(frozen.trailing_items().iter().any(
            |item| matches!(item, StatusItem::Text(label) if label.as_ref() == "mode: frozen")
        ));
    }
}
