pub use nive_core::{ErrorPresentation, OperationStatusPresentation, ResourceStatusPresentation};

/// Shared by `ResourceStatusLine` and `SectionHeader` so both agree on state derivation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResourceStatusPhase {
    Idle,
    Refreshing,
    StaleError,
}

pub(crate) fn resource_status_phase(
    presentation: &impl ResourceStatusPresentation,
) -> ResourceStatusPhase {
    if presentation.is_refreshing() && presentation.has_value() {
        ResourceStatusPhase::Refreshing
    } else if presentation.error().is_some() && presentation.has_value() {
        ResourceStatusPhase::StaleError
    } else {
        ResourceStatusPhase::Idle
    }
}
