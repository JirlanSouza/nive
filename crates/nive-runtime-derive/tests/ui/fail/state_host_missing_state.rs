extern crate self as nive_runtime;

#[path = "../support.rs"]
mod support;

pub mod devtools {
    pub use crate::support::*;
}

use nive_runtime_derive::DevtoolStateHost;

#[derive(DevtoolStateHost)]
struct Host {
    value: String,
}

fn main() {}
