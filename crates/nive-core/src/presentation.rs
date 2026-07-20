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

impl ToastTone {
    /// The live-region announcement politeness this tone SHALL use.
    ///
    /// Preparatory only: no native AccessKit live-region emission exists in
    /// this Iced version, so nothing consumes this yet. It fixes the
    /// tone-to-politeness mapping now so `ToastHost` (or whatever wires the
    /// eventual accessibility surface) has one settled contract to implement
    /// against, rather than each caller inventing its own.
    pub fn announcement_politeness(self) -> AnnouncementPoliteness {
        match self {
            ToastTone::Info | ToastTone::Success => AnnouncementPoliteness::Polite,
            ToastTone::Warning => AnnouncementPoliteness::NonAggressive,
            ToastTone::Danger => AnnouncementPoliteness::Assertive,
        }
    }
}

/// Live-region announcement urgency, independent of any concrete
/// accessibility backend. See [`ToastTone::announcement_politeness`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnouncementPoliteness {
    /// Read at the next opportunity; never interrupts current speech.
    Polite,
    /// Noticeable without the interruption an assertive announcement makes.
    NonAggressive,
    /// Interrupts current speech immediately.
    Assertive,
}

/// A read-only view over a presentable toast.
///
/// Deliberately message-free: the runtime-owned action (an application
/// `Message` a caller can attach to a toast) lives on `nive-runtime`'s
/// concrete `Toast`/`ToastItem`, never on this neutral contract, so
/// `nive-ui` can render any `ToastPresentation` without knowing about
/// application messages at all.
///
/// ```compile_fail
/// use nive_core::ToastPresentation;
///
/// fn takes_message<T: ToastPresentation>(toast: &T) {
///     let _ = toast.action();
/// }
/// ```
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

    #[test]
    fn announcement_politeness_follows_tone() {
        assert_eq!(
            ToastTone::Info.announcement_politeness(),
            AnnouncementPoliteness::Polite
        );
        assert_eq!(
            ToastTone::Success.announcement_politeness(),
            AnnouncementPoliteness::Polite
        );
        assert_eq!(
            ToastTone::Warning.announcement_politeness(),
            AnnouncementPoliteness::NonAggressive
        );
        assert_eq!(
            ToastTone::Danger.announcement_politeness(),
            AnnouncementPoliteness::Assertive
        );
    }
}
