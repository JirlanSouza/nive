use std::future::Future;
use std::marker::PhantomData;

use nive_core::{ErrorPresentation, OperationStatusPresentation};

use crate::UserFacingError;

use super::request::{
    CancelSignal, CancellationReason, Request, RequestCancellation, RequestControl, RequestId,
    RequestPolicy, RequestTask, SettleOutcome, Settled,
};
use super::scope::ScopeId;

#[derive(Debug, PartialEq, Eq, Default)]
enum OperationPhase<C> {
    #[default]
    Idle,
    Running(C),
    Failed {
        input: C,
        error: UserFacingError,
    },
    Cancelled {
        input: C,
        reason: CancellationReason,
    },
}

/// Opinionated single-lane state machine for an asynchronous mutation.
///
/// An active lane has one logical owner, so operations are deliberately affine:
///
/// ```compile_fail
/// use nive_runtime::Operation;
/// let operation = Operation::<()>::idle();
/// let duplicate = operation.clone();
/// # let _ = duplicate;
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Operation<C, T = ()> {
    phase: OperationPhase<C>,
    active: Option<RequestControl>,
    _output: PhantomData<fn() -> T>,
}

impl<C, T> Default for Operation<C, T> {
    fn default() -> Self {
        Self {
            phase: OperationPhase::Idle,
            active: None,
            _output: PhantomData,
        }
    }
}

impl<C, T> Operation<C, T> {
    pub fn idle() -> Self {
        Self::default()
    }

    /// Starts the explicit untracked/manual tier and returns its correlation ID.
    pub fn begin(&mut self, input: C) -> RequestId {
        let control = RequestControl::new();
        self.active = Some(control.clone());
        self.phase = OperationPhase::Running(input);
        control.id()
    }

    /// Uses `DropNew` and clones the presentation input into service intent.
    pub fn request(&mut self, scope: ScopeId, input: C) -> Option<Request<T, C>>
    where
        C: Clone,
    {
        let intent = input.clone();
        self.request_with(scope, input, intent)
    }

    /// Uses `DropNew` while separating presentation input from service intent.
    pub fn request_with<I>(
        &mut self,
        scope: ScopeId,
        input: C,
        intent: I,
    ) -> Option<Request<T, I>> {
        self.request_with_policy(scope, input, intent, RequestPolicy::DropNew)
    }

    pub fn request_with_policy<I>(
        &mut self,
        scope: ScopeId,
        input: C,
        intent: I,
        policy: RequestPolicy,
    ) -> Option<Request<T, I>> {
        if self.active.is_some() && policy == RequestPolicy::DropNew {
            return None;
        }
        let replaces = self.active.take().map(RequestCancellation::replaced);
        let control = RequestControl::new();
        self.active = Some(control.clone());
        self.phase = OperationPhase::Running(input);
        Some(Request::new(control, replaces, scope, intent))
    }

    pub fn settle(&mut self, settled: Settled<T>) -> SettleOutcome<(C, T)> {
        if self.active.as_ref().map(RequestControl::id) != Some(settled.request()) {
            return SettleOutcome::Stale;
        }
        self.active = None;
        let phase = std::mem::take(&mut self.phase);
        let OperationPhase::Running(input) = phase else {
            self.phase = phase;
            return SettleOutcome::Stale;
        };
        match settled {
            Settled::Succeeded { value, .. } => {
                self.phase = OperationPhase::Idle;
                SettleOutcome::Succeeded((input, value))
            }
            Settled::Failed { error, .. } => {
                self.phase = OperationPhase::Failed { input, error };
                SettleOutcome::Failed
            }
            Settled::Cancelled { reason, .. } => {
                self.phase = OperationPhase::Cancelled { input, reason };
                SettleOutcome::Cancelled(reason)
            }
        }
    }

    pub fn cancel(&mut self) -> Option<RequestCancellation> {
        let control = self.active.take()?;
        let phase = std::mem::take(&mut self.phase);
        self.phase = match phase {
            OperationPhase::Running(input) => OperationPhase::Cancelled {
                input,
                reason: CancellationReason::Explicit,
            },
            other => other,
        };
        Some(RequestCancellation::explicit(control))
    }

    pub fn reset(&mut self) -> Option<RequestCancellation> {
        let cancellation = self.active.take().map(RequestCancellation::explicit);
        self.phase = OperationPhase::Idle;
        cancellation
    }

    pub fn dismiss_error(&mut self) {
        if matches!(self.phase, OperationPhase::Failed { .. }) {
            self.phase = OperationPhase::Idle;
        }
    }

