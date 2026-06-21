use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::unix_now;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuntimeEventKind {
    Info,
    Warning,
    Error,
    Panic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeEvent {
    pub timestamp: i64,
    pub kind: RuntimeEventKind,
    pub category: Cow<'static, str>,
    pub message: Cow<'static, str>,
}

impl RuntimeEvent {
    pub fn new(
        kind: RuntimeEventKind,
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
        Self::new(RuntimeEventKind::Info, category, message)
    }

    pub fn warning(
        category: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::new(RuntimeEventKind::Warning, category, message)
    }

    pub fn error(
        category: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::new(RuntimeEventKind::Error, category, message)
    }

    pub fn panic(
        category: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::new(RuntimeEventKind::Panic, category, message)
    }
}
