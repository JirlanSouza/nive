use nive::prelude::*;

use super::{AppCommand, DocumentId, Message, PanelActionId, WorkbenchMonitor};

impl WorkbenchMonitor {
    pub(super) fn toolbar(&self) -> Element<'_, Message> {
        let latest_alert = self
            .model
            .active_alerts()
            .next()
            .map(|alert| Message::ShowAlert(alert.id));

        Toolbar::new()
            .fill_width()
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
            .into()
    }

    pub(super) fn left_panels(
        &self,
    ) -> Vec<WorkbenchPanel<'_, &'static str, PanelActionId, Message>> {
        vec![
            WorkbenchPanel::new("services", "Services", self.services_view())
                .icon(IconRole::Folder)
                .badge(self.model.services.len().to_string())
                .status(self.overall_tone())
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
                .badge(self.model.hosts.len().to_string())
                .status(self.host_tone()),
            WorkbenchPanel::new("alerts", "Alerts", self.alerts_left_view())
                .icon(IconRole::DialogWarning)
                .badge(self.active_alert_count().to_string())
                .status(self.alert_tone()),
            WorkbenchPanel::new("dashboards", "Dashboards", self.dashboards_view())
                .icon(IconRole::DialogInformation)
                .status(ToneRole::Accent),
            WorkbenchPanel::new("settings", "Settings", self.settings_view())
                .icon(IconRole::PreferencesSystem),
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
                .badge(self.model.logs.len().to_string())
                .status(ToneRole::Accent),
            bottom_panel_slot("events", "Events", self.events_view())
                .icon(IconRole::MailInbox)
                .badge(self.model.events.len().to_string())
                .action(PanelAction::icon(
                    PanelActionId::Clear,
                    IconRole::EditDelete,
                    "Clear events",
                )),
            operations_panel_slot("jobs", self.jobs_view())
                .badge(self.model.running_jobs().to_string())
                .status(if self.model.running_jobs() == 0 {
                    ToneRole::Success
                } else {
                    ToneRole::Accent
                }),
        ]
    }

    pub(super) fn document_tabs(&self) -> Vec<WorkbenchDocument<'static, DocumentId>> {
        self.documents
            .iter()
            .copied()
            .map(|id| {
                let mut tab = WorkbenchDocument::new(id, self.document_label(id))
                    .icon(self.document_icon(id))
                    .closable(matches!(id, DocumentId::Service(_)));
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
            .item(StatusItem::text("Workbench monitor"))
            .item(StatusItem::severity(
                self.overall_tone(),
                format!("env: {}", self.model.environment_label()),
            ))
            .item(StatusItem::severity(
                self.alert_tone(),
                format!("{} active alerts", self.active_alert_count()),
            ))
            .item(StatusItem::operation_summary(
                self.model.running_jobs(),
                "jobs",
            ))
            .item(StatusItem::Spacer)
            .item(StatusItem::severity(self.connection_tone(), "connected"))
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
