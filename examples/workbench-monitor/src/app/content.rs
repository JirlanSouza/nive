use nive::prelude::*;
use nive::widget::{column, container, row, scrollable, text};

use super::tone::tone_label;
use super::{AppCommand, Message, WorkbenchMonitor};

impl WorkbenchMonitor {
    pub(super) fn services_view(&self) -> Element<'_, Message> {
        let rows = self.model.services.iter().map(|service| {
            DataRow::new(service.name)
                .tone(service.health)
                .value(format!(
                    "{} rpm · {} ms",
                    service.requests_per_minute, service.latency_ms
                ))
                .trailing(button("Open").on_press(Message::OpenService(service.id)))
                .fill_width()
                .into()
        });

        scrollable(column(rows).spacing(8).padding(12)).into()
    }

    pub(super) fn hosts_view(&self) -> Element<'_, Message> {
        let rows = self.model.hosts.iter().map(|host| {
            DataRow::new(host.name)
                .tone(host.health)
                .value(format!("{} · cpu {}%", host.zone, host.cpu_percent))
                .trailing(button("Inspect").on_press(Message::InspectHost(host.id)))
                .fill_width()
                .into()
        });

        scrollable(column(rows).spacing(8).padding(12)).into()
    }

    pub(super) fn alerts_left_view(&self) -> Element<'_, Message> {
        let rows = self.model.active_alerts().map(|alert| {
            DataRow::new(alert.title)
                .tone(alert.severity)
                .value(alert.service_id)
                .trailing(button("Details").on_press(Message::ShowAlert(alert.id)))
                .fill_width()
                .into()
        });

        scrollable(column(rows).spacing(8).padding(12)).into()
    }

    pub(super) fn dashboards_view(&self) -> Element<'_, Message> {
        column![
            DataRow::new("Fleet overview")
                .tone(ToneRole::Accent)
                .value("Pinned dashboard")
                .fill_width(),
            DataRow::new("Unsaved filter")
                .tone(if self.dirty_filter {
                    ToneRole::Warning
                } else {
                    ToneRole::Success
                })
                .value(if self.dirty_filter { "dirty" } else { "clean" })
                .trailing(button("Toggle").on_press(Message::ToggleFilterDirty))
                .fill_width(),
        ]
        .spacing(8)
        .padding(12)
        .into()
    }

    pub(super) fn settings_view(&self) -> Element<'_, Message> {
        column![
            DataRow::new("Environment")
                .value(self.model.environment_label())
                .trailing(
                    button("Switch").on_press(Message::Command(AppCommand::SwitchEnvironment))
                )
                .fill_width(),
            DataRow::new("Theme")
                .value(if matches!(self.theme, ThemePreference::Dark) {
                    "dark"
                } else {
                    "light"
                })
                .trailing(button("Toggle").on_press(Message::ToggleTheme))
                .fill_width(),
        ]
        .spacing(8)
        .padding(12)
        .into()
    }

    pub(super) fn dashboard_document(&self) -> Element<'_, Message> {
        let active_alerts = self.active_alert_count() as i128;
        let running_jobs = self.model.running_jobs() as i128;
        let total_rpm: i128 = self
            .model
            .services
            .iter()
            .map(|service| service.requests_per_minute as i128)
            .sum();

        let cards = row![
            MetricCard::new("requests/min", total_rpm),
            MetricCard::new("active alerts", active_alerts),
            MetricCard::new("running jobs", running_jobs),
        ]
        .spacing(12);

        let services = self.model.services.iter().map(|service| {
            DataRow::new(service.name)
                .tone(service.health)
                .value(format!(
                    "{} ms · {}% uptime",
                    service.latency_ms, service.uptime_percent
                ))
                .fill_width()
                .into()
        });

        container(
            scrollable(
                column![
                    text("Fleet overview").size(28),
                    cards,
                    column(services).spacing(8),
                    button("Toggle unsaved dashboard filter").on_press(Message::ToggleFilterDirty),
                ]
                .spacing(16)
                .padding(24),
            )
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub(super) fn service_document(&self, service_id: &'static str) -> Element<'_, Message> {
        let Some(service) = self.model.service(service_id) else {
            return EmptyState::new("Service not found").into();
        };

        let host = self.model.host(service.host_id);
        let cards = row![
            MetricCard::new("latency ms", service.latency_ms as i128),
            MetricCard::new("uptime %", service.uptime_percent as i128),
            MetricCard::new("error %", service.error_rate_percent as i128),
        ]
        .spacing(12);
        let host_label = host.map(|host| host.name).unwrap_or("unknown");

        container(
            column![
                text(service.name).size(28),
                cards,
                KeyValueList::new()
                    .item(MetadataItem::new("Host", host_label))
                    .item(MetadataItem::new(
                        "Environment",
                        self.model.environment_label()
                    ))
                    .item(
                        MetadataItem::new("Health", tone_label(service.health))
                            .tone(service.health)
                    )
                    .fill_width(),
                row![
                    button("Inspect service").on_press(Message::InspectService(service.id)),
                    button("Run health check")
                        .on_press(Message::Command(AppCommand::RunHealthCheck)),
                ]
                .spacing(8),
            ]
            .spacing(16)
            .padding(24),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub(super) fn logs_view(&self) -> Element<'_, Message> {
        let lines = self.model.logs.iter().map(|line| text(line).into());
        scrollable(column(lines).spacing(4).padding(12)).into()
    }

    pub(super) fn events_view(&self) -> Element<'_, Message> {
        let rows = self.model.events.iter().rev().map(|event| {
            DataRow::new(event.as_str())
                .value(format!("tick {}", self.model.tick))
                .fill_width()
                .into()
        });

        scrollable(column(rows).spacing(8).padding(12)).into()
    }

    pub(super) fn jobs_view(&self) -> Element<'_, Message> {
        if self.model.jobs.is_empty() {
            return EmptyState::new("No jobs")
                .description("Run a health check to exercise operation progress.")
                .icon(IconRole::ViewRefresh)
                .action(
                    button("Run health check")
                        .on_press(Message::Command(AppCommand::RunHealthCheck)),
                )
                .into();
        }

        let rows = self.model.jobs.iter().rev().map(|job| {
            column![
                DataRow::new(job.label)
                    .tone(if job.running {
                        ToneRole::Accent
                    } else {
                        ToneRole::Success
                    })
                    .value(format!(
                        "#{} · {}",
                        job.id,
                        if job.running { "running" } else { "complete" }
                    ))
                    .fill_width(),
                ProgressBar::percent(job.progress).fill_width(),
            ]
            .spacing(6)
            .into()
        });

        scrollable(column(rows).spacing(12).padding(12)).into()
    }
}
