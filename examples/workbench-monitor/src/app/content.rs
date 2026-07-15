use nive::prelude::*;
use nive::widget::{column, container, row, scrollable};
use nive::widgets::{button as nive_button, text as nive_text};

use super::tone::tone_label;
use super::{AppCommand, Message, WorkbenchMonitor};

impl WorkbenchMonitor {
    pub(super) fn services_view(&self) -> Element<'_, Message> {
        let rows = self.model.services.iter().map(|service| {
            SelectableItem::new(service.name)
                .selected(
                    matches!(self.selected, super::Selection::Service(id) if id == service.id),
                )
                .leading_color(theme::active().tone(service.health).color)
                .trailing(nive_text::caption(format!(
                    "{} rpm · {} ms",
                    service.requests_per_minute, service.latency_ms
                )))
                .on_press(Message::OpenService(service.id))
                .into()
        });

        scrollable(column(rows).spacing(8).padding(12))
            .direction(scrollable::Direction::Vertical(overlay_scrollbar()))
            .into()
    }

    pub(super) fn hosts_view(&self) -> Element<'_, Message> {
        let rows = self.model.hosts.iter().map(|host| {
            SelectableItem::new(host.name)
                .selected(matches!(self.selected, super::Selection::Host(id) if id == host.id))
                .leading_color(theme::active().tone(host.health).color)
                .trailing(nive_text::caption(format!(
                    "{} · cpu {}%",
                    host.zone, host.cpu_percent
                )))
                .on_press(Message::InspectHost(host.id))
                .into()
        });

        scrollable(column(rows).spacing(8).padding(12))
            .direction(scrollable::Direction::Vertical(overlay_scrollbar()))
            .into()
    }

    pub(super) fn alerts_left_view(&self) -> Element<'_, Message> {
        let rows = self.model.active_alerts().map(|alert| {
            SelectableItem::new(alert.title)
                .selected(matches!(self.selected, super::Selection::Alert(id) if id == alert.id))
                .leading_color(theme::active().tone(alert.severity).color)
                .trailing_text(alert.service_id)
                .on_press(Message::ShowAlert(alert.id))
                .into()
        });

        scrollable(column(rows).spacing(8).padding(12))
            .direction(scrollable::Direction::Vertical(overlay_scrollbar()))
            .into()
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
                .trailing(nive_button::secondary("Toggle").on_press(Message::ToggleFilterDirty))
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
                    nive_button::secondary("Switch")
                        .on_press(Message::Command(AppCommand::SwitchEnvironment))
                )
                .fill_width(),
            DataRow::new("Theme")
                .value(if matches!(self.theme, ThemePreference::Dark) {
                    "dark"
                } else {
                    "light"
                })
                .trailing(nive_button::secondary("Toggle").on_press(Message::ToggleTheme))
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
            Card::new(MetricCard::new("requests/min", total_rpm))
                .shape_md()
                .padding(14)
                .fill_width(),
            Card::new(MetricCard::new("active alerts", active_alerts))
                .shape_md()
                .padding(14)
                .fill_width(),
            Card::new(MetricCard::new("running jobs", running_jobs))
                .shape_md()
                .padding(14)
                .fill_width(),
        ]
        .spacing(12);

        let alert_summary: Element<'_, Message> = if let Some(alert) =
            self.model.active_alerts().next()
        {
            InlineAlert::new(alert.title)
                .tone(alert.severity)
                .body("Open the alert to inspect service impact and acknowledge it.")
                .action(nive_button::secondary("Details").on_press(Message::ShowAlert(alert.id)))
                .into()
        } else {
            InlineAlert::new("No active fleet alerts")
                .success()
                .body("All monitored services are currently within threshold.")
                .into()
        };

        let services = self.model.services.iter().map(|service| {
            SelectableItem::new(service.name)
                .selected(
                    matches!(self.selected, super::Selection::Service(id) if id == service.id),
                )
                .leading_color(theme::active().tone(service.health).color)
                .trailing(nive_text::caption(format!(
                    "{} rpm · {} ms · {}% uptime",
                    service.requests_per_minute, service.latency_ms, service.uptime_percent
                )))
                .on_press(Message::OpenService(service.id))
                .into()
        });

        container(
            scrollable(
                column![
                    DocumentHeader::new("Fleet overview")
                        .icon(IconRole::DialogInformation)
                        .title_tooltip("Fleet overview")
                        .trailing(
                            ActionGroup::new().action(
                                ToolbarAction::icon_label(
                                    IconRole::ViewRefresh,
                                    "Run health check"
                                )
                                .tooltip("Run fleet health check")
                                .on_press(Message::Command(AppCommand::RunHealthCheck))
                            )
                        ),
                    cards,
                    alert_summary,
                    Card::new(
                        column![
                            SectionHeader::new("Services")
                                .icon(IconRole::Folder)
                                .badge(self.model.services.len().to_string()),
                            column(services).spacing(6),
                        ]
                        .spacing(8)
                    )
                    .shape_md()
                    .padding(14)
                    .fill_width(),
                    Card::new(
                        column![
                            SectionHeader::new("Dashboard state")
                                .icon(IconRole::TabPinned)
                                .status(SectionHeaderStatus::icon_label(
                                    if self.dirty_filter {
                                        IconRole::DialogWarning
                                    } else {
                                        IconRole::DialogSuccess
                                    },
                                    if self.dirty_filter {
                                        "unsaved"
                                    } else {
                                        "saved"
                                    },
                                    if self.dirty_filter {
                                        ToneRole::Warning
                                    } else {
                                        ToneRole::Success
                                    },
                                )),
                            DataRow::new("Unsaved dashboard filter")
                                .tone(if self.dirty_filter {
                                    ToneRole::Warning
                                } else {
                                    ToneRole::Success
                                })
                                .value(if self.dirty_filter { "dirty" } else { "clean" })
                                .trailing(
                                    nive_button::secondary("Toggle")
                                        .on_press(Message::ToggleFilterDirty)
                                )
                                .fill_width(),
                        ]
                        .spacing(8)
                    )
                    .shape_md()
                    .padding(14)
                    .fill_width(),
                ]
                .spacing(16)
                .padding(24),
            )
            .direction(scrollable::Direction::Vertical(overlay_scrollbar()))
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
            Card::new(MetricCard::new("latency ms", service.latency_ms as i128))
                .shape_md()
                .padding(14)
                .fill_width(),
            Card::new(MetricCard::new("uptime %", service.uptime_percent as i128))
                .shape_md()
                .padding(14)
                .fill_width(),
            Card::new(MetricCard::new(
                "error %",
                service.error_rate_percent as i128
            ))
            .shape_md()
            .padding(14)
            .fill_width(),
        ]
        .spacing(12);
        let host_label = host.map(|host| host.name).unwrap_or("unknown");

        container(
            column![
                DocumentHeader::new(service.name)
                    .icon(IconRole::Folder)
                    .title_tooltip(service.name)
                    .trailing(ToneDot::new(service.health).sm()),
                cards,
                Card::new(
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
                        .fill_width()
                )
                .shape_md()
                .padding(14)
                .fill_width(),
                ActionGroup::new()
                    .action(
                        ToolbarAction::icon_label(IconRole::ViewReveal, "Inspect service")
                            .on_press(Message::InspectService(service.id))
                    )
                    .action(
                        ToolbarAction::icon_label(IconRole::ViewRefresh, "Run health check")
                            .loading(self.model.running_jobs() > 0)
                            .on_press(Message::Command(AppCommand::RunHealthCheck))
                    ),
            ]
            .spacing(16)
            .padding(24),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub(super) fn logs_view(&self) -> Element<'_, Message> {
        let lines = self
            .model
            .logs
            .iter()
            .map(|line| nive_text::code_small(line).into());
        scrollable(column(lines).spacing(4).padding(12))
            .direction(scrollable::Direction::Vertical(overlay_scrollbar()))
            .into()
    }

    pub(super) fn events_view(&self) -> Element<'_, Message> {
        let rows = self.model.events.iter().rev().map(|event| {
            DataRow::new(event.as_str())
                .value(format!("tick {}", self.model.tick))
                .fill_width()
                .into()
        });

        scrollable(column(rows).spacing(8).padding(12))
            .direction(scrollable::Direction::Vertical(overlay_scrollbar()))
            .into()
    }

    pub(super) fn jobs_view(&self) -> Element<'_, Message> {
        if self.model.jobs.is_empty() {
            return EmptyState::new("No jobs")
                .description("Run a health check to exercise operation progress.")
                .icon(IconRole::ViewRefresh)
                .action(
                    nive_button::primary("Run health check")
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

        scrollable(column(rows).spacing(12).padding(12))
            .direction(scrollable::Direction::Vertical(overlay_scrollbar()))
            .into()
    }
}
