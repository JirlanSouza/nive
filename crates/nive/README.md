# Nive

Nive is the umbrella crate for the Rust/Iced Nive framework. It re-exports the
runtime and UI crates so application code can depend on `nive` as its framework
boundary.

Use `nive::prelude::*` for the minimal scaffolded app surface and
`nive::prelude::ui::*` for the extended app surface.
