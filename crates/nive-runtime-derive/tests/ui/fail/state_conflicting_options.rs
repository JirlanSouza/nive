extern crate self as nive_runtime;

#[path = "../support.rs"]
mod support;

pub mod devtools {
    pub use crate::support::*;
}

use nive_runtime_derive::DevtoolStateCatalog;
use support::{AsyncState, DevtoolFixture};

fn fixtures(_path: &str) -> Vec<DevtoolFixture<String>> {
    Vec::new()
}

#[derive(DevtoolStateCatalog)]
struct State {
    #[devtool(nested, fixtures = fixtures)]
    resource: AsyncState<String>,
}

fn main() {}
