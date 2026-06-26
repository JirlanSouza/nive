use std::future::Future;

use iced::Task;
use nive_ui::widgets::{ErrorPresentation, ResourceStatusPresentation};

use crate::UserFacingError;

use super::request::{RequestCounter, RequestId, Settled};

/// Opinionated state machine for an asynchronously loaded value.
///
/// Tracks `Idle`, `Loading` (retaining a stale value while refreshing),
/// `Loaded(T)`, and `Failed` (retaining a stale value plus a
/// [`UserFacingError`]). Implements [`ResourceStatusPresentation`] so
/// `nive-ui` feedback widgets render it without depending on `T`.
///
/// ## Lifecycle
///
/// ```text
/// begin() → Loading (retains prior value)
/// settle(Settled::new(token, Ok(v)))  → Loaded(v)   (if token is current)
/// settle(Settled::new(token, Err(e))) → Failed       (if token is current; retains value)
/// reset()         → Idle
/// dismiss_error() → Loaded(v) if a value was retained, else Idle
/// ```
///
/// Stale responses (token mismatch) are silently ignored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource<T> {
    inner: ResourceInner<T>,
    token: Option<RequestId>,
    counter: RequestCounter,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum ResourceInner<T> {
    #[default]
    Idle,
    Loading {
        value: Option<T>,
    },
    Loaded(T),
    Failed {
        value: Option<T>,
        error: UserFacingError,
    },
}

impl<T> Default for Resource<T> {
    fn default() -> Self {
        Self {
            inner: ResourceInner::Idle,
            token: None,
            counter: RequestCounter::default(),
        }
    }
}

impl<T> Resource<T> {
    /// Constructs an `Idle` resource. Equivalent to `Resource::default()`.
    pub fn idle() -> Self {
        Self::default()
    }

    /// Constructs a `Loaded(value)` resource with a pre-loaded value.
    pub fn ready(value: T) -> Self {
        Self {
            inner: ResourceInner::Loaded(value),
            token: None,
            counter: RequestCounter::default(),
        }
    }

    /// Transitions to `Loading`, retaining any previously loaded value
    /// (stale-while-revalidate by default), and mints a fresh in-flight
    /// [`RequestId`] internally.
    ///
    /// On the blessed path, call [`load`](Self::load) instead — it fuses
    /// `begin` with task spawning so the token is invisible plumbing.
    pub fn begin(&mut self) -> RequestId {
        let id = self.counter.next();
        let value = self.take_value();
        self.inner = ResourceInner::Loading { value };
        self.token = Some(id);
        id
    }

    /// Applies `Ok(value)` → `Loaded` or `Err(error)` → `Failed`, but only
    /// if the token carried by `settled` matches the most recent token minted
    /// by [`begin`](Self::begin). Stale responses are silently ignored.
    ///
    /// On failure the previously loaded value is retained.
    pub fn settle(&mut self, settled: Settled<T>) {
        if self.token != Some(settled.token) {
            return;
        }
        self.token = None;
        match settled.result {
            Ok(value) => {
                self.inner = ResourceInner::Loaded(value);
            }
            Err(error) => {
                let value = self.take_value();
                self.inner = ResourceInner::Failed { value, error };
            }
        }
    }

    /// Returns to `Idle`, clearing any retained value and in-flight token.
    pub fn reset(&mut self) {
        self.inner = ResourceInner::Idle;
        self.token = None;
    }

    /// Clears a `Failed` error. If a value was retained returns to `Loaded`;
    /// otherwise returns to `Idle`.
    pub fn dismiss_error(&mut self) {
        let current = std::mem::take(&mut self.inner);
        self.inner = match current {
            ResourceInner::Failed {
                value: Some(value), ..
            } => ResourceInner::Loaded(value),
            ResourceInner::Failed { value: None, .. } => ResourceInner::Idle,
            other => other,
        };
        self.token = None;
    }

    /// Returns the loaded value if one is available (including while loading
    /// or failed with a retained stale value).
    pub fn value(&self) -> Option<&T> {
        match &self.inner {
            ResourceInner::Loaded(value)
            | ResourceInner::Loading { value: Some(value) }
            | ResourceInner::Failed {
                value: Some(value), ..
            } => Some(value),
            _ => None,
        }
    }

    /// Returns the current error, if any.
    pub fn error(&self) -> Option<&UserFacingError> {
        match &self.inner {
            ResourceInner::Failed { error, .. } => Some(error),
            _ => None,
        }
    }

    /// Returns `true` while a load is in progress (initial or refresh).
    pub fn is_loading(&self) -> bool {
        matches!(self.inner, ResourceInner::Loading { .. })
    }

    /// Returns `true` while loading *and* a prior value is retained
    /// (stale-while-revalidate). `false` during initial loading.
    pub fn is_refreshing(&self) -> bool {
        matches!(self.inner, ResourceInner::Loading { value: Some(_) })
    }

    /// Returns `true` while loading with no prior value (the first load).
    pub fn is_initial_loading(&self) -> bool {
        matches!(self.inner, ResourceInner::Loading { value: None })
    }

    /// Returns `true` when the resource is `Idle` (no load ever started or
    /// after [`reset`](Self::reset)).
    pub fn is_idle(&self) -> bool {
        matches!(self.inner, ResourceInner::Idle)
    }

