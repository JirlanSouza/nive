use nive::prelude::*;
use nive::widget::{column, container, row, scrollable};
use nive::widgets::{button as nive_button, text as nive_text};

use super::format::grouped;
use super::tone::tone_label;
use super::{AppCommand, Message, MonitorFilter, ServiceScope, WorkbenchMonitor};
use crate::sim::{Environment, Service};
use crate::icons::IconSymbol;

/// Maximum readable measure for document content columns. Surplus canvas
/// width falls outside the content instead of stretching the gaps inside
/// every row; this stays app-side rather than pushed into a shared widget.
const CONTENT_MEASURE: f32 = 960.0;

impl WorkbenchMonitor {
    pub(super) fn services_view(&self) -> Element<'_, Message> {
        let rows = self.model.services.iter().map(|service| {
            SelectableItem::new(service.name)
                .selected(
                    matches!(self.selected, super::Selection::Service(id) if id == service.id),
                )
                .status_text(service.health, tone_label(service.health))
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
                .status_text(host.health, tone_label(host.health))
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
                .status_text(alert.severity, tone_label(alert.severity))
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
                .trailing(
                    ActionGroup::new().action(
                        ContentAction::label("Toggle").on_press(Message::ToggleFilterDirty)
                    )
                )
                .fill_width(),
        ]
        .spacing(8)
        .padding(12)
        .into()
    }

