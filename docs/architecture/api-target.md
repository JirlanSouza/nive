# Target API — Phase 1

This document records the target public contract for Nive apps before the first
publication. It complements [`api-surface.md`](api-surface.md), which describes
the surface that exists; here are the decisions that should steer breaking
changes, templates, examples, and the next roadmap phases.

## Principle

A real desktop app should be able to start from `nive::prelude::*`, grow into
`nive::prelude::ui::*` when it uses async state, dialogs, or runtime windows, and
reach for `nive::runtime::*` or `nive::ui::*` only when working directly in that
layer.

Crate-root re-exports stay convenient before publication, but the scaffold,
examples, and docs should treat the preludes as the primary contract.

## Target prelude

| Path | Role | Should contain |
|------|------|----------------|
| `nive::prelude::*` | Default app tier and simple scaffold | `Application`, `ApplicationConfig`, `run`, `Effect`, `MessageContext`, `MessageSource`, `Context`, `ScreenView`, `Task`, `Subscription`, basic theming, `Toast`, `Action`, `ActionId`, `ActionMap`, `RuntimeEvent`, window declaration types (`WindowSpec`, `WindowRole`, `WindowCardinality`, `WindowCommand`), basic settings/session, runtime errors, Iced geometry, and `nive_ui::prelude::*`. |
| `nive::prelude::ui::*` | Extended app tier | Everything in the default tier plus `Resource`, `Operation`, `OperationRegistry`, `DialogRequest`, `DialogDismiss`, `ScreenEffect`, `UserFacingError`, `BootstrapSpec`, `BrandContent`, `ToastDuration`, `ToastTone`, `WindowHandle`, `WindowRegistry`, `WindowMode`, `WindowChrome`, and the file-picker params when that feature is on. |
| `nive::runtime::prelude::*` | Direct runtime consumer | The same runtime slice as the tiers above, without depending on the umbrella facade. Should stay useful for crates that do not want to import widgets. |
| `nive::ui::prelude::*` | Direct design-system consumer | `Element`, `Renderer`, common Iced layout, the theme, hosts, presentation contracts, and the UI facade's public widgets. |

Decision: keep `nive::prelude::*` and `nive::prelude::ui::*` as the end user's
happy path. `nive::runtime::prelude::*` and `nive::ui::prelude::*` are stable for
per-layer consumers, but should never be needed in the CLI-generated scaffold.

## Action, Command, and Shortcut

Decision: `Action` remains the central name for a product intent the user can see
and activate.

- `ActionId` identifies a product action stably.
- `Action<M>` carries the label, description, enabled state, an optional
  shortcut, and the message emitted on activation.
- `ActionMap<M>` is the ordered catalogue feeding shortcuts, the command palette,
  menus, and toolbars.
- `ShortcutMap<M>` still exists for shortcuts that need not appear in the visual
  action catalogue.

`Command` does not replace `Action` now. In the target vocabulary, `Command` is
reserved for the imperative runtime or platform effects that already exist, such
as `RuntimeCommand` and `WindowCommand`. A `CommandRegistry` should not be
created before the first real app proves `ActionMap` insufficient for menus,
toolbars, shortcuts, and the command palette.

## Effect and RuntimeCommand

Original Phase 1 decision: keep the names `Update`, `AppUpdate`, and
`RuntimeCommand`.

Revised decision (the `refine-runtime-effect-window-commands` change): unify
`Update`/`AppUpdate` into a single public `Effect<M, K = Never>` and drop the
generic outcome axis, which `Application`'s hooks never produced — the runtime
discarded that outcome. A screen or component's typed outcome still exists, but
in `ScreenEffect` (child-to-parent `Output`), not in the application's effect
contract.

`RuntimeCommand<K>` is no longer re-exported from the prelude or crate root: it
became a crate-internal type, drained by the runner. App authors build effects
through direct constructors (`Effect::task`, `Effect::toast`, `Effect::window`,
`Effect::theme`, `Effect::exit`) and compose with `with_task`, `with_toast`,
`with_window`, `with_theme`, and `with_exit` — never naming `RuntimeCommand`.

`RuntimeCommand<K>` still represents only the effects the runtime executes after
the app's `update`:

- `Toast(Toast)`
- `Window(WindowCommand<K>)`
- `Theme(ThemePreference)`
- `Exit`

Do not rename `RuntimeCommand` to `Action` or `Command`. The target separation is:
`Action` describes something the user can activate; `RuntimeCommand` describes an
effect the runtime must drain — while the type itself stays internal to the crate.

## Context

Decision: `Context` stays small, cheap, and read-only.

It should expose app identity, the active and preferred theme, window lookup, and
exit state. It must not become a service locator. Product clients, repositories,
and external services arrive through `Bootstrap` and live in the app's state.

New runtime capabilities should enter `Context` only when they are:

- global to the runtime;
- read-only, or represented by an explicit command in `Effect`;
- needed in more than one area of the app contract.

## Resource and Operation

Decision: keep `Resource`, `Operation`, `OperationRegistry`, `RequestId`, and
`Settled` as the target vocabulary for async state.

- `Resource<T>` represents an asynchronously loaded value with
  stale-while-revalidate and stale-response rejection.
- `Operation<C>` represents an async mutation with no persistent value,
  preserving its input while running or after failing.
- `OperationRegistry` represents a panel or register of named in-flight
  operations, particularly for global status, progress, and cancellation.
- `RequestId` and `Settled<T>` stay public because they are part of the contract
  for receiving tasks, but belong to the extended tier.

