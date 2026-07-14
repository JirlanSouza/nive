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

## Visual foundations review

The monitor receives Inter Regular/SemiBold and Geist Mono Regular/Medium from
`nive-runtime`; it intentionally performs no app-local font registration.
Runtime defaults the application font to Inter.

Use the monitor to review both light and dark themes after foundation changes.
Check that document titles remain visually stronger than 12 px section
headers, adjacent shell surfaces remain distinct without full outlines,
selection persists through hover/press, and keyboard focus remains visible
independently of selection. Icons inherit their host color and control-owned
icons follow the active `ControlSize` metrics.

Run it:

```sh
cargo run --manifest-path examples/workbench-monitor/Cargo.toml
```

Check it:

```sh
cargo check --manifest-path examples/workbench-monitor/Cargo.toml
just examples-check
```
