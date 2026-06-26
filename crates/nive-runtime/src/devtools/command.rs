/// The forcing action the devtools panel wants to apply to a single
/// [`SimulableState`] leaf identified by its dotted path.
///
/// [`SimulableState`]: crate::inspect::SimulableState
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimulateAction {
    Idle,
    /// Force loading without preserving the current value.
    Loading,
    /// Force an operation to running with its configured sample input.
    Start,
    /// Force loading while preserving the current value (refresh).
    Refreshing,
    /// Force an error with the given message.
    Error {
        message: String,
    },
    /// Force the default value when the field declared `#[inspect(default)]`.
    Default,
    /// Force the configured sample payload.
    Sample,
    /// Dismiss the current error without changing the load state.
    DismissError,
}

/// Identifies a row in the devtools panel for per-row error display.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DevtoolsRowId {
    Resource(String),
    Operation(String),
}

impl DevtoolsRowId {
    pub fn path(&self) -> &str {
        match self {
            Self::Resource(p) | Self::Operation(p) => p,
        }
    }
}

/// Result of applying a simulate action to the app state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimulateResult {
    Applied,
    Unsupported { reason: String },
    NotFound,
}

impl SimulateResult {
    pub fn applied() -> Self {
        Self::Applied
    }

    pub fn unsupported(reason: impl Into<String>) -> Self {
        Self::Unsupported {
            reason: reason.into(),
        }
    }

    pub fn not_found() -> Self {
        Self::NotFound
    }

    pub fn applied_result(&self) -> bool {
        matches!(self, Self::Applied)
    }

    pub fn unsupported_reason(&self) -> Option<&str> {
        match self {
            Self::Unsupported { reason } => Some(reason.as_str()),
            _ => None,
        }
    }

    pub fn panel_error(&self, path: &str) -> Option<String> {
        match self {
            Self::Applied => None,
            Self::Unsupported { reason } => Some(reason.clone()),
            Self::NotFound => Some(format!("No state matched `{path}`")),
        }
    }
}