    pub(super) fn settings_view(&self) -> Element<'_, Message> {
        column![
            Switch::setting("Automatic refresh", self.auto_refresh)
                .description("Apply incoming monitor updates immediately")
                .on_toggle(Message::AutoRefreshChanged),
            nive_text::section_label("Environment"),
            SegmentedControl::new(
                "Monitor environment",
                self.model.environment,
                [
                    SegmentedOption::new(Environment::Production, "Production"),
                    SegmentedOption::new(Environment::Staging, "Staging"),
                ],
            )
            .on_select(Message::EnvironmentChanged)
            .fill_width(),
            nive_text::section_label("Dashboard filter"),
            SegmentedControl::new(
                "Dashboard filter",
                self.monitor_filter,
                [
                    SegmentedOption::new(MonitorFilter::All, "All services"),
                    SegmentedOption::new(MonitorFilter::Attention, "Attention"),
                ],
            )
            .linked()
            .on_select(Message::MonitorFilterChanged)
            .fill_width(),
            DataRow::new("Theme")
                .value(if matches!(self.theme, ThemePreference::Dark) {
                    "dark"
                } else {
                    "light"
                })
                .trailing(
                    ActionGroup::new()
                        .action(ContentAction::label("Toggle").on_press(Message::ToggleTheme))
                )
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
            .filter(|service| self.service_scope.includes(service.id))
            .map(|service| service.requests_per_minute as i128)
            .sum();

        let mut service_options = vec![SelectOption::new(ServiceScope::All, "All services")];
        service_options.extend(
            self.model
                .services
                .iter()
                .map(|service| SelectOption::new(ServiceScope::Service(service.id), service.name)),
        );
        let service_scope = Field::new(
            "Service scope",
            Select::new(service_options, Some(self.service_scope))
                .on_select(Message::ServiceScopeChanged),
        )
        .hint("Filters dashboard metrics and rows without changing workbench navigation")
        .sm();

        let cards = row![
            Card::new(
                MetricCard::new("Requests", grouped(total_rpm))
                    .unit("rpm")
                    .trend(nive_text::body_small("live fleet total"))
            )
            .fill_width(),
            Card::new(
                MetricCard::new("Active alerts", grouped(active_alerts)).status(
                    Badge::status(if active_alerts > 0 {
                        "attention"
                    } else {
                        "clear"
                    })
                    .tone(if active_alerts > 0 {
                        ToneRole::Warning
                    } else {
                        ToneRole::Success
                    })
                )
            )
            .fill_width(),
            Card::new(
                MetricCard::new("Running jobs", grouped(running_jobs)).trend(
                    nive_text::body_small(if running_jobs > 0 {
                        "in progress"
                    } else {
                        "idle"
                    })
                )
            )
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

        let services = self
            .model
            .services
            .iter()
            .filter(|service| self.service_scope.includes(service.id))
            .map(|service| {
                SelectableItem::new(service.name)
                    .selected(
                        matches!(self.selected, super::Selection::Service(id) if id == service.id),
                    )
                    .status_text(service.health, tone_label(service.health))
                    .trailing(nive_text::caption(format!(
                        "{} rpm · {} ms · {}% uptime",
                        grouped(service.requests_per_minute),
                        grouped(service.latency_ms),
                        grouped(service.uptime_percent)
                    )))
                    .on_press(Message::OpenService(service.id))
                    .into()
            });

        let content = column![
            DocumentHeader::new("Fleet overview")
                .icon(IconSymbol::Dashboard)
                .trailing(
                    ActionGroup::new().action(
                        ContentAction::icon_label(IconRole::ViewActivity, "Run health check")
                            .on_press(Message::Command(AppCommand::RunHealthCheck))
                    )
                ),
            service_scope,
            cards,
            alert_summary,
            Card::new(
                column![
                    SectionHeader::new("Services")
                        .icon(IconSymbol::Server)
                        .count_badge(self.model.services.len() as u64),
                    column(services).spacing(6),
                ]
                .spacing(8)
            )
            .outlined()
            .fill_width(),
            Card::new(
                column![
                    SectionHeader::new("Dashboard state")
                        .icon(IconSymbol::Dashboard)
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
                        .trailing(ActionGroup::new().action(
                            ContentAction::label("Toggle").on_press(Message::ToggleFilterDirty)
                        ))
                        .fill_width(),
                ]
                .spacing(8)
            )
            .elevated()
            .fill_width(),
        ]
        .spacing(16)
        .padding(24)
        .max_width(CONTENT_MEASURE)
        .width(Length::Fill);

        container(
            scrollable(
                container(content)
                    .width(Length::Fill)
                    .center_x(Length::Fill),
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
            Card::new(MetricCard::new("Latency", grouped(service.latency_ms)).unit("ms"))
                .fill_width(),
            Card::new(MetricCard::new("Uptime", grouped(service.uptime_percent)).unit("%"))
                .fill_width(),
            Card::new(MetricCard::new("Error rate", grouped(service.error_rate_percent)).unit("%"))
                .fill_width(),
        ]
        .spacing(12);
        let host_label = host.map_or("unknown", |host| host.name);

        let content = column![
            DocumentHeader::new(service.name)
                .icon(IconSymbol::Server)
                .trailing(StatusIndicator::new(
                    service.health,
                    tone_label(service.health)
                )),
            cards,
            Card::new(
                KeyValueList::new()
                    .item(MetadataItem::new("Host", host_label))
                    .item(MetadataItem::new(
                        "Environment",
                        self.model.environment_label()
                    ))
                    .item(
                        MetadataItem::new("Deployment revision identifier", "")
                            .code_value("release-2026.07.15+build.9f31ad7c0ffee")
                    )
                    .fill_width()
            )
            .fill_width(),
            self.host_and_runtime_card(host),
            self.dependencies_card(service),
            self.recent_activity_card(service),
            ActionGroup::new()
                .fill_width()
                .wrap()
                .action(
                    ContentAction::icon_label(IconRole::ViewReveal, "Inspect service")
                        .on_press(Message::InspectService(service.id))
                )
                .action(
                    ContentAction::icon_label(IconRole::ViewActivity, "Run health check")
                        .loading(self.model.running_jobs() > 0)
                        .on_press(Message::Command(AppCommand::RunHealthCheck))
                )
                .separator()
                .action(
                    ContentAction::label("Restart service")
                        .destructive()
                        .disabled(true)
                ),
        ]
        .spacing(16)
        .padding(24)
        .max_width(CONTENT_MEASURE)
        .width(Length::Fill);

        container(
            scrollable(
                container(content)
                    .width(Length::Fill)
                    .center_x(Length::Fill),
            )
            .direction(scrollable::Direction::Vertical(overlay_scrollbar()))
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Host and runtime section: the resolved host's zone, CPU, and memory,
    /// so the host relationship is visible without switching panels.
    fn host_and_runtime_card(&self, host: Option<&crate::sim::Host>) -> Element<'_, Message> {
        let body: Element<'_, Message> = match host {
            Some(host) => KeyValueList::new()
                .item(MetadataItem::new("Host", host.name))
                .item(MetadataItem::new("Zone", host.zone))
                .item(MetadataItem::new("Health", tone_label(host.health)).status(host.health))
                .item(MetadataItem::new("CPU", format!("{}%", host.cpu_percent)))
                .item(MetadataItem::new(
                    "Memory",
                    format!("{}%", host.memory_percent),
                ))
                .fill_width()
                .into(),
            None => EmptyState::new("Host unavailable")
                .description("No host record for this service.")
                .into(),
        };

        Card::new(
            column![
                SectionHeader::new("Host and runtime").icon(IconSymbol::Server),
                body,
            ]
            .spacing(8),
        )
        .outlined()
        .fill_width()
        .into()
    }

    /// Dependency section: the other seeded services this one calls, with
    /// their health, so the canvas carries a second selectable collection.
    fn dependencies_card(&self, service: &Service) -> Element<'_, Message> {
        let dependencies = self.model.dependencies_of(service);
        let rows = dependencies.iter().map(|dependency| {
            SelectableItem::new(dependency.name)
                .selected(
                    matches!(self.selected, super::Selection::Service(id) if id == dependency.id),
                )
                .status_text(dependency.health, tone_label(dependency.health))
                .on_press(Message::InspectService(dependency.id))
                .into()
        });

        Card::new(
            column![
                SectionHeader::new("Dependencies")
                    .icon(IconSymbol::Dependency)
                    .count_badge(dependencies.len() as u64),
                column(rows).spacing(6),
            ]
            .spacing(8),
        )
        .outlined()
        .fill_width()
        .into()
    }

    /// Recent-activity section: logs and events scoped to this service,
    /// using the same treatments already used by the bottom panels.
    fn recent_activity_card(&self, service: &Service) -> Element<'_, Message> {
        let (logs, events) = self.model.recent_activity_for(service);
        let rows: Vec<Element<'_, Message>> = events
            .iter()
            .take(4)
            .map(|event| DataRow::new(*event).fill_width().into())
            .chain(
                logs.iter()
                    .take(4)
                    .map(|line| nive_text::code_small(*line).into()),
            )
            .collect();
        let count = rows.len();

        Card::new(
            column![
                SectionHeader::new("Recent activity")
                    .icon(IconRole::ViewActivity)
                    .count_badge(count as u64),
                column(rows).spacing(6),
            ]
            .spacing(8),
        )
        .outlined()
        .fill_width()
        .into()
    }

