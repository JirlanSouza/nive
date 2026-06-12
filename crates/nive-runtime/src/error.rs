use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserFacingErrorKind {
    Bootstrap,
    Devtools,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserFacingError {
    kind: UserFacingErrorKind,
    summary: String,
    detail: String,
}

pub type UserFacingResult<T> = Result<T, UserFacingError>;

impl UserFacingError {
    pub fn new(kind: UserFacingErrorKind, error: impl fmt::Display) -> Self {
        let detail = error.to_string();
        let summary = error_summary(detail.as_str()).to_string();

        Self {
            kind,
            summary,
            detail,
        }
    }

    pub fn bootstrap(error: impl fmt::Display) -> Self {
        Self::new(UserFacingErrorKind::Bootstrap, error)
    }

    pub fn devtools(error: impl fmt::Display) -> Self {
        Self::new(UserFacingErrorKind::Devtools, error)
    }

    pub fn custom(kind: impl Into<String>, error: impl fmt::Display) -> Self {
        Self::new(UserFacingErrorKind::Custom(kind.into()), error)
    }

    pub fn kind(&self) -> &UserFacingErrorKind {
        &self.kind
    }

    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }

    pub fn detail(&self) -> &str {
        self.detail.as_str()
    }

    pub fn has_diagnostic_detail(&self) -> bool {
        self.detail != self.summary
    }
}

impl fmt::Display for UserFacingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.detail.as_str())
    }
}

impl std::error::Error for UserFacingError {}

fn error_summary(detail: &str) -> &str {
    detail
        .split_once(" (")
        .map(|(summary, _)| summary)
        .unwrap_or(detail)
}

#[cfg(test)]
mod user_facing_error_tests {
    use super::*;

    #[test]
    fn summary_strips_context_suffix() {
        let error =
            UserFacingError::custom("project_catalog", "Project not found (project_id: p1)");

        assert_eq!(error.summary(), "Project not found");
        assert_eq!(error.detail(), "Project not found (project_id: p1)");
        assert!(error.has_diagnostic_detail());
    }

    #[test]
    fn diagnostic_detail_is_absent_when_summary_matches_detail() {
        let error = UserFacingError::custom("project_catalog", "Catalog unavailable");

        assert_eq!(error.summary(), "Catalog unavailable");
        assert_eq!(error.detail(), "Catalog unavailable");
        assert!(!error.has_diagnostic_detail());
    }

    #[test]
    fn bootstrap_convenience_constructor() {
        let error = UserFacingError::bootstrap("DB connection failed");

        assert_eq!(error.kind(), &UserFacingErrorKind::Bootstrap);
        assert_eq!(error.summary(), "DB connection failed");
    }

    #[test]
    fn devtools_convenience_constructor() {
        let error = UserFacingError::devtools("Refresh failed");

        assert_eq!(error.kind(), &UserFacingErrorKind::Devtools);
        assert_eq!(error.summary(), "Refresh failed");
    }

    #[test]
    fn custom_kind_error() {
        let error = UserFacingError::custom("project_catalog", "Load failed");

        assert!(matches!(error.kind(), UserFacingErrorKind::Custom(_)));
        assert_eq!(error.summary(), "Load failed");
    }
}
