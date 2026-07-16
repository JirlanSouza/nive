use nive::prelude::*;

use super::tone::tone_label;
use super::{AppCommand, DocumentId, Message, PanelActionId, WorkbenchMonitor};

impl WorkbenchMonitor {
    pub(super) fn toolbar(&self) -> Toolbar<'_, Message> {
        let latest_alert = self
            .model
            .active_alerts()
            .next()
            .map(|alert| Message::ShowAlert(alert.id));

        Toolbar::new()
            .group(
                ToolbarGroup::new()
                    .action(
                        ToolbarAction::icon_label(IconRole::ViewRefresh, "Run health check")
                            .tooltip("Run fleet health check")
                            .loading(self.model.running_jobs() > 0)
                            .on_press(Message::Command(AppCommand::RunHealthCheck)),
                    )
                    .action(
                        ToolbarAction::icon_label(IconRole::EditFind, "Command palette")
                            .tooltip("Open command palette")
                            .on_press(Message::OpenPalette),
                    ),
            )
            .separator()
            .group(
                ToolbarGroup::new()
                    .action(
                        ToolbarAction::icon_label(IconRole::PreferencesSystem, "Switch env")
                            .tooltip("Switch monitor environment")
                            .on_press(Message::Command(AppCommand::SwitchEnvironment)),
                    )
                    .action(
                        ToolbarAction::icon_label(IconRole::DialogInformation, "Theme")
                            .tooltip("Toggle light/dark theme")
                            .selected(matches!(self.theme, ThemePreference::Dark))
                            .on_press(Message::ToggleTheme),
                    ),
            )
            .separator()
            .group(
                ToolbarGroup::new()
                    .action(
                        ToolbarAction::icon_label(IconRole::DialogWarning, "Latest alert")
                            .tooltip("Open latest active alert")
                            .disabled(latest_alert.is_none())
                            .on_press_maybe(latest_alert),
                    )
                    .action(
                        ToolbarAction::icon_label(IconRole::WindowClose, "Clear selection")
                            .tooltip("Clear inspector selection")
                            .disabled(matches!(self.selected, super::Selection::None))
                            .on_press(Message::ClearSelection),
                    ),
            )
    }

    pub(super) fn left_panels(
        &self,
    ) -> Vec<WorkbenchPanel<'_, &'static str, PanelActionId, Message>> {
        vec![
            WorkbenchPanel::new("services", "Services", self.services_view())
                .icon(IconRole::Folder)
                .count_badge(self.model.services.len() as u64)
                .status_text(self.overall_tone(), tone_label(self.overall_tone()))
                .action(PanelAction::icon(
                    PanelActionId::RunHealth,
                    IconRole::ViewRefresh,
                    "Run health check",
                ))
                .action(PanelAction::icon(
                    PanelActionId::Refresh,
                    IconRole::ViewReveal,
                    "Refresh panel",
                )),
            WorkbenchPanel::new("hosts", "Hosts", self.hosts_view())
                .icon(IconRole::OpenMenu)
                .count_badge(self.model.hosts.len() as u64)
                .status_text(self.host_tone(), tone_label(self.host_tone())),
            WorkbenchPanel::new("alerts", "Alerts", self.alerts_left_view())
                .icon(IconRole::DialogWarning)
                .count_badge(self.active_alert_count() as u64)
                .status_text(self.alert_tone(), tone_label(self.alert_tone())),
            WorkbenchPanel::new("dashboards", "Dashboards", self.dashboards_view())
                .icon(IconRole::DialogInformation)
                .status_text(ToneRole::Accent, "Active"),
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

        vec![inspector_panel("inspector", state)]
    }

    pub(super) fn bottom_panels(
        &self,
    ) -> Vec<WorkbenchPanel<'_, &'static str, PanelActionId, Message>> {
        vec![
            ProblemsPanel::new(self.problems()).into_panel("alerts"),
            logs_panel_slot("logs", self.logs_view())
                .count_badge(self.model.logs.len() as u64)
                .status_text(ToneRole::Accent, "Live"),
            bottom_panel_slot("events", "Events", self.events_view())
                .icon(IconRole::MailInbox)
                .count_badge(self.model.events.len() as u64)
                .action(PanelAction::icon(
                    PanelActionId::Clear,
                    IconRole::EditDelete,
                    "Clear events",
                )),
            operations_panel_slot("jobs", self.jobs_view())
                .count_badge(self.model.running_jobs() as u64)
                .status_text(if self.model.running_jobs() == 0 {
                    ToneRole::Success
                } else {
                    ToneRole::Accent
                }, if self.model.running_jobs() == 0 { "Idle" } else { "Running" }),
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
            .leading(StatusItem::severity(
                self.overall_tone(),
                "fleet health",
            ))
            .leading(StatusItem::severity(
                self.alert_tone(),
                format!("{} active alerts", self.active_alert_count()),
            ))
            .trailing(StatusItem::operation_summary(
                self.model.running_jobs(),
                "jobs",
            ))
            .trailing(StatusItem::severity(self.connection_tone(), "connected"))
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

    const fn document_icon(&self, id: DocumentId) -> IconRole {
        match id {
            DocumentId::Dashboard(_) => IconRole::DialogInformation,
            DocumentId::Service(_) => IconRole::Folder,
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
}
