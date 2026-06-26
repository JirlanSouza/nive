# Devtools Contract

Devtools is an optional `nive-runtime` capability. Default builds do not expose
or compile the `nive_runtime::devtools` module, Devtools window, panel state,
messages or generic UI.

```toml
[features]
default = []
devtools = ["nive-runtime-derive/devtools"]
```

With the feature enabled, `nive-runtime` exposes the `devtools` module,
`run_with_devtools`, the `Inspect` trait, and the `#[derive(Inspect)]`
traversal implementation. The derive macro remains available without the
feature and expands to nothing, so applications can keep it in source with no
production simulator code.

Applications implement `DevtoolsApp` with mutable access to an inspectable
state struct. The runtime owns the host, generic view, panel state, auxiliary
window, title, lifecycle, keyboard shortcut and message routing. Product window
kinds and application messages do not include Devtools variants.

Devtools starts closed. `Cmd+Option+I` on macOS or `Ctrl+Alt+I` on
Windows/Linux opens the single auxiliary window or focuses it when already
open. Set `NIVE_DEVTOOLS=1` or `NIVE_DEVTOOLS=open` to open it during startup,
and set `NIVE_DEVTOOLS_TAB=resources|operations` to choose the initial tab.

`#[derive(Inspect)]` recurses through every field, following the same model as
serde derives. `Resource<T>` and `Operation<C>` register simulator entries;
`OperationRegistry` registers a read-only registry snapshot; common scalar
types are no-op leaves; `Vec<T>`, `Option<T>` and `Box<T>` recurse when `T:
Inspect`. Use `#[inspect(skip)]` for fields that should not be traversed.

Payload-bearing simulator controls are opt-in:

```rust
#[derive(nive_runtime::Inspect)]
struct State {
    #[inspect(default)]
    cached: nive_runtime::Resource<Vec<Project>>,
    #[inspect(sample = sample_projects)]
    projects: nive_runtime::Resource<Vec<Project>>,
    #[inspect(input = SaveInput::devtools_sample)]
    save: nive_runtime::Operation<SaveInput>,
    operations: nive_runtime::OperationRegistry,
}
```

`#[inspect(default)]` requires `T: Default`. `#[inspect(sample = path)]` uses a
`fn() -> T` factory for Resource sample data. `#[inspect(input = path)]` uses a
`fn() -> C` factory for Operation start/fail simulation. Missing capabilities
are rendered as disabled controls with tooltips explaining the required
attribute, and unsupported simulator actions return an explicit unsupported
result instead of being treated as success.
