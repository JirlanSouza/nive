use std::future::Future;

use iced::Task;
use nive_core::{ErrorPresentation, OperationStatusPresentation};

use crate::UserFacingError;

use super::request::{RequestCounter, RequestId, Settled};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum OperationInner<C> {
    #[default]
    Idle,
    Running {
        token: RequestId,
        input: C,
    },
    Failed {
        error: UserFacingError,
        input: C,
    },
}

/// Opinionated state machine for an asynchronous mutation carrying an input.
///
/// Tracks `Idle`, `Running { input }`, and `Failed { input, error }`.
/// Implements [`OperationStatusPresentation`] so `nive-ui` feedback widgets
/// render it without depending on `C`.
///
/// ## Lifecycle
///
/// ```text
/// begin(input) → Running { input }
/// settle(Settled::new(token, Ok(()))) → Idle  (returns Some(input))
/// settle(Settled::new(token, Err(e))) → Failed { input, error }  (returns None)
/// reset()         → Idle
/// dismiss_error() → Idle
/// ```
///
/// Stale responses (token mismatch) are silently ignored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation<C> {
    inner: OperationInner<C>,
    counter: RequestCounter,
}

impl<C> Default for Operation<C> {
    fn default() -> Self {
        Self {
            inner: OperationInner::Idle,
            counter: RequestCounter::default(),
        }
    }
}

impl<C> Operation<C> {
    /// Constructs an `Idle` operation. Equivalent to `Operation::default()`.
    pub fn idle() -> Self {
        Self::default()
    }

    /// Transitions to `Running { input }`, recording the input and minting a
    /// fresh in-flight [`RequestId`] internally.
    ///
    /// On the blessed path, call [`run`](Self::run) instead — it fuses
    /// `begin` with task spawning so the token is invisible plumbing.
    pub fn begin(&mut self, input: C) -> RequestId {
        let token = self.counter.next();
        self.inner = OperationInner::Running { token, input };
        token
    }

    /// Applies the settled result, guarded by the carried token.
    ///
    /// - `Ok(())` → transitions to `Idle` and returns `Some(input)`.
    /// - `Err(error)` → transitions to `Failed { input, error }` and returns `None`.
    /// - Stale token → no change, returns `None`.
    pub fn settle(&mut self, settled: Settled<()>) -> Option<C> {
        let current = std::mem::take(&mut self.inner);
        match current {
            OperationInner::Running { token, input } if token == settled.token => {
                match settled.result {
                    Ok(()) => {
                        self.inner = OperationInner::Idle;
                        Some(input)
                    }
                    Err(error) => {
                        self.inner = OperationInner::Failed { error, input };
                        None
                    }
                }
            }
            other => {
                self.inner = other;
                None
            }
        }
    }

    /// Returns to `Idle`, clearing any in-progress or failed state.
    pub fn reset(&mut self) {
        self.inner = OperationInner::Idle;
    }

    /// Clears a `Failed` error, returning to `Idle`.
    pub fn dismiss_error(&mut self) {
        if matches!(self.inner, OperationInner::Failed { .. }) {
            self.inner = OperationInner::Idle;
        }
    }

    /// Returns `true` while the operation is `Running`.
    pub fn is_running(&self) -> bool {
        matches!(self.inner, OperationInner::Running { .. })
    }

    /// Returns `true` when the operation is `Idle`.
    pub fn is_idle(&self) -> bool {
        matches!(self.inner, OperationInner::Idle)
    }

    /// Returns the current error, if any.
    pub fn error(&self) -> Option<&UserFacingError> {
        match &self.inner {
            OperationInner::Failed { error, .. } => Some(error),
            _ => None,
        }
    }

    /// Returns `Some(&input)` while the operation is `Running` or `Failed`;
    /// `None` when `Idle`.
    pub fn input(&self) -> Option<&C> {
        match &self.inner {
            OperationInner::Running { input, .. } | OperationInner::Failed { input, .. } => {
                Some(input)
            }
            OperationInner::Idle => None,
        }
    }

