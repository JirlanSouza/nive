// Pass: flat struct with Resource and Operation fields derives Inspect.
extern crate self as nive_runtime;

#[path = "../support.rs"]
mod support;

pub use support::{Inspect, InspectPath, InspectSink, Operation, Resource, SimulableState};
pub mod __inspect {
    pub use crate::support::{
        Inspect, InspectPath, InspectSink, OperationSimulator, ResourceSimulator,
    };
}

use nive_runtime_derive::Inspect;

#[derive(Inspect)]
struct AppState {
    users: Resource<Vec<String>>,
    save: Operation<String>,
    count: i32,
}

fn main() {
    fn _assert<T: Inspect>() {}
    _assert::<AppState>();
}
