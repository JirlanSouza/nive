# Application Contract

`nive-runtime` exposes the stable-Rust application contract through
`nive_runtime::prelude`.

The app-facing public API is the crate root plus `nive_runtime::prelude`.
Applications should import runtime contracts from those facades instead of
depending on runner module paths. The public contract includes:

- `Application`, `ApplicationConfig`, `Context`, `WindowContext` and `run`
- `Action`, `ActionId`, `ActionMap`, `DuplicateActionId` and neutral shortcut types
- `Effect`, `MessageContext`, `MessageSource` and `perform`
- lifecycle/window contracts such as `WindowSpec`, `WindowCommand`,
  `CloseDecision`, `ExitDecision`, `BootstrapSpec` and `RuntimeEvent`
- reusable feedback/state helpers such as `Toast`, `UserFacingError`,
  `Resource`, `Operation`, tracked requests/scopes, settlement, cancellation,
  and clock helpers
- `Task`, `Subscription`, `time`, `window`, `Point` and `Size`
  aliases/reexports used by app hooks
- settings/session contracts such as `SettingsConfig`, `RuntimeSession`,
  `WindowSession`, `WindowSessionSize` and `WindowSessionPosition`
- `Theme`, `ThemeBuilder`, `ThemeCatalog`, `ThemeMode` and `ThemePreference`
- feature-gated platform/devtools APIs

Implementation modules are intentionally not exposed as stable integration
points. Public platform and devtools modules are extension surfaces; private
application/lifecycle/feedback/state/screen modules remain runner internals.

Applications implement `Application` with product-owned message, window and
bootstrap types. Apps without bootstrap use `type Bootstrap = ();`. Rust stable
does not support the approved associated-type default syntax, so the explicit
associated type is required.

## Actions And Shortcuts

Applications can expose product commands through `Application::actions`. The
immutable action types are owned by zero-dependency `nive-core` and re-exported
by runtime and the umbrella crate, so `nive-ui` controls consume the exact same
values without depending on runtime. An `ActionMap` contains ordered `Action`
values with stable `ActionId`s,
user-facing labels, optional descriptions, optional shortcut bindings, enabled
state and the product message to emit when activated.

The runtime uses action shortcuts before legacy shortcuts, after framework
reserved shortcuts:

1. Framework-reserved shortcuts: Escape, Tab/Shift+Tab focus traversal and
   feature-gated Devtools toggle.
2. Enabled `Application::actions()` shortcuts.
3. `Application::shortcuts()` fallback bindings.

Disabled actions remain visible to future command surfaces but do not dispatch
from shortcut routing. `ActionMap::validate()` reports duplicate IDs through
`DuplicateActionId`; apps and tests can call it at catalog construction
boundaries without making keyboard routing panic in release builds.

Use `ShortcutBinding::primary_character` for app commands that should follow
the platform primary modifier (`Cmd` on macOS, `Ctrl` elsewhere) without
depending on Iced keyboard modifier constants in app code.

For explicit shortcuts, use the neutral `ShortcutModifiers` bit flags and
`NamedShortcutKey` values:

```rust
# use nive_runtime::{NamedShortcutKey, ShortcutBinding, ShortcutModifiers};
let save_as = ShortcutBinding::character(
    's',
    ShortcutModifiers::CONTROL | ShortcutModifiers::SHIFT,
);
let cancel = ShortcutBinding::named(
    NamedShortcutKey::Escape,
    ShortcutModifiers::NONE,
);
# let _ = (save_as, cancel);
```

`Application::shortcuts` remains the low-level hook for bindings that do not
represent application actions. App-facing commands should prefer actions so
the same command can power toolbar, menu and command-palette surfaces.

Apps that surface a `nive-ui` command palette project each action through
`CommandPaletteItem::from_action`. `ToolbarAction::from_action` provides the
same command semantics for a text toolbar action, while
`ToolbarAction::from_action_with_icon` accepts UI-owned icon decoration. The
canonical `CommandPalette` composite is in `nive-ui`: it owns its own
placement, focus, and keyboard navigation, so apps host it directly (not
through `DialogRequest`) and own only `open(bool)`, the controlled query, and
`on_dismiss`.

## Native Menus, Tray, And Global Shortcuts

Iced 0.14 does not expose a native menu bar, system tray, or global
shortcut API. The only `window` Task is `show_system_menu`, which
surfaces the OS-provided window context menu (maximize / minimize /
close on right-click of the title bar) on Windows and Linux. Nive
therefore defers native menu integration until Iced adds upstream
support.

