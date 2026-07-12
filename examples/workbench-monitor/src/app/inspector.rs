use nive::prelude::*;
use nive::widget::{column, row, text};

use crate::sim::Alert;

use super::tone::{problem_severity, tone_label};
use super::{Message, Selection, WorkbenchMonitor};

impl WorkbenchMonitor {
    pub(super) fn inspector_content(&self) -> Option<Element<'_, Message>> {
        match self.selected {
            Selection::None => None,
            Selection::Service(id) => self.model.service(id).map(|service| {
                KeyValueList::new()
                    .item(MetadataItem::new("Service", service.name))
                    .item(MetadataItem::new("Host", service.host_id))
                    .item(
                        MetadataItem::new("Health", tone_label(service.health))
                            .tone(service.health),
                    )
                    .item(
                        MetadataItem::new("Latency", "")
                            .value(text(format!("{} ms", service.latency_ms))),
                    )
                    .item(
                        MetadataItem::new("RPM", "")
                            .value(text(service.requests_per_minute.to_string())),
                    )
                    .fill_width()
                    .into()
            }),
            Selection::Host(id) => self.model.host(id).map(|host| {
                KeyValueList::new()
                    .item(MetadataItem::new("Host", host.name))
                    .item(MetadataItem::new("Zone", host.zone))
                    .item(MetadataItem::new("Health", tone_label(host.health)).tone(host.health))
                    .item(
                        MetadataItem::new("CPU", "").value(text(format!("{}%", host.cpu_percent))),
                    )
                    .item(
                        MetadataItem::new("Memory", "")
                            .value(text(format!("{}%", host.memory_percent))),
                    )
                    .fill_width()
                    .into()
            }),
            Selection::Alert(id) => self.model.alert(id).map(|alert| {
                KeyValueList::new()
                    .item(MetadataItem::new("Alert", alert.title))
                    .item(MetadataItem::new("Service", alert.service_id))
                    .item(
                        MetadataItem::new("Severity", tone_label(alert.severity))
                            .tone(alert.severity),
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
        column![
            text(alert.title).size(24),
            KeyValueList::new()
                .item(MetadataItem::new("Service", alert.service_id))
                .item(
                    MetadataItem::new("Severity", tone_label(alert.severity)).tone(alert.severity)
                )
                .item(MetadataItem::new(
                    "Environment",
                    self.model.environment_label()
                ))
                .fill_width(),
            row![
                button("Acknowledge").on_press(Message::AcknowledgeAlert(alert.id)),
                button("Close").on_press(Message::CloseDialog),
            ]
            .spacing(8),
        ]
        .spacing(16)
        .padding(24)
        .into()
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
