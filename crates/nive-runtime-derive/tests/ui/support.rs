#![allow(dead_code)]

use std::marker::PhantomData;

pub trait SimulableState {}

pub struct InspectPath {
    segments: Vec<String>,
}

impl InspectPath {
    pub fn new() -> Self {
        Self { segments: Vec::new() }
    }

    pub fn push(&mut self, segment: &str) {
        self.segments.push(segment.to_owned());
    }

    pub fn pop(&mut self) {
        self.segments.pop();
    }

    pub fn as_str(&self) -> String {
        self.segments.join(".")
    }
}

pub trait InspectSink {
    fn register(&mut self, path: &str, state: &mut dyn SimulableState);
}

pub trait Inspect {
    fn inspect(&mut self, path: &mut InspectPath, sink: &mut dyn InspectSink);
}

pub struct Resource<T>(pub PhantomData<T>);

impl<T> SimulableState for Resource<T> {}

impl<T> Inspect for Resource<T> {
    fn inspect(&mut self, path: &mut InspectPath, sink: &mut dyn InspectSink) {
        sink.register(&path.as_str(), self);
    }
}

pub struct ResourceSimulator<'a, T>(&'a mut Resource<T>);

impl<'a, T> ResourceSimulator<'a, T> {
    pub fn new(resource: &'a mut Resource<T>) -> Self {
        Self(resource)
    }

    pub fn with_sample(self, _sample: fn() -> T) -> Self {
        self
    }
}

impl<'a, T: Default> ResourceSimulator<'a, T> {
    pub fn with_default(self) -> Self {
        self
    }
}

impl<T> SimulableState for ResourceSimulator<'_, T> {}

pub struct Operation<C>(pub PhantomData<C>);

impl<C> SimulableState for Operation<C> {}

impl<C> Inspect for Operation<C> {
    fn inspect(&mut self, path: &mut InspectPath, sink: &mut dyn InspectSink) {
        sink.register(&path.as_str(), self);
    }
}

pub struct OperationSimulator<'a, C>(&'a mut Operation<C>);

impl<'a, C> OperationSimulator<'a, C> {
    pub fn new(operation: &'a mut Operation<C>) -> Self {
        Self(operation)
    }

    pub fn with_input(self, _input: fn() -> C) -> Self {
        self
    }
}

impl<C> SimulableState for OperationSimulator<'_, C> {}

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

macro_rules! noop_inspect {
    ($($t:ty),*) => {
        $(
            impl Inspect for $t {
                fn inspect(&mut self, _path: &mut InspectPath, _sink: &mut dyn InspectSink) {}
            }
        )*
    };
}

noop_inspect!(bool, i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, String, usize, isize);
