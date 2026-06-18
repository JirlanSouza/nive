extern crate self as nive_runtime;

#[path = "../support.rs"]
mod support;

pub mod devtools {
    pub use crate::support::*;
}

use nive_runtime_derive::DevtoolStateCatalog;
use support::AsyncState;

#[derive(DevtoolStateCatalog)]
struct State {
    resource: AsyncState<String>,
}

fn main() {}
