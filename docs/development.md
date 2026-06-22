# Nive Development Guide

This guide is for contributors working on the Nive framework itself.

## Requirements

- Rust 1.92 or later
- Cargo
- just
- curl (for icon syncing)

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

The framework maintains a set of essential icons in `nive-ui`.

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
just create-app test-app

# Or manually
cargo run --package create-nive-app -- test-app
```

## Architecture


## Migration (v0.1.0 contract changes)

The v0.1 "API ergonomics" change rewrote several public-contract surfaces.
Apps written against the pre-v0.1 snapshot need the updates below. New apps
scaffolded via `cargo run --package create-nive-app -- my-app` already use
the new defaults.

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

### Runtime-managed `OperationRegistry`

`AppUpdate` exposes `op_start / op_complete / op_fail / op_cancel` for driving
the runtime-managed `OperationRegistry` (visible by devtools). Apps that
previously held their own `OperationRegistry` field may keep it for in-app
rendering, but should NOT also call `op_start` for the same ids — the two
registries are independent and do not stay in sync. Pick one path per id.

### `AsyncState` stale-request guarding

`AsyncState<T>` gained `set_loading_with(RequestId)`, `set_loaded_with(RequestId, T)`,
and `set_failed_with(RequestId, UserFacingError)`. Stale responses are
silently ignored. The unguarded `set_loading / set_loaded / set_failed`
methods keep their pre-change behaviour (no id check) — existing apps do not
need to migrate; new apps should prefer the `_with(...)` forms.

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

When ready to publish:

```bash
# Dry run
cargo publish --package nive-ui --dry-run
cargo publish --package nive-runtime --dry-run
cargo publish --package nive --dry-run
cargo publish --package create-nive-app --dry-run

# Publish
cargo publish --package nive-ui
cargo publish --package nive-runtime
cargo publish --package nive
cargo publish --package create-nive-app
```

Note: Publish order matters due to dependencies.