Do not create another family of names such as `AsyncResource`, `AsyncTask`, or
`Mutation` before the first real app. The next evolution should improve helpers,
progress, and cancellation without swapping the central vocabulary.

## Toast, Error, and Feedback

Decision: keep the headless/runtime versus presentation/UI separation.

- `Toast` is a temporary user-facing event emitted by `Effect::toast`.
- `ToastState` is the runtime queue and state; it may stay public for tests,
  custom hosts, and advanced integration, but does not enter the scaffold's happy
  path.
- `UserFacingError` is the presentable error used by the runtime, `Resource`, and
  `Operation`.
- Feedback widgets in `nive-ui` should depend on presentation traits, not on
  concrete `nive-runtime` types.

Phase 3: `ToastRequest` was removed from the public API. `Toast` is the target
name.

## Window lifecycle

Decision: keep the current window model as the target contract.

- `WindowSpec` declares initial appearance and behaviour.
- `WindowRole` separates app windows from auxiliary ones.
- `WindowCardinality` bounds single versus multiple.
- `WindowCommand<K>` is the effect the app emits to open, focus, or close
  windows.
- `WindowHandle<K>` and `WindowRegistry<K>` represent runtime state and stay in
  the extended tier.
- `WindowMode` and `WindowChrome` remain public concepts, but need not be in the
  minimal tier.

Phase 3: `open_window` became an internal runtime/devtools helper. Apps emit
`WindowCommand` through `Effect`. `WindowRegistration` stays out of the preludes
and remains reachable through the `nive_runtime::application` module only as the
type returned by `ApplicationConfig`'s introspectors.

## The nive-core decision

Original Phase 1 decision: do not create `nive-core` at that time.

Reason: the candidates then still belonged clearly to an existing layer.
`Error`/`Result` are runtime input; `RequestId`, `OperationId`, and `ActionId`
carry semantics tied to runtime, state, and actions; metadata and capabilities did
not yet form a shared contract independent of UI and runtime.

Decision revised after Phase 3: create a minimal `nive-core` in Phase 4 for shared
presentation and status contracts.

Reason: `nive-ui` defined traits such as `ErrorPresentation`,
`ResourceStatusPresentation`, `OperationStatusPresentation`, and
`ToastPresentation`, while `nive-runtime` implemented them for
`UserFacingError`, `Resource<T>`, `Operation<C>`, and `ToastItem`. That inverted
a conceptual boundary: the UI defined headless contracts describing runtime state
and feedback.

**Done in Phase 4:** `nive-core` exists as a workspace member
(`crates/nive-core`, zero dependencies). The four traits and `ToastTone` moved
there; `nive-ui` re-exports them at the same public paths (`widgets`,
`widgets::feedback`, `overlays`, `prelude`, and the root for
`ToastPresentation`/`ToastTone`), and `nive-runtime` implements them importing
from `nive_core` rather than `nive_ui::widgets`. `nive_runtime::ToastTone` and
`nive_runtime::ToastPosition` stopped being types of their own: the first is a
re-export of `nive_core::ToastTone`, the second of `nive_ui::ToastPosition` —
removing both duplicated `impl From` pairs and the ambiguous-glob workaround in
`crates/nive/src/lib.rs`.

`nive-core`'s Phase 4 scope:

- error presentation contracts (`ErrorPresentation`);
- toast presentation contracts (`ToastPresentation`);
- resource/operation status contracts (`ResourceStatusPresentation`,
  `OperationStatusPresentation`);
- `ToastTone`, moved because `ToastPresentation::tone()` returns it and the
  runtime/UI duplication had already leaked to the user.

Out of the initial scope, staying in their current layers:

- `Resource`, `Operation`, `OperationRegistry`;
- `UserFacingError`, `Toast`, `ToastState`, `ToastItem`;
- `Action`, `ActionMap`;
- `WindowSpec`, `RuntimeCommand`;
- runtime `Error`/`Result`;
- strong IDs still tied to a specific layer;
- `ToastPosition` (UI layout vocabulary; stays in `nive-ui` and is re-exported by
  `nive-runtime`);
- metadata, capabilities, and version, with no concrete consumer.

## Target renames

| Current | Decision |
|---------|----------|
| `ToastRequest` | Removed in Phase 3; use `Toast`. |
| `Action` | Keep; do not rename to `Command`. |
| `RuntimeCommand` | Keep the name; no longer re-exported from the prelude or crate root (crate-internal). |
| `WindowCommand` | Keep as the window-specific effect. |
| `Resource` / `Operation` | Keep as the final async-state names. |

## Target removals and restrictions

- Do not add a `Command` alias for `Action`.
- Do not add a `CommandRegistry` until a real app validates the need.
- Do not promote `WindowRegistration` into the prelude.
- Keep `open_window` an internal helper; apps use `WindowCommand`.
- `ToastRequest` is already removed; do not reintroduce a legacy alias.
- Treat crate-root wildcard usage as an alpha convenience; templates and examples
  should prefer the preludes.

## Phase 1 acceptance criteria

- A new app has a predictable path: `nive::prelude::*` first,
  `nive::prelude::ui::*` when it needs the extended tier.
- `Action`, `RuntimeCommand`, and `WindowCommand` do not compete semantically.
- `Context` does not accumulate product services.
- `Resource` and `Operation` are the final vocabulary for request/response and
  async mutations.
- The revised `nive-core` decision is explicit: create only the minimal core of
  neutral contracts in Phase 4.
- The next phase can reorganise `nive-ui` without relitigating the primary app
  contract.
