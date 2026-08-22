use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::num::NonZeroU64;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::Task;
use tokio_util::sync::CancellationToken;

use crate::{UserFacingError, UserFacingResult};

use super::scope::ScopeId;

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// Identifies one admitted async request without exposing its numeric value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(NonZeroU64);

impl RequestId {
    pub(crate) fn mint() -> Self {
        let value = NEXT_REQUEST_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                (current != 0).then_some(current.checked_add(1).unwrap_or(0))
            })
            .unwrap_or_else(|_| panic!("Nive request identity space exhausted"));
        Self(NonZeroU64::new(value).expect("request identity is never zero"))
    }
}

/// Explains why an admitted request ended without succeeding or failing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CancellationReason {
    Explicit,
    Replaced,
    ScopeClosed,
}

/// Carries every possible terminal result together with its correlation ID.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Settled<T> {
    Succeeded {
        request: RequestId,
        value: T,
    },
    Failed {
        request: RequestId,
        error: UserFacingError,
    },
    Cancelled {
        request: RequestId,
        reason: CancellationReason,
    },
}

impl<T> Settled<T> {
    pub fn succeeded(request: RequestId, value: T) -> Self {
        Self::Succeeded { request, value }
    }

    pub fn failed(request: RequestId, error: UserFacingError) -> Self {
        Self::Failed { request, error }
    }

    pub fn cancelled(request: RequestId, reason: CancellationReason) -> Self {
        Self::Cancelled { request, reason }
    }

    pub fn from_result(request: RequestId, result: UserFacingResult<T>) -> Self {
        match result {
            Ok(value) => Self::succeeded(request, value),
            Err(error) => Self::failed(request, error),
        }
    }

    pub fn request(&self) -> RequestId {
        match self {
            Self::Succeeded { request, .. }
            | Self::Failed { request, .. }
            | Self::Cancelled { request, .. } => *request,
        }
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Succeeded { value, .. } => Some(value),
            _ => None,
        }
    }

    pub fn error(&self) -> Option<&UserFacingError> {
        match self {
            Self::Failed { error, .. } => Some(error),
            _ => None,
        }
    }

    pub fn cancellation_reason(&self) -> Option<CancellationReason> {
        match self {
            Self::Cancelled { reason, .. } => Some(*reason),
            _ => None,
        }
    }
}

/// Reports whether a terminal result was applied to its state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
#[non_exhaustive]
pub enum SettleOutcome<P = ()> {
    Succeeded(P),
    Failed,
    Cancelled(CancellationReason),
    Stale,
}

/// Controls admission into a single visible async-state lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RequestPolicy {
    Restart,
    DropNew,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RequestTiming {
    Timeout(Duration),
    Deadline(Instant),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum CancellationMode {
    Hard,
    Graceful(Duration),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StopCause {
    Explicit = 1,
    Replaced = 2,
}

pub(crate) struct RequestControlInner {
    id: RequestId,
    stop: CancellationToken,
    cause: AtomicU8,
    suppress_message: AtomicBool,
}

#[derive(Clone)]
pub(crate) struct RequestControl(Arc<RequestControlInner>);

impl RequestControl {
    pub(crate) fn new() -> Self {
        Self(Arc::new(RequestControlInner {
            id: RequestId::mint(),
            stop: CancellationToken::new(),
            cause: AtomicU8::new(0),
            suppress_message: AtomicBool::new(false),
        }))
    }

    pub(crate) fn id(&self) -> RequestId {
        self.0.id
    }

    fn cancel(&self, cause: StopCause) {
        let _ = self
            .0
            .cause
            .compare_exchange(0, cause as u8, Ordering::AcqRel, Ordering::Acquire);
        self.0.suppress_message.store(true, Ordering::Release);
        self.0.stop.cancel();
    }

    fn stop_cause(&self) -> Option<StopCause> {
        match self.0.cause.load(Ordering::Acquire) {
            1 => Some(StopCause::Explicit),
            2 => Some(StopCause::Replaced),
            _ => None,
        }
    }

    fn suppress_message(&self) -> bool {
        self.0.suppress_message.load(Ordering::Acquire)
    }
}

impl fmt::Debug for RequestControl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RequestControl")
            .field("id", &self.id())
            .finish_non_exhaustive()
    }
}

