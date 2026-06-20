extern crate self as nive_runtime;

#[path = "../support.rs"]
mod support;

pub mod devtools {
    pub use crate::support::*;

    pub mod probe {
        pub use crate::support::{ProbeCatalogEntry, ProbeErrorScope, ProbeMeta};
    }
}

use std::marker::PhantomData;

use nive_runtime_derive::{
    DevtoolOperationContext, DevtoolStateCatalog, DevtoolStateHost, UiErrorProbeCatalog,
};
use support::{AsyncState, DevtoolFixture, OperationState};

#[derive(DevtoolOperationContext)]
struct OperationContext {
    project_id: String,
}

#[derive(DevtoolStateCatalog)]
struct ChildState {
    operation: OperationState<OperationContext>,
}

fn fixtures<T>(_path: &str) -> Vec<DevtoolFixture<T>> {
    Vec::new()
}

#[derive(DevtoolStateCatalog)]
struct AppState<T>
where
    T: Clone,
{
    #[devtool(fixtures = fixtures::<T>)]
    resource: AsyncState<T>,
    #[devtool(nested)]
    child: ChildState,
    ignored: PhantomData<T>,
}

#[derive(DevtoolStateHost)]
struct Host<T>
where
    T: Clone,
{
    state: AppState<T>,
}

#[derive(Clone, Copy, UiErrorProbeCatalog)]
enum Probe {
    Bootstrap,
    LoadProjects,
}

fn main() {}
