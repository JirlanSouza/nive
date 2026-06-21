use std::borrow::Cow;

use crate::UserFacingError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OperationId(pub &'static str);

impl OperationId {
    pub const fn new(id: &'static str) -> Self {
        Self(id)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl From<&'static str> for OperationId {
    fn from(id: &'static str) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for OperationId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationProgress {
    Indeterminate,
    Fraction { completed: u32, total: u32 },
    Message(Cow<'static, str>),
}

impl OperationProgress {
    pub fn fraction(completed: u32, total: u32) -> Self {
        Self::Fraction { completed, total }
    }

    pub fn message(message: impl Into<Cow<'static, str>>) -> Self {
        Self::Message(message.into())
    }

    pub fn ratio(&self) -> Option<f32> {
        match self {
            Self::Fraction { completed, total } if *total > 0 => {
                Some((*completed as f32) / (*total as f32))
            }
            Self::Indeterminate | Self::Message(_) | Self::Fraction { .. } => None,
        }
    }
}

impl Default for OperationProgress {
    fn default() -> Self {
        Self::Indeterminate
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationStatus {
    Running,
    Completed,
    Failed(UserFacingError),
    Cancelled,
}

impl OperationStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed(_) | Self::Cancelled)
    }

    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationDescriptor {
    pub id: OperationId,
    pub title: Cow<'static, str>,
    pub progress: OperationProgress,
    pub cancellable: bool,
}

impl OperationDescriptor {
    pub fn new(id: impl Into<OperationId>, title: impl Into<Cow<'static, str>>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            progress: OperationProgress::default(),
            cancellable: false,
        }
    }

    pub fn progress(mut self, progress: OperationProgress) -> Self {
        self.progress = progress;
        self
    }

    pub fn cancellable(mut self, cancellable: bool) -> Self {
        self.cancellable = cancellable;
        self
    }
}
