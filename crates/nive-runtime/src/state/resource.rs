use std::future::Future;

use nive_core::{ErrorPresentation, ResourceStatusPresentation};

use crate::UserFacingError;

use super::request::{
    CancelSignal, CancellationReason, Request, RequestCancellation, RequestControl, RequestId,
    RequestPolicy, RequestTask, SettleOutcome, Settled,
};
use super::scope::ScopeId;

/// Opinionated state machine for an asynchronously loaded value.
///
/// An active lane has one logical owner, so resources are deliberately affine:
///
/// ```compile_fail
/// use nive_runtime::Resource;
/// let resource = Resource::<()>::idle();
/// let duplicate = resource.clone();
/// # let _ = duplicate;
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Resource<T> {
    value: Option<T>,
    phase: ResourcePhase,
    active: Option<RequestControl>,
}

#[derive(Debug, PartialEq, Eq, Default)]
enum ResourcePhase {
    #[default]
    Idle,
    Ready,
    Failed(UserFacingError),
    Cancelled(CancellationReason),
}

impl<T> Default for Resource<T> {
    fn default() -> Self {
        Self {
            value: None,
            phase: ResourcePhase::Idle,
            active: None,
        }
    }
}

impl<T> Resource<T> {
    pub fn idle() -> Self {
        Self::default()
    }

    pub fn ready(value: T) -> Self {
        Self {
            value: Some(value),
            phase: ResourcePhase::Ready,
            active: None,
        }
    }

    /// Starts the explicit untracked/manual tier and returns its correlation ID.
    pub fn begin(&mut self) -> RequestId {
        let (control, _) = self.admit(RequestPolicy::Restart).expect("restart admits");
        control.id()
    }

    /// Mints an affine tracked request using the default `Restart` policy.
    pub fn request(&mut self, scope: ScopeId) -> Request<T> {
        self.request_with(scope, ())
    }

    /// Mints an affine tracked request with application-owned intent.
    pub fn request_with<I>(&mut self, scope: ScopeId, intent: I) -> Request<T, I> {
        self.request_with_policy(scope, intent, RequestPolicy::Restart)
            .expect("restart always admits")
    }

    /// Attempts to admit a request using an explicit single-lane policy.
    pub fn request_with_policy<I>(
        &mut self,
        scope: ScopeId,
        intent: I,
        policy: RequestPolicy,
    ) -> Option<Request<T, I>> {
        let (control, replaces) = self.admit(policy)?;
        Some(Request::new(control, replaces, scope, intent))
    }

    pub fn settle(&mut self, settled: Settled<T>) -> SettleOutcome {
        if self.active.as_ref().map(RequestControl::id) != Some(settled.request()) {
            return SettleOutcome::Stale;
        }
        self.active = None;
        match settled {
            Settled::Succeeded { value, .. } => {
                self.value = Some(value);
                self.phase = ResourcePhase::Ready;
                SettleOutcome::Succeeded(())
            }
            Settled::Failed { error, .. } => {
                self.phase = ResourcePhase::Failed(error);
                SettleOutcome::Failed
            }
            Settled::Cancelled { reason, .. } => {
                self.phase = ResourcePhase::Cancelled(reason);
                SettleOutcome::Cancelled(reason)
            }
        }
    }

    /// Cancels active tracked work and records an explicit terminal phase.
    pub fn cancel(&mut self) -> Option<RequestCancellation> {
        let control = self.active.take()?;
        self.phase = ResourcePhase::Cancelled(CancellationReason::Explicit);
        Some(RequestCancellation::explicit(control))
    }

    /// Clears local state and returns cancellation for any active tracked work.
    pub fn reset(&mut self) -> Option<RequestCancellation> {
        let cancellation = self.active.take().map(RequestCancellation::explicit);
        self.value = None;
        self.phase = ResourcePhase::Idle;
        cancellation
    }

    pub fn dismiss_error(&mut self) {
        if matches!(self.phase, ResourcePhase::Failed(_)) {
            self.phase = self.resting_phase();
        }
    }