impl PartialEq for RequestControl {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for RequestControl {}

/// Linear descriptor that asks the runtime to stop a tracked request.
#[must_use = "return this cancellation through Effect"]
pub struct RequestCancellation {
    control: RequestControl,
    cause: StopCause,
}

impl fmt::Debug for RequestCancellation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RequestCancellation")
            .field("request", &self.control.id())
            .field("cause", &self.cause)
            .finish()
    }
}

impl RequestCancellation {
    pub(crate) fn explicit(control: RequestControl) -> Self {
        Self {
            control,
            cause: StopCause::Explicit,
        }
    }

    pub(crate) fn replaced(control: RequestControl) -> Self {
        Self {
            control,
            cause: StopCause::Replaced,
        }
    }

    pub(crate) fn apply(self) -> RequestId {
        let id = self.control.id();
        self.control.cancel(self.cause);
        id
    }
}

/// Observation-only cancellation signal passed to application futures.
#[derive(Clone)]
pub struct CancelSignal {
    request: CancellationToken,
    scope: CancellationToken,
}

impl CancelSignal {
    pub(crate) fn new(control: &RequestControl, scope: &ScopeId) -> Self {
        Self {
            request: control.0.stop.clone(),
            scope: scope.token(),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.request.is_cancelled() || self.scope.is_cancelled()
    }

    pub async fn cancelled(&self) {
        tokio::select! {
            biased;
            _ = self.request.cancelled() => {}
            _ = self.scope.cancelled() => {}
        }
    }
}

impl fmt::Debug for CancelSignal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CancelSignal")
            .field("cancelled", &self.is_cancelled())
            .finish()
    }
}

/// Affine handle connecting state-machine admission to application-owned work.
///
/// ```compile_fail
/// use nive_runtime::Request;
/// fn duplicate(request: Request<()>) {
///     let _ = request.clone();
/// }
/// ```
#[must_use = "a request must be performed or completed"]
pub struct Request<T, I = ()> {
    pub(crate) control: RequestControl,
    pub(crate) replaces: Option<RequestCancellation>,
    scope: ScopeId,
    intent: Option<I>,
    timing: Option<RequestTiming>,
    cancellation: CancellationMode,
    consumed: bool,
    _target: PhantomData<fn() -> T>,
}

impl<T, I> Request<T, I> {
    pub(crate) fn new(
        control: RequestControl,
        replaces: Option<RequestCancellation>,
        scope: ScopeId,
        intent: I,
    ) -> Self {
        Self {
            control,
            replaces,
            scope,
            intent: Some(intent),
            timing: None,
            cancellation: CancellationMode::Hard,
            consumed: false,
            _target: PhantomData,
        }
    }

    pub fn intent(&self) -> &I {
        self.intent.as_ref().expect("request intent is available")
    }

    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timing = Some(RequestTiming::Timeout(duration));
        self
    }

    pub fn deadline(mut self, deadline: Instant) -> Self {
        self.timing = Some(RequestTiming::Deadline(deadline));
        self
    }

    pub fn graceful(mut self, grace: Duration) -> Self {
        self.cancellation = CancellationMode::Graceful(grace);
        self
    }

    pub fn perform<Message, Fut>(
        mut self,
        run: impl FnOnce(I, CancelSignal) -> Fut,
        settled: impl FnOnce(Settled<T>) -> Message + Send + 'static,
    ) -> RequestTask<Message>
    where
        T: Send + 'static,
        I: 'static,
        Message: Send + 'static,
        Fut: Future<Output = UserFacingResult<T>> + Send + 'static,
    {
        self.consumed = true;
        let intent = self.intent.take().expect("request is consumed once");
        let signal = CancelSignal::new(&self.control, &self.scope);
        let future = run(intent, signal);
        RequestTask::new(
            self.control.clone(),
            self.replaces.take(),
            self.scope.clone(),
            self.timing,
            self.cancellation,
            future,
            settled,
        )
    }

    pub fn into_settled(mut self, result: UserFacingResult<T>) -> Settled<T> {
        self.consumed = true;
        self.intent.take();
        Settled::from_result(self.control.id(), result)
    }

    pub fn into_cancelled(mut self, reason: CancellationReason) -> Settled<T> {
        self.consumed = true;
        self.intent.take();
        Settled::cancelled(self.control.id(), reason)
    }
}

