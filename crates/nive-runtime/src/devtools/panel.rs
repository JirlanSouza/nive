use std::collections::{BTreeMap, BTreeSet};

use iced::{window, Task};

use crate::{lifecycle::open_window, WindowChrome, WindowHandle, WindowMode, WindowSpec};

use super::command::{DevtoolsRowId, SimulateAction, SimulateResult};
use super::types::SimulatorEntry;

const DEFAULT_ERROR_MESSAGE: &str = "Devtools injected failure";
const NIVE_DEVTOOLS_ENV_VAR: &str = "NIVE_DEVTOOLS";
const NIVE_DEVTOOLS_TAB_ENV_VAR: &str = "NIVE_DEVTOOLS_TAB";

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DevtoolsConfig {
    enabled: bool,
    initial_tab: Option<DevtoolsPanelTab>,
    probe_env: Option<&'static str>,
}

impl DevtoolsConfig {
    pub fn from_env() -> Self {
        let mut config = Self::from_env_var(NIVE_DEVTOOLS_ENV_VAR);
        if let Ok(raw) = std::env::var(NIVE_DEVTOOLS_TAB_ENV_VAR) {
            if let Some(tab) = parse_devtools_tab(&raw) {
                config.initial_tab = Some(tab);
            }
        }
        config
    }

    pub fn from_env_var(env_var: &str) -> Self {
        Self::from_env_value(std::env::var(env_var).ok().as_deref())
    }

    pub fn from_env_value(value: Option<&str>) -> Self {
        Self {
            enabled: value.is_some_and(env_flag_enabled),
            ..Default::default()
        }
    }

    pub fn enabled(self) -> bool {
        self.enabled
    }

    pub fn initial_tab(self) -> Option<DevtoolsPanelTab> {
        self.initial_tab
    }

    pub fn with_initial_tab(mut self, tab: DevtoolsPanelTab) -> Self {
        self.initial_tab = Some(tab);
        self
    }

    pub fn probe_env(mut self, env_var: &'static str) -> Self {
        self.probe_env = Some(env_var);
        self
    }

    pub fn probe_env_var(self) -> Option<&'static str> {
        self.probe_env
    }
}

// ---------------------------------------------------------------------------
// Tabs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevtoolsPanelTab {
    Resources,
    Operations,
}

