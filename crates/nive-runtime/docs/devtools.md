# Devtools Contract

Devtools is an optional `nive-runtime` capability.

```toml
[features]
default = []
devtools = ["dep:nive-runtime-derive"]
```

With the feature enabled, `nive-runtime` reexports the root `Devtools` derive.
Applications derive that marker and implement `DevtoolsApp`, including the
associated `Probe` catalog and state snapshot, command, probe snapshot and
probe-effect hooks.

Use `run_with_devtools::<A>()` to install the capability. The runtime owns the
host, generic view, panel state, auxiliary window, title, lifecycle, keyboard
shortcut and message routing. Product window kinds and application messages do
not include Devtools variants.

Devtools starts closed. `Cmd+Option+I` on macOS or `Ctrl+Alt+I` on
Windows/Linux opens the single auxiliary window or focuses it when already
open. Set `NIVE_DEVTOOLS=1` or `NIVE_DEVTOOLS=open` to open it during startup,
and set `NIVE_DEVTOOLS_TAB=probes|resources|operations` to choose the initial
tab.

State, host, operation, runtime-client and probe derives are available through
`nive-runtime`; apps do not depend directly on `nive-runtime-derive`. Derive
inputs are parsed through `syn`, preserve generics and emit errors on the
unsupported item or field span.

`DevtoolStateCatalog` requires explicit field annotations for state that cannot
be inferred safely:

```rust
#[derive(nive_runtime::DevtoolStateCatalog)]
struct State {
    #[devtool(fixtures = project_fixtures)]
    projects: nive_runtime::AsyncState<Vec<Project>>,
    #[devtool(nested)]
    selection: SelectionState,
    save: nive_runtime::OperationState<SaveContext>,
}
```

Every `AsyncState<_>` field supplies a fixture-provider function with signature
`fn(&str) -> Vec<DevtoolFixture<T>>`. Nested catalogs use `#[devtool(nested)]`.
`OperationState<_>` fields are recognized structurally and their context must
implement `DevtoolOperationContext`. Other fields are ignored.