impl<T, I> fmt::Debug for Request<T, I> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Request")
            .field("request", &self.control.id())
            .field("scope", &self.scope)
            .finish_non_exhaustive()
    }
}

impl<T, I> Drop for Request<T, I> {
    fn drop(&mut self) {
        if cfg!(debug_assertions) && !self.consumed && self.intent.is_some() {
            log::warn!(target: "nive_runtime::request", "request.dropped request={:?}", self.control.id());
        }
    }
}

#[derive(Clone)]
pub(crate) struct RequestEvent<M> {
    pub(crate) request: RequestId,
    pub(crate) message: Option<M>,
}

/// Opaque carrier retaining tracked-request metadata until runtime admission.
///
/// ```compile_fail
/// use nive_runtime::RequestTask;
/// fn duplicate(task: RequestTask<()>) {
///     let _ = task.clone();
/// }
/// ```
#[must_use = "a request task must be returned through Effect or ScreenEffect"]
pub struct RequestTask<M> {
    pub(crate) task: Task<RequestEvent<M>>,
    pub(crate) request: RequestId,
    pub(crate) scope_key: NonZeroU64,
    pub(crate) control: RequestControl,
    pub(crate) replaces: Option<RequestCancellation>,
    consumed: bool,
}

impl<M> RequestTask<M> {
    #[allow(clippy::too_many_arguments)]
    fn new<T, Fut>(
        control: RequestControl,
        replaces: Option<RequestCancellation>,
        scope: ScopeId,
        timing: Option<RequestTiming>,
        cancellation: CancellationMode,
        future: Fut,
        settled: impl FnOnce(Settled<T>) -> M + Send + 'static,
    ) -> Self
    where
        T: Send + 'static,
        M: Send + 'static,
        Fut: Future<Output = UserFacingResult<T>> + Send + 'static,
    {
        let request = control.id();
        let task_control = control.clone();
        let scope_token = scope.token();
        let task = Task::perform(
            run_request(
                request,
                task_control,
                scope_token,
                timing,
                cancellation,
                future,
                settled,
            ),
            |event| event,
        );
        Self {
            task,
            request,
            scope_key: scope.key(),
            control,
            replaces,
            consumed: false,
        }
    }

    pub fn map<N>(mut self, map: impl Fn(M) -> N + Send + Sync + 'static) -> RequestTask<N>
    where
        M: Send + 'static,
        N: Send + 'static,
    {
        self.consumed = true;
        let task = std::mem::replace(&mut self.task, Task::none());
        RequestTask {
            task: task.map(move |event| RequestEvent {
                request: event.request,
                message: event.message.map(&map),
            }),
            request: self.request,
            scope_key: self.scope_key,
            control: self.control.clone(),
            replaces: self.replaces.take(),
            consumed: false,
        }
    }

    pub(crate) fn into_parts(mut self) -> (Task<RequestEvent<M>>, RequestRegistration) {
        self.consumed = true;
        (
            std::mem::replace(&mut self.task, Task::none()),
            RequestRegistration {
                request: self.request,
                scope_key: self.scope_key,
                control: self.control.clone(),
                replaces: self.replaces.take(),
            },
        )
    }
}

impl<M> fmt::Debug for RequestTask<M> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RequestTask")
            .field("request", &self.request)
            .finish_non_exhaustive()
    }
}

impl<M> Drop for RequestTask<M> {
    fn drop(&mut self) {
        if cfg!(debug_assertions) && !self.consumed {
            log::warn!(target: "nive_runtime::request", "request_task.dropped request={:?}", self.request);
        }
    }
}

