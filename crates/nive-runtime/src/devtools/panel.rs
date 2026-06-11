use std::collections::{BTreeMap, BTreeSet};

use crate::{
    probe_drafts_from_snapshot, update_probe_drafts, ProbeCatalogEntry, ProbeDraft,
    ProbeInjectionSnapshot, ProbePanelEffect, ProbePanelMessage,
};

use super::{DevtoolCommand, DevtoolCommandResult, DevtoolsRowId};

const DEFAULT_DEVTOOLS_ERROR: &str = "Devtools injected failure";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DevtoolsConfig {
    enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevtoolsPanelMessage<P> {
    SelectTab(DevtoolsPanelTab),
    SearchChanged(String),
    ToggleActiveOnly(bool),
    ToggleRowExpanded(DevtoolsRowId),
    ClearRowCommandError(DevtoolsRowId),
    Probe(ProbePanelMessage<P>),
    CommandInputChanged {
        path: String,
        field_name: String,
        value: String,
    },
    CommandErrorChanged {
        path: String,
        value: String,
    },
    SetResourceIdle(String),
    SetResourceLoading {
        path: String,
        preserve_value: bool,
    },
    SetResourceFailedEmpty(String),
    SetResourceFailedCached(String),
    SetResourceLoadedFixture {
        path: String,
        fixture_id: String,
    },
    DismissResourceError(String),
    SetOperationIdle(String),
    SetOperationRunning(String),
    SetOperationFailed(String),
    ClearCommandInputs(String),
    ClearCommandError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevtoolsPanelTab {
    Probes,
    Resources,
    Operations,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevtoolsPanelEffect<P> {
    Command(DevtoolCommand),
    Probe(ProbePanelEffect<P>),
}

pub fn run_devtools_panel_effect<P>(
    effect: DevtoolsPanelEffect<P>,
    apply_command: impl FnOnce(&DevtoolCommand) -> DevtoolCommandResult,
    apply_probe: impl FnOnce(ProbePanelEffect<P>),
) -> Option<(DevtoolCommand, DevtoolCommandResult)> {
    match effect {
        DevtoolsPanelEffect::Command(command) => {
            let result = apply_command(&command);
            Some((command, result))
        }
        DevtoolsPanelEffect::Probe(effect) => {
            apply_probe(effect);
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProbePanelState<P> {
    pub drafts: Vec<ProbeDraft<P>>,
}

#[derive(Debug, Clone)]
pub struct DevtoolsPanelState<P> {
    pub active_tab: DevtoolsPanelTab,
    pub probes: ProbePanelState<P>,
    query: String,
    active_only: bool,
    expanded_rows: BTreeSet<DevtoolsRowId>,
    command_inputs: BTreeMap<String, BTreeMap<String, String>>,
    command_errors: BTreeMap<String, String>,
    command_failures: BTreeMap<DevtoolsRowId, String>,
    last_command_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DevtoolsHostState<P> {
    panel: Option<DevtoolsPanelState<P>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DevtoolsWindowSpec {
    pub size: iced::Size,
    pub min_size: iced::Size,
}

impl DevtoolsConfig {
    pub fn from_env_var(env_var: &str) -> Self {
        Self::from_env_value(std::env::var(env_var).ok().as_deref())
    }

    pub fn enabled(self) -> bool {
        self.enabled
    }

    pub fn from_env_value(value: Option<&str>) -> Self {
        Self {
            enabled: value.is_some_and(env_flag_enabled),
        }
    }
}

impl Default for DevtoolsConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

impl<P> DevtoolsHostState<P> {
    pub fn new(panel: Option<DevtoolsPanelState<P>>) -> Self {
        Self { panel }
    }

    pub fn disabled() -> Self {
        Self { panel: None }
    }

    pub fn is_enabled(&self) -> bool {
        self.panel.is_some()
    }

    pub fn panel(&self) -> Option<&DevtoolsPanelState<P>> {
        self.panel.as_ref()
    }
}

impl<P: Copy + Eq> DevtoolsHostState<P> {
    pub fn update(&mut self, message: DevtoolsPanelMessage<P>) -> Option<DevtoolsPanelEffect<P>> {
        self.panel.as_mut()?.update(message)
    }

    pub fn record_command_result(
        &mut self,
        command: &DevtoolCommand,
        result: DevtoolCommandResult,
    ) {
        if let Some(panel) = &mut self.panel {
            panel.record_command_result(command, result);
        }
    }
}

impl Default for DevtoolsWindowSpec {
    fn default() -> Self {
        Self {
            size: iced::Size::new(940.0, 640.0),
            min_size: iced::Size::new(820.0, 520.0),
        }
    }
}

impl<P: ProbeCatalogEntry> ProbePanelState<P> {
    pub fn from_snapshot(snapshot: &ProbeInjectionSnapshot<P>) -> Self {
        Self {
            drafts: probe_drafts_from_snapshot(snapshot),
        }
    }
}

impl<P: Copy + Eq> ProbePanelState<P> {
    pub fn update(&mut self, message: ProbePanelMessage<P>) -> Option<ProbePanelEffect<P>> {
        update_probe_drafts(&mut self.drafts, message)
    }

    pub fn active_count(&self) -> usize {
        self.drafts.iter().filter(|draft| draft.enabled).count()
    }
}

impl<P: ProbeCatalogEntry> DevtoolsPanelState<P> {
    pub fn from_probe_snapshot(snapshot: &ProbeInjectionSnapshot<P>) -> Self {
        Self::new(ProbePanelState::from_snapshot(snapshot))
    }
}

impl<P: Copy + Eq> DevtoolsPanelState<P> {
    pub fn new(probes: ProbePanelState<P>) -> Self {
        Self {
            active_tab: DevtoolsPanelTab::Resources,
            probes,
            query: String::new(),
            active_only: false,
            expanded_rows: BTreeSet::new(),
            command_inputs: BTreeMap::new(),
            command_errors: BTreeMap::new(),
            command_failures: BTreeMap::new(),
            last_command_error: None,
        }
    }

    pub fn update(&mut self, message: DevtoolsPanelMessage<P>) -> Option<DevtoolsPanelEffect<P>> {
        match message {
            DevtoolsPanelMessage::SelectTab(tab) => {
                self.active_tab = tab;
                None
            }
            DevtoolsPanelMessage::SearchChanged(query) => {
                self.query = query;
                None
            }
            DevtoolsPanelMessage::ToggleActiveOnly(active_only) => {
                self.active_only = active_only;
                None
            }
            DevtoolsPanelMessage::ToggleRowExpanded(row_id) => {
                if !self.expanded_rows.remove(&row_id) {
                    self.expanded_rows.insert(row_id);
                }
                None
            }
            DevtoolsPanelMessage::ClearRowCommandError(row_id) => {
                self.command_failures.remove(&row_id);
                None
            }
            DevtoolsPanelMessage::Probe(message) => {
                self.probes.update(message).map(DevtoolsPanelEffect::Probe)
            }
            DevtoolsPanelMessage::CommandInputChanged {
                path,
                field_name,
                value,
            } => {
                self.command_inputs
                    .entry(path)
                    .or_default()
                    .insert(field_name, value);
                self.last_command_error = None;
                None
            }
            DevtoolsPanelMessage::CommandErrorChanged { path, value } => {
                self.command_errors.insert(path, value);
                self.last_command_error = None;
                None
            }
            DevtoolsPanelMessage::SetResourceIdle(path) => Some(DevtoolsPanelEffect::Command(
                DevtoolCommand::SetResourceIdle { path },
            )),
            DevtoolsPanelMessage::SetResourceLoading {
                path,
                preserve_value,
            } => Some(DevtoolsPanelEffect::Command(
                DevtoolCommand::SetResourceLoading {
                    path,
                    preserve_value,
                },
            )),
            DevtoolsPanelMessage::SetResourceFailedEmpty(path) => {
                let message = self.command_error(&path);
                Some(DevtoolsPanelEffect::Command(
                    DevtoolCommand::SetResourceFailed {
                        path,
                        message,
                        preserve_value: false,
                    },
                ))
            }
            DevtoolsPanelMessage::SetResourceFailedCached(path) => {
                let message = self.command_error(&path);
                Some(DevtoolsPanelEffect::Command(
                    DevtoolCommand::SetResourceFailed {
                        path,
                        message,
                        preserve_value: true,
                    },
                ))
            }
            DevtoolsPanelMessage::SetResourceLoadedFixture { path, fixture_id } => {
                Some(DevtoolsPanelEffect::Command(
                    DevtoolCommand::SetResourceLoadedFixture { path, fixture_id },
                ))
            }
            DevtoolsPanelMessage::DismissResourceError(path) => Some(DevtoolsPanelEffect::Command(
                DevtoolCommand::DismissResourceError { path },
            )),
            DevtoolsPanelMessage::SetOperationIdle(path) => Some(DevtoolsPanelEffect::Command(
                DevtoolCommand::SetOperationIdle { path },
            )),
            DevtoolsPanelMessage::SetOperationRunning(path) => Some(DevtoolsPanelEffect::Command(
                DevtoolCommand::SetOperationRunning {
                    input: self.command_input(&path),
                    path,
                },
            )),
            DevtoolsPanelMessage::SetOperationFailed(path) => {
                let message = self.command_error(&path);
                Some(DevtoolsPanelEffect::Command(
                    DevtoolCommand::SetOperationFailed {
                        input: self.command_input(&path),
                        path,
                        message,
                    },
                ))
            }
            DevtoolsPanelMessage::ClearCommandInputs(path) => {
                self.command_inputs.remove(&path);
                self.command_errors.remove(&path);
                self.command_failures
                    .retain(|row_id, _| row_id.path() != path);
                self.last_command_error = None;
                None
            }
            DevtoolsPanelMessage::ClearCommandError => {
                self.last_command_error = None;
                self.command_failures.clear();
                None
            }
        }
    }

    pub fn record_command_result(
        &mut self,
        command: &DevtoolCommand,
        result: DevtoolCommandResult,
    ) {
        let row_id = command.row_id();
        match result.into_parts() {
            (_, Some(error)) => {
                self.command_failures.insert(row_id, error.clone());
                self.last_command_error = Some(error);
            }
            (false, None) => {
                let error = format!("No devtools state matched `{}`", command.path());
                self.command_failures.insert(row_id, error.clone());
                self.last_command_error = Some(error);
            }
            (true, None) => {
                self.command_failures.remove(&row_id);
                self.last_command_error = None;
            }
        };
    }

    pub fn active_probe_count(&self) -> usize {
        self.probes.active_count()
    }

    pub fn command_value(&self, path: &str, field_name: &str) -> String {
        self.command_inputs
            .get(path)
            .and_then(|values| values.get(field_name))
            .cloned()
            .unwrap_or_default()
    }

    pub fn command_error_value(&self, path: &str) -> String {
        self.command_errors
            .get(path)
            .cloned()
            .unwrap_or_else(|| DEFAULT_DEVTOOLS_ERROR.to_string())
    }

    pub fn last_command_error(&self) -> Option<&str> {
        self.last_command_error.as_deref()
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn active_only(&self) -> bool {
        self.active_only
    }

    pub fn is_row_expanded(&self, row_id: &DevtoolsRowId) -> bool {
        self.expanded_rows.contains(row_id)
    }

    pub fn row_command_error(&self, row_id: &DevtoolsRowId) -> Option<&str> {
        self.command_failures.get(row_id).map(String::as_str)
    }

    fn command_input(&self, path: &str) -> BTreeMap<String, String> {
        self.command_inputs.get(path).cloned().unwrap_or_default()
    }

    fn command_error(&self, path: &str) -> String {
        self.command_error_value(path)
    }
}

fn env_flag_enabled(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "y" | "on"
    )
}

#[cfg(test)]
mod tests {
    use crate::{ProbeEffect, ProbeErrorScope, ProbeInjectionConfig, ProbeMeta};

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestProbe {
        One,
    }

    impl ProbeCatalogEntry for TestProbe {
        const ALL: &'static [Self] = &[Self::One];

        fn meta(self) -> ProbeMeta {
            ProbeMeta::new("test.one", "one", "Couldn't run test", ProbeErrorScope::Tag)
        }
    }

    fn panel() -> DevtoolsPanelState<TestProbe> {
        let snapshot = crate::ProbeInjectionStore::new(ProbeInjectionConfig::<TestProbe> {
            scenarios: Vec::new(),
            unknown: Vec::new(),
        })
        .snapshot();

        DevtoolsPanelState::from_probe_snapshot(&snapshot)
    }

    #[test]
    fn devtools_config_defaults_to_disabled() {
        assert!(!DevtoolsConfig::default().enabled());
        assert!(!DevtoolsConfig::from_env_value(None).enabled());
    }

    #[test]
    fn devtools_config_accepts_enabled_env_values() {
        for value in ["1", "true", "yes", "y", "on", " TRUE "] {
            assert!(DevtoolsConfig::from_env_value(Some(value)).enabled());
        }

        for value in ["", "0", "false", "off", "no"] {
            assert!(!DevtoolsConfig::from_env_value(Some(value)).enabled());
        }
    }

    #[test]
    fn devtools_panel_tracks_search_active_filter_and_row_expansion() {
        let mut panel = panel();
        let row_id = DevtoolsRowId::Probe("test.one".to_string());

        panel.update(DevtoolsPanelMessage::SearchChanged("test".to_string()));
        panel.update(DevtoolsPanelMessage::ToggleActiveOnly(true));
        panel.update(DevtoolsPanelMessage::ToggleRowExpanded(row_id.clone()));

        assert_eq!(panel.query(), "test");
        assert!(panel.active_only());
        assert!(panel.is_row_expanded(&row_id));

        panel.update(DevtoolsPanelMessage::ToggleRowExpanded(row_id.clone()));

        assert!(!panel.is_row_expanded(&row_id));
    }

    #[test]
    fn devtools_panel_records_command_failure_on_matching_row() {
        let mut panel = panel();
        let command = DevtoolCommand::SetResourceIdle {
            path: "welcome.tags".to_string(),
        };
        let row_id = DevtoolsRowId::Resource("welcome.tags".to_string());

        panel.record_command_result(&command, DevtoolCommandResult::failed("No fixture"));

        assert_eq!(panel.row_command_error(&row_id), Some("No fixture"));

        panel.record_command_result(&command, DevtoolCommandResult::changed());

        assert_eq!(panel.row_command_error(&row_id), None);
    }

    #[test]
    fn devtools_host_state_is_disabled_without_panel() {
        let mut host = DevtoolsHostState::<TestProbe>::disabled();

        assert!(!host.is_enabled());
        assert!(host.panel().is_none());
        assert_eq!(
            host.update(DevtoolsPanelMessage::SetResourceIdle(
                "welcome.tags".to_string()
            )),
            None
        );
    }

    #[test]
    fn devtools_host_state_routes_updates_and_records_results() {
        let mut host = DevtoolsHostState::new(Some(panel()));
        let command = DevtoolCommand::SetResourceIdle {
            path: "welcome.tags".to_string(),
        };
        let row_id = DevtoolsRowId::Resource("welcome.tags".to_string());

        assert!(host.is_enabled());
        assert!(matches!(
            host.update(DevtoolsPanelMessage::SetResourceIdle(
                "welcome.tags".to_string()
            )),
            Some(DevtoolsPanelEffect::Command(
                DevtoolCommand::SetResourceIdle { .. }
            ))
        ));

        host.record_command_result(&command, DevtoolCommandResult::failed("No match"));
        assert_eq!(
            host.panel()
                .and_then(|panel| panel.row_command_error(&row_id)),
            Some("No match")
        );

        host.record_command_result(&command, DevtoolCommandResult::changed());
        assert_eq!(
            host.panel()
                .and_then(|panel| panel.row_command_error(&row_id)),
            None
        );
    }

    #[test]
    fn devtools_window_spec_uses_runtime_defaults() {
        let spec = DevtoolsWindowSpec::default();

        assert_eq!(spec.size, iced::Size::new(940.0, 640.0));
        assert_eq!(spec.min_size, iced::Size::new(820.0, 520.0));
    }

    #[test]
    fn run_devtools_panel_effect_returns_command_result_for_recording() {
        let outcome = run_devtools_panel_effect::<TestProbe>(
            DevtoolsPanelEffect::Command(DevtoolCommand::SetResourceIdle {
                path: "welcome.tags".to_string(),
            }),
            |command| {
                assert_eq!(command.path(), "welcome.tags");
                DevtoolCommandResult::changed()
            },
            |_| panic!("probe effect should not run for commands"),
        );

        assert_eq!(
            outcome,
            Some((
                DevtoolCommand::SetResourceIdle {
                    path: "welcome.tags".to_string()
                },
                DevtoolCommandResult::changed()
            ))
        );
    }

    #[test]
    fn run_devtools_panel_effect_applies_probe_effects() {
        let mut applied = None;

        let outcome = run_devtools_panel_effect(
            DevtoolsPanelEffect::Probe(ProbePanelEffect::ClearAll),
            |_| panic!("command effect should not run for probes"),
            |effect| applied = Some(effect),
        );

        assert_eq!(outcome, None);
        assert_eq!(applied, Some(ProbePanelEffect::<TestProbe>::ClearAll));
    }

    #[test]
    fn devtools_panel_builds_explicit_resource_failure_modes() {
        let mut panel = panel();

        let empty = panel
            .update(DevtoolsPanelMessage::SetResourceFailedEmpty(
                "welcome.tags".to_string(),
            ))
            .unwrap();
        let cached = panel
            .update(DevtoolsPanelMessage::SetResourceFailedCached(
                "welcome.tags".to_string(),
            ))
            .unwrap();

        assert!(matches!(
            empty,
            DevtoolsPanelEffect::Command(DevtoolCommand::SetResourceFailed {
                preserve_value: false,
                ..
            })
        ));
        assert!(matches!(
            cached,
            DevtoolsPanelEffect::Command(DevtoolCommand::SetResourceFailed {
                preserve_value: true,
                ..
            })
        ));
    }

    #[test]
    fn devtools_panel_routes_probe_updates_as_effects() {
        let mut panel = panel();

        let effect = panel
            .update(DevtoolsPanelMessage::Probe(
                ProbePanelMessage::EffectChanged(TestProbe::One, ProbeEffect::DelayOnly),
            ))
            .unwrap();

        assert!(matches!(
            effect,
            DevtoolsPanelEffect::Probe(ProbePanelEffect::SetProbeConfig(TestProbe::One, _))
        ));
    }
}
