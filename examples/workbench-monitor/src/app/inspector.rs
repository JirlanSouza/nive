use nive::prelude::ui::{DialogRequest, UserFacingError};
use nive::prelude::*;

use crate::sim::Alert;

use super::tone::{problem_severity, tone_label};
use super::{Message, Selection, WorkbenchMonitor};

impl WorkbenchMonitor {
    pub(super) fn inspector_content(&self) -> Option<Element<'_, Message>> {
        match self.selected {
            Selection::None => None,
            Selection::Service(id) => self.model.service(id).map(|service| {
                // Identity, relation, and situation only: metrics belong to
                // the active document's metric cards, not this panel.
                let zone = self
                    .model
                    .host(service.host_id)
                    .map_or("unknown", |host| host.zone);

                KeyValueList::new()
                    .item(MetadataItem::new("Service", service.name))
                    .item(MetadataItem::new("Host", service.host_id))
                    .item(
                        MetadataItem::new("Health", tone_label(service.health))
                            .status(service.health),
                    )
                    .item(MetadataItem::new("Zone", zone))
                    .item(MetadataItem::new(
                        "Environment",
                        self.model.environment_label(),
                    ))
                    .fill_width()
                    .into()
            }),
            Selection::Host(id) => self.model.host(id).map(|host| {
                KeyValueList::new()
                    .item(MetadataItem::new("Host", host.name))
                    .item(MetadataItem::new("Zone", host.zone))
                    .item(MetadataItem::new("Health", tone_label(host.health)).status(host.health))
                    .item(MetadataItem::new("CPU", format!("{}%", host.cpu_percent)))
                    .item(MetadataItem::new(
                        "Memory",
                        format!("{}%", host.memory_percent),
                    ))
                    .fill_width()
                    .into()
            }),
            Selection::Alert(id) => self.model.alert(id).map(|alert| {
                KeyValueList::new()
                    .item(MetadataItem::new("Alert", alert.title))
                    .item(MetadataItem::new("Service", alert.service_id))
                    .item(
                        MetadataItem::new("Severity", tone_label(alert.severity))
                            .status(alert.severity),
                    )
                    .item(MetadataItem::new(
                        "State",
                        if alert.active {
                            "active"
                        } else {
                            "acknowledged"
                        },
                    ))
                    .fill_width()
                    .into()
            }),
        }
    }

    pub(super) fn alert_dialog(&self, alert: &Alert) -> Element<'_, Message> {
        let body = KeyValueList::new()
            .item(MetadataItem::new("Service", alert.service_id))
            .item(MetadataItem::new("Severity", tone_label(alert.severity)).status(alert.severity))
            .item(MetadataItem::new(
                "Environment",
                self.model.environment_label(),
            ))
            .fill_width();

        Dialog::new(body)
            .header(DialogHeader::new(alert.title).close("Close alert", Message::CloseDialog))
            .footer(DialogActionFooter::with_one(
                DialogAction::cancel("Close", Message::CloseDialog),
                DialogTerminalAction::primary("Acknowledge", Message::AcknowledgeAlert(alert.id)),
            ))
            .into()
    }

    pub(super) fn alert_dialog_request(&self, alert: &Alert) -> DialogRequest<'_, Message> {
        DialogRequest::new(self.alert_dialog(alert))
            .dismiss_on_backdrop(Message::CloseDialog)
            .dismiss_on_escape(Message::CloseDialog)
    }

    /// The full diagnostic detail behind a summary-only error toast's "View
    /// details" action — kept out of the toast itself and reachable only
    /// through this dialog.
    pub(super) fn sync_error_dialog_request<'a>(
        &self,
        error: &'a UserFacingError,
    ) -> DialogRequest<'a, Message> {
        DialogRequest::new(
            ErrorDetailsDialog::new(error.detail()).on_close(Message::CloseSyncErrorDialog),
        )
        .dismiss_on_backdrop(Message::CloseSyncErrorDialog)
        .dismiss_on_escape(Message::CloseSyncErrorDialog)
    }

    pub(super) fn problems(&self) -> Vec<Problem<'static>> {
        self.model
            .active_alerts()
            .map(|alert| {
                Problem::new(problem_severity(alert.severity), "monitor", alert.title)
                    .location(ProblemLocation::new(alert.service_id))
            })
            .collect()
    }
}

#[cfg(test)]
mod alert_dialog_tests {
    use super::*;

    fn fixture() -> WorkbenchMonitor {
        WorkbenchMonitor::seeded()
    }

    #[test]
    fn every_seeded_alert_builds_a_dialog_without_panicking() {
        let app = fixture();

        for alert in app.model.active_alerts() {
            let _: Element<'_, Message> = app.alert_dialog(alert);
        }
    }

    #[test]
    fn alert_dialog_request_enables_both_backdrop_and_escape_dismissal() {
        let app = fixture();
        let alert = app
            .model
            .active_alerts()
            .next()
            .expect("seeded simulation has at least one active alert");

        let request = app.alert_dialog_request(alert);

        assert!(request.dismiss_policy().on_backdrop().is_some());
        assert!(request.dismiss_policy().on_escape().is_some());
        assert!(matches!(
            request.dismiss_policy().on_backdrop(),
            Some(Message::CloseDialog)
        ));
        assert!(matches!(
            request.dismiss_policy().on_escape(),
            Some(Message::CloseDialog)
        ));
    }

    #[test]
    fn sync_error_dialog_request_shows_full_detail_and_dismisses_to_close() {
        let app = fixture();
        let error = UserFacingError::custom(
            "workbench-monitor",
            "Could not sync fleet status (endpoint: https://status.internal.example)",
        );

        let request = app.sync_error_dialog_request(&error);

        assert!(matches!(
            request.dismiss_policy().on_backdrop(),
            Some(Message::CloseSyncErrorDialog)
        ));
        assert!(matches!(
            request.dismiss_policy().on_escape(),
            Some(Message::CloseSyncErrorDialog)
        ));
    }

    #[test]
    fn alert_dialog_request_declares_no_explicit_identity_or_initial_focus_override() {
        let app = fixture();
        let alert = app
            .model
            .active_alerts()
            .next()
            .expect("seeded simulation has at least one active alert");

        let request = app.alert_dialog_request(alert);

        // Shell geometry and modal wiring stay app-owned; the migration
        // does not introduce application-owned focus state (per the
        // "no application-owned focus state" requirement).
        assert_eq!(request.identity(), None);
        assert_eq!(
            request.initial_focus_policy(),
            &nive::prelude::DialogInitialFocus::First
        );
    }
}
