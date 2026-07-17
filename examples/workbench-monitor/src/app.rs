mod commands;
mod content;
mod events;
mod inspector;
mod shell;
mod tone;

use std::{borrow::Cow, time::Duration};

use iced::keyboard;
use nive::prelude::ui::DialogRequest;
use nive::{prelude::*, Action, ActionMap};

use crate::sim::Simulation;
use crate::sim::Environment;

pub(crate) type WorkbenchMsg = WorkbenchEvent<DocumentId, &'static str, PanelActionId>;

#[derive(Debug, Clone)]
pub(crate) struct WorkbenchMonitor {
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
    auto_refresh: bool,
    monitor_filter: MonitorFilter,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
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
    AutoRefreshChanged(bool),
    EnvironmentChanged(Environment),
    MonitorFilterChanged(MonitorFilter),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DocumentId {
    Dashboard(&'static str),
    Service(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Selection {
    None,
    Service(&'static str),
    Host(&'static str),
    Alert(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MonitorFilter {
    All,
    Attention,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppCommand {
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
pub(crate) enum PanelActionId {
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
            .theme_catalog(commands::workbench_theme_catalog())
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, impl Into<Effect<Self::Message, Self::Window>>) {
        (Self::seeded(), ())
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
            Message::AutoRefreshChanged(value) => self.auto_refresh = value,
            Message::EnvironmentChanged(environment) => self.model.environment = environment,
            Message::MonitorFilterChanged(filter) => self.monitor_filter = filter,
        }

        Effect::none()
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        let shell = WorkbenchShell::new(self.layout.clone(), Message::Workbench)
            .chrome_size(ControlSize::Sm)
            .pane_constraints(WorkbenchPaneConstraints::default())
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
                .dismiss_on_backdrop(Message::Palette(WorkbenchCommandPaletteEvent::Dismissed))
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
    fn seeded() -> Self {
        let mut layout =
            WorkbenchLayoutState::default().with_active_document(DocumentId::Dashboard("fleet"));
        layout.set_active_panel(WorkbenchRegion::Left, "services");
        layout.set_active_panel(WorkbenchRegion::Right, "inspector");
        layout.set_active_panel(WorkbenchRegion::Bottom, "alerts");

        Self {
            model: Simulation::seeded(),
            layout,
            documents: vec![
                DocumentId::Dashboard("fleet"),
                DocumentId::Dashboard(
                    "Regional capacity forecast with an intentionally long document label",
                ),
                DocumentId::Service("api"),
                DocumentId::Service("billing"),
                DocumentId::Service("search"),
            ],
            selected: Selection::Service("api"),
            inspector_loading_until: None,
            palette: CommandPaletteState::new(),
            commands: commands::commands(),
            theme: ThemePreference::Dark,
            alert_dialog: None,
            dirty_filter: false,
            auto_refresh: true,
            monitor_filter: MonitorFilter::All,
        }
    }
}
