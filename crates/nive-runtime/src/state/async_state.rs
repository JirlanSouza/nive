use crate::{RequestId, UserFacingError};
use nive_ui::widgets::{ErrorPresentation, ResourceStatusPresentation};

/// Reusable state machine for an async-loaded resource.
///
/// Tracks idle, loading (optionally retaining a stale value for refreshing),
/// loaded, and failed states. Implements the UI presentation contracts
/// ([`ErrorPresentation`], [`ResourceStatusPresentation`]) so `nive-ui`
/// feedback widgets can render it without depending on app-domain types.
///
/// ## Stale-request guarding
///
/// Use [`AsyncState::set_loading_with`]/[`AsyncState::set_loaded_with`]/
/// [`AsyncState::set_failed_with`] (with a [`RequestId`] from a
/// [`crate::RequestCounter`]) to silently ignore responses that don't match
/// the most recent in-flight request. The legacy
/// [`AsyncState::set_loading`]/[`AsyncState::set_loaded`]/[`AsyncState::set_failed`]
/// methods **do not guard** against stale responses — they preserve
/// pre-change behaviour for apps that hand-roll their own id checks or do
/// not need them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncState<T> {
    inner: AsyncStateInner<T>,
    /// The id of the most recent in-flight request, if any. Required for
    /// stale-response guards; `None` when no guarded request is in flight
    /// (e.g. after legacy `set_loading()` or after `set_loaded_with`).
    request_id: Option<RequestId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum AsyncStateInner<T> {
    /// No value has been requested yet.
    #[default]
    Idle,
    /// A load is in progress, optionally retaining a stale value while refreshing.
    Loading { value: Option<T> },
    /// A value was loaded successfully.
    Loaded(T),
    /// The last load failed, optionally retaining a stale value.
    Failed {
        value: Option<T>,
        error: UserFacingError,
    },
}

impl<T> Default for AsyncState<T> {
    fn default() -> Self {
        Self {
            inner: AsyncStateInner::Idle,
            request_id: None,
        }
    }
}

impl<T> AsyncState<T> {
    /// Constructs an `Idle` async state.
    pub fn idle() -> Self {
        Self::default()
    }

    /// Constructs a `Loading` async state with no retained value.
    pub fn loading() -> Self {
        Self {
            inner: AsyncStateInner::Loading { value: None },
            request_id: None,
        }
    }

    pub fn new(value: T) -> Self {
        Self {
            inner: AsyncStateInner::Loaded(value),
            request_id: None,
        }
    }