    /// Fuses [`begin`](Self::begin) with task spawning on the send side.
    ///
    /// Calls `begin()` internally, then spawns `future` and maps its
    /// `UserFacingResult<T>` into a `Settled<T>` passed to the `settled`
    /// message constructor. Returns the resulting `Task<Message>`.
    ///
    /// The receive side stays explicit:
    /// ```ignore
    /// Msg::ProjectsSettled(s) => self.projects.settle(s),
    /// ```
    pub fn load<Message, Fut>(
        &mut self,
        future: Fut,
        settled: impl Fn(Settled<T>) -> Message + Send + 'static,
    ) -> Task<Message>
    where
        T: Send + 'static,
        Message: Send + 'static,
        Fut: Future<Output = crate::UserFacingResult<T>> + Send + 'static,
    {
        let token = self.begin();
        Task::perform(future, move |result| settled(Settled::new(token, result)))
    }

    fn take_value(&mut self) -> Option<T> {
        let current = std::mem::take(&mut self.inner);
        match current {
            ResourceInner::Loaded(value)
            | ResourceInner::Loading { value: Some(value) }
            | ResourceInner::Failed {
                value: Some(value), ..
            } => Some(value),
            other => {
                self.inner = other;
                None
            }
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
        self.error().map(|e| e as &dyn ErrorPresentation)
    }
}

#[cfg(test)]
mod resource_tests {
    use super::*;
    use crate::UserFacingError;
    use nive_ui::widgets::ResourceStatusPresentation;

    fn error(msg: &str) -> UserFacingError {
        UserFacingError::custom("test", msg)
    }

    #[test]
    fn idle_by_default() {
        let r = Resource::<u32>::idle();
        assert!(r.is_idle());
        assert!(r.value().is_none());
        assert!(r.error().is_none());
    }

    #[test]
    fn ready_is_loaded_with_value() {
        let r = Resource::ready(42u32);
        assert_eq!(r.value(), Some(&42));
        assert!(!r.is_loading());
    }

    #[test]
    fn begin_retains_prior_value() {
        let mut r = Resource::ready("cached");
        r.begin();
        assert!(r.is_loading());
        assert_eq!(r.value(), Some(&"cached"));
        assert!(r.is_refreshing());
        assert!(!r.is_initial_loading());
    }

    #[test]
    fn begin_from_idle_has_no_value() {
        let mut r = Resource::<u32>::idle();
        r.begin();
        assert!(r.is_loading());
        assert!(r.value().is_none());
        assert!(r.is_initial_loading());
        assert!(!r.is_refreshing());
    }

    #[test]
    fn each_begin_mints_a_distinct_token() {
        let mut r = Resource::<u32>::idle();
        let t1 = r.begin();
        let t2 = r.begin();
        assert_ne!(t1, t2);
    }

    #[test]
    fn fresh_success_transitions_to_loaded() {
        let mut r = Resource::<u32>::idle();
        let t = r.begin();
        r.settle(Settled::new(t, Ok(99)));
        assert_eq!(r.value(), Some(&99));
        assert!(!r.is_loading());
    }

    #[test]
    fn stale_response_is_silently_ignored() {
        let mut r = Resource::<u32>::idle();
        let t1 = r.begin();
        let _t2 = r.begin();
        r.settle(Settled::new(t1, Ok(42)));
        assert!(r.is_loading());
    }

    #[test]
    fn failure_retains_prior_value() {
        let mut r = Resource::ready("cached");
        let t = r.begin();
        r.settle(Settled::new(t, Err(error("boom"))));
        assert!(r.error().is_some());
        assert_eq!(r.value(), Some(&"cached"));
    }

    #[test]
    fn reset_clears_everything() {
        let mut r = Resource::ready("data");
        r.reset();
        assert!(r.is_idle());
        assert!(r.value().is_none());
    }

    #[test]
    fn dismiss_error_keeps_retained_value() {
        let mut r = Resource::ready("cached");
        let t = r.begin();
        r.settle(Settled::new(t, Err(error("boom"))));
        r.dismiss_error();
        assert_eq!(r.value(), Some(&"cached"));
        assert!(r.error().is_none());
    }

    #[test]
    fn dismiss_error_without_value_returns_to_idle() {
        let mut r = Resource::<u32>::idle();
        let t = r.begin();
        r.settle(Settled::new(t, Err(error("boom"))));
        r.dismiss_error();
        assert!(r.is_idle());
    }

    #[test]
    fn presentation_distinguishes_refreshing_from_initial_load() {
        let mut refreshing = Resource::ready("cached");
        refreshing.begin();
        let mut initial = Resource::<&str>::idle();
        initial.begin();

        assert!(ResourceStatusPresentation::is_refreshing(&refreshing));
        assert!(!ResourceStatusPresentation::is_refreshing(&initial));
        assert!(ResourceStatusPresentation::has_value(&refreshing));
        assert!(!ResourceStatusPresentation::has_value(&initial));
    }

    #[test]
    fn presentation_exposes_error() {
        use nive_ui::widgets::ErrorPresentation;
        let mut r = Resource::<u32>::idle();
        let t = r.begin();
        r.settle(Settled::new(t, Err(error("load failed"))));

        assert_eq!(
            ResourceStatusPresentation::error(&r).map(ErrorPresentation::summary),
            Some("load failed")
        );
    }
}
