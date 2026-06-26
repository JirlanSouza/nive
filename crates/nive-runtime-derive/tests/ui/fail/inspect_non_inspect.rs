// Fail: a struct field whose type does not implement Inspect and is not skipped
// should produce a compile error.
extern crate self as nive_runtime;

#[path = "../support.rs"]
mod support;

pub use support::{Inspect, InspectPath, InspectSink, Resource, SimulableState};
pub mod __inspect {
    pub use crate::support::{
        Inspect, InspectPath, InspectSink, OperationSimulator, ResourceSimulator,
    };
}

use nive_runtime_derive::Inspect;

struct NotInspect;

#[derive(Inspect)]
struct Wrapper {
    inner: NotInspect,
}

fn main() {}
