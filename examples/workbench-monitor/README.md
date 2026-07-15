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

The reference composition uses `DocumentHeader` for principal service and
dashboard titles, `SectionHeader` for compact sections, explicit StatusBar
lanes, transparent toolbar groups, Panel-owned internal seams with body-owned
inset, 12/6 overlay scrollbars, dedicated bottom controls, and public
`WorkbenchPaneConstraints`.

Document tabs use the refined controlled `TabBar`; side selectors use public
edge-rail presentation; service and Inspector choices use semantic
`SelectableItem` rows. Bottom panels use content-sized leading tabs with
contained wheel overflow and a protected trailing action lane instead of an
equal-width segmented selector. Labels, counts, ordering, ratios, and sample
status data remain local to this example.

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

Review wide, narrow, and low viewports in both themes. Exercise toolbar and tab
overflow, long title tooltips, scrollbar hover/drag, SplitPane hover/drag/focus,
document menu/keyboard/disabled/dirty/pinned/close/drag/cancellation states,
side rails, constrained bottom-tab wheel overflow, collapsed side and bottom
regions, and maximized panels. Fixed chrome extents, protected trailing
controls, and all region bounds should remain contained.

Run it:

```sh
cargo run --manifest-path examples/workbench-monitor/Cargo.toml
```

Check it:

```sh
cargo check --manifest-path examples/workbench-monitor/Cargo.toml
just examples-check
```
