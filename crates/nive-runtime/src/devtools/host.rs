use crate::inspect::{Inspect, InspectPath, InspectSink, SimulableSnapshot, SimulableState};
use crate::Application;

use super::command::SimulateAction;
use super::types::{DevtoolStateSnapshot, RegistryEntry, SimulatorEntry};

// ---------------------------------------------------------------------------
// DevtoolsApp
// ---------------------------------------------------------------------------

/// Application contract for devtools support.
///
/// Implement this (or derive it via framework scaffolding) to enable the
/// devtools window. The `State` associated type must implement [`Inspect`] so
/// the devtools panel can enumerate and force-simulate every `Resource` and
/// `Operation` field without any extra boilerplate.
pub trait DevtoolsApp: Application {
    /// The part of the application state that devtools can inspect. Typically
    /// a struct that `#[derive(Inspect)]` is applied to.
    type State: Inspect;

    /// Mutable access to the inspectable state.
    fn devtool_state_mut(&mut self) -> &mut Self::State;
}

// ---------------------------------------------------------------------------
// Collection and simulation helpers (called from program.rs)
// ---------------------------------------------------------------------------

/// Build a fresh [`DevtoolStateSnapshot`] by traversing the inspectable state
/// and reading each leaf's [`SimulableSnapshot`].
///
/// Called during the `update` cycle (requires `&mut A`) so that the panel's
/// cached `entries` stay up to date without needing a mutable borrow in `view`.
pub fn collect_snapshot<A: DevtoolsApp>(app: &mut A) -> DevtoolStateSnapshot {
    collect_state_snapshot(app.devtool_state_mut())
}

fn collect_state_snapshot(state: &mut impl Inspect) -> DevtoolStateSnapshot {
    #[derive(Default)]
    struct CollectSink {
        entries: Vec<SimulatorEntry>,
        registry: Vec<RegistryEntry>,
    }

    impl InspectSink for CollectSink {
        fn register(&mut self, path: &str, state: &mut dyn SimulableState) {
            let label = label_from_path(path);
            self.entries.push(SimulatorEntry {
                path: path.to_string(),
                label,
                kind: state.kind(),
                capabilities: state.capabilities(),
                snapshot: state.snapshot(),
            });
        }

        fn register_registry(&mut self, _path: &str, registry: &crate::state::OperationRegistry) {
            self.registry.extend(
                registry
                    .iter()
                    .map(|(id, entry)| RegistryEntry::from_registry_entry(id.clone(), entry)),
            );
        }
    }

    let mut sink = CollectSink::default();
    let mut path = InspectPath::new();
    state.inspect(&mut path, &mut sink);
    DevtoolStateSnapshot {
        entries: sink.entries,
        registry: sink.registry,
    }
}

/// Apply a [`SimulateAction`] to the leaf at `target_path` in the inspectable
/// state, returning whether it applied, was unsupported, or was not found.
pub fn apply_simulate<A: DevtoolsApp>(
    app: &mut A,
    target_path: &str,
    action: &SimulateAction,
) -> crate::devtools::SimulateResult {
    struct SimulateSink<'a> {
        path: &'a str,
        action: &'a SimulateAction,
        result: crate::devtools::SimulateResult,
    }

    impl InspectSink for SimulateSink<'_> {
        fn register(&mut self, path: &str, state: &mut dyn SimulableState) {
            if path != self.path
                || !matches!(&self.result, crate::devtools::SimulateResult::NotFound)
            {
                return;
            }
            self.result = state.apply(self.action);
        }
    }

    let mut sink = SimulateSink {
        path: target_path,
        action,
        result: crate::devtools::SimulateResult::not_found(),
    };
    let mut path = InspectPath::new();
    app.devtool_state_mut().inspect(&mut path, &mut sink);
    sink.result
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn label_from_path(path: &str) -> String {
    let segment = path.split('.').next_back().unwrap_or(path);
    super::helpers::label_from_field_name(segment)
}

// ---------------------------------------------------------------------------
// Retain public symbols needed by existing test code in program.rs
// ---------------------------------------------------------------------------

/// Snapshot for the panel produced by collecting from an `A: DevtoolsApp`.
pub fn devtool_snapshot<A: DevtoolsApp>(app: &mut A) -> DevtoolStateSnapshot {
    collect_snapshot(app)
}

/// Apply a simulate action from the panel effect.
pub fn devtool_apply<A: DevtoolsApp>(
    app: &mut A,
    path: &str,
    action: &SimulateAction,
) -> crate::devtools::SimulateResult {
    apply_simulate(app, path, action)
}

// ---------------------------------------------------------------------------
// SimulableSnapshot helpers re-exported for view use
// ---------------------------------------------------------------------------

pub fn snapshot_has_value(s: &SimulableSnapshot) -> bool {
    match s {
        SimulableSnapshot::Loading { has_value } | SimulableSnapshot::Failed { has_value, .. } => {
            *has_value
        }
        SimulableSnapshot::Loaded => true,
        _ => false,
    }
}

pub fn snapshot_has_error(s: &SimulableSnapshot) -> bool {
    matches!(
        s,
        SimulableSnapshot::Failed { .. } | SimulableSnapshot::OperationFailed { .. }
    )
}