    pub fn value(&self) -> Option<&T> {
        match &self.inner {
            AsyncStateInner::Loaded(value)
            | AsyncStateInner::Loading { value: Some(value) }
            | AsyncStateInner::Failed {
                value: Some(value), ..
            } => Some(value),
            AsyncStateInner::Idle
            | AsyncStateInner::Loading { value: None }
            | AsyncStateInner::Failed { value: None, .. } => None,
        }
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.inner, AsyncStateInner::Loading { .. })
    }

    pub fn error(&self) -> Option<&UserFacingError> {
        match &self.inner {
            AsyncStateInner::Failed { error, .. } => Some(error),
            AsyncStateInner::Idle
            | AsyncStateInner::Loading { .. }
            | AsyncStateInner::Loaded(_) => None,
        }
    }

    /// Resets to `Idle` (no id guard). Clears any in-flight `RequestId`.
    pub fn set_idle(&mut self) {
        self.inner = AsyncStateInner::Idle;
        self.request_id = None;
    }

    /// Transitions to `Loading { value: None }` (no id guard).
    /// Clears any in-flight `RequestId`. Use [`Self::set_loading_with`] for
    /// stale-response guarding.
    pub fn set_loading(&mut self) {
        self.inner = AsyncStateInner::Loading { value: None };
        self.request_id = None;
    }

    pub fn set_refreshing(&mut self) {
        let value = self.take_value();
        self.inner = AsyncStateInner::Loading { value };
        self.request_id = None;
    }

    /// Transitions to `Loaded(value)` (no id guard).
    pub fn set_loaded(&mut self, value: T) {
        self.inner = AsyncStateInner::Loaded(value);
        self.request_id = None;
    }

    pub fn set_failed(&mut self, error: UserFacingError) {
        let value = self.take_value();
        self.inner = AsyncStateInner::Failed { value, error };
        self.request_id = None;
    }

    pub fn set_failed_empty(&mut self, error: UserFacingError) {
        self.inner = AsyncStateInner::Failed { value: None, error };
        self.request_id = None;
    }

    /// Transitions to `Loading { value: None }` and records `id` as the
    /// in-flight request. Subsequent `set_loaded_with(other_id, ...)` or
    /// `set_failed_with(other_id, ...)` calls are silently ignored.
    pub fn set_loading_with(&mut self, id: RequestId) {
        self.inner = AsyncStateInner::Loading { value: None };
        self.request_id = Some(id);
    }

    /// Transitions to `Loaded(value)` only if `id` matches the most recent
    /// in-flight request recorded by [`Self::set_loading_with`]. Stale
    /// responses (mismatched ids) are silently ignored — no state transition
    /// and no side effect.
    pub fn set_loaded_with(&mut self, id: RequestId, value: T) {
        if self.request_id == Some(id) {
            self.inner = AsyncStateInner::Loaded(value);
            self.request_id = None;
        }
    }

    /// Transitions to `Failed { value, error }` only if `id` matches the most
    /// recent in-flight request. Stale responses are silently ignored. Like
    /// [`Self::set_failed`], this preserves the cached value (or `None`).
    pub fn set_failed_with(&mut self, id: RequestId, error: UserFacingError) {
        if self.request_id == Some(id) {
            let value = self.take_value();
            self.inner = AsyncStateInner::Failed { value, error };
            self.request_id = None;
        }
    }

    /// Returns the `RequestId` of the most recent in-flight request, if any.
    /// Cleared by the un-guarded setters and by transitional helper setters
    /// once a terminal state is reached.
    pub fn request_id(&self) -> Option<RequestId> {
        self.request_id
    }

    pub fn dismiss_error(&mut self) {
        let current_inner = std::mem::take(&mut self.inner);
        self.inner = match current_inner {
            AsyncStateInner::Failed {
                value: Some(value), ..
            } => AsyncStateInner::Loaded(value),
            AsyncStateInner::Failed { value: None, .. } => AsyncStateInner::Idle,
            other => other,
        };
        self.request_id = None;
    }

    fn take_inner(&mut self) -> AsyncStateInner<T> {
        std::mem::take(&mut self.inner)
    }

    fn take_value(&mut self) -> Option<T> {
        match self.take_inner() {
            AsyncStateInner::Loaded(value)
            | AsyncStateInner::Loading { value: Some(value) }
            | AsyncStateInner::Failed {
                value: Some(value), ..
            } => {
                self.inner = AsyncStateInner::Idle;
                Some(value)
            }
            AsyncStateInner::Idle
            | AsyncStateInner::Loading { value: None }
            | AsyncStateInner::Failed { value: None, .. } => None,
        }
    }
}

impl<T> ResourceStatusPresentation for AsyncState<T> {
    fn is_refreshing(&self) -> bool {
        matches!(self.inner, AsyncStateInner::Loading { value: Some(_) })
    }

    fn has_value(&self) -> bool {
        self.value().is_some()
    }

    fn error(&self) -> Option<&dyn ErrorPresentation> {
        self.error().map(|error| error as &dyn ErrorPresentation)
    }
}

#[cfg(test)]
mod async_state_tests {
    use super::*;
    use nive_ui::widgets::ResourceStatusPresentation;

    #[test]
    fn set_loaded_updates_value_and_clears_error() {
        let mut state = AsyncState::new(1);
        state.set_failed(UserFacingError::custom(
            "record_catalog",
            "Record not found (record_id: r1)",
        ));

        state.set_loaded(2);

        assert_eq!(state.value(), Some(&2));
        assert!(!state.is_loading());
        assert!(state.error().is_none());
    }

    #[test]
    fn set_refreshing_preserves_cached_value() {
        let mut state = AsyncState::new("cached");

        state.set_refreshing();

        assert!(state.is_loading());
        assert_eq!(state.value(), Some(&"cached"));
    }

    #[test]
    fn set_loading_clears_cached_value() {
        let mut state = AsyncState::new("cached");

        state.set_loading();

        assert!(state.is_loading());
        assert!(state.value().is_none());
    }

    #[test]
    fn set_failed_preserves_cached_value_and_exposes_error() {
        let mut state = AsyncState::new("cached");

        state.set_failed(UserFacingError::custom(
            "record_catalog",
            "Record not found (record_id: r1)",
        ));

        assert_eq!(state.value(), Some(&"cached"));
        assert_eq!(
            state.error().map(UserFacingError::summary),
            Some("Record not found")
        );
    }