    /// Fuses [`begin`](Self::begin) with task spawning on the send side.
    ///
    /// Builds the future from a borrow of `input` (single source of input),
    /// then calls `begin(input)` and spawns the task. Returns a
    /// `Task<Message>` delivering `Settled<()>` via `settled`.
    ///
    /// The receive side stays explicit:
    /// ```ignore
    /// Msg::SaveSettled(s) => { self.save.settle(s); },
    /// ```
    pub fn run<Message, Fut>(
        &mut self,
        input: C,
        make_future: impl FnOnce(&C) -> Fut,
        settled: impl Fn(Settled<()>) -> Message + Send + 'static,
    ) -> Task<Message>
    where
        C: Send + 'static,
        Message: Send + 'static,
        Fut: Future<Output = crate::UserFacingResult<()>> + Send + 'static,
    {
        let future = make_future(&input);
        let token = self.begin(input);
        Task::perform(future, move |result| settled(Settled::new(token, result)))
    }
}

impl<C> OperationStatusPresentation for Operation<C> {
    fn is_running(&self) -> bool {
        self.is_running()
    }

    fn error(&self) -> Option<&dyn ErrorPresentation> {
        self.error().map(|e| e as &dyn ErrorPresentation)
    }
}

#[cfg(test)]
mod operation_tests {
    use super::*;
    use crate::UserFacingError;
    use nive_core::OperationStatusPresentation;

    fn error(msg: &str) -> UserFacingError {
        UserFacingError::custom("test", msg)
    }

    #[test]
    fn idle_by_default() {
        let op = Operation::<String>::idle();
        assert!(op.is_idle());
        assert!(!op.is_running());
        assert!(op.input().is_none());
        assert!(op.error().is_none());
    }

    #[test]
    fn begin_records_input_and_runs() {
        let mut op = Operation::idle();
        op.begin("save_ctx".to_string());
        assert!(op.is_running());
        assert_eq!(op.input(), Some(&"save_ctx".to_string()));
    }

    #[test]
    fn fresh_success_returns_input() {
        let mut op = Operation::idle();
        let t = op.begin("ctx".to_string());
        let returned = op.settle(Settled::new(t, Ok(())));
        assert_eq!(returned, Some("ctx".to_string()));
        assert!(op.is_idle());
    }

    #[test]
    fn failure_retains_input_and_exposes_error() {
        let mut op = Operation::idle();
        let t = op.begin("ctx".to_string());
        let returned = op.settle(Settled::new(t, Err(error("save failed"))));
        assert!(returned.is_none());
        assert_eq!(op.input(), Some(&"ctx".to_string()));
        assert!(op.error().is_some());
    }

    #[test]
    fn stale_settle_is_ignored() {
        let mut op = Operation::idle();
        let t1 = op.begin("a".to_string());
        let _t2 = op.begin("b".to_string());
        op.settle(Settled::new(t1, Ok(())));
        assert!(op.is_running());
        assert_eq!(op.input(), Some(&"b".to_string()));
    }

    #[test]
    fn reset_returns_to_idle() {
        let mut op = Operation::idle();
        op.begin("ctx".to_string());
        op.reset();
        assert!(op.is_idle());
    }

    #[test]
    fn dismiss_error_returns_to_idle() {
        let mut op = Operation::idle();
        let t = op.begin("ctx".to_string());
        op.settle(Settled::new(t, Err(error("boom"))));
        op.dismiss_error();
        assert!(op.is_idle());
        assert!(op.error().is_none());
    }

    #[test]
    fn presentation_is_running() {
        let mut op = Operation::idle();
        op.begin("ctx".to_string());
        assert!(OperationStatusPresentation::is_running(&op));
        assert!(OperationStatusPresentation::error(&op).is_none());
    }

    #[test]
    fn presentation_exposes_error() {
        use nive_core::ErrorPresentation;
        let mut op = Operation::idle();
        let t = op.begin("ctx".to_string());
        op.settle(Settled::new(t, Err(error("op failed"))));
        assert_eq!(
            OperationStatusPresentation::error(&op).map(ErrorPresentation::summary),
            Some("op failed")
        );
    }
}
