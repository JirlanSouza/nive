use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::unix_now;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagnosticEventKind {
    Info,
    Warning,
    Error,
    Panic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticEvent {
    pub timestamp: i64,
    pub kind: DiagnosticEventKind,
    pub category: Cow<'static, str>,
    pub message: Cow<'static, str>,
}

impl DiagnosticEvent {
    pub fn new(
        kind: DiagnosticEventKind,
        category: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            timestamp: unix_now(),
            kind,
            category: category.into(),
            message: message.into(),
        }
    }

    pub fn info(
        category: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::new(DiagnosticEventKind::Info, category, message)
    }

    pub fn warning(
        category: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::new(DiagnosticEventKind::Warning, category, message)
    }

    pub fn error(
        category: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::new(DiagnosticEventKind::Error, category, message)
    }

    pub fn panic(
        category: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::new(DiagnosticEventKind::Panic, category, message)
    }
}
