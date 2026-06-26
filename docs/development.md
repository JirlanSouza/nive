# Nive Development Guide

This guide is for contributors working on the Nive framework itself.

## Requirements

- Rust 1.92 or later
- Cargo
- just

## Setup

Clone the repository:

```bash
git clone https://github.com/nive-rs/nive.git
cd nive
```

## Development Commands

```bash
# Format code
just fmt

# Check formatting
just fmt-check

# Check all crates
just check

# Lint all crates
just lint

# Run all tests
just test

# Build documentation
just doc

# Build all crates
just build

# Build release
just release
```

## Icon Management

The framework maintains a set of essential icons in `nive-ui`. For external
apps, use `nive icons` (see [adding-icons.md](guides/adding-icons.md)).

```bash
# List framework icons
just icons-list

# Sync framework icons
just icons-sync

# Check icons are up to date
just icons-check

# Add icon to framework
just icons-add <Variant> <lucide-name>
```

## Creating Test Apps

```bash
# Create a new app using the framework
nive new test-app
```

## Architecture


## Migration (v0.1.0 contract changes)

The v0.1 "API ergonomics" change rewrote several public-contract surfaces.
Apps written against the pre-v0.1 snapshot need the updates below. New apps
scaffolded via `nive new my-app` already use the new defaults.

### `Application` trait

- `type Window` and `type Bootstrap` are no longer trait-defaulted (that
  feature requires nightly `associated_type_defaults`). Apps set
  `type Window = ();` and `type Bootstrap = ();` explicitly to opt into the
  `SimpleApplication` marker; the runtime auto-registers one
  `WindowSpec::app()` when the config has zero `.window(...)` calls and the
  app's `Window = ()`.
- `Application::init` and `Application::update` now return
  `impl Into<AppUpdate<...>>`, so `()` is a valid return ("no side effects").
  `impl From<()> for AppUpdate<M, K>` produces `AppUpdate::none()`.
- `Application::window_title` returns `impl Into<Cow<'a, str>> + 'a`, so
  apps no longer need to `use std::borrow::Cow;` to return a `&'static str`.
- `Application::theme(ctx, Option<WindowContext>) -> ThemePreference`
  (default `ThemePreference::System`) lets the app influence the global
  theme singleton. The runtime consults it on startup, on
  `AppUpdate::theme(pref)` emissions, and on OS theme changes; the
  method's return value wins over the emitted preference except when it
  returns `System` (then the OS / persisted preference wins).

### `Toast` ≠ `ToastRequest`

`ToastRequest` is renamed `Toast` consistently across `nive-runtime`,
`nive-ui`, preludes, and docs. A deprecated `pub use Toast as ToastRequest;`
alias remains for one release cycle (v0.1) and is removed in v0.2.
Existing apps should rename `ToastRequest` references to `Toast`.

### `OperationId` becomes `Cow<'static, str>`

`OperationId(pub &'static str)` is now `OperationId(Cow<'static, str>)` with
two constructors: `OperationId::from_static("...")` (zero-cost) and
`OperationId::from_owned(format!(...))` (allocating). Apps that constructed
`OperationId::new("...")` need no changes — `new` is a `const` alias for
`from_static`. Apps that declared `OperationId` `Copy` and `*id`-derefs must
switch to `.clone()` (Cow is not `Copy`).

### App-owned `OperationRegistry`

`OperationRegistry` is app-owned state. Apps hold it as a field, drive it from
`update`, and render it in product UI as needed. Devtools discover the same
field read-only through `#[derive(Inspect)]`; there is no runtime-managed mirror
and no `AppUpdate::op_*` API.

### `Resource` stale-request guarding

`Resource<T>` owns its request counter. `begin()` transitions to loading and
returns a `RequestId`; `settle(Settled<T>)` applies only when the carried token
matches the most recent `begin`. The blessed `load` helper hides the token in
the message value, while manual code can still construct `Settled::new(token,
result)`.

### `ErrorCode::new` returns `Result`

`ErrorCode::new(...)` now returns `Result<ErrorCode, InvalidErrorCode>`
instead of silently downgrading invalid codes to `"application"`. Construct
codes via `ErrorCode::new("valid-code")?` or the result-handling helper.
Apps that relied on the silent downgrade should switch to explicit validation.

### `file-picker` feature alignment