For now, app navigation uses in-app menus. The canonical Menu work projects
command entries from `Action<M>` so menus share the catalog used by shortcuts,
toolbars, and the command palette. Checkbox, radio-group, separator, and
submenu entries remain typed Menu categories because their state and hierarchy
are surface-specific.

## Accessibility And Keyboard

The framework's accessibility contract is documented in
`crates/nive-ui/docs/components.md`. The runtime contributes the
keyboard helpers:

- `is_escape_key_press(&Event)` detects the Escape key for overlay
  dismissal.
- `DialogDismiss` carries the dismiss message for Escape, backdrop, or
  both; `DialogRequest` wires it into the modal overlay.
- `keyboard_navigation_subscription` and the focus trap helpers cycle
  focus on Tab and Shift+Tab. Modifier-only Tab is left to the
  application.

Application code that introduces new keyboard affordances (custom
shortcuts, in-app menus) should reuse these helpers instead of
parsing Iced events directly so the framework's escape/focus
behavior remains consistent.

## Operation Registry

`OperationRegistry` is the app-wide presentation store for long-running operations.
It complements the per-operation `Operation<C>` state machine and
is the surface app-wide progress, cancellation, and retry UI can read
from.

Core types:

- `OperationId(&'static str)` — stable, product-owned identifier. The
  registry keys entries by this id; products that need multiple
  concurrent instances of the same logical operation should give each
  instance a unique id.
- `OperationDescriptor` — registered metadata: title, progress and
  `cancellable` flag.
- `OperationProgress` — `Indeterminate`, `Fraction { completed, total }`
  with a `ratio()` helper, or `Message(Cow<'static, str>)` for textual
  status. `ratio()` returns `None` for indeterminate and message-only
  progress so UI can fall back to spinners.
- `OperationStatus` — `Running`, `Completed`, `Failed(UserFacingError)`,
  `Cancelled`. `is_running()` and `is_terminal()` are explicit.
- `OperationEntry` — `descriptor` plus `status` for the live entry.
- `OperationRegistry` — `BTreeMap`-backed store with `register`,
  `update_progress`, `complete`, `fail`, `cancel`, `remove`, `iter`,
  `running`, `cancellable`, `running_count`, `clear_terminal`.

Invariants:

- `register` overwrites an existing entry's descriptor and resets the
  status to `Running`. The previous descriptor is returned so apps can
  surface "the same operation restarted" UI if they need it.
- `update_progress` is a no-op when the entry is missing or already
  terminal. Apps that want a progress notification after completion
  should restart the operation.
- `cancel` only succeeds on running, cancellable entries. The runtime
  surfaces cancel via `ActionMap` so the same product action can be
  bound to a button, a menu item, and a command palette entry.
- `clear_terminal` removes only completed/failed/cancelled entries
  and returns the count. Apps can use it to bound the visible history.

`OperationRegistry::cancel` only changes registry presentation state. A tracked
`Operation<C, T>` uses its separate `cancel()` descriptor and
`Effect::cancel` to stop the underlying request. Apps keep both models aligned
when they choose to display a tracked request in the registry.

Slice 10 does not introduce UI components. The registry is the
runtime model only; presentation can be added in a later slice that
consumes `iter()` / `running()` / `cancellable()` without coupling
the registry to a specific widget.

`ApplicationConfig` declares product windows, initial windows, theme preference,
optional custom `ThemeCatalog`, toast position, fonts, a shared window icon and
optional bootstrap configuration. `Context` and `WindowContext` are read-only
views; their `app_scope()` and `task_scope()` methods expose only opaque
lifetime capabilities, not runtime-owned mutable state.

Runtime settings/session persistence is opt-in and documented in `settings.md`.
Apps supply the settings file path; runtime-owned persistence must not absorb
product/domain settings.

`Effect<M, K = Never>` combines raw Iced tasks, tracked `RequestTask` values,
linear request cancellations, and ordered runtime side effects: toasts, window
commands, theme changes and application exit requests.
Application hooks return `impl Into<Effect<...>>`; returning `()` is equivalent
to `Effect::none()`. Typed child-to-parent outputs live in `ScreenEffect`, not
in the application effect contract.

## Tracked Async Requests

State machines mint a process-unique request only after admission. `Resource`
uses restart semantics by default; `Operation<C, T>` uses drop-new semantics so
a busy submit lane creates no second request. `Request<T, I>::perform` connects
application-owned intent and services to an observation-only `CancelSignal` and
returns an opaque `RequestTask<Message>`.

The runtime registers tracked work before returning it to Iced and removes the
entry before delivering its optional terminal application message. Explicit
cancel/reset and replacement transition local state immediately and suppress a
duplicate message. Scope closure remains message-borne as
`Settled::Cancelled`, because a surviving app state must apply its own terminal
transition. Timeout/deadline settle as failure.

