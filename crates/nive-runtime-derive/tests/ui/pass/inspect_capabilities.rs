// Pass: explicit simulator capability attributes compile on Resource and Operation fields.
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

#[derive(Debug)]
struct Project;

#[derive(Debug)]
struct SaveInput;

fn sample_projects() -> Vec<Project> {
    vec![Project]
}

fn sample_input() -> SaveInput {
    SaveInput
}

#[derive(Inspect)]
struct AppState {
    #[inspect(default)]
    empty_projects: Resource<Vec<Project>>,
    #[inspect(sample = sample_projects)]
    sample_projects: Resource<Vec<Project>>,
    #[inspect(default, sample = sample_projects)]
    both_projects: Resource<Vec<Project>>,
    #[inspect(input = sample_input)]
    save: Operation<SaveInput>,
}

fn main() {
    fn _assert<T: Inspect>() {}
    _assert::<AppState>();
}
