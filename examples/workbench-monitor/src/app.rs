mod commands;
mod content;
mod events;
mod explorer;
mod format;
mod inspector;
mod shell;
mod tone;

use std::{borrow::Cow, time::Duration};

use nive::prelude::ui::{ToastInsets, UserFacingError};
use nive::{prelude::*, ActionMap};

use crate::sim::Environment;
use crate::sim::Simulation;
use crate::sim::SimulationMode;

use explorer::{ExplorerContextMenuState, ExplorerNodeId};

pub(crate) type WorkbenchMsg = WorkbenchEvent<DocumentId, &'static str, PanelActionId>;

#[derive(Debug, Clone)]
pub(crate) struct WorkbenchMonitor {
    model: Simulation,
    mode: SimulationMode,
    layout: WorkbenchLayoutState<DocumentId, &'static str>,
    documents: Vec<DocumentId>,
    selected: Selection,
    inspector_loading_until: Option<u64>,
    palette_open: bool,
    palette_query: String,
    commands: ActionMap<Message>,
    theme: ThemePreference,
    alert_dialog: Option<u32>,
    sync_error: Option<UserFacingError>,
    dirty_filter: bool,
    auto_refresh: bool,
    monitor_filter: MonitorFilter,
    service_scope: ServiceScope,
    explorer: TreeState<ExplorerNodeId>,
    explorer_diagnostics_failed: bool,
    explorer_diagnostics_loading: bool,
    explorer_context_menu: Option<ExplorerContextMenuState>,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Tick,
    Workbench(WorkbenchMsg),
    PaletteQueryChanged(String),
    PaletteDismissed,
    Command(AppCommand),
    OpenPalette,
    OpenService(&'static str),
    InspectService(&'static str),
    InspectHost(&'static str),
    ShowAlert(u32),
    AcknowledgeAlert(u32),
    CloseDialog,
    SimulateSyncFailure,
    ShowSyncErrorDetails(UserFacingError),
    CloseSyncErrorDialog,
    ToggleTheme,
    ToggleFilterDirty,
    AutoRefreshChanged(bool),
    EnvironmentChanged(Environment),
    MonitorFilterChanged(MonitorFilter),
    ServiceScopeChanged(ServiceScope),
    ExplorerEvent(TreeEvent<ExplorerNodeId>),
    ExplorerDiagnosticsFailed,
    ExplorerContextAction(&'static str),
    ExplorerContextDismissed,
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
pub(crate) enum ServiceScope {
    All,
    Service(&'static str),
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PanelActionId {
    Refresh,
    RunHealth,
    Clear,
    ClearSelection,
}

impl Application for WorkbenchMonitor {
    type Message = Message;
    type Window = ();
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("nive-example-workbench-monitor")
            .name("Workbench Monitor")
            .theme_catalog(commands::workbench_theme_catalog())
            // Explicit rather than relying on the default, so the status-bar
            // safe inset below reads as deliberate too. `chrome_size` here
            // must match `WorkbenchShell::chrome_size` in `view()`, since
            // that's what the status bar actually renders at.
            .toast_position(ToastPosition::BottomEnd)
            .toast_insets(ToastInsets {
                bottom: StatusBar::height(ControlSize::Sm),
                ..ToastInsets::NONE
            })
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
            Message::PaletteQueryChanged(query) => self.palette_query = query,
            Message::PaletteDismissed => self.close_palette(),
            Message::Command(command) => self.run_command(command),
            Message::OpenPalette => self.palette_open = true,
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
            Message::CloseDialog => self.alert_dialog = None,
            Message::SimulateSyncFailure => {
                self.close_palette();
                let error = UserFacingError::custom(
                    "workbench-monitor",
                    "Could not sync fleet status (endpoint: https://status.internal.example)",
                );
                return Effect::toast(
                    Toast::error(error.clone())
                        .with_action("View details", Message::ShowSyncErrorDetails(error)),
                );
            }
            Message::ShowSyncErrorDetails(error) => self.sync_error = Some(error),
            Message::CloseSyncErrorDialog => self.sync_error = None,
            Message::ToggleTheme => {
                self.close_palette();
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
            Message::ServiceScopeChanged(scope) => {
                self.service_scope = scope;
                self.dirty_filter = true;
            }
            Message::ExplorerEvent(event) => {
                if let Some(task) = self.apply_explorer_event(event) {
                    return Effect::task(task);
                }
            }
            Message::ExplorerDiagnosticsFailed => self.apply_explorer_diagnostics_failed(),
            Message::ExplorerContextAction(action) => self.apply_explorer_context_action(action),
            Message::ExplorerContextDismissed => self.explorer_context_menu = None,
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

        let all_items = self.palette_items();
        let visible = command_palette_filter(&self.palette_query, &all_items);
        let items: Vec<_> = visible
            .into_iter()
            .map(|index| all_items[index].clone())
            .collect();
        let content = CommandPalette::new(shell)
            .open(self.palette_open)
            .query(self.palette_query.as_str())
            .items(items)
            .placeholder("Search commands")
            .on_query_change(Message::PaletteQueryChanged)
            .on_dismiss(Message::PaletteDismissed);

        let mut view = ScreenView::new(content);

        if let Some(alert_id) = self.alert_dialog {
            if let Some(alert) = self.model.alert(alert_id) {
                view = view.dialog(self.alert_dialog_request(alert));
            }
        } else if let Some(error) = &self.sync_error {
            view = view.dialog(self.sync_error_dialog_request(error));
        }

        view
    }

    fn subscription(&self, _context: Context<'_, Self::Window>) -> Subscription<Self::Message> {
        if self.installs_tick_timer() {
            iced::time::every(Duration::from_millis(900)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn theme(
        &self,
        _context: Context<'_, Self::Window>,
        _window: Option<WindowContext<Self::Window>>,
    ) -> ThemePreference {
        self.theme
    }

    fn actions(&self, _context: Context<'_, Self::Window>) -> ActionMap<Self::Message> {
        self.commands.clone()
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
    /// Only `Live` mode installs the 900ms tick timer; `Frozen` advances only
    /// from explicit user actions.
    fn installs_tick_timer(&self) -> bool {
        matches!(self.mode, SimulationMode::Live)
    }

    fn palette_items(&self) -> Vec<CommandPaletteItem<'_, Message>> {
        action_palette_items(&self.commands)
            .into_iter()
            .filter(|item| item.id != "monitor.palette")
            .collect()
    }

    fn seeded() -> Self {
        let mut layout =
            WorkbenchLayoutState::default().with_active_document(DocumentId::Dashboard("fleet"));
        layout.set_active_panel(WorkbenchRegion::Left, "services");
        layout.set_active_panel(WorkbenchRegion::Right, "inspector");
        layout.set_active_panel(WorkbenchRegion::Bottom, "alerts");
        layout.set_region_size(WorkbenchRegion::Bottom, 180.0);

        let mode = SimulationMode::from_env();

        Self {
            model: match mode {
                SimulationMode::Live => Simulation::seeded(),
                SimulationMode::Frozen => Simulation::frozen(),
            },
            mode,
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
            palette_open: false,
            palette_query: String::new(),
            commands: commands::commands(),
            theme: ThemePreference::Dark,
            alert_dialog: None,
            sync_error: None,
            dirty_filter: false,
            auto_refresh: true,
            monitor_filter: MonitorFilter::All,
            service_scope: ServiceScope::All,
            explorer: TreeState::default(),
            explorer_diagnostics_failed: false,
            explorer_diagnostics_loading: false,
            explorer_context_menu: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_layout_owns_the_bottom_size_without_leaking_a_framework_default() {
        let app = WorkbenchMonitor::seeded();
        let sizes = app.layout.pane_sizes();

        assert_eq!(sizes.bottom, 180.0);
        // Region widths in logical pixels, both left at the framework default.
        assert_eq!(sizes.left, 240.0);
        assert_eq!(sizes.right, 300.0);
        assert_eq!(WorkbenchPaneSizes::default().bottom, 240.0);
    }

    #[test]
    fn only_live_mode_installs_the_tick_timer() {
        let mut app = WorkbenchMonitor::seeded();

        app.mode = SimulationMode::Live;
        assert!(app.installs_tick_timer());

        app.mode = SimulationMode::Frozen;
        assert!(!app.installs_tick_timer());
    }

    #[test]
    fn palette_does_not_offer_its_own_open_action() {
        let app = WorkbenchMonitor::seeded();
        let items = app.palette_items();

        assert!(items.iter().all(|item| item.id != "monitor.palette"));
        assert!(items
            .iter()
            .any(|item| item.id == "monitor.demo_sync_failure"));
    }
}