Use `Context::app_scope()` for work that must survive individual windows,
`WindowContext::task_scope()` for window-owned work, and an owned child
`TaskScope` for a screen/component lifetime. Dropping a child scope cancels its
descendants without affecting its parent or siblings. Messages routed after a
screen was removed must be treated as a normal no-op by the parent router.

The four supported tiers are direct (`load`/`run`), reducer-friendly handle
(`request*` then `perform`), external-owner (`into_settled`), and manual
(`begin` plus raw `RequestId`). Only the first two are Nive-owned and scoped.

`run::<A>()` owns the private Iced daemon state. Product view messages are
correlated with their source window automatically. `Application::update`
receives `MessageContext` so apps can distinguish view, task, subscription and
action-originated messages. The runner processes ordered effects, configured
initial windows, dynamic titles, app subscriptions, declared product shortcuts
and runtime events.

`BootstrapSpec` accepts a task factory so retries create independent attempts.
When configured, the runner opens an internal splash, correlates results with
their attempt, enforces the minimum splash duration and calls `Application::init`
only after success. The bootstrap value is transferred into `init`; the runtime
does not retain product clients or services afterward. The initial `Effect`
runtime side effects are processed before configured initial product windows open,
while the app task from `init` runs concurrently and cannot block initial window
opening.

Failure, retry, diagnostic details and close-during-bootstrap are runtime-owned.
Closing the splash exits without constructing the application. Apps without
bootstrap continue to use `Bootstrap = ()`.

## Theme Runtime

`ThemeController` owns the configured `ThemePreference`, optional custom
`ThemeCatalog`, current system mode, effective Nive theme, initial system-theme
detection task and system-change subscription. The runner applies it internally
and emits `RuntimeEvent::ThemeChanged` when the effective theme changes.

Only `ThemeController` synchronizes the global `nive-ui` active-theme snapshot.
Application code must not call Iced system-theme APIs or mutate the snapshot
directly.

Apps that need product-specific branding build light/dark themes with
`ThemeBuilder`, pass them as `ThemeCatalog::new(light, dark)`, and attach the
catalog through `ApplicationConfig::theme_catalog`.

## Toast Runtime

The runner owns the toast queue, expiration timers, hover pause/resume and
manual dismiss. `Effect::toast` enqueues a `Toast`; the runtime assigns
identity, shows up to three visible toasts,
keeps overflow queued and starts queued toast expiry only when promoted. A time
subscription ticks only while toasts are visible, expiring due items and
pausing expiration while the host is hovered. Default durations are
info/success 4s, warning 6s and danger/error 8s.

The runner applies `nive-ui`'s `ToastHost` automatically to app-role windows
with visible toasts, using the configured `ToastPosition`. Auxiliary windows
and the internal splash are not decorated. Toasts do not capture focus and may
remain visible alongside a modal dialog. Applications emit toasts through
`Effect::toast` and never own toast state or the host widget.

## Clock Helpers

`unix_now()` provides the current Unix timestamp in seconds, and
`relative_time_label(updated_at, now)` formats shared compact relative-time
labels. Keeping the clock input explicit makes relative-time presentation
deterministic in state and widget tests.

## File Picker

The optional `file-picker` feature exposes cross-platform file dialog
primitives backed by `rfd`:

- `FileFilter` and `PickFileParams` for opening a single file.
- `PickFileParams` and `pick_files` for opening one or more files.
- `pick_folder` for opening a directory.
- `SaveFileParams` and `save_file` for picking a save destination, with
  optional default name pre-fill.

All dialog calls return `Task<Option<PathBuf>>` (or
`Task<Option<Vec<PathBuf>>>` for `pick_files`) so apps compose them with the
runtime `Task`/`Message` flow. The `PickFileParams` and `SaveFileParams` types
are part of the public contract and are always available; only the dialog
task constructors are gated behind the feature. Apps that opt in to the
`file-picker` feature get the full open/folder/save set as reusable platform
primitives.

## Devtools Runtime

With the `devtools` feature enabled, `run_with_devtools::<A>()` monomorphizes
the runner with the inspectable `A::State` and installs the internal Devtools host. The standard
`run::<A>()` path has no Devtools runtime. Default builds do not expose or
compile the `nive_runtime::devtools` module or simulator APIs.

The runner owns the auxiliary window, title, window policy, keyboard shortcut,
panel message routing and simulator effects. Devtools is closed by default;
`Cmd+Option+I` on macOS or `Ctrl+Alt+I` on Windows/Linux opens it and focuses the
existing window on later toggles.