// ---------------------------------------------------------------------------
// Messages and effects
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevtoolsPanelMessage {
    SelectTab(DevtoolsPanelTab),
    SearchChanged(String),
    ToggleActiveOnly(bool),
    ToggleRowExpanded(DevtoolsRowId),
    ClearRowCommandError(DevtoolsRowId),
    ClearLastError,
    /// Update the error message text field for the given path.
    ErrorMessageChanged {
        path: String,
        value: String,
    },
    /// Apply a simulate action to the state at `path`.
    Simulate {
        path: String,
        action: SimulateAction,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevtoolsPanelEffect {
    Simulate {
        path: String,
        action: SimulateAction,
    },
}

// ---------------------------------------------------------------------------
// Panel state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DevtoolsPanelState {
    pub active_tab: DevtoolsPanelTab,
    /// Cached simulator entries, refreshed after each simulate action.
    pub entries: Vec<SimulatorEntry>,
    query: String,
    active_only: bool,
    expanded_rows: BTreeSet<DevtoolsRowId>,
    /// Per-path error message inputs (shown in the expanded row).
    error_inputs: BTreeMap<String, String>,
    /// Per-row simulation errors.
    row_errors: BTreeMap<DevtoolsRowId, String>,
    last_error: Option<String>,
}

impl DevtoolsPanelState {
    pub fn new() -> Self {
        Self {
            active_tab: DevtoolsPanelTab::Resources,
            entries: Vec::new(),
            query: String::new(),
            active_only: false,
            expanded_rows: BTreeSet::new(),
            error_inputs: BTreeMap::new(),
            row_errors: BTreeMap::new(),
            last_error: None,
        }
    }

    pub fn with_config(mut self, config: DevtoolsConfig) -> Self {
        if let Some(tab) = config.initial_tab() {
            self.active_tab = tab;
        }
        self
    }

    pub fn update(&mut self, message: DevtoolsPanelMessage) -> Option<DevtoolsPanelEffect> {
        match message {
            DevtoolsPanelMessage::SelectTab(tab) => {
                self.active_tab = tab;
                None
            }
            DevtoolsPanelMessage::SearchChanged(query) => {
                self.query = query;
                None
            }
            DevtoolsPanelMessage::ToggleActiveOnly(v) => {
                self.active_only = v;
                None
            }
            DevtoolsPanelMessage::ToggleRowExpanded(row_id) => {
                if !self.expanded_rows.remove(&row_id) {
                    self.expanded_rows.insert(row_id);
                }
                None
            }
            DevtoolsPanelMessage::ClearRowCommandError(row_id) => {
                self.row_errors.remove(&row_id);
                None
            }
            DevtoolsPanelMessage::ClearLastError => {
                self.last_error = None;
                self.row_errors.clear();
                None
            }
            DevtoolsPanelMessage::ErrorMessageChanged { path, value } => {
                self.error_inputs.insert(path, value);
                None
            }
            DevtoolsPanelMessage::Simulate { path, action } => {
                Some(DevtoolsPanelEffect::Simulate { path, action })
            }
        }
    }

    /// Record the result of a simulate action and update per-row error state.
    pub fn record_simulate_result(
        &mut self,
        path: &str,
        is_resource: bool,
        result: SimulateResult,
    ) {
        let row_id = if is_resource {
            DevtoolsRowId::Resource(path.to_string())
        } else {
            DevtoolsRowId::Operation(path.to_string())
        };
        match result.panel_error(path) {
            Some(error) => {
                self.row_errors.insert(row_id, error.clone());
                self.last_error = Some(error);
            }
            None => {
                self.row_errors.remove(&row_id);
                self.last_error = None;
            }
        }
    }

    // --- Accessors ---

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn active_only(&self) -> bool {
        self.active_only
    }

    pub fn is_row_expanded(&self, row_id: &DevtoolsRowId) -> bool {
        self.expanded_rows.contains(row_id)
    }

    pub fn row_error(&self, row_id: &DevtoolsRowId) -> Option<&str> {
        self.row_errors.get(row_id).map(String::as_str)
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    pub fn error_input(&self, path: &str) -> String {
        self.error_inputs
            .get(path)
            .cloned()
            .unwrap_or_else(|| DEFAULT_ERROR_MESSAGE.to_string())
    }
}

impl Default for DevtoolsPanelState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Host state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DevtoolsHostState {
    panel: Option<DevtoolsPanelState>,
}

impl DevtoolsHostState {
    pub fn new(panel: Option<DevtoolsPanelState>) -> Self {
        Self { panel }
    }

    pub fn disabled() -> Self {
        Self { panel: None }
    }

    pub fn is_enabled(&self) -> bool {
        self.panel.is_some()
    }

    pub fn panel(&self) -> Option<&DevtoolsPanelState> {
        self.panel.as_ref()
    }

    pub fn panel_mut(&mut self) -> Option<&mut DevtoolsPanelState> {
        self.panel.as_mut()
    }

    pub fn update(&mut self, message: DevtoolsPanelMessage) -> Option<DevtoolsPanelEffect> {
        self.panel.as_mut()?.update(message)
    }

    pub fn open_sidecar_window<K, Message>(
        &self,
        kind: K,
        icon: Option<window::Icon>,
        on_open: impl Fn(window::Id) -> Message + Send + 'static,
    ) -> Option<(WindowHandle<K>, Task<Message>)>
    where
        Message: Send + 'static,
    {
        if !self.is_enabled() {
            return None;
        }
        let (window_id, open_task) =
            open_window(DevtoolsWindowSpec::default().window_spec(), icon, on_open);
        Some((WindowHandle::auxiliary(kind, window_id), open_task))
    }
}

// ---------------------------------------------------------------------------
// Window spec
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DevtoolsWindowSpec {
    pub size: iced::Size,
    pub min_size: iced::Size,
}

impl Default for DevtoolsWindowSpec {
    fn default() -> Self {
        Self {
            size: iced::Size::new(940.0, 640.0),
            min_size: iced::Size::new(820.0, 520.0),
        }
    }
}

impl DevtoolsWindowSpec {
    pub fn window_spec(self) -> WindowSpec {
        WindowSpec {
            role: crate::WindowRole::Auxiliary,
            cardinality: crate::WindowCardinality::Single,
            size: self.size,
            position: window::Position::Centered,
            min_size: Some(self.min_size),
            max_size: None,
            resizable: true,
            decorations: true,
            transparent: false,
            mode: WindowMode::Windowed,
            chrome: WindowChrome::UnifiedTitlebar,
            level: window::Level::AlwaysOnTop,
            session_key: None,
        }
    }

    pub fn title_for_app(app_name: &str) -> String {
        format!("{app_name} · Devtools")
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn env_flag_enabled(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "y" | "on" | "open"
    )
}

fn parse_devtools_tab(value: &str) -> Option<DevtoolsPanelTab> {
    match value.trim().to_ascii_lowercase().as_str() {
        "resources" => Some(DevtoolsPanelTab::Resources),
        "operations" => Some(DevtoolsPanelTab::Operations),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tests (task 9.5 — panel enumeration + forcing wiring)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devtools::types::DevtoolStateSnapshot;
    use crate::devtools::{SimulatorCapabilities, SimulatorKind};
    use crate::inspect::SimulableSnapshot;

    fn panel_with_entries(entries: Vec<SimulatorEntry>) -> DevtoolsPanelState {
        let mut p = DevtoolsPanelState::new();
        p.entries = entries;
        p
    }

    fn resource_entry(path: &str) -> SimulatorEntry {
        SimulatorEntry {
            path: path.to_string(),
            label: path.to_string(),
            kind: SimulatorKind::Resource,
            capabilities: SimulatorCapabilities::default(),
            snapshot: SimulableSnapshot::Idle,
        }
    }

    fn operation_entry(path: &str) -> SimulatorEntry {
        SimulatorEntry {
            path: path.to_string(),
            label: path.to_string(),
            kind: SimulatorKind::Operation,
            capabilities: SimulatorCapabilities::default(),
            snapshot: SimulableSnapshot::Idle,
        }
    }

    #[test]
    fn devtools_config_defaults_to_disabled() {
        assert!(!DevtoolsConfig::default().enabled());
        assert!(!DevtoolsConfig::from_env_value(None).enabled());
    }

    #[test]
    fn devtools_config_accepts_enabled_env_values() {
        for value in ["1", "true", "yes", "y", "on", "open", " TRUE "] {
            assert!(DevtoolsConfig::from_env_value(Some(value)).enabled());
        }
        for value in ["", "0", "false", "off", "no"] {
            assert!(!DevtoolsConfig::from_env_value(Some(value)).enabled());
        }
    }

    #[test]
    fn devtools_config_probe_env_builder_sets_var_name() {
        let config = DevtoolsConfig::default().probe_env("APP_DEV_ERROR");
        assert_eq!(config.probe_env_var(), Some("APP_DEV_ERROR"));
    }

    #[test]
    fn devtools_config_with_initial_tab_applies_tab() {
        let config = DevtoolsConfig::default().with_initial_tab(DevtoolsPanelTab::Operations);
        let panel = DevtoolsPanelState::new().with_config(config);
        assert_eq!(panel.active_tab, DevtoolsPanelTab::Operations);
    }

    #[test]
    fn panel_tracks_search_and_row_expansion() {
        let mut panel = DevtoolsPanelState::new();
        let row_id = DevtoolsRowId::Resource("auth.profile".to_string());

        panel.update(DevtoolsPanelMessage::SearchChanged("auth".to_string()));
        panel.update(DevtoolsPanelMessage::ToggleActiveOnly(true));
        panel.update(DevtoolsPanelMessage::ToggleRowExpanded(row_id.clone()));

        assert_eq!(panel.query(), "auth");
        assert!(panel.active_only());
        assert!(panel.is_row_expanded(&row_id));

        panel.update(DevtoolsPanelMessage::ToggleRowExpanded(row_id.clone()));
        assert!(!panel.is_row_expanded(&row_id));
    }

    #[test]
    fn panel_simulate_message_produces_effect() {
        let mut panel = DevtoolsPanelState::new();
        let effect = panel.update(DevtoolsPanelMessage::Simulate {
            path: "auth.profile".to_string(),
            action: SimulateAction::Loading,
        });
        assert_eq!(
            effect,
            Some(DevtoolsPanelEffect::Simulate {
                path: "auth.profile".to_string(),
                action: SimulateAction::Loading,
            })
        );
    }

    #[test]
    fn panel_records_simulate_failure_and_clears_on_success() {
        let mut panel = panel_with_entries(vec![resource_entry("auth.profile")]);
        let path = "auth.profile";
        let row_id = DevtoolsRowId::Resource(path.to_string());

        panel.record_simulate_result(path, true, SimulateResult::not_found());
        assert!(panel.row_error(&row_id).is_some());
        assert!(panel.last_error().is_some());

        panel.record_simulate_result(path, true, SimulateResult::applied());
        assert!(panel.row_error(&row_id).is_none());
        assert!(panel.last_error().is_none());
    }

    #[test]
    fn host_disabled_returns_none_on_update() {
        let mut host = DevtoolsHostState::disabled();
        assert!(!host.is_enabled());
        assert!(host
            .update(DevtoolsPanelMessage::Simulate {
                path: "x".to_string(),
                action: SimulateAction::Idle,
            })
            .is_none());
    }

    #[test]
    fn host_enabled_routes_simulate_message_to_effect() {
        let mut host = DevtoolsHostState::new(Some(DevtoolsPanelState::new()));
        let effect = host.update(DevtoolsPanelMessage::Simulate {
            path: "auth.profile".to_string(),
            action: SimulateAction::Loading,
        });
        assert!(matches!(effect, Some(DevtoolsPanelEffect::Simulate { .. })));
    }

    #[test]
    fn snapshot_resource_and_operation_splitting() {
        let mut panel = panel_with_entries(vec![
            resource_entry("auth.profile"),
            operation_entry("auth.login"),
        ]);
        // manually set operation snapshot
        panel.entries[1].snapshot = SimulableSnapshot::Running;

        let snapshot = DevtoolStateSnapshot {
            entries: panel.entries.clone(),
            registry: Vec::new(),
        };

        assert_eq!(snapshot.resources().count(), 1);
        assert_eq!(snapshot.operations().count(), 1);
    }
}
