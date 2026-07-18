use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use iced::{
    advanced::widget::{operation, Id},
    Rectangle,
};

#[derive(Debug, Default, PartialEq, Eq)]
struct ManagedState {
    visits: usize,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct UnknownState {
    visits: usize,
}

#[derive(Debug, Default)]
struct SharedProbe {
    managed_visits: usize,
}

struct OperationWrapper<T> {
    inner: T,
    shared: Arc<Mutex<SharedProbe>>,
}

impl<T> OperationWrapper<T> {
    fn new(inner: T, shared: Arc<Mutex<SharedProbe>>) -> Self {
        Self { inner, shared }
    }
}

impl<T, Output> operation::Operation<Output> for OperationWrapper<T>
where
    T: operation::Operation<Output> + 'static,
    Output: Send + 'static,
{
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation<Output>)) {
        self.inner.traverse(&mut |inner| {
            operate(&mut OperationRef {
                inner,
                shared: Arc::clone(&self.shared),
            });
        });
    }

    fn custom(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn Any) {
        if let Some(managed) = state.downcast_mut::<ManagedState>() {
            managed.visits += 1;
            self.shared
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .managed_visits += 1;
        } else {
            self.inner.custom(id, bounds, state);
        }
    }

    fn finish(&self) -> operation::Outcome<Output> {
        match self.inner.finish() {
            operation::Outcome::None => operation::Outcome::None,
            operation::Outcome::Some(output) => operation::Outcome::Some(output),
            operation::Outcome::Chain(next) => operation::Outcome::Chain(Box::new(
                OperationWrapper::new(next, Arc::clone(&self.shared)),
            )),
        }
    }
}

struct OperationRef<'a, Output> {
    inner: &'a mut dyn operation::Operation<Output>,
    shared: Arc<Mutex<SharedProbe>>,
}

impl<Output> operation::Operation<Output> for OperationRef<'_, Output>
where
    Output: Send + 'static,
{
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation<Output>)) {
        self.inner.traverse(&mut |inner| {
            operate(&mut OperationRef {
                inner,
                shared: Arc::clone(&self.shared),
            });
        });
    }

    fn custom(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn Any) {
        if let Some(managed) = state.downcast_mut::<ManagedState>() {
            managed.visits += 1;
            self.shared
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .managed_visits += 1;
        } else {
            self.inner.custom(id, bounds, state);
        }
    }
}

struct PhaseOperation {
    phase: usize,
    last_phase: usize,
}

impl operation::Operation<usize> for PhaseOperation {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation<usize>)) {
        operate(self);
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Any) {
        if let Some(unknown) = state.downcast_mut::<UnknownState>() {
            unknown.visits += 1;
        }
    }

    fn finish(&self) -> operation::Outcome<usize> {
        if self.phase == self.last_phase {
            operation::Outcome::Some(self.phase)
        } else {
            operation::Outcome::Chain(Box::new(Self {
                phase: self.phase + 1,
                last_phase: self.last_phase,
            }))
        }
    }
}

fn assert_send<T: Send>() {}

#[test]
fn wrapper_is_send_and_wraps_traversal_and_every_chained_phase() {
    assert_send::<OperationWrapper<PhaseOperation>>();

    let shared = Arc::new(Mutex::new(SharedProbe::default()));
    let mut operation: Box<dyn operation::Operation<usize>> = Box::new(OperationWrapper::new(
        PhaseOperation {
            phase: 1,
            last_phase: 3,
        },
        Arc::clone(&shared),
    ));
    let mut managed = ManagedState::default();
    let mut unknown = UnknownState::default();

    for expected_phase in 1..=3 {
        operation.traverse(&mut |phase| {
            phase.custom(None, Rectangle::default(), &mut managed);
            phase.custom(None, Rectangle::default(), &mut unknown);
        });

        match operation.finish() {
            operation::Outcome::Chain(next) => {
                assert!(expected_phase < 3);
                operation = next;
            }
            operation::Outcome::Some(phase) => {
                assert_eq!(expected_phase, 3);
                assert_eq!(phase, 3);
            }
            operation::Outcome::None => panic!("phase operation unexpectedly returned no outcome"),
        }
    }

    assert_eq!(managed.visits, 3);
    assert_eq!(unknown.visits, 3);
    assert_eq!(
        shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .managed_visits,
        3
    );
}
