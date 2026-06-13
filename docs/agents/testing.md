# Testing Guidelines

## Commands

- `just fmt`: format active Rust crates.
- `just fmt-check`: check formatting for active Rust crates.
- `just check`: check active Rust crates.
- `just lint`: run clippy for active Rust crates.
- `just rust-test`: run tests for active Rust crates.
- `just test`: run active Rust tests plus parser tests.
- `just app-gui-check-dev`: check `app-gui` with the `dev` feature enabled for Devtools/probe changes.
- `just app-gui-test-dev`: test `app-gui` with the `dev` feature enabled for Devtools/probe changes.

## Rust Tests

Keep tests close to the module they cover, following the existing `*_tests.rs` pattern.

Use descriptive `snake_case` test names without redundant prefixes. The `#[test]` attribute and module path already provide context, so prefer `rejects_incompatible_format` over names like `test_payload_reader_rejects_incompatible_format`.

When a test module grows large, split it into cohesive submodules:

- a `*_tests.rs` entry point declaring submodules
- `support.rs` for shared helpers, fixtures, builders, and assertions
- one file per logical group of tests

Follow the `<name>.rs` plus `<name>/` module convention. Do not use `mod.rs`. Shared helpers should use `pub(super)` visibility.

## Coverage

Add or update tests with every behavior change. Scale test coverage with risk and blast radius:

- parsing, chunking, indexing, and persistence changes need focused tests
- reducer/state/action behavior should be covered when user-visible flows change
- tiny mechanical edits or documentation-only changes do not require automated tests
