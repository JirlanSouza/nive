use crate::devtools::{
    SimulateAction, SimulateResult, SimulatorCapabilities, SimulatorKind, DEFAULT_CAPABILITY_HINT,
    INPUT_CAPABILITY_HINT, REFRESH_CAPABILITY_HINT, SAMPLE_CAPABILITY_HINT,
};
use crate::feedback::UserFacingError;
use crate::state::{Operation, OperationRegistry, Resource, Settled};

// ---------------------------------------------------------------------------
// InspectPath
// ---------------------------------------------------------------------------

/// Tracks the dotted path of field names from the root state struct to the
/// current traversal position. Passed to every [`Inspect::inspect`] call.
pub struct InspectPath {
    segments: Vec<String>,
}

impl InspectPath {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn push(&mut self, segment: &str) {
        self.segments.push(segment.to_owned());
    }

    pub fn pop(&mut self) {
        self.segments.pop();
    }

    /// The current dotted path (e.g. `"settings.theme"`).
    pub fn as_str(&self) -> String {
        self.segments.join(".")
    }
}

impl Default for InspectPath {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// SimulableSnapshot
// ---------------------------------------------------------------------------

/// A type-erased snapshot of the current state for devtools display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimulableSnapshot {
    Idle,
    Loading { has_value: bool },
    Loaded,
    Failed { has_value: bool, summary: String },
    Running,
    OperationFailed { summary: String },
}

// ---------------------------------------------------------------------------
// SimulableState
// ---------------------------------------------------------------------------

/// Type-erased interface for forcing a `Resource` or `Operation` into a
/// specific state from the devtools simulator.
pub trait SimulableState {
    fn kind(&self) -> SimulatorKind;
    fn capabilities(&self) -> SimulatorCapabilities;
    fn snapshot(&self) -> SimulableSnapshot;
    fn apply(&mut self, action: &SimulateAction) -> SimulateResult;
}

// ---------------------------------------------------------------------------
// InspectSink
// ---------------------------------------------------------------------------

/// Receives [`SimulableState`] leaves discovered during an [`Inspect`]
/// traversal. The devtools panel holds an `impl InspectSink` that collects
/// all reachable `Resource`/`Operation` fields with their paths.
pub trait InspectSink {
    fn register(&mut self, path: &str, state: &mut dyn SimulableState);
    fn register_registry(&mut self, _path: &str, _registry: &OperationRegistry) {}
}

// ---------------------------------------------------------------------------
// Inspect trait
// ---------------------------------------------------------------------------

/// Recursive state-tree traversal for devtools inspection.
///
/// Derived via `#[derive(Inspect)]` for structs. `Resource<T>` and
/// `Operation<C>` are leaves: their `inspect` impl registers simulator
/// adapters with [`InspectSink::register`]. All other types recurse into
/// their fields.
pub trait Inspect {
    fn inspect(&mut self, path: &mut InspectPath, sink: &mut dyn InspectSink);
}

// ---------------------------------------------------------------------------
// Std type no-op impls (task 7.2)
// ---------------------------------------------------------------------------

macro_rules! noop_inspect {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl Inspect for $ty {
                fn inspect(&mut self, _: &mut InspectPath, _: &mut dyn InspectSink) {}
            }
        )+
    };
}

noop_inspect!(
    (),
    bool,
    char,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    f32,
    f64,
    String,
);

impl Inspect for &str {
    fn inspect(&mut self, _: &mut InspectPath, _: &mut dyn InspectSink) {}
}

// ---------------------------------------------------------------------------
// Recursive std impls (task 7.3)
// ---------------------------------------------------------------------------

impl<T: Inspect> Inspect for Vec<T> {
    fn inspect(&mut self, path: &mut InspectPath, sink: &mut dyn InspectSink) {
        for (i, item) in self.iter_mut().enumerate() {
            let idx = i.to_string();
            path.push(&idx);
            item.inspect(path, sink);
            path.pop();
        }
    }
}

impl<T: Inspect> Inspect for Option<T> {
    fn inspect(&mut self, path: &mut InspectPath, sink: &mut dyn InspectSink) {
        if let Some(inner) = self.as_mut() {
            inner.inspect(path, sink);
        }
    }
}

impl<T: Inspect> Inspect for Box<T> {
    fn inspect(&mut self, path: &mut InspectPath, sink: &mut dyn InspectSink) {
        (**self).inspect(path, sink);
    }
}

