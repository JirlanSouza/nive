extern crate self as nive_runtime;

#[path = "../support.rs"]
mod support;

pub mod devtools {
    pub mod probe {
        pub use crate::support::{ProbeCatalogEntry, ProbeErrorScope, ProbeMeta};
    }
}

use nive_runtime_derive::UiErrorProbeCatalog;

#[derive(UiErrorProbeCatalog)]
enum Probe {
    Load(String),
}

fn main() {}