    pub(super) fn logs_view(&self) -> Element<'_, Message> {
        let lines = self
            .model
            .logs
            .iter()
            .map(|line| nive_text::code_small(line).into());
        scrollable(column(lines).spacing(4).padding(12).width(Length::Fill))
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
                .icon(IconSymbol::Job)
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

impl ServiceScope {
    fn includes(self, service_id: &str) -> bool {
        match self {
            Self::All => true,
            Self::Service(selected) => selected == service_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_dashboard_and_service_documents_build_refined_content_compositions() {
        let app = WorkbenchMonitor::seeded();

        let _: Element<'_, Message> = app.dashboard_document();
        for service in &app.model.services {
            let _: Element<'_, Message> = app.service_document(service.id);
        }
    }

    #[test]
    fn unknown_service_id_resolves_to_the_empty_state_instead_of_a_partial_document() {
        let app = WorkbenchMonitor::seeded();

        assert!(app.model.service("does-not-exist").is_none());
        let _: Element<'_, Message> = app.service_document("does-not-exist");
    }

    #[test]
    fn every_seeded_service_lists_a_real_dependency() {
        let app = WorkbenchMonitor::seeded();

        for service in &app.model.services {
            let dependencies = app.model.dependencies_of(service);
            assert!(
                !dependencies.is_empty(),
                "{} should list a resolvable dependency",
                service.id
            );
        }
    }

    #[test]
    fn every_seeded_service_has_scoped_recent_activity() {
        let app = WorkbenchMonitor::seeded();

        for service in &app.model.services {
            let (logs, events) = app.model.recent_activity_for(service);
            assert!(
                !logs.is_empty() || !events.is_empty(),
                "{} should have scoped recent activity",
                service.id
            );
        }
    }

    #[test]
    fn service_scope_filters_app_data_without_replacing_navigation() {
        assert!(ServiceScope::All.includes("api"));
        assert!(ServiceScope::Service("api").includes("api"));
        assert!(!ServiceScope::Service("api").includes("billing"));

        let mut app = WorkbenchMonitor::seeded();
        app.service_scope = ServiceScope::Service("billing");
        let _: Element<'_, Message> = app.dashboard_document();
        let _: Element<'_, Message> = app.services_view();
    }
}