// ---------------------------------------------------------------------------
// Resource<T> — leaf Inspect + explicit simulator adapter
// ---------------------------------------------------------------------------

impl<T> Inspect for Resource<T> {
    fn inspect(&mut self, path: &mut InspectPath, sink: &mut dyn InspectSink) {
        let mut simulator = ResourceSimulator::new(self);
        sink.register(&path.as_str(), &mut simulator);
    }
}

#[doc(hidden)]
pub struct ResourceSimulator<'a, T> {
    state: &'a mut Resource<T>,
    default: Option<fn() -> T>,
    sample: Option<fn() -> T>,
}

impl<'a, T> ResourceSimulator<'a, T> {
    pub fn new(state: &'a mut Resource<T>) -> Self {
        Self {
            state,
            default: None,
            sample: None,
        }
    }

    pub fn with_sample(mut self, sample: fn() -> T) -> Self {
        self.sample = Some(sample);
        self
    }

    fn load_value(&mut self, value: T) {
        let token = self.state.begin();
        self.state.settle(Settled::new(token, Ok(value)));
    }

    fn force_error(&mut self, message: &str) {
        let token = self.state.begin();
        self.state
            .settle(Settled::new(token, Err(UserFacingError::devtools(message))));
    }
}

impl<'a, T: Default> ResourceSimulator<'a, T> {
    pub fn with_default(mut self) -> Self {
        self.default = Some(T::default);
        self
    }
}

