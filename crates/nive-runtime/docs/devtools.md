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

The existing state, operation, runtime-client and probe derives remain
available through `nive-runtime` while their parsing is hardened separately.
Apps do not depend directly on `nive-runtime-derive`.
