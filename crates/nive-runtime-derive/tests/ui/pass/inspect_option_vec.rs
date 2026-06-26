// Pass: Option<T> and Vec<T> fields where T: Inspect compile without extra bounds.
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

#[derive(Inspect)]
struct Panel {
    title: Resource<String>,
}

#[derive(Inspect)]
struct Dashboard {
    panels: Vec<Panel>,
    active: Option<Panel>,
}

fn main() {
    fn _assert<T: Inspect>() {}
    _assert::<Dashboard>();
}
