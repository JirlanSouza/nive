# nive-runtime

## Role

`nive-runtime` owns shared app runtime foundation types that are reused by Rust/Iced apps without depending on app-domain services.

Current scope:

- `AsyncState`
- `Application`, `ApplicationConfig`, `Context`, `WindowContext`, `CoreEvent`, and `run` — stable product contract and private runtime-owned Iced program
- `Update`, `AppUpdate`, and `RuntimeCommand` — ordered task and runtime-effect composition
- `BootstrapSpec` and the private bootstrap controller — repeatable task attempts, stale-result rejection, minimum splash duration, pending success, retry, failure details, cancellation and transfer into `Application::init`
- `client_task`
- optional `devtools` feature: command, input schema, state snapshot, host state, generic view, panel config/state/message/effect reducer and effect runner, auxiliary window lifecycle/title/shortcut, resource/operation view models, state-field collection/application helpers, `devtools::probe` generic probe catalog/list helpers, config parsing, injection store, injected client task behavior, probe panel state/messages/effects/drafts, panel filtering/summary helpers, runtime config and snapshots, and `DevtoolStateCatalog`, `DevtoolStateHost`, and `DevtoolsApp` host trait contracts
- `DialogDismiss` and `DialogRequest`
- `focus_trap` compatibility exports from `nive-ui` — `FocusDirection`, `direction_from_event`, `direction_from_keyboard_event`
- `OperationState`
- `platform::app_icon` — cross-platform app icon installer, accepting icon PNG bytes from the app layer
- `platform::file_picker` — `FileFilter`, `PickFileParams`, `pick_file`, `pick_files`, `pick_folder` (feature-gated behind `file-picker`, requires `rfd`)
- `RequestId` and `RequestCounter`
- `ScreenView` and `ScreenUpdate`
- `ToastState`, `ToastRequest`, `ToastMessage`, `ToastTone`, and `ToastItem` — generic toast state, requests, visible/queued item tracking, promotion, expiration, pause/resume behavior, and timer tick handling
- `UserFacingError` and `UserFacingResult` (including the `Devtools` error kind for injected failures)
- `WindowSpec`, `WindowMode`, `WindowChrome`, `WindowCommand`, `WindowRole`, and the private ID-keyed registry — generic Rust/Iced window contracts, cardinality, opening/open lifecycle, focus selection, command rejection and close/exit handshakes

## Internal Layout

Keep the crate root as a public façade. Prefer grouping implementation files by
runtime context instead of adding standalone modules at `src/` root:

- `application/` for the stable app contract, config, context, app-level
  events, task helpers, theme runtime, update composition and private Iced
  runner
- `lifecycle/` for bootstrap, window lifecycle/settings mechanics, window
  commands, close/exit decisions and command rejection types
- `state/` for reusable state/data helpers such as async resource state,
  operation state, request IDs and clock helpers
- `feedback/` for user-facing errors and toast runtime state
- `screen/` for screen view composition, screen update return values, dialog
  requests and dialog dismissal
- `input/` for keyboard navigation and shortcut bindings
- `platform/` and `devtools/` for their existing optional/platform-specific
  boundaries

## Boundaries

- Keep concrete `app-core` and `app-models` clients out of this crate until a domain-specific extraction is intentionally planned.
- Keep app-specific probe metadata, env var ownership, concrete probe store application, and the local `devtools::probe::ProbeCatalogEntry` implementation in `app-gui`; composed app/generated probe IDs, generated metadata aggregation, generic devtools/probe panel state and view, reducer/effect handling, auxiliary window lifecycle/spec/title/shortcut, injection store, and key/name lookup stay in `nive-runtime::devtools::probe`.
- Keep proc-macro declarations in `nive-runtime-derive`; runtime owns the generated target contracts and derive expansions target `nive_runtime::devtools` when the `devtools` feature is enabled.
- Keep app-domain fixture data in `app-gui` through explicit fixture-provider functions referenced by `#[devtool(fixtures = path)]`; do not reintroduce an app-owned state-field bridge or fixture-source trait.
- `DevtoolValue` — generic fixture source trait for async resource devtools values; app-domain values use explicit fixture-provider functions instead of direct runtime-owned trait impls when orphan rules would apply
- `DevtoolStateField` — generic state field collection/application trait for `AsyncState<T>` and `OperationState<C>`, used by `DevtoolStateCatalog` derive expansions
- Keep product brand assets (icon PNG bytes, brand theme tokens) in `app-gui`; the installer pattern in `nive-runtime::platform::app_icon` accepts generic icon bytes passed from the app.
- Keep app-specific logical window enums, titles, dimensions, fonts and icon construction in `app-gui`; runtime owns reusable window specs, settings conversion, registry mechanics, opening, focus and close/exit routing.
- Keep widget-layer focus and overlay behavior in `nive-ui`; runtime may re-export stable helper APIs while lifecycle and shell code still consumes them.
- Keep visual toast composition in `nive-ui` (`ToastHost`); runtime owns generic toast state/types, visible/queued overflow, promotion, expiration, pause/resume, timer tick handling, and applies the host automatically to app-role windows. `ToastItem` implements `nive-ui`'s `ToastPresentation`. `ScreenUpdate` remains generic over the feedback payload.
- Keep bootstrap lifecycle state private. Apps provide only the task factory,
  result type, assets and copy; product clients and services transfer into
  `Application::init` and are not retained by the runtime.
- Prefer behavior-preserving moves from `app-gui` into this crate, with tests moved alongside the extracted types.
