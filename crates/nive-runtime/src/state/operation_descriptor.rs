use std::borrow::Cow;

use crate::UserFacingError;

/// Stable identifier for an operation tracked by an [`OperationRegistry`].
///
/// Backed by [`Cow<'static, str>`] so that apps can declare zero-cost
/// constant ids via [`OperationId::from_static`] (no allocation) or construct
/// runtime-derived ids via [`OperationId::from_owned`]. `BTreeMap` ordering is
/// by string content, so static and owned ids with the same string compare
/// equal.
///
/// [`OperationRegistry`]: super::OperationRegistry
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OperationId(Cow<'static, str>);

impl OperationId {
    /// Zero-cost constructor for compile-time-declared ids.
    pub const fn from_static(id: &'static str) -> Self {
        Self(Cow::Borrowed(id))
    }

    /// Allocating constructor for runtime-derived ids (per-user, per-record).
    pub fn from_owned(id: String) -> Self {
        Self(Cow::Owned(id))
    }

    /// Legacy convenience constructor for `&'static str` ids. Equivalent to
    /// [`OperationId::from_static`]. Kept for source compatibility with code
    /// written against the pre-Cow API.
    pub const fn new(id: &'static str) -> Self {
        Self::from_static(id)
    }

    /// View the id's string content.
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<&'static str> for OperationId {
    fn from(id: &'static str) -> Self {
        Self::from_static(id)
    }
}

impl From<String> for OperationId {
    fn from(id: String) -> Self {
        Self::from_owned(id)
    }
}

impl std::fmt::Display for OperationId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum OperationProgress {
    #[default]
    Indeterminate,
    Fraction {
        completed: u32,
        total: u32,
    },
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
