use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DevtoolsRowId {
    Resource(String),
    Operation(String),
    Probe(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevtoolCommand {
    SetResourceIdle {
        path: String,
    },
    SetResourceLoading {
        path: String,
        preserve_value: bool,
    },
    SetResourceFailed {
        path: String,
        message: String,
        preserve_value: bool,
    },
    SetResourceLoadedFixture {
        path: String,
        fixture_id: String,
    },
    DismissResourceError {
        path: String,
    },
    SetOperationIdle {
        path: String,
    },
    SetOperationRunning {
        path: String,
        input: BTreeMap<String, String>,
    },
    SetOperationFailed {
        path: String,
        input: BTreeMap<String, String>,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevtoolCommandResult {
    handled: bool,
    state_changed: bool,
    error: Option<String>,
}

impl DevtoolCommand {
    pub fn path(&self) -> &str {
        match self {
            Self::SetResourceIdle { path }
            | Self::SetResourceLoading { path, .. }
            | Self::SetResourceFailed { path, .. }
            | Self::SetResourceLoadedFixture { path, .. }
            | Self::DismissResourceError { path }
            | Self::SetOperationIdle { path }
            | Self::SetOperationRunning { path, .. }
            | Self::SetOperationFailed { path, .. } => path,
        }
    }

    pub fn row_id(&self) -> DevtoolsRowId {
        match self {
            Self::SetResourceIdle { path }
            | Self::SetResourceLoading { path, .. }
            | Self::SetResourceFailed { path, .. }
            | Self::SetResourceLoadedFixture { path, .. }
            | Self::DismissResourceError { path } => DevtoolsRowId::Resource(path.clone()),
            Self::SetOperationIdle { path }
            | Self::SetOperationRunning { path, .. }
            | Self::SetOperationFailed { path, .. } => DevtoolsRowId::Operation(path.clone()),
        }
    }
}

impl DevtoolsRowId {
    pub fn path(&self) -> &str {
        match self {
            Self::Resource(path) | Self::Operation(path) | Self::Probe(path) => path,
        }
    }
}

impl DevtoolCommandResult {
    pub fn changed() -> Self {
        Self {
            handled: true,
            state_changed: true,
            error: None,
        }
    }

    pub fn failed(error: impl Into<String>) -> Self {
        Self {
            handled: true,
            state_changed: false,
            error: Some(error.into()),
        }
    }

    pub fn not_handled() -> Self {
        Self {
            handled: false,
            state_changed: false,
            error: None,
        }
    }

    pub fn handled(&self) -> bool {
        self.handled
    }

    pub fn state_changed(&self) -> bool {
        self.state_changed
    }

    pub fn into_parts(self) -> (bool, Option<String>) {
        (self.handled, self.error)
    }
}