`FileFilter`, `PickFileParams`, and `SaveFileParams` are now
`#[cfg(feature = "file-picker")]`-gated — turning the feature off removes
the param structs (no "orphan types"). Apps that import these structs
must set the `file-picker` feature on the `nive` or `nive-runtime` crate
(it is forwarded via `nive`'s `file-picker` feature flag).

### Two-tier prelude

`nive::prelude::*` is the minimal template-stable surface. Apps that use
toasts, async state, dialogs, file picker params, theming, shortcuts, or
window-handle types switch to `nive::prelude::ui::*`. The Bootstrap-related
types (`BootstrapSpec`, `BrandContent`, `SplashBackground`, `BackgroundFit`)
moved out of the minimal tier into `prelude::ui`.

### `AsyncState` → `Resource`, `OperationState` → `Operation`

The `AsyncState<T>` and `OperationState<C>` types have been replaced. The
setter-method API (`set_loading`, `set_loaded_with`, `set_failed`) has been
removed in favour of a token-based `begin`/`settle` contract:

| Old (pre-v0.1) | New (v0.1) |
|---|---|
| `AsyncState<T>` | `Resource<T>` |
| `OperationState<C>` | `Operation<C>` |
| `state.set_loading()` | `let tok = resource.begin()` |
| `state.set_loading_with(id)` | `let tok = resource.begin()` (token is the guard) |
| `state.set_loaded(val)` | `resource.settle(Settled::new(tok, Ok(val)))` |
| `state.set_loaded_with(id, val)` | `resource.settle(Settled::new(tok, Ok(val)))` |
| `state.set_failed(err)` | `resource.settle(Settled::new(tok, Err(err)))` |
| `AppUpdate::op_start(id, desc)` | removed — use `Operation<C>` fields + `#[derive(Inspect)]` |
| `AppUpdate::op_complete(id)` | removed |
| `AppUpdate::op_fail(id, err)` | removed |
| `#[probe]` / `ProbeCatalog` | `#[derive(Inspect)]` on state struct |

Stale-request safety is automatic: `Resource::settle` silently drops results
whose token doesn't match the most recent `begin` call, so the explicit `_with`
variants are no longer needed.

### Devtools: `#[derive(Inspect)]` replaces probe catalog

The devtools simulator no longer requires a separate probe catalog or the
`#[probe]` attribute. Instead:

1. Derive `Inspect` on the application's state struct (or implement it
   manually for custom traversal).
2. Declare `impl DevtoolsApp for MyApp` with `type State = MyState;` and
   `fn devtool_state_mut(&mut self) -> &mut MyState`.
3. Run with `nive_runtime::run_with_devtools::<MyApp>()`.

The simulator panel discovers all `Resource` and `Operation` fields
automatically at runtime. Payload-bearing controls are explicit:
`#[inspect(default)]` enables default Resource data,
`#[inspect(sample = path)]` enables Resource sample data, and
`#[inspect(input = path)]` enables Operation start/fail simulation. Missing
capabilities remain visible as disabled controls with tooltips.

## Testing

Run the full test suite:

```bash
just test
```

Run tests for a specific crate:

```bash
cargo test --package nive-ui
cargo test --package nive-runtime
```

## Documentation

Build and open documentation:

```bash
just doc
```

Or manually:

```bash
cargo doc --workspace --no-deps --open
```

## Publishing

Publish in dependency order (leaves first, then the umbrella, then the CLI):

```bash
# 1. nive-ui (no intra-workspace deps)
cargo publish --package nive-ui --dry-run
cargo publish --package nive-ui

# 2. nive-runtime-derive (no intra-workspace deps)
cargo publish --package nive-runtime-derive --dry-run
cargo publish --package nive-runtime-derive

# 3. nive-runtime (depends on nive-ui, nive-runtime-derive)
cargo publish --package nive-runtime --dry-run
cargo publish --package nive-runtime

# 4. nive (umbrella, depends on nive-ui, nive-runtime)
cargo publish --package nive --dry-run
cargo publish --package nive

# 5. nive-cli (binary, no workspace deps)
cargo publish --package nive-cli --dry-run
cargo publish --package nive-cli
```

Post-publish verification:
- `cargo install nive-cli` from a clean container
- `nive new smoke-test && cd smoke-test && cargo build`
- `nive new dashboard-test --dashboard && cd dashboard-test && cargo build`
- `docs.rs/nive` resolves within 24 hours
