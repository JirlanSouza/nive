# Workbench Monitor

`workbench-monitor` is the visual validation app for `nive-workbench`.

It renders a deterministic service-monitoring desktop shell with simulated
services, hosts, alerts, logs, events, jobs, command palette actions, dialogs,
toasts, document tabs, side rails, bottom tabs, and a status bar.

## Chrome sizing

The monitor deliberately uses `ThemeDensity::Compact` for its global theme
metrics and `WorkbenchShell::chrome_size(ControlSize::Sm)` for its local
workbench scale. Density changes the resolved theme metrics; the shell chrome
size applies one shared control size to its toolbar, status bar, tabs, rails,
panel headers, bottom selector, and split panes.

Its shell receives typed `Toolbar` and `StatusBar` values. The shell retains
them until rendering so its selected chrome size takes precedence over any
toolbar size chosen by the caller.

Run it:

```sh
cargo run --manifest-path examples/workbench-monitor/Cargo.toml
```

Check it:

```sh
cargo check --manifest-path examples/workbench-monitor/Cargo.toml
just examples-check
```
