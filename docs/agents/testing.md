# Testing Guidelines

## Commands

- `just fmt`: format active Rust crates.
- `just fmt-check`: check formatting for active Rust crates.
- `just check`: run `cargo check --workspace --all-targets --all-features`.
- `just lint`: run clippy with all targets and all features, treating warnings as errors.
- `just test`: run workspace tests with all features.
- `just doc-check`: build workspace docs with all features and warnings denied.
- `just examples-check`: run `cargo check` for every standalone example under `examples/*/Cargo.toml`.
- `just scaffold-smoke`: scaffold basic and dashboard apps outside the workspace with a temporary local `nive` patch and run `cargo check`.
- `just package-check`: run package-readiness checks for publishable crates in dependency order.
- `just readiness`: run the local CI-like readiness suite.

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

- icon manifest, source generation, path planning, and scaffold changes need offline tests or smoke coverage
- runtime, widget, and public API changes should be covered by focused unit or compile-contract tests
- tiny mechanical edits or documentation-only changes do not require automated tests
