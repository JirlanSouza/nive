// Pass: a field annotated #[inspect(skip)] is excluded and its type need not
// implement Inspect.
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
struct AppState {
    data: Resource<String>,
    #[inspect(skip)]
    cache: NotInspect,
}

fn main() {
    fn _assert<T: Inspect>() {}
    _assert::<AppState>();
}
