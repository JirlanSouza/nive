use std::fmt;

use nive_core::ErrorPresentation;

/// A stable, human-facing error code derived from an error kind.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ErrorCode(String);

/// Error returned when an error code string is not a valid code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidErrorCode;

/// The category of a [`UserFacingError`], used to derive its [`ErrorCode`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserFacingErrorKind {
    /// An error that occurred during application bootstrap.
    Bootstrap,
    /// An error injected by the devtools layer.
    Devtools,
    /// An application-defined error category.
    Custom(String),
}

/// A user-facing error carrying a stable code, kind, summary, and detail.
///
/// Implements [`ErrorPresentation`] so `nive-ui` feedback widgets can render it
/// without depending on app-domain types. The `summary` is a short headline;
/// the `detail` is the full diagnostic text (which may equal the summary when
/// no extra detail is available).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserFacingError {
    code: ErrorCode,
    kind: UserFacingErrorKind,
    summary: String,
    detail: String,
}

/// Convenience [`Result`] for operations that produce a [`UserFacingError`].
pub type UserFacingResult<T> = Result<T, UserFacingError>;

impl UserFacingError {
    pub fn new(kind: UserFacingErrorKind, error: impl fmt::Display) -> Self {
        let detail = error.to_string();
        let summary = error_summary(detail.as_str()).to_string();
        let code = ErrorCode::from_kind(&kind);

        Self {
            code,
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

    /// Constructs a user-facing error of [`UserFacingErrorKind::Custom`]
    /// with the provided code and message.
    ///
    /// # Message convention
    ///
    /// The `error` message is split on the first occurrence of the substring
    /// `" ("` to derive a short summary (title) and a diagnostic detail. Apps
    /// SHOULD follow the `"Short summary (context: value)"` convention.
    /// `Toast::error` uses only the summary as its title and never places the
    /// diagnostic detail in the toast body; the detail is reserved for
    /// `ErrorDetailsDialog` behind explicit access.
    ///
    /// ```
    /// use nive_runtime::{Toast, UserFacingError};
    ///
    /// let error = UserFacingError::custom(
    ///     "record_catalog",
    ///     "Record not found (record_id: r1)",
    /// );
    /// let toast: Toast = Toast::error(error);
    ///
    /// assert_eq!(toast.title(), "Record not found");
    /// assert_eq!(toast.body(), None);
    /// ```
    ///
    /// When the message contains no parenthesised context, the summary
    /// equals the detail and there is no diagnostic detail to withhold.
    pub fn custom(kind: impl Into<String>, error: impl fmt::Display) -> Self {
        Self::new(UserFacingErrorKind::Custom(kind.into()), error)
    }

    pub fn kind(&self) -> &UserFacingErrorKind {
        &self.kind
    }

    pub fn code(&self) -> &ErrorCode {
        &self.code
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

    /// Extracts the parenthesised context — the part after `" ("` and
    /// before a trailing `")"` — when present. Returns `None` when the
    /// message carries no context suffix.
    ///
    /// `Toast::error` never surfaces this; it is reserved for
    /// `ErrorDetailsDialog` behind explicit access.
    pub fn diagnostic_detail(&self) -> Option<&str> {
        if !self.has_diagnostic_detail() {
            return None;
        }
        let (_, body) = self.detail.split_once(" (")?;
        Some(body.strip_suffix(')').unwrap_or(body))
    }
}

impl ErrorCode {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidErrorCode> {
        let value = value.into();
        if is_valid_error_code(value.as_str()) {
            Ok(Self(value))
        } else {
            Err(InvalidErrorCode)
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    fn from_kind(kind: &UserFacingErrorKind) -> Self {
        match kind {
            UserFacingErrorKind::Bootstrap => Self(String::from("bootstrap")),
            UserFacingErrorKind::Devtools => Self(String::from("devtools")),
            UserFacingErrorKind::Custom(code) => {
                Self::new(code.clone()).unwrap_or_else(|_| Self(String::from("application")))
            }
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl fmt::Display for InvalidErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("error code must use lowercase ASCII letters, digits, '-' or '_'")
    }
}

impl std::error::Error for InvalidErrorCode {}

impl fmt::Display for UserFacingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.detail.as_str())
    }
}

impl std::error::Error for UserFacingError {}

impl ErrorPresentation for UserFacingError {
    fn summary(&self) -> &str {
        self.summary()
    }

    fn detail(&self) -> &str {
        self.detail()
    }

    fn has_diagnostic_detail(&self) -> bool {
        self.has_diagnostic_detail()
    }
}

fn error_summary(detail: &str) -> &str {
    detail
        .split_once(" (")
        .map(|(summary, _)| summary)
        .unwrap_or(detail)
}

fn is_valid_error_code(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

#[cfg(test)]
mod user_facing_error_tests {
    use super::*;

    #[test]
    fn summary_strips_context_suffix() {
        let error = UserFacingError::custom("record_catalog", "Record not found (record_id: r1)");

        assert_eq!(error.summary(), "Record not found");
        assert_eq!(error.detail(), "Record not found (record_id: r1)");
        assert!(error.has_diagnostic_detail());
    }

    #[test]
    fn diagnostic_detail_is_absent_when_summary_matches_detail() {
        let error = UserFacingError::custom("record_catalog", "Catalog unavailable");

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
        let error = UserFacingError::custom("record_catalog", "Load failed");

        assert!(matches!(error.kind(), UserFacingErrorKind::Custom(_)));
        assert_eq!(error.code().as_str(), "record_catalog");
        assert_eq!(error.summary(), "Load failed");
    }

    #[test]
    fn rejects_invalid_error_codes() {
        assert!(ErrorCode::new("Project Catalog").is_err());
        assert!(ErrorCode::new("").is_err());
    }

    #[test]
    fn accepts_valid_error_codes() {
        assert!(ErrorCode::new("project-catalog").is_ok());
        assert!(ErrorCode::new("record_catalog").is_ok());
    }

    #[test]
    fn question_mark_operator_propagates_invalid_code_through_result_helper() {
        fn try_build(code: &str) -> Result<ErrorCode, InvalidErrorCode> {
            let parsed = ErrorCode::new(code)?;
            Ok(parsed)
        }

        assert!(try_build("project-catalog").is_ok());
        assert!(try_build("Project Catalog").is_err());
    }
}
