# Nive Development Guide

This guide is for contributors working on the Nive framework itself.

## Requirements

- Rust 1.92 or later
- Cargo
- just

## Setup

Clone the repository:

```bash
git clone https://github.com/JirlanSouza/nive.git
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

# Run CI-like readiness checks
just readiness

# Run the widget gallery with terminal-triggered reload
just widget-gallery-dev

# Run the widget gallery with devtools explicitly enabled
just widget-gallery-devtools

# Run any standalone example with terminal-triggered reload
just example-dev widget-gallery

# Build all crates
just build

# Build release
just release
```

## Icon Management

The framework maintains semantic icon role defaults in `nive-ui`. For external
apps, use the provider-neutral `nive icons` workflow (see
[adding-icons.md](guides/adding-icons.md)).

```bash
# List framework icons
just icons-list

# Sync framework icons
just icons-sync

# Check icons are up to date
just icons-check

# Add framework icon symbol
just icons-add-symbol <Variant> <provider-ref>

# Set framework icon role
just icons-set-role <role-name> <provider-ref>
```

## Creating Test Apps

```bash
# Create a new app using the framework
nive new test-app
```

## Architecture

See [docs/agents/architecture.md](agents/architecture.md) for framework architecture.

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
- `Application::init` and `Application::update` return
  `impl Into<Effect<...>>`, so `()` is a valid return ("no side effects").
  `impl From<()> for Effect<M, K>` produces `Effect::none()`. `Effect` has no
  outcome type parameter — it carries only a task and ordered runtime
  commands, built via direct constructors (`Effect::task`, `Effect::toast`,
  `Effect::window`, `Effect::theme`, `Effect::exit`) and `with_*` combinators.
- `Application::update` receives a `MessageContext<Self::Window>` (window +
  `MessageSource::{View, Task, Subscription, Action}`) instead of
  `Option<WindowContext<Self::Window>>`.
- `Application::window_title` returns `impl Into<Cow<'a, str>> + 'a`, so
  apps no longer need to `use std::borrow::Cow;` to return a `&'static str`.
- `Application::theme(ctx, Option<WindowContext>) -> ThemePreference`
  (default `ThemePreference::System`) lets the app influence the global
  theme singleton. The runtime consults it on startup, on
  `Effect::theme(pref)` emissions, and on OS theme changes; the
  method's return value wins over the emitted preference except when it
  returns `System` (then the OS / persisted preference wins).
- `Application::on_core_event`/`CoreEvent` are renamed
  `Application::on_runtime_event`/`RuntimeEvent`. `ExitDecision::Accept` is
  renamed `ExitDecision::Exit`.

### `Toast` ≠ `ToastRequest`

`ToastRequest` is renamed `Toast` consistently across `nive-runtime`,
`nive-ui`, preludes, and docs. The deprecated `ToastRequest` alias has been
removed before the first public compatibility promise. Existing local apps
should rename `ToastRequest` references to `Toast`.

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
and no `Effect::op_*` API.

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

`nive::prelude::*` is the minimal template-stable surface. It already includes
basic toasts (`Toast`), theming (`Theme`/`ThemeBuilder`/`ThemeController`),
shortcuts (`ShortcutMap`), actions, core events, and the window *declaration*
types (`WindowSpec`/`WindowRole`). Apps switch to `nive::prelude::ui::*` for
async state (`Resource`/`Operation`), dialogs, `UserFacingError`, file picker
params, runtime window handles (`WindowHandle`/`WindowRegistry`/`WindowMode`),
`ToastDuration`, and the Bootstrap-related types (`BootstrapSpec`,
`BrandContent`, `BackgroundFit`) — all of which live in the extended tier.

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

## Readiness Checks

Local readiness recipes mirror CI categories where the local environment
allows:

```bash
just fmt-check
just check
just test
just lint
just doc-check
just examples-check
just scaffold-smoke
just scaffold-smoke-github
just package-check
just icons-check
```

`just icons-check` must remain offline: it validates checked-in `icons.toml`,
generated Rust modules, generated SVG assets, custom SVG references, stale
assets, and required role coverage without fetching Lucide data.

`just scaffold-smoke` creates temporary apps outside the workspace, patches the
generated app to the local `nive` checkout via `[patch.crates-io]`, and runs
`nive icons check` plus `cargo check` for both the basic and dashboard
templates.

`just scaffold-smoke-github` creates temporary apps outside the workspace using
a temporary Git snapshot of the current working tree. This validates the exact
dependency shape that real apps use with GitHub alpha tags without requiring a
pushed tag or a pre-existing commit for local changes.

