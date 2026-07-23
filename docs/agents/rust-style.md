# Rust Style Guidelines

## Formatting

- Run `just fmt` before opening a PR or finishing broad Rust edits.
- Follow default Rust formatting.
- Keep imports grouped as `std`, external crates, `super::`/`self::`/`crate::`.
- Within each import group, merge imports from the same crate path when practical, for example `use std::{borrow::Cow, time::Duration};`.

## Naming And Modules

- Use `snake_case` for files, modules, functions, and tests.
- Follow the `<name>.rs` plus `<name>/` module pattern; workspace Clippy lints reject `mod.rs`.
- Divide modules into submodules when they become too large over 300 to 350 lines.
- Divide functions that grow past roughly 150 lines into smaller handlers or helpers; keep exceptions rare and justify them in review.
- Trait impls cannot span files; keep the trait impl a thin dispatcher and move method bodies into inherent impls in sibling submodules. Prefer inherent methods; fall back to free functions only when borrow splitting demands it.
- Keep Rust tests close to the module they cover.

## Automated Lints

- Workspace crates inherit `[workspace.lints]` from the root `Cargo.toml`.
- `clippy.toml` stores thresholds for configurable lints only; lint levels live in `Cargo.toml`.
- Clippy enforces self-named module files and ordinary wildcard imports. Prelude re-exports and `super::*` in tests remain allowed.
- Module length, function length, import grouping order, and test placement are review rules rather than Clippy-enforced rules.

## Ownership

- Service methods should prefer references such as `&str` instead of owned `String` values for IDs when ownership is not required.
- Command or UI boundary layers should act as data owners and pass references down into services.

## Comments

Do not add comments unless a public Rust API truly needs a short doc comment or a complex block benefits from a concise orientation note.