pub(crate) struct RequestRegistration {
    pub(crate) request: RequestId,
    pub(crate) scope_key: NonZeroU64,
    pub(crate) control: RequestControl,
    pub(crate) replaces: Option<RequestCancellation>,
}

async fn run_request<T, M, Fut>(
    request: RequestId,
    control: RequestControl,
    scope: CancellationToken,
    timing: Option<RequestTiming>,
    cancellation: CancellationMode,
    future: Fut,
    settled: impl FnOnce(Settled<T>) -> M,
) -> RequestEvent<M>
where
    Fut: Future<Output = UserFacingResult<T>>,
{
    let mut future = Box::pin(future);
    let mut timer: Pin<Box<dyn Future<Output = ()> + Send>> = match timing {
        Some(RequestTiming::Timeout(duration)) => Box::pin(tokio::time::sleep(duration)),
        Some(RequestTiming::Deadline(deadline)) => Box::pin(tokio::time::sleep_until(
            tokio::time::Instant::from_std(deadline),
        )),
        None => Box::pin(std::future::pending()),
    };

    enum Terminal<T> {
        Result(UserFacingResult<T>),
        RequestCancelled,
        ScopeClosed,
        TimedOut,
    }

    let terminal = tokio::select! {
        biased;
        _ = control.0.stop.cancelled() => Terminal::RequestCancelled,
        _ = scope.cancelled() => Terminal::ScopeClosed,
        _ = &mut timer => Terminal::TimedOut,
        result = &mut future => Terminal::Result(result),
    };

    let terminal = match terminal {
        Terminal::Result(result) => Settled::from_result(request, result),
        Terminal::RequestCancelled => {
            finish_gracefully(cancellation, &mut future).await;
            let reason = match control.stop_cause() {
                Some(StopCause::Replaced) => CancellationReason::Replaced,
                _ => CancellationReason::Explicit,
            };
            Settled::cancelled(request, reason)
        }
        Terminal::ScopeClosed => {
            finish_gracefully(cancellation, &mut future).await;
            Settled::cancelled(request, CancellationReason::ScopeClosed)
        }
        Terminal::TimedOut => {
            control.0.stop.cancel();
            finish_gracefully(cancellation, &mut future).await;
            Settled::failed(
                request,
                UserFacingError::custom("request_timeout", "Request timed out"),
            )
        }
    };
    let message = if control.suppress_message() {
        None
    } else {
        Some(settled(terminal))
    };
    RequestEvent { request, message }
}

