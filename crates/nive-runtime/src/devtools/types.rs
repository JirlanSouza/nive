use crate::inspect::SimulableSnapshot;
use crate::state::OperationId;

pub const DEFAULT_CAPABILITY_HINT: &str =
    "Add #[inspect(default)] to enable default payload simulation.";
pub const SAMPLE_CAPABILITY_HINT: &str =
    "Add #[inspect(sample = path::to_fn)] to enable sample payload simulation.";
pub const INPUT_CAPABILITY_HINT: &str =
    "Add #[inspect(input = path::to_fn)] to enable operation input simulation.";
pub const REFRESH_CAPABILITY_HINT: &str =
    "Load or simulate a value before refreshing this resource.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulatorKind {
    Resource,
    Operation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SimulatorCapabilities {
    pub default_value: bool,
    pub sample_value: bool,
    pub input: bool,
}

impl SimulatorCapabilities {
    pub const fn resource(default_value: bool, sample_value: bool) -> Self {
        Self {
            default_value,
            sample_value,
            input: false,
        }
    }

    pub const fn operation(input: bool) -> Self {
        Self {
            default_value: false,
            sample_value: false,
            input,
        }
    }
}

/// One entry in the devtools simulator — a single `Resource` or `Operation`
/// field discovered via [`crate::inspect::Inspect`] traversal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulatorEntry {
    /// Dotted path from the root inspect state (e.g. `"auth.profile"`).
    pub path: String,
    /// Human-readable label derived from the last path segment.
    pub label: String,
    pub kind: SimulatorKind,
    pub capabilities: SimulatorCapabilities,
    /// Current state snapshot, used to render status and decide which
    /// controls to enable.
    pub snapshot: SimulableSnapshot,
}

impl SimulatorEntry {
    /// True when the entry represents a `Resource`-style state.
    pub fn is_resource(&self) -> bool {
        self.kind == SimulatorKind::Resource
    }

    /// True when the entry has a cached value that could be refreshed.
    pub fn has_value(&self) -> bool {
        match &self.snapshot {
            SimulableSnapshot::Loading { has_value }
            | SimulableSnapshot::Failed { has_value, .. } => *has_value,
            SimulableSnapshot::Loaded => true,
            _ => false,
        }
    }

    /// True when the entry is currently in a Failed/OperationFailed state.
    pub fn has_error(&self) -> bool {
        matches!(
            self.snapshot,
            SimulableSnapshot::Failed { .. } | SimulableSnapshot::OperationFailed { .. }
        )
    }
}

/// Status label for an [`OperationRegistry`] entry, pre-formatted for display.
///
/// [`OperationRegistry`]: crate::state::OperationRegistry
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryStatus {
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

/// A read-only summary of one entry from the app-owned [`OperationRegistry`].
///
/// Rendered in the Operations tab as a separate read-only section (task 9.3).
///
/// [`OperationRegistry`]: crate::state::OperationRegistry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryEntry {
    pub id: OperationId,
    pub label: String,
    pub status: RegistryStatus,
}

impl RegistryEntry {
    pub fn from_registry_entry(id: OperationId, entry: &crate::state::OperationEntry) -> Self {
        use crate::state::OperationStatus;
        let status = match &entry.status {
            OperationStatus::Running => RegistryStatus::Running,
            OperationStatus::Completed => RegistryStatus::Completed,
            OperationStatus::Failed(e) => RegistryStatus::Failed(e.summary().to_string()),
            OperationStatus::Cancelled => RegistryStatus::Cancelled,
        };
        Self {
            id,
            label: entry.descriptor.title.to_string(),
            status,
        }
    }
}

/// Snapshot passed to the devtools view function each frame.
///
/// Built during the `update` cycle (requires `&mut` for `Inspect` traversal)
/// and cached in [`DevtoolsPanelState`] so `view` can render it without
/// mutating the app.
///
/// [`DevtoolsPanelState`]: super::panel::DevtoolsPanelState
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DevtoolStateSnapshot {
    pub entries: Vec<SimulatorEntry>,
    pub registry: Vec<RegistryEntry>,
}

impl DevtoolStateSnapshot {
    pub fn resources(&self) -> impl Iterator<Item = &SimulatorEntry> {
        self.entries.iter().filter(|e| e.is_resource())
    }

    pub fn operations(&self) -> impl Iterator<Item = &SimulatorEntry> {
        self.entries.iter().filter(|e| !e.is_resource())
    }
}