    pub fn dismiss_cancellation(&mut self) {
        if matches!(self.phase, OperationPhase::Cancelled { .. }) {
            self.phase = OperationPhase::Idle;
        }
    }

    pub fn is_running(&self) -> bool {
        self.active.is_some()
    }

    pub fn is_idle(&self) -> bool {
        self.active.is_none() && matches!(self.phase, OperationPhase::Idle)
    }

    pub fn error(&self) -> Option<&UserFacingError> {
        match &self.phase {
            OperationPhase::Failed { error, .. } => Some(error),
            _ => None,
        }
    }

    pub fn cancellation_reason(&self) -> Option<CancellationReason> {
        match self.phase {
            OperationPhase::Cancelled { reason, .. } => Some(reason),
            _ => None,
        }
    }

    pub fn input(&self) -> Option<&C> {
        match &self.phase {
            OperationPhase::Running(input)
            | OperationPhase::Failed { input, .. }
            | OperationPhase::Cancelled { input, .. } => Some(input),
            OperationPhase::Idle => None,
        }
    }

    pub fn run<Message, Fut>(
        &mut self,
        scope: ScopeId,
        input: C,
        run: impl FnOnce(&C, CancelSignal) -> Fut,
        settled: impl FnOnce(Settled<T>) -> Message + Send + 'static,
    ) -> Option<RequestTask<Message>>
    where
        C: Send + 'static,
        T: Send + 'static,
        Message: Send + 'static,
        Fut: Future<Output = crate::UserFacingResult<T>> + Send + 'static,
    {
        if self.active.is_some() {
            return None;
        }
        let control = RequestControl::new();
        let cancel = CancelSignal::new(&control, &scope);
        let future = run(&input, cancel);
        self.active = Some(control.clone());
        self.phase = OperationPhase::Running(input);
        Some(Request::new(control, None, scope, future).perform(|future, _cancel| future, settled))
    }
}

impl<C, T> OperationStatusPresentation for Operation<C, T> {
    fn is_running(&self) -> bool {
        self.is_running()
    }

    fn error(&self) -> Option<&dyn ErrorPresentation> {
        self.error().map(|error| error as &dyn ErrorPresentation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error(message: &str) -> UserFacingError {
        UserFacingError::custom("test", message)
    }

    #[test]
    fn typed_success_returns_input_and_output() {
        let mut operation = Operation::<String, u64>::idle();
        let request = operation.begin(String::from("draft"));
        assert_eq!(
            operation.settle(Settled::succeeded(request, 42)),
            SettleOutcome::Succeeded((String::from("draft"), 42))
        );
        assert!(operation.is_idle());
    }

    #[test]
    fn failure_and_cancellation_retain_input() {
        let mut operation = Operation::<String>::idle();
        let failed = operation.begin(String::from("draft"));
        assert_eq!(
            operation.settle(Settled::failed(failed, error("failed"))),
            SettleOutcome::Failed
        );
        assert_eq!(operation.input().map(String::as_str), Some("draft"));

        let cancelled = operation.begin(String::from("again"));
        assert_eq!(
            operation.settle(Settled::cancelled(
                cancelled,
                CancellationReason::ScopeClosed
            )),
            SettleOutcome::Cancelled(CancellationReason::ScopeClosed)
        );
        assert_eq!(operation.input().map(String::as_str), Some("again"));
        assert!(operation.error().is_none());
    }

    #[test]
    fn stale_settlement_is_explicit() {
        let mut operation = Operation::<String>::idle();
        let stale = operation.begin(String::from("first"));
        let current = operation.begin(String::from("second"));
        assert_eq!(
            operation.settle(Settled::succeeded(stale, ())),
            SettleOutcome::Stale
        );
        assert!(operation.is_running());
        assert_eq!(
            operation.settle(Settled::succeeded(current, ())),
            SettleOutcome::Succeeded((String::from("second"), ()))
        );
    }

    #[test]
    fn drop_new_preserves_the_first_lane_without_replacing_input() {
        let root = ScopeId::root("app");
        let mut operation = Operation::<String>::idle();
        let first = operation.request(root.id(), String::from("first"));
        let second = operation.request(root.id(), String::from("second"));

        assert!(first.is_some());
        assert!(second.is_none());
        assert_eq!(operation.input().map(String::as_str), Some("first"));
    }

    #[test]
    fn direct_run_accepts_non_clone_input() {
        struct NonClone;

        let root = ScopeId::root("app");
        let mut operation = Operation::<NonClone>::idle();
        let task = operation.run(
            root.id(),
            NonClone,
            |_input, _cancel| async { Ok(()) },
            |_| (),
        );

        assert!(task.is_some());
    }
}