impl<T> SimulableState for ResourceSimulator<'_, T> {
    fn kind(&self) -> SimulatorKind {
        SimulatorKind::Resource
    }

    fn capabilities(&self) -> SimulatorCapabilities {
        SimulatorCapabilities::resource(self.default.is_some(), self.sample.is_some())
    }

    fn snapshot(&self) -> SimulableSnapshot {
        if let Some(error) = self.state.error() {
            return SimulableSnapshot::Failed {
                has_value: self.state.value().is_some(),
                summary: error.summary().to_string(),
            };
        }
        if self.state.is_loading() {
            return SimulableSnapshot::Loading {
                has_value: self.state.value().is_some(),
            };
        }
        if self.state.value().is_some() {
            SimulableSnapshot::Loaded
        } else {
            SimulableSnapshot::Idle
        }
    }

    fn apply(&mut self, action: &SimulateAction) -> SimulateResult {
        match action {
            SimulateAction::Idle => {
                self.state.reset();
                SimulateResult::applied()
            }
            SimulateAction::Loading => {
                self.state.reset();
                self.state.begin();
                SimulateResult::applied()
            }
            SimulateAction::Refreshing => {
                if self.state.value().is_none() {
                    return SimulateResult::unsupported(REFRESH_CAPABILITY_HINT);
                }
                self.state.begin();
                SimulateResult::applied()
            }
            SimulateAction::Error { message } => {
                self.force_error(message);
                SimulateResult::applied()
            }
            SimulateAction::Default => match self.default {
                Some(factory) => {
                    self.load_value(factory());
                    SimulateResult::applied()
                }
                None => SimulateResult::unsupported(DEFAULT_CAPABILITY_HINT),
            },
            SimulateAction::Sample => match self.sample {
                Some(factory) => {
                    self.load_value(factory());
                    SimulateResult::applied()
                }
                None => SimulateResult::unsupported(SAMPLE_CAPABILITY_HINT),
            },
            SimulateAction::DismissError => {
                self.state.dismiss_error();
                SimulateResult::applied()
            }
            SimulateAction::Start => {
                SimulateResult::unsupported("Start is only supported for Operation fields.")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Operation<C> — leaf Inspect + explicit simulator adapter
// ---------------------------------------------------------------------------

impl<C> Inspect for Operation<C> {
    fn inspect(&mut self, path: &mut InspectPath, sink: &mut dyn InspectSink) {
        let mut simulator = OperationSimulator::new(self);
        sink.register(&path.as_str(), &mut simulator);
    }
}

#[doc(hidden)]
pub struct OperationSimulator<'a, C> {
    state: &'a mut Operation<C>,
    input: Option<fn() -> C>,
}

impl<'a, C> OperationSimulator<'a, C> {
    pub fn new(state: &'a mut Operation<C>) -> Self {
        Self { state, input: None }
    }

    pub fn with_input(mut self, input: fn() -> C) -> Self {
        self.input = Some(input);
        self
    }

    fn start(&mut self) -> SimulateResult {
        match self.input {
            Some(factory) => {
                self.state.begin(factory());
                SimulateResult::applied()
            }
            None => SimulateResult::unsupported(INPUT_CAPABILITY_HINT),
        }
    }

    fn fail(&mut self, message: &str) -> SimulateResult {
        match self.input {
            Some(factory) => {
                let token = self.state.begin(factory());
                self.state
                    .settle(Settled::new(token, Err(UserFacingError::devtools(message))));
                SimulateResult::applied()
            }
            None => SimulateResult::unsupported(INPUT_CAPABILITY_HINT),
        }
    }
}

impl<C> SimulableState for OperationSimulator<'_, C> {
    fn kind(&self) -> SimulatorKind {
        SimulatorKind::Operation
    }

    fn capabilities(&self) -> SimulatorCapabilities {
        SimulatorCapabilities::operation(self.input.is_some())
    }

    fn snapshot(&self) -> SimulableSnapshot {
        if let Some(error) = self.state.error() {
            SimulableSnapshot::OperationFailed {
                summary: error.summary().to_string(),
            }
        } else if self.state.is_running() {
            SimulableSnapshot::Running
        } else {
            SimulableSnapshot::Idle
        }
    }

    fn apply(&mut self, action: &SimulateAction) -> SimulateResult {
        match action {
            SimulateAction::Idle => {
                self.state.reset();
                SimulateResult::applied()
            }
            SimulateAction::Start => self.start(),
            SimulateAction::Error { message } => self.fail(message),
            SimulateAction::DismissError => {
                self.state.dismiss_error();
                SimulateResult::applied()
            }
            SimulateAction::Loading => self.start(),
            SimulateAction::Refreshing => SimulateResult::unsupported(
                "Refresh is only supported for Resource fields with a retained value.",
            ),
            SimulateAction::Default => {
                SimulateResult::unsupported("Default is only supported for Resource fields.")
            }
            SimulateAction::Sample => {
                SimulateResult::unsupported("Sample is only supported for Resource fields.")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// OperationRegistry — read-only Inspect leaf (task 7.5)
// ---------------------------------------------------------------------------

// OperationRegistry entries are shown in the devtools panel as a read-only
// section, not force-simulated.
impl Inspect for OperationRegistry {
    fn inspect(&mut self, path: &mut InspectPath, sink: &mut dyn InspectSink) {
        sink.register_registry(&path.as_str(), self);
    }
}

// ---------------------------------------------------------------------------
// Tests (task 8.4)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_loading_needs_no_value() {
        let mut r: Resource<String> = Resource::idle();
        let mut simulator = ResourceSimulator::new(&mut r);

        assert_eq!(
            simulator.apply(&SimulateAction::Loading),
            SimulateResult::applied()
        );

        assert!(r.is_loading());
        assert!(!r.is_refreshing());
    }

    #[test]
    fn resource_refreshing_retains_value() {
        let mut r: Resource<String> = Resource::ready("hello".to_string());
        let mut simulator = ResourceSimulator::new(&mut r);

        assert_eq!(
            simulator.apply(&SimulateAction::Refreshing),
            SimulateResult::applied()
        );

        assert!(r.is_loading());
        assert!(r.is_refreshing());
        assert_eq!(r.value(), Some(&"hello".to_string()));
    }

    #[test]
    fn resource_refreshing_without_value_is_unsupported() {
        let mut r: Resource<String> = Resource::idle();
        let mut simulator = ResourceSimulator::new(&mut r);

        assert_eq!(
            simulator.apply(&SimulateAction::Refreshing),
            SimulateResult::unsupported(REFRESH_CAPABILITY_HINT)
        );

        assert!(r.is_idle());
    }

    #[test]
    fn resource_error_applies() {
        let mut r: Resource<String> = Resource::idle();
        let mut simulator = ResourceSimulator::new(&mut r);

        assert_eq!(
            simulator.apply(&SimulateAction::Error {
                message: "test failure".to_string(),
            }),
            SimulateResult::applied()
        );

        assert!(r.error().is_some());
    }

    #[test]
    fn resource_default_without_capability_is_unsupported() {
        let mut r: Resource<Vec<String>> = Resource::idle();
        let mut simulator = ResourceSimulator::new(&mut r);

        assert_eq!(
            simulator.apply(&SimulateAction::Default),
            SimulateResult::unsupported(DEFAULT_CAPABILITY_HINT)
        );

        assert!(r.value().is_none());
        assert!(r.is_idle());
    }

    #[test]
    fn resource_default_capability_applies() {
        let mut r: Resource<Vec<String>> = Resource::idle();
        let mut simulator = ResourceSimulator::new(&mut r).with_default();

        assert_eq!(
            simulator.apply(&SimulateAction::Default),
            SimulateResult::applied()
        );

        assert_eq!(r.value(), Some(&Vec::<String>::default()));
    }

    #[test]
    fn resource_sample_capability_applies() {
        fn sample() -> Vec<String> {
            vec!["sample".to_string()]
        }

        let mut r: Resource<Vec<String>> = Resource::idle();
        let mut simulator = ResourceSimulator::new(&mut r).with_sample(sample);

        assert_eq!(
            simulator.apply(&SimulateAction::Sample),
            SimulateResult::applied()
        );

        assert_eq!(r.value(), Some(&vec!["sample".to_string()]));
    }

    #[test]
    fn resource_idle_applies() {
        let mut r: Resource<String> = Resource::ready("x".to_string());
        let mut simulator = ResourceSimulator::new(&mut r);

        assert_eq!(
            simulator.apply(&SimulateAction::Idle),
            SimulateResult::applied()
        );

        assert!(r.is_idle());
        assert!(r.value().is_none());
    }

    #[test]
    fn operation_start_without_input_is_unsupported() {
        let mut op: Operation<String> = Operation::idle();
        let mut simulator = OperationSimulator::new(&mut op);

        assert_eq!(
            simulator.apply(&SimulateAction::Start),
            SimulateResult::unsupported(INPUT_CAPABILITY_HINT)
        );

        assert!(op.is_idle());
    }

    #[test]
    fn operation_input_capability_starts_with_sample_input() {
        fn input() -> String {
            "sample input".to_string()
        }

        let mut op: Operation<String> = Operation::idle();
        let mut simulator = OperationSimulator::new(&mut op).with_input(input);

        assert_eq!(
            simulator.apply(&SimulateAction::Start),
            SimulateResult::applied()
        );

        assert!(op.is_running());
        assert_eq!(op.input(), Some(&"sample input".to_string()));
    }

    #[test]
    fn operation_input_capability_fails_with_sample_input() {
        fn input() -> String {
            "sample input".to_string()
        }

        let mut op: Operation<String> = Operation::idle();
        let mut simulator = OperationSimulator::new(&mut op).with_input(input);

        assert_eq!(
            simulator.apply(&SimulateAction::Error {
                message: "failed".to_string(),
            }),
            SimulateResult::applied()
        );

        assert!(op.error().is_some());
        assert_eq!(op.input(), Some(&"sample input".to_string()));
    }

    #[test]
    fn operation_idle_applies() {
        let mut op: Operation<String> = Operation::idle();
        op.begin("action".to_string());
        let mut simulator = OperationSimulator::new(&mut op);

        assert_eq!(
            simulator.apply(&SimulateAction::Idle),
            SimulateResult::applied()
        );

        assert!(op.is_idle());
    }

    #[test]
    fn operation_registry_is_collected_through_inspect() {
        use crate::state::{OperationDescriptor, OperationStatus};

        struct Sink {
            registry: Vec<String>,
        }

        impl InspectSink for Sink {
            fn register(&mut self, _: &str, _: &mut dyn SimulableState) {}

            fn register_registry(&mut self, _: &str, registry: &OperationRegistry) {
                self.registry.extend(
                    registry
                        .iter()
                        .filter(|(_, entry)| matches!(entry.status, OperationStatus::Running))
                        .map(|(id, _)| id.to_string()),
                );
            }
        }

        let mut registry = OperationRegistry::new();
        registry.register(OperationDescriptor::new("sync", "Sync"));
        let mut sink = Sink { registry: vec![] };
        let mut path = InspectPath::new();

        registry.inspect(&mut path, &mut sink);

        assert_eq!(sink.registry, vec!["sync".to_string()]);
    }
}