`just package-check` verifies package readiness only. It runs `cargo package`
for `nive-core`, `nive-ui`, `nive-runtime-derive`, `nive-runtime`, `nive`, and
`nive-cli` in dependency order. For crates that depend on unpublished internal Nive crates,
the recipe passes temporary Cargo patch config so verification models the local
dependency chain without publishing anything. It does not publish crates, tag a
release, or perform post-publish docs.rs verification.

## Documentation

Build and open documentation:

```bash
just doc
```

Or manually:

```bash
cargo doc --workspace --no-deps --open
```

## Alpha Release Ritual (GitHub pre-crates.io)

GitHub alpha releases let you dogfood the framework from real apps before the
irreversible crates.io publication. The distribution channel is
`https://github.com/JirlanSouza/nive` with annotated `v0.1.0-alpha.N` tags.

### Releasing an alpha

1. **Bump versions** — update all publishable crate versions to `0.1.0-alpha.N`
   in their `Cargo.toml` files: `nive-core`, `nive-ui`, `nive-runtime-derive`,
   `nive-runtime`, `nive`, `nive-cli`. Update internal dependency version
   requirements to match.

2. **Run readiness** — all checks must pass before tagging:

   ```bash
   just readiness
   ```

   `just readiness` now includes both the local `[patch.crates-io]` scaffold
   smoke and the GitHub consumer smoke (`just scaffold-smoke-github`).

3. **Create an annotated tag:**

   ```bash
   git tag -a v0.1.0-alpha.1 -m "v0.1.0-alpha.1 — initial GitHub dogfood release"
   git push origin v0.1.0-alpha.1
   ```

4. **Create a GitHub pre-release** for the tag. Mark it as a pre-release (not
   a latest release). Include install and dependency snippets in the release
   notes (see template below).

5. **Dogfood verification** — install the CLI from the tag on a clean machine
   and create a test app:

   ```bash
   cargo install --git https://github.com/JirlanSouza/nive --tag v0.1.0-alpha.1 --locked nive-cli
   nive new my-alpha-app --git https://github.com/JirlanSouza/nive --tag v0.1.0-alpha.1
   cd my-alpha-app && cargo build
   ```

### Rollback strategy

Do **not** delete tags that may already be consumed by real apps. If an alpha is
bad, document it in the GitHub release notes and publish `v0.1.0-alpha.N+1` with
the fix.

### Release notes template

Widget API changes in alpha releases should link migration notes when relevant.
For the public API semantics cleanup, use
[`migrations/widget-public-api-semantics.md`](migrations/widget-public-api-semantics.md).

```markdown
## v0.1.0-alpha.1

Pre-crates.io dogfood release. Install from GitHub:

### Install the CLI

\`\`\`bash
cargo install --git https://github.com/JirlanSouza/nive --tag v0.1.0-alpha.1 --locked nive-cli
\`\`\`

### Create an app

\`\`\`bash
nive new my-app --git https://github.com/JirlanSouza/nive --tag v0.1.0-alpha.1
cd my-app && cargo build
\`\`\`

### Add as a dependency

\`\`\`toml
nive = { git = "https://github.com/JirlanSouza/nive", tag = "v0.1.0-alpha.1" }
# or with the file-picker feature:
nive = { git = "https://github.com/JirlanSouza/nive", tag = "v0.1.0-alpha.1", features = ["file-picker"] }
\`\`\`
```

## Publishing (crates.io — final v0.1.0 release)

Publishing to crates.io is a separate maintainer action after all alpha
dogfooding is complete. Bump all crate versions to `0.1.0` before publishing.
Publish in dependency order (leaves first, then the umbrella, then the CLI):

```bash
# 1. nive-core (zero dependencies)
cargo publish --package nive-core --dry-run
cargo publish --package nive-core

# 2. nive-ui (depends on nive-core)
cargo publish --package nive-ui --dry-run
cargo publish --package nive-ui

# 3. nive-runtime-derive (no intra-workspace deps)
cargo publish --package nive-runtime-derive --dry-run
cargo publish --package nive-runtime-derive

# 4. nive-runtime (depends on nive-core, nive-ui, nive-runtime-derive)
cargo publish --package nive-runtime --dry-run
cargo publish --package nive-runtime

# 5. nive (umbrella, depends on nive-ui, nive-runtime)
cargo publish --package nive --dry-run
cargo publish --package nive

# 6. nive-cli (binary, no workspace deps)
cargo publish --package nive-cli --dry-run
cargo publish --package nive-cli
```

Post-publish verification:
- `cargo install nive-cli` from a clean container
- `nive new smoke-test && cd smoke-test && cargo build`
- `nive new dashboard-test --dashboard && cd dashboard-test && cargo build`
- `docs.rs/nive` resolves within 24 hours
