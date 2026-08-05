# Contributing to Nive

Thank you for helping improve Nive.

## Before You Start

Small bug fixes, documentation corrections, tests, and focused maintenance can
go directly to a pull request. For new features, public API changes, breaking
changes, or work spanning several crates, open an Issue or Discussion first so
the problem, scope, and compatibility expectations can be agreed before
implementation.

Use the existing architecture and public APIs as the starting point. Prefer a
focused change over introducing a new abstraction without a demonstrated need.

## Development Setup

See the [development guide](docs/development.md) for prerequisites, commands,
testing, examples, and release checks. The
[architecture overview](docs/architecture/README.md) describes the workspace
and crate boundaries.

## Rust Style

- Follow default `rustfmt` formatting and run `just fmt` before submitting
  broad Rust changes.
- Group imports as standard library, external crates, then
  `super::`/`self::`/`crate::`. Merge imports from the same path when practical.
- Use `snake_case` for files, modules, functions, and tests.
- Follow the `<name>.rs` plus `<name>/` module pattern; do not introduce
  `mod.rs` files.
- Split modules that grow beyond roughly 300 to 350 lines and functions that
  grow beyond roughly 150 lines when cohesive helpers or handlers exist.
- Keep trait implementations thin when their method bodies can live in
  inherent implementations in sibling modules.
- Prefer borrowed parameters such as `&str` when ownership is not required.
- Add comments only when a public API needs concise rustdoc or complex logic
  needs orientation that the code cannot provide clearly.

Workspace crates inherit lint levels from the root `Cargo.toml`. Configurable
thresholds live in `clippy.toml`.

## Tests and Documentation

Add or update tests for behavior changes. Scale validation with the risk and
blast radius of the change; documentation-only and mechanical changes do not
need new automated tests. Keep public documentation, examples, and migration
guidance aligned with behavior and API changes.

The [development guide](docs/development.md#testing) describes test placement
and the available validation commands.

## Commits

Use imperative, specific Conventional Commit messages, for example:

- `feat: add logical toast positions`
- `fix(runtime): ignore stale resource results`
- `refactor(ui): centralize popup row state`

## Pull Requests

Pull requests should include:

- a concise problem statement;
- a summary of user-visible changes;
- linked Issues or Discussions when applicable;
- validation commands and results;
- screenshots or recordings for visual UI changes;
- explicit notes for breaking API changes, migrations, or compatibility impact.

