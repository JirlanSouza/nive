mod sim;

use std::borrow::Cow;
use std::time::Duration;

use iced::keyboard;
use nive::prelude::ui::DialogRequest;
use nive::widget::{column, container, row, scrollable, text};
use nive::{prelude::*, Action, ActionMap};

use sim::{Alert, Environment, Simulation};

type WorkbenchMsg = WorkbenchEvent<DocumentId, &'static str, PanelActionId>;

#[derive(Debug, Clone)]
struct WorkbenchMonitor {
    model: Simulation,
    layout: WorkbenchLayoutState<DocumentId, &'static str>,
    documents: Vec<DocumentId>,
    selected: Selection,
    inspector_loading_until: Option<u64>,
    palette: CommandPaletteState,
    commands: Vec<WorkbenchCommand<'static, AppCommand>>,
    theme: ThemePreference,
    alert_dialog: Option<u32>,
    dirty_filter: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    Workbench(WorkbenchMsg),
    Palette(WorkbenchCommandPaletteEvent<AppCommand>),
    PaletteMove(isize),
    Command(AppCommand),
    OpenPalette,
    OpenService(&'static str),
    InspectService(&'static str),
    InspectHost(&'static str),
    ShowAlert(u32),
    AcknowledgeAlert(u32),
    ClearSelection,
    CloseDialog,
    ToggleTheme,
    ToggleFilterDirty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocumentId {
    Dashboard(&'static str),
    Service(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Selection {
    None,
    Service(&'static str),
    Host(&'static str),
    Alert(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppCommand {
    OpenApi,
    OpenBilling,
    AcknowledgeFirstAlert,
    RunHealthCheck,
    SwitchEnvironment,
    ToggleLeft,
    ToggleBottom,
    ToggleTheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PanelActionId {
    Refresh,
    RunHealth,
    Clear,
}

impl Application for WorkbenchMonitor {
    type Message = Message;
    type Window = ();
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("nive-example-workbench-monitor")
            .name("Workbench Monitor")
            .theme_catalog(workbench_theme_catalog())
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, impl Into<Effect<Self::Message, Self::Window>>) {
        let mut layout = WorkbenchLayoutState::default()
            .with_active_document(DocumentId::Dashboard("fleet"));
        layout.set_active_panel(WorkbenchRegion::Left, "services");
        layout.set_active_panel(WorkbenchRegion::Right, "inspector");
        layout.set_active_panel(WorkbenchRegion::Bottom, "alerts");

        (
            Self {
                model: Simulation::seeded(),
                layout,
                documents: vec![DocumentId::Dashboard("fleet"), DocumentId::Service("api")],
                selected: Selection::Service("api"),
                inspector_loading_until: None,
                palette: CommandPaletteState::new(),
                commands: commands(),
                theme: ThemePreference::Dark,
                alert_dialog: None,
                dirty_filter: false,
            },
            (),
        )
    }

    fn update(
        &mut self,
        _context: Context<'_, Self::Window>,
        _message_context: MessageContext<Self::Window>,
        message: Self::Message,
    ) -> impl Into<Effect<Self::Message, Self::Window>> {
        match message {
            Message::Tick => {
                if self.model.advance() {
                    return Effect::toast(Toast::success("Run health check completed"));
                }
            }
            Message::Workbench(event) => self.apply_workbench_event(event),
            Message::Palette(event) => self.apply_palette_event(event),
            Message::PaletteMove(delta) => {
                self.palette.move_highlight(delta, self.commands.len());
                self.apply_palette_event(WorkbenchCommandPaletteEvent::Highlighted(
                    self.palette.highlighted,
                ));
            }
            Message::Command(command) => self.run_command(command),
            Message::OpenPalette => self.palette.open(),
            Message::OpenService(id) => {
                self.open_document(DocumentId::Service(id));
                self.select(Selection::Service(id));
            }
            Message::InspectService(id) => self.select(Selection::Service(id)),
            Message::InspectHost(id) => self.select(Selection::Host(id)),
            Message::ShowAlert(id) => {
                self.alert_dialog = Some(id);
                self.select(Selection::Alert(id));
            }
            Message::AcknowledgeAlert(id) => self.model.acknowledge_alert(id),
            Message::ClearSelection => self.select(Selection::None),
            Message::CloseDialog => self.alert_dialog = None,
            Message::ToggleTheme => {
                self.theme = match self.theme {
                    ThemePreference::Dark => ThemePreference::Light,
                    _ => ThemePreference::Dark,
                };
                return Effect::theme(self.theme);
            }
            Message::ToggleFilterDirty => self.dirty_filter = !self.dirty_filter,
        }

        Effect::none()
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        let shell = WorkbenchShell::new(self.layout.clone(), Message::Workbench)
            .toolbar(self.toolbar())
            .left_panels(self.left_panels())
            .documents(self.document_tabs())
            .document_content(self.document_content())
            .right_panels(self.right_panels())
            .bottom_panels(self.bottom_panels())
            .status(self.status_bar())
            .view();

        let mut view = ScreenView::new(shell);

        if self.palette.open {
            view = view.dialog(
                DialogRequest::new(
                    WorkbenchCommandPalette::new(&self.palette, &self.commands, Message::Palette)
                        .view(),
                )
                .dismiss_on_backdrop(Message::Palette(
                    WorkbenchCommandPaletteEvent::Dismissed,
                ))
                .dismiss_on_escape(Message::Palette(WorkbenchCommandPaletteEvent::Dismissed)),
            );
        }

        if let Some(alert_id) = self.alert_dialog {
            if let Some(alert) = self.model.alert(alert_id) {
                view = view.dialog(
                    DialogRequest::new(self.alert_dialog(alert))
                        .dismiss_on_backdrop(Message::CloseDialog)
                        .dismiss_on_escape(Message::CloseDialog),
                );
            }
        }

        view
    }

    fn subscription(&self, _context: Context<'_, Self::Window>) -> Subscription<Self::Message> {
        let timer = iced::time::every(Duration::from_millis(900)).map(|_| Message::Tick);
        let keys = keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(key),
                modifiers,
                ..
            } if modifiers.command() && key.eq_ignore_ascii_case("k") => Some(Message::OpenPalette),
            keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::ArrowDown),
                ..
            } => Some(Message::PaletteMove(1)),
            keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::ArrowUp),
                ..
            } => Some(Message::PaletteMove(-1)),
            _ => None,
        });

        Subscription::batch([timer, keys])
    }

    fn theme(
        &self,
        _context: Context<'_, Self::Window>,
        _window: Option<WindowContext<Self::Window>>,
    ) -> ThemePreference {
        self.theme
    }

    fn actions(&self, _context: Context<'_, Self::Window>) -> ActionMap<Self::Message> {
        ActionMap::new()
            .action(Action::new(
                "monitor.palette",
                "Open command palette",
                Message::OpenPalette,
            ))
            .action(Action::new(
                "monitor.health",
                "Run health check",
                Message::Command(AppCommand::RunHealthCheck),
            ))
            .action(Action::new(
                "monitor.theme",
                "Toggle theme",
                Message::ToggleTheme,
            ))
    }

    fn window_title<'a>(
        &'a self,
        _context: Context<'a, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> impl Into<Cow<'a, str>> + 'a {
        Cow::Borrowed("Workbench Monitor")
    }
}

impl WorkbenchMonitor {
    fn apply_workbench_event(&mut self, event: WorkbenchMsg) {
        event.apply_to(&mut self.layout);

        match event {
            WorkbenchEvent::Document(WorkbenchDocumentEvent::CloseRequested { id, .. }) => {
                self.close_document(id);
            }
            WorkbenchEvent::Document(WorkbenchDocumentEvent::Reorder { dragged, target }) => {
                self.reorder_document(dragged, target);
            }
            WorkbenchEvent::Document(WorkbenchDocumentEvent::TearOffRequested { dragged, .. }) => {
                self.model.events.push(format!("Tear-off requested for {:?}", dragged));
            }
            WorkbenchEvent::Document(WorkbenchDocumentEvent::ContextRequested { id, .. }) => {
                self.model
                    .events
                    .push(format!("Document context requested for {:?}", id));
            }
            WorkbenchEvent::Panel(WorkbenchPanelEvent::Action { action_id, .. }) => match action_id {
                PanelActionId::Refresh => self.model.events.push("Panel refresh requested".into()),
                PanelActionId::RunHealth => self.model.run_health_check(),
                PanelActionId::Clear => self.model.events.clear(),
            },
            WorkbenchEvent::Panel(WorkbenchPanelEvent::CloseRequested { region, .. }) => {
                self.layout.collapse_region(region);
            }
            WorkbenchEvent::Layout(_) | WorkbenchEvent::Document(_) | WorkbenchEvent::Panel(_) => {}
            _ => {}
        }
    }

    fn apply_palette_event(&mut self, event: WorkbenchCommandPaletteEvent<AppCommand>) {
        match event {
            WorkbenchCommandPaletteEvent::QueryChanged(query) => self.palette.set_query(query),
            WorkbenchCommandPaletteEvent::Highlighted(index) => self.palette.highlighted = index,
            WorkbenchCommandPaletteEvent::Submitted(command) => {
                self.palette.close();
                self.run_command(command);
            }
            WorkbenchCommandPaletteEvent::Dismissed => self.palette.close(),
            _ => {}
        }
    }

    fn run_command(&mut self, command: AppCommand) {
        match command {
            AppCommand::OpenApi => self.open_document(DocumentId::Service("api")),
            AppCommand::OpenBilling => self.open_document(DocumentId::Service("billing")),
            AppCommand::AcknowledgeFirstAlert => {
                let first_alert = self.model.active_alerts().next().map(|alert| alert.id);
                if let Some(id) = first_alert {
                    self.model.acknowledge_alert(id);
                }
            }
            AppCommand::RunHealthCheck => self.model.run_health_check(),
            AppCommand::SwitchEnvironment => self.model.toggle_environment(),
            AppCommand::ToggleLeft => self.toggle_region(WorkbenchRegion::Left),
            AppCommand::ToggleBottom => self.toggle_region(WorkbenchRegion::Bottom),
            AppCommand::ToggleTheme => {
                self.theme = match self.theme {
                    ThemePreference::Dark => ThemePreference::Light,
                    _ => ThemePreference::Dark,
                };
            }
        }
    }

    fn toolbar(&self) -> Element<'_, Message> {
        row![
            button("Run health check").on_press(Message::Command(AppCommand::RunHealthCheck)),
            button("Command palette").on_press(Message::OpenPalette),
            button("Toggle theme").on_press(Message::ToggleTheme),
            button("Switch environment").on_press(Message::Command(AppCommand::SwitchEnvironment)),
            button("Clear selection").on_press(Message::ClearSelection),
            button("Open latest alert").on_press_maybe(
                self.model
                    .active_alerts()
                    .next()
                    .map(|alert| Message::ShowAlert(alert.id))
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    }

    fn left_panels(&self) -> Vec<WorkbenchPanel<'_, &'static str, PanelActionId, Message>> {
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

    fn right_panels(&self) -> Vec<WorkbenchPanel<'_, &'static str, PanelActionId, Message>> {
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

    fn bottom_panels(&self) -> Vec<WorkbenchPanel<'_, &'static str, PanelActionId, Message>> {
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

    fn document_tabs(&self) -> Vec<WorkbenchDocument<'static, DocumentId>> {
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

    fn document_content(&self) -> Element<'_, Message> {
        match self.layout.active_document().copied() {
            Some(DocumentId::Service(id)) => self.service_document(id),
            Some(DocumentId::Dashboard(_)) | None => self.dashboard_document(),
        }
    }

    fn status_bar(&self) -> StatusBar<'static> {
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

    fn services_view(&self) -> Element<'_, Message> {
        let rows = self.model.services.iter().map(|service| {
            DataRow::new(service.name)
                .tone(service.health)
                .value(format!("{} rpm · {} ms", service.requests_per_minute, service.latency_ms))
                .trailing(button("Open").on_press(Message::OpenService(service.id)))
                .fill_width()
                .into()
        });

        scrollable(column(rows).spacing(8).padding(12)).into()
    }

    fn hosts_view(&self) -> Element<'_, Message> {
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

    fn alerts_left_view(&self) -> Element<'_, Message> {
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

    fn dashboards_view(&self) -> Element<'_, Message> {
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

    fn settings_view(&self) -> Element<'_, Message> {
        column![
            DataRow::new("Environment")
                .value(self.model.environment_label())
                .trailing(button("Switch").on_press(Message::Command(AppCommand::SwitchEnvironment)))
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

    fn dashboard_document(&self) -> Element<'_, Message> {
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

    fn service_document(&self, service_id: &'static str) -> Element<'_, Message> {
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
                    .item(MetadataItem::new("Environment", self.model.environment_label()))
                    .item(MetadataItem::new("Health", tone_label(service.health)).tone(service.health))
                    .fill_width(),
                row![
                    button("Inspect service").on_press(Message::InspectService(service.id)),
                    button("Run health check").on_press(Message::Command(AppCommand::RunHealthCheck)),
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

    fn inspector_content(&self) -> Option<Element<'_, Message>> {
        match self.selected {
            Selection::None => None,
            Selection::Service(id) => self.model.service(id).map(|service| {
                KeyValueList::new()
                    .item(MetadataItem::new("Service", service.name))
                    .item(MetadataItem::new("Host", service.host_id))
                    .item(MetadataItem::new("Health", tone_label(service.health)).tone(service.health))
                    .item(MetadataItem::new("Latency", "").value(text(format!("{} ms", service.latency_ms))))
                    .item(MetadataItem::new("RPM", "").value(text(service.requests_per_minute.to_string())))
                    .fill_width()
                    .into()
            }),
            Selection::Host(id) => self.model.host(id).map(|host| {
                KeyValueList::new()
                    .item(MetadataItem::new("Host", host.name))
                    .item(MetadataItem::new("Zone", host.zone))
                    .item(MetadataItem::new("Health", tone_label(host.health)).tone(host.health))
                    .item(MetadataItem::new("CPU", "").value(text(format!("{}%", host.cpu_percent))))
                    .item(MetadataItem::new("Memory", "").value(text(format!("{}%", host.memory_percent))))
                    .fill_width()
                    .into()
            }),
            Selection::Alert(id) => self.model.alert(id).map(|alert| {
                KeyValueList::new()
                    .item(MetadataItem::new("Alert", alert.title))
                    .item(MetadataItem::new("Service", alert.service_id))
                    .item(MetadataItem::new("Severity", tone_label(alert.severity)).tone(alert.severity))
                    .item(MetadataItem::new("State", if alert.active { "active" } else { "acknowledged" }))
                    .fill_width()
                    .into()
            }),
        }
    }

    fn logs_view(&self) -> Element<'_, Message> {
        let lines = self.model.logs.iter().map(|line| text(line).into());
        scrollable(column(lines).spacing(4).padding(12)).into()
    }

    fn events_view(&self) -> Element<'_, Message> {
        let rows = self.model.events.iter().rev().map(|event| {
            DataRow::new(event.as_str())
                .value(format!("tick {}", self.model.tick))
                .fill_width()
                .into()
        });

        scrollable(column(rows).spacing(8).padding(12)).into()
    }

    fn jobs_view(&self) -> Element<'_, Message> {
        if self.model.jobs.is_empty() {
            return EmptyState::new("No jobs")
                .description("Run a health check to exercise operation progress.")
                .icon(IconRole::ViewRefresh)
                .action(button("Run health check").on_press(Message::Command(AppCommand::RunHealthCheck)))
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

    fn alert_dialog(&self, alert: &Alert) -> Element<'_, Message> {
        column![
            text(alert.title).size(24),
            KeyValueList::new()
                .item(MetadataItem::new("Service", alert.service_id))
                .item(MetadataItem::new("Severity", tone_label(alert.severity)).tone(alert.severity))
                .item(MetadataItem::new("Environment", self.model.environment_label()))
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

    fn problems(&self) -> Vec<Problem<'static>> {
        self.model
            .active_alerts()
            .map(|alert| {
                Problem::new(problem_severity(alert.severity), "monitor", alert.title)
                    .location(ProblemLocation::new(alert.service_id))
            })
            .collect()
    }

    fn select(&mut self, selection: Selection) {
        self.selected = selection;
        self.inspector_loading_until = Some(self.model.tick + 2);
    }

    fn open_document(&mut self, id: DocumentId) {
        if !self.documents.contains(&id) {
            self.documents.push(id);
        }
        self.layout.set_active_document(Some(id));
    }

    fn close_document(&mut self, id: DocumentId) {
        self.documents.retain(|document| *document != id);
        if self.layout.active_document() == Some(&id) {
            self.layout
                .set_active_document(self.documents.first().copied());
        }
    }

    fn reorder_document(
        &mut self,
        dragged: Vec<DocumentId>,
        target: WorkbenchDocumentDropTarget<DocumentId>,
    ) {
        let Some(document) = dragged.first().copied() else {
            return;
        };
        self.documents.retain(|existing| *existing != document);
        let index = match target {
            WorkbenchDocumentDropTarget::Before(target) => self
                .documents
                .iter()
                .position(|existing| *existing == target)
                .unwrap_or(0),
            WorkbenchDocumentDropTarget::After(target) => self
                .documents
                .iter()
                .position(|existing| *existing == target)
                .map(|index| index + 1)
                .unwrap_or(self.documents.len()),
            WorkbenchDocumentDropTarget::Unknown => self.documents.len(),
            _ => self.documents.len(),
        };
        self.documents.insert(index.min(self.documents.len()), document);
    }

    fn toggle_region(&mut self, region: WorkbenchRegion) {
        if self.layout.is_collapsed(region) {
            self.layout.restore_region(region);
        } else {
            self.layout.collapse_region(region);
        }
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

    fn active_alert_count(&self) -> usize {
        self.model.active_alerts().count()
    }

    fn overall_tone(&self) -> ToneRole {
        if self
            .model
            .services
            .iter()
            .any(|service| service.health == ToneRole::Danger)
        {
            ToneRole::Danger
        } else if self
            .model
            .services
            .iter()
            .any(|service| service.health == ToneRole::Warning)
        {
            ToneRole::Warning
        } else {
            ToneRole::Success
        }
    }

    fn host_tone(&self) -> ToneRole {
        if self
            .model
            .hosts
            .iter()
            .any(|host| host.health == ToneRole::Warning)
        {
            ToneRole::Warning
        } else {
            ToneRole::Success
        }
    }

    fn alert_tone(&self) -> ToneRole {
        if self.active_alert_count() == 0 {
            ToneRole::Success
        } else {
            ToneRole::Warning
        }
    }

    fn connection_tone(&self) -> ToneRole {
        match self.model.environment {
            Environment::Production => ToneRole::Success,
            Environment::Staging => ToneRole::Info,
        }
    }
}

fn problem_severity(tone: ToneRole) -> ProblemSeverity {
    match tone {
        ToneRole::Danger => ProblemSeverity::Error,
        ToneRole::Warning => ProblemSeverity::Warning,
        ToneRole::Neutral | ToneRole::Accent | ToneRole::Info | ToneRole::Success => {
            ProblemSeverity::Info
        }
    }
}

fn tone_label(tone: ToneRole) -> &'static str {
    match tone {
        ToneRole::Neutral => "neutral",
        ToneRole::Accent => "active",
        ToneRole::Info => "info",
        ToneRole::Success => "healthy",
        ToneRole::Warning => "warning",
        ToneRole::Danger => "critical",
    }
}

fn commands() -> Vec<WorkbenchCommand<'static, AppCommand>> {
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

fn workbench_theme_catalog() -> ThemeCatalog {
    ThemeCatalog::new(
        Theme::builder("Workbench Monitor Light", ThemeMode::Light)
            .density(ThemeDensity::Compact)
            .build(),
        Theme::builder("Workbench Monitor Dark", ThemeMode::Dark)
            .density(ThemeDensity::Compact)
            .build(),
    )
}

fn main() -> nive::Result {
    nive::run::<WorkbenchMonitor>()
}
