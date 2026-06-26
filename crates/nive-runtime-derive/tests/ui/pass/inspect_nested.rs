// Pass: nested structs both deriving Inspect are traversed without a marker attribute.
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
struct SettingsPanel {
    theme: Resource<String>,
}

#[derive(Inspect)]
struct AppState<T>
where
    T: Inspect,
{
    settings: SettingsPanel,
    items: Resource<Vec<String>>,
    extra: T,
}

fn main() {
    fn _assert<S: Inspect>() {}
    _assert::<AppState<Resource<String>>>();
}