    #[test]
    fn set_failed_empty_clears_cached_value_and_exposes_error() {
        let mut state = AsyncState::new("cached");

        state.set_failed_empty(UserFacingError::custom(
            "record_catalog",
            "Record not found (record_id: r1)",
        ));

        assert!(state.value().is_none());
        assert_eq!(
            state.error().map(UserFacingError::summary),
            Some("Record not found")
        );
    }

    #[test]
    fn dismiss_error_preserves_cached_value() {
        let mut state = AsyncState::new("cached");
        state.set_failed(UserFacingError::custom(
            "record_catalog",
            "Record not found (record_id: r1)",
        ));

        state.dismiss_error();

        assert_eq!(state.value(), Some(&"cached"));
        assert!(state.error().is_none());
    }

    #[test]
    fn dismiss_error_without_cache_returns_to_idle() {
        let mut state = AsyncState::<&str>::idle();
        state.set_failed_empty(UserFacingError::custom(
            "record_catalog",
            "Record not found (record_id: r1)",
        ));

        state.dismiss_error();

        assert_eq!(state, AsyncState::<&str>::idle());
    }

    #[test]
    fn resource_presentation_distinguishes_refresh_from_initial_load() {
        let refreshing = {
            let mut state = AsyncState::new("cached");
            state.set_refreshing();
            state
        };
        let loading = AsyncState::<&str>::loading();

        assert!(ResourceStatusPresentation::is_refreshing(&refreshing));
        assert!(!ResourceStatusPresentation::is_refreshing(&loading));
        assert!(ResourceStatusPresentation::has_value(&refreshing));
        assert!(!ResourceStatusPresentation::has_value(&loading));
    }

    #[test]
    fn resource_presentation_exposes_failed_error() {
        let state = {
            let mut state = AsyncState::new("cached");
            state.set_failed(UserFacingError::custom("record_catalog", "Refresh failed"));
            state
        };

        assert_eq!(
            ResourceStatusPresentation::error(&state).map(ErrorPresentation::summary),
            Some("Refresh failed")
        );
    }

    // ----- Stale-request guard tests (Section 5.5) -----

    fn counter() -> crate::RequestCounter {
        crate::RequestCounter::default()
    }

    #[test]
    fn fresh_request_then_set_loaded_with_matches_transitions_to_loaded() {
        let mut state = AsyncState::<u8>::idle();
        let mut counter = counter();

        let id_a = counter.next();
        state.set_loading_with(id_a);
        state.set_loaded_with(id_a, 42);

        assert_eq!(state.value(), Some(&42));
        assert!(!state.is_loading());
    }

    #[test]
    fn stale_set_loaded_with_is_silently_ignored() {
        let mut state = AsyncState::<u8>::idle();
        let mut counter = counter();

        let id_a = counter.next();
        let id_b = counter.next();
        state.set_loading_with(id_a);
        // a stale response for the prior request arrives
        state.set_loaded_with(id_a, 42);

        // a newer request supersedes the in-flight id (id_b)
        state.set_loading_with(id_b);
        // stale response for id_a arrives — should be ignored
        state.set_loaded_with(id_a, 7);

        assert!(state.is_loading());
        assert!(state.value().is_none());
    }

    #[test]
    fn fresh_failure_is_applied_while_stale_failure_is_ignored() {
        let mut state = AsyncState::<&str>::idle();
        let mut counter = counter();

        let id_a = counter.next();
        let id_b = counter.next();

        // First request: id_a in flight.
        state.set_loading_with(id_a);

        // A second, newer request supersedes it (state stays Loading, request_id -> id_b).
        state.set_loading_with(id_b);

        // Stale failure for the older request (id_a) is ignored.
        state.set_failed_with(id_a, UserFacingError::custom("catalog", "stale"));
        assert!(state.is_loading());
        assert!(state.error().is_none());

        // Fresh failure for the current in-flight request (id_b) is applied.
        state.set_failed_with(id_b, UserFacingError::custom("catalog", "fresh"));
        assert!(state.error().is_some());
        assert_eq!(state.error().map(UserFacingError::summary), Some("fresh"));
    }

    #[test]
    fn legacy_set_loaded_has_no_id_guard() {
        let mut state = AsyncState::<u8>::idle();
        let mut counter = counter();

        let id_a = counter.next();
        state.set_loading_with(id_a);
        // Legacy `set_loaded` ignores any in-flight request_id guarantee.
        state.set_loaded(42);

        assert_eq!(state.value(), Some(&42));
        assert_eq!(state.request_id(), None);
    }
}
