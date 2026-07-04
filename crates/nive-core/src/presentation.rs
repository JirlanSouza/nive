//! Read-only presentation contracts for error, toast, and async-state status.
//!
//! Every trait here must stay dyn-compatible (`ErrorPresentation` is used in
//! `&dyn ErrorPresentation` position by `ResourceStatusPresentation` and
//! `OperationStatusPresentation`). Do not add generic methods or associated
//! types to `ErrorPresentation`.

/// A read-only view over a presentable error: a short summary and full detail.
pub trait ErrorPresentation {
    fn summary(&self) -> &str;

    fn detail(&self) -> &str;

    fn has_diagnostic_detail(&self) -> bool {
        self.detail() != self.summary()
    }
}

/// A read-only view over the status of an async resource (`Resource<T>`).
pub trait ResourceStatusPresentation {
    fn is_refreshing(&self) -> bool;

    fn has_value(&self) -> bool;

    fn error(&self) -> Option<&dyn ErrorPresentation>;
}

/// A read-only view over the status of an async operation (`Operation<C>`).
pub trait OperationStatusPresentation {
    fn is_running(&self) -> bool;

    fn error(&self) -> Option<&dyn ErrorPresentation>;
}

/// The semantic tone of a toast, shared by runtime toast state and UI
/// rendering so both sides speak the same type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastTone {
    Info,
    Success,
    Warning,
    Danger,
}

/// A read-only view over a presentable toast.
pub trait ToastPresentation {
    type Id: Copy;

    fn id(&self) -> Self::Id;
    fn title(&self) -> &str;
    fn body(&self) -> Option<&str>;
    fn tone(&self) -> ToastTone;
}

#[cfg(test)]
mod presentation_tests {
    use super::*;

    struct TestError {
        summary: &'static str,
        detail: &'static str,
    }

    impl ErrorPresentation for TestError {
        fn summary(&self) -> &str {
            self.summary
        }

        fn detail(&self) -> &str {
            self.detail
        }
    }

    #[test]
    fn default_has_diagnostic_detail_compares_detail_to_summary() {
        let matching = TestError {
            summary: "Failed",
            detail: "Failed",
        };
        let differing = TestError {
            summary: "Failed",
            detail: "Failed (context: value)",
        };

        assert!(!matching.has_diagnostic_detail());
        assert!(differing.has_diagnostic_detail());
    }

    #[test]
    fn error_presentation_is_dyn_compatible() {
        let error = TestError {
            summary: "Failed",
            detail: "Failed",
        };
        let _: &dyn ErrorPresentation = &error;
    }
}
