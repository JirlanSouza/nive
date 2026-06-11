use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserFacingErrorKind {
    Bootstrap,
    ProjectCatalog,
    Tag,
    Devtools,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserFacingError {
    kind: UserFacingErrorKind,
    summary: String,
    detail: String,
}

pub type UserFacingResult<T> = Result<T, UserFacingError>;

impl UserFacingError {
    pub fn bootstrap(error: impl fmt::Display) -> Self {
        Self::new(UserFacingErrorKind::Bootstrap, error)
    }

    pub fn project_catalog(error: impl fmt::Display) -> Self {
        Self::new(UserFacingErrorKind::ProjectCatalog, error)
    }

    pub fn tag(error: impl fmt::Display) -> Self {
        Self::new(UserFacingErrorKind::Tag, error)
    }

    pub fn devtools(error: impl fmt::Display) -> Self {
        Self::new(UserFacingErrorKind::Devtools, error)
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

    fn new(kind: UserFacingErrorKind, error: impl fmt::Display) -> Self {
        let detail = error.to_string();
        let summary = error_summary(detail.as_str()).to_string();

        Self {
            kind,
            summary,
            detail,
        }
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
        let error = UserFacingError::project_catalog("Project not found (project_id: p1)");

        assert_eq!(error.summary(), "Project not found");
        assert_eq!(error.detail(), "Project not found (project_id: p1)");
        assert!(error.has_diagnostic_detail());
    }

    #[test]
    fn diagnostic_detail_is_absent_when_summary_matches_detail() {
        let error = UserFacingError::project_catalog("Catalog unavailable");

        assert_eq!(error.summary(), "Catalog unavailable");
        assert_eq!(error.detail(), "Catalog unavailable");
        assert!(!error.has_diagnostic_detail());
    }
}
