# Workbench Monitor

`workbench-monitor` is the visual validation app for `nive-workbench`.

It renders a deterministic service-monitoring desktop shell with simulated
services, hosts, alerts, logs, events, jobs, command palette actions, dialogs,
toasts, document tabs, side rails, bottom tabs, and a status bar.

The command palette (Cmd+K, or the "Open command palette" action) hosts the
canonical `nive_ui::widgets::CommandPalette` directly, projecting its items
from the shell's shared `ActionMap` via `nive_workbench::action_palette_items`
— the monitor owns only `open` and the controlled query. It preserves focus,
horizontal long-value behavior, filtered actions, overlay placement, and
shell geometry at narrow viewports.

The runtime installs one logical-focus root around the complete window,
including workbench content and overlay hosts. The bottom-panel tab track uses
one outer managed target while preserving its active/roving tab internally;
no application focus manager or second overlay root is required.

The settings area uses a titled immediate `Switch::setting`, a two-option typed
environment `SegmentedControl`, and a genuine dashboard
`Select<ServiceScope>` that filters application-owned service data. Toolbar
theme actions and document/panel navigation keep their specialized ownership;
they are not modeled as form selection controls.

Framework-owned icon-only, truncated, and non-obvious actions use shared
Tooltip disclosure while retaining independent semantic names. Complete visible
action labels do not duplicate themselves in Tooltip. TabBar's all-tabs
overflow uses canonical typed Menu in one Popover while preserving pinned-first
order, current/disabled state, dirty/pinned metadata, selection, dismissal, and
focus return.

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

Dashboard/service content uses framework-owned Md card shape and semantic
content padding. Metric labels are separate from baseline units and optional
status/trend content. Document peer actions use `ActionGroup` with
`ContentAction`; Toolbar continues to use `ToolbarAction`. The service action
group opts into whole-control wrapping so `800x600` and `1024x480` expose the
narrow layout without app-local control resizing.

Inspector `KeyValueList` content is surface-neutral and uses framework-owned
14 px Text/Code values; its surrounding workbench panel owns the only surface.
Panel and bottom-tab counts use `BadgeContent::Count`; status uses complete
visible `StatusIndicator` labels and omits unlabeled dots. Problems and other
`DataRow` content keep principal and source/value metadata clustered, reserve
status slots where needed, and use a one-item `ActionGroup`/`ContentAction`
for protected peer actions. Selectable service/host/alert rows own whole-row
interaction and labelled status explicitly.

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

At `1440x900`, `800x600`, and `1024x480`, also verify filled, outlined, and
elevated cards, label-before-value metric hierarchy, baseline units and support
content, loading/disabled content actions, keyboard focus, and wrapping with no
orphaned separator or split action.

Also confirm the titled Switch and both typed selectors preserve their values,
focus, and finite geometry in Light/Dark at all three reference viewports.
For logical-focus review, alternate pointer and keyboard entry through the
command Input, document tabs, bottom tabs, SplitPane, dialogs, and popovers;
verify pointer-hidden versus keyboard-visible rings, empty-press continuation,
window deactivate/reactivate behavior, and conditional nested-overlay restore.
This remains a user-screenshot/manual boundary even though state and traversal
are covered by automated tests.

For the CommandPalette pass, open it and verify: rendering and the controlled
query as you type; commands projected from the shared `ActionMap` (including
"Run health check", "Toggle theme", and the panel/document toggles); and that
selecting or dismissing it closes the palette without leaving stale query
text. Confirm it replaces rather than stacks with the alert Dialog if both are
triggered.

For the anchored-popup pass, verify icon-only Tooltip disclosure without
redundant labelled-action copies, all-tabs Menu order/selection/focus/overflow,
nested and outside dismissal, and the dashboard `Select<ServiceScope>` filter.
Start/End and submenu keys are currently physical LTR; popup and chevron visuals
change immediately without interpolated motion; arbitrary EdgeToEdge Popover
descendants use rectangular Iced 0.14 clipping; and retained semantics do not
yet emit native accessibility-tree roles, names, expanded state,
active-descendant relations, or announcements.

For the Tree pass, open the left "Explorer" panel: it groups seeded hosts and
their services under `Tree`, plus a `diagnostics` branch that always fails
after a short delay through `TreeChildren::Failed`. Expand it to see the
canonical error row and press its retry affordance to re-trigger the same
failure. Right-click a host or service row to confirm Tree emits
`ContextRequested` only and the app hosts the canonical `Menu` (Inspect, plus
Open document for services) at the pointer position — Tree owns no menu of
its own. Activating a service or host row updates the shared `Selection` and
Inspector panel the same way the existing Services/Hosts lists do.

Run it:

```sh
rtk just example-dev workbench-monitor
```

Equivalent standalone run from the repository root:

```sh
rtk cargo run --manifest-path examples/workbench-monitor/Cargo.toml
```

Check it:

```sh
rtk cargo test --manifest-path examples/workbench-monitor/Cargo.toml
rtk cargo check --manifest-path examples/workbench-monitor/Cargo.toml
rtk just examples-check
```

For manual sign-off, the agent launches
`rtk just example-dev workbench-monitor` and keeps
it running. The user captures and attaches Light/Dark screenshots at
`1440x900`, `800x600`, and `1024x480`, including Tooltip, all-tabs Menu,
nested/outside dismissal, service-filter states, the CommandPalette
(rendering, query, and action projection), and the Explorer Tree
(expanded hosts/services, the failed `diagnostics` row with retry, and the
hosted context-menu-via-Menu). The agent reviews only
those user-supplied images, applies corrections, and requests replacements; it
does not capture screenshots itself. Sign-off remains open until the user
confirms the final supplied evidence.
