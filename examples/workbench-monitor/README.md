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

Toasts are emitted only through the runtime effect flow (`Effect::toast`),
never by constructing `ToastHost` directly. A completed "Run health check"
emits a success toast at the `BottomEnd` default, with a safe inset kept
clear of the status bar (`StatusBar::height` fed into
`ApplicationConfig::toast_insets`, computed once from the fixed
`ControlSize::Sm` chrome). "Simulate sync failure" emits a `Toast::error`
showing only the safe summary; its "View details" action opens
`ErrorDetailsDialog` with the full diagnostic text, which the toast itself
never shows.

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

For the Toast pass, trigger "Run health check" (toolbar or command palette)
and let it complete — confirm a success toast appears bottom-right, clear of
the status bar. Press "Simulate sync failure" and confirm the toast shows
only the safe summary; press its "View details" action and confirm the full
diagnostic text — never shown in the toast itself — appears in
`ErrorDetailsDialog`, dismissible by `Escape`, an outside press, or its close
control. Open the alert Dialog or the CommandPalette while a toast is visible
and confirm existing toasts render beneath its scrim and stop counting down,
and that a newly triggered toast stays queued until the modal closes.

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

## Canonical review configuration

The reference composition for visual review is `ThemeDensity::Compact` with
`WorkbenchShell::chrome_size(ControlSize::Sm)`, in both Light and Dark, at the
canonical `1440x900`, `800x600`, and `1024x480` viewports. The Monitor owns
its own bottom region height (`180` logical pixels, set on its seeded
`WorkbenchLayoutState`) so the document canvas holds the dominant share of the
body; the generic `WorkbenchPaneSizes::default()` (`240`) is untouched by this
app-local value.

## Document and panel structure

The dashboard document (`DocumentId::Dashboard`) and every service document
(`DocumentId::Service`) each own exactly one vertical scroll region
(`container(scrollable(...))` with the overlay scrollbar direction) and
constrain their content column to a maximum readable measure so an ultrawide
window does not separate a row's leading label from its trailing values.
A service document renders, in order: the header with health status, the
metric card row, service metadata, a **Host and runtime** section, a
**Dependencies** section (the service's other seeded dependencies, selectable
into the Inspector), and a **Recent activity** section (logs and events
scoped to the service), followed by its action group.

The Inspector and the active document divide entity information by data
type instead of duplicating it: the document owns metrics and detail, the
Inspector owns identity, relation, and situation (Service, Host, Health,
Zone, Environment) for the current selection — it does not repeat a metric
value the active document already shows as a metric card. The shared
`inspector_panel` helper (`nive-workbench`) also owns body inset and
scrolling for its `Content` state, and contributes no panel status there,
since the body already shows the content; `NoSelection`, `Loading`, and
`Error` keep their status because the label is the only thing stating why
the body looks the way it does. `ProblemsPanel` similarly never restates its
own title or count badge in its status label.

## Deterministic frozen simulation mode

The Monitor supports two simulation modes, resolved once at startup:

- **Live** (default): the interactive product scenario, driven by a 900ms
  tick. `Simulation::advance()` mutates services, hosts, alerts, logs,
  events, and jobs over time.
- **Frozen** (`NIVE_MONITOR_FROZEN=1`): a deterministic fixture for review.
  No tick subscription is installed — `Simulation::advance()` runs only from
  explicit user actions (e.g. "Run health check"). The seeded fixture already
  exposes both alert severities as active, one running job at a fixed
  progress value plus one complete job, and populated logs and events, so
  every reviewable content state is visible without waiting for a tick.
  Frozen mode does **not** freeze user interaction: document/panel
  navigation, selection, dialogs, the command palette, and toasts all still
  respond normally; only the passive tick timer is absent. The active mode
  renders as a trailing status-bar item (`mode: live` / `mode: frozen`).

Both modes share one `Simulation` type and one set of views; the mode
affects only the seed fixture and whether the tick subscription is
installed.

## Shell and content state matrix

Reachable through the app's own actions, in both simulation modes: all
regions expanded; left/right/bottom collapsed independently; one panel
maximized and restored; the default dashboard and a service document; tab
overflow with long labels and dirty/pinned/closable states; and
loading/loaded/empty/error/operation-running content. CommandPalette,
Dialog, and Toast are hosted through their real application paths.

Run it:

```sh
rtk just example-dev workbench-monitor
```

Run it in deterministic frozen mode for review:

```sh
rtk just workbench-monitor-frozen
```

Equivalent standalone runs from the repository root:

```sh
rtk cargo run --manifest-path examples/workbench-monitor/Cargo.toml
NIVE_MONITOR_FROZEN=1 rtk cargo run --manifest-path examples/workbench-monitor/Cargo.toml
```

Check it:

```sh
rtk cargo test --manifest-path examples/workbench-monitor/Cargo.toml
rtk cargo check --manifest-path examples/workbench-monitor/Cargo.toml
rtk just examples-check
```

For manual sign-off, the agent launches
`rtk just workbench-monitor-frozen` and keeps
it running so the review is frame-for-frame reproducible. The user captures and attaches Light/Dark screenshots at
`1440x900`, `800x600`, and `1024x480`, including Tooltip, all-tabs Menu,
nested/outside dismissal, service-filter states, the CommandPalette
(rendering, query, and action projection), the Explorer Tree
(expanded hosts/services, the failed `diagnostics` row with retry, and the
hosted context-menu-via-Menu), and Toast (success, status-bar-safe position,
error-summary with `ErrorDetailsDialog` diagnostics, and beneath-scrim/
modal-pause behavior). The agent reviews only
those user-supplied images, applies corrections, and requests replacements; it
does not capture screenshots itself. Sign-off remains open until the user
confirms the final supplied evidence.