async fn finish_gracefully<T>(
    cancellation: CancellationMode,
    future: &mut Pin<Box<impl Future<Output = T>>>,
) {
    if let CancellationMode::Graceful(grace) = cancellation {
        tokio::select! {
            _ = future => {}
            _ = tokio::time::sleep(grace) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn globally_minted_ids_are_distinct() {
        assert_ne!(RequestId::mint(), RequestId::mint());
    }

    #[test]
    fn settled_accessors_distinguish_terminals() {
        let id = RequestId::mint();
        let success = Settled::succeeded(id, 7);
        assert_eq!(success.request(), id);
        assert_eq!(success.value(), Some(&7));
        assert!(success.error().is_none());
    }

    #[test]
    fn async_contract_sizes_are_recorded() {
        use std::mem::size_of;

        let sizes = [
            ("RequestId", size_of::<RequestId>()),
            ("ScopeId", size_of::<ScopeId>()),
            ("Request<(), ()>", size_of::<Request<(), ()>>()),
            ("Resource<u64>", size_of::<crate::Resource<u64>>()),
            (
                "Operation<u64, u64>",
                size_of::<crate::Operation<u64, u64>>(),
            ),
        ];

        for (name, bytes) in sizes {
            eprintln!("{name}: {bytes} bytes");
        }
        assert_eq!(size_of::<RequestId>(), 8);
        assert_eq!(size_of::<ScopeId>(), size_of::<usize>());
        assert!(size_of::<Request<(), ()>>() <= 96);
        assert!(size_of::<crate::Resource<u64>>() <= 128);
        assert!(size_of::<crate::Operation<u64, u64>>() <= 128);
    }

    #[tokio::test]
    async fn closed_scope_prevents_the_inner_future_first_poll() {
        let scope = ScopeId::root("closed");
        let token = scope.id().token();
        scope.close();
        let polls = Arc::new(AtomicUsize::new(0));
        let future_polls = Arc::clone(&polls);
        let future = std::future::poll_fn(move |_| {
            future_polls.fetch_add(1, Ordering::SeqCst);
            std::task::Poll::<UserFacingResult<()>>::Pending
        });
        let control = RequestControl::new();
        let request = control.id();

        let event = run_request(
            request,
            control,
            token,
            None,
            CancellationMode::Hard,
            future,
            |settled| settled,
        )
        .await;

        assert_eq!(polls.load(Ordering::SeqCst), 0);
        assert!(matches!(
            event.message,
            Some(Settled::Cancelled {
                reason: CancellationReason::ScopeClosed,
                ..
            })
        ));
    }

    #[tokio::test]
    async fn pre_registration_explicit_cancellation_suppresses_message_and_poll() {
        let scope = ScopeId::root("app");
        let polls = Arc::new(AtomicUsize::new(0));
        let future_polls = Arc::clone(&polls);
        let future = std::future::poll_fn(move |_| {
            future_polls.fetch_add(1, Ordering::SeqCst);
            std::task::Poll::<UserFacingResult<()>>::Pending
        });
        let control = RequestControl::new();
        let request = control.id();
        RequestCancellation::explicit(control.clone()).apply();

        let event = run_request(
            request,
            control,
            scope.id().token(),
            None,
            CancellationMode::Hard,
            future,
            |settled| settled,
        )
        .await;

        assert_eq!(polls.load(Ordering::SeqCst), 0);
        assert!(event.message.is_none());
    }

    #[tokio::test]
    async fn timeout_is_a_failure_and_drops_the_future() {
        struct DropProbe(Arc<AtomicBool>);
        impl Drop for DropProbe {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        let dropped = Arc::new(AtomicBool::new(false));
        let probe = DropProbe(Arc::clone(&dropped));
        let future = async move {
            let _probe = probe;
            std::future::pending::<UserFacingResult<()>>().await
        };
        let scope = ScopeId::root("app");
        let control = RequestControl::new();
        let request = control.id();

        let event = run_request(
            request,
            control,
            scope.id().token(),
            Some(RequestTiming::Timeout(Duration::from_millis(1))),
            CancellationMode::Hard,
            future,
            |settled| settled,
        )
        .await;

        assert!(matches!(event.message, Some(Settled::Failed { .. })));
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[test]
    fn two_resource_group_sketch_type_checks_without_public_group_api() {
        struct GroupMessage {
            generation: u64,
            member: usize,
            settled: Settled<usize>,
        }

        let root = ScopeId::root("app");
        let group = root.child("group");
        let mut first = crate::Resource::<usize>::idle();
        let mut second = crate::Resource::<usize>::idle();
        let first_request = first.request(group.id());
        let second_request = second.request(group.id());
        let generation = 7;
        let tasks = [
            first_request.perform(
                |(), _cancel| async { Ok(1) },
                move |settled| GroupMessage {
                    generation,
                    member: 0,
                    settled,
                },
            ),
            second_request.perform(
                |(), _cancel| async { Ok(2) },
                move |settled| GroupMessage {
                    generation,
                    member: 1,
                    settled,
                },
            ),
        ];
        let partial_results: [Option<usize>; 2] = [None, None];
        let progress = partial_results.iter().flatten().count();

        assert_eq!(tasks.len(), 2);
        assert_eq!(progress, 0);
        let probe = GroupMessage {
            generation,
            member: 0,
            settled: Settled::succeeded(RequestId::mint(), 1),
        };
        assert_eq!((probe.generation, probe.member), (7, 0));
        assert_eq!(probe.settled.value(), Some(&1));
        for task in tasks {
            let _parts = task.into_parts();
        }
    }
}