    pub fn dismiss_cancellation(&mut self) {
        if matches!(self.phase, ResourcePhase::Cancelled(_)) {
            self.phase = self.resting_phase();
        }
    }

    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn error(&self) -> Option<&UserFacingError> {
        match &self.phase {
            ResourcePhase::Failed(error) => Some(error),
            _ => None,
        }
    }

    pub fn cancellation_reason(&self) -> Option<CancellationReason> {
        match self.phase {
            ResourcePhase::Cancelled(reason) => Some(reason),
            _ => None,
        }
    }

    pub fn is_loading(&self) -> bool {
        self.active.is_some()
    }

    pub fn is_refreshing(&self) -> bool {
        self.active.is_some() && self.value.is_some()
    }

    pub fn is_initial_loading(&self) -> bool {
        self.active.is_some() && self.value.is_none()
    }

    pub fn is_idle(&self) -> bool {
        self.active.is_none() && matches!(self.phase, ResourcePhase::Idle)
    }

    /// Admits and performs a directly constructed scoped request.
    pub fn load<Message, Fut>(
        &mut self,
        scope: ScopeId,
        run: impl FnOnce(CancelSignal) -> Fut,
        settled: impl FnOnce(Settled<T>) -> Message + Send + 'static,
    ) -> RequestTask<Message>
    where
        T: Send + 'static,
        Message: Send + 'static,
        Fut: Future<Output = crate::UserFacingResult<T>> + Send + 'static,
    {
        self.request(scope)
            .perform(|(), cancel| run(cancel), settled)
    }

    fn admit(
        &mut self,
        policy: RequestPolicy,
    ) -> Option<(RequestControl, Option<RequestCancellation>)> {
        if self.active.is_some() && policy == RequestPolicy::DropNew {
            return None;
        }
        let replaces = self.active.take().map(RequestCancellation::replaced);
        let control = RequestControl::new();
        self.active = Some(control.clone());
        self.phase = self.resting_phase();
        Some((control, replaces))
    }

    fn resting_phase(&self) -> ResourcePhase {
        if self.value.is_some() {
            ResourcePhase::Ready
        } else {
            ResourcePhase::Idle
        }
    }
}

impl<T> ResourceStatusPresentation for Resource<T> {
    fn is_refreshing(&self) -> bool {
        self.is_refreshing()
    }

    fn has_value(&self) -> bool {
        self.value().is_some()
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
    fn refresh_retains_value_and_applies_success() {
        let mut resource = Resource::ready("cached");
        let request = resource.begin();
        assert!(resource.is_refreshing());
        assert_eq!(resource.value(), Some(&"cached"));

        assert_eq!(
            resource.settle(Settled::succeeded(request, "fresh")),
            SettleOutcome::Succeeded(())
        );
        assert_eq!(resource.value(), Some(&"fresh"));
    }

    #[test]
    fn failure_and_cancellation_retain_value_but_are_distinct() {
        let mut resource = Resource::ready(1);
        let failed = resource.begin();
        assert_eq!(
            resource.settle(Settled::failed(failed, error("failed"))),
            SettleOutcome::Failed
        );
        assert_eq!(resource.value(), Some(&1));
        assert!(resource.error().is_some());

        let cancelled = resource.begin();
        assert_eq!(
            resource.settle(Settled::cancelled(
                cancelled,
                CancellationReason::ScopeClosed
            )),
            SettleOutcome::Cancelled(CancellationReason::ScopeClosed)
        );
        assert_eq!(resource.value(), Some(&1));
        assert!(resource.error().is_none());
    }

    #[test]
    fn stale_settlement_changes_nothing() {
        let mut resource = Resource::<u32>::idle();
        let stale = resource.begin();
        let current = resource.begin();
        assert_eq!(
            resource.settle(Settled::succeeded(stale, 1)),
            SettleOutcome::Stale
        );
        assert!(resource.is_loading());
        assert_eq!(
            resource.settle(Settled::succeeded(current, 2)),
            SettleOutcome::Succeeded(())
        );
    }

    #[test]
    fn reset_returns_cancellation_and_clears_value() {
        let mut resource = Resource::ready(1);
        resource.begin();
        assert!(resource.reset().is_some());
        assert!(resource.is_idle());
        assert!(resource.value().is_none());
    }
}
