# nive-runtime

## Role

`nive-runtime` owns shared app runtime foundation types that are reused by Rust/Iced apps without depending on app-domain services.

Current scope:

- `AppPhase` — generic lifecycle phase enum (`Booting`, `BootFailed`, `Ready`) parameterized over error, pending-success, and ready-state types, with splash duration, pending-success, and phase query methods
- `AsyncState`
- `SplashConfig` — configurable minimum splash duration with a `DEFAULT` constant (900ms)
- `client_task`, `injected_client_task`, `ClientTaskInjection`, and `ProbeEffect`
- `devtools` command, input schema, state snapshot, host state, panel config/state/message/effect reducer and effect runner, window spec, resource/operation view models, state-field collection/application helpers, and `DevtoolStateCatalog`, `DevtoolStateHost`, and `DevtoolsApp` host trait contracts
- `DialogDismiss` and `DialogRequest`
- `focus_trap` compatibility exports from `nive-ui` — `FocusDirection`, `direction_from_event`, `direction_from_keyboard_event`
- `OperationState`
- `platform::app_icon` — cross-platform app icon installer, accepting icon PNG bytes from the app layer
- `platform::file_picker` — `FileFilter`, `PickFileParams`, `pick_file`, `pick_files`, `pick_folder` (feature-gated behind `file-picker`, requires `rfd`)
- `ProbeCatalogEntry`, `ProbeMeta`, `ProbeMetaCatalog`, `ComposedProbeId`, `ProbeErrorScope`, generic probe catalog/list helpers, config parsing, injection store, probe injection by catalog entry or key/name, probe panel state/messages/effects/drafts, panel filtering/summary helpers, runtime config, and snapshots
- `RequestId` and `RequestCounter`
- `ScreenView` and `ScreenUpdate`
- `UserFacingError` and `UserFacingResult` (including the `Devtools` error kind for injected failures)

## Boundaries

- Keep concrete `app-core` and `app-models` clients out of this crate until a domain-specific extraction is intentionally planned.
- Keep app-specific probe metadata, env var ownership, concrete probe store application, and the local `ProbeCatalogEntry` implementation in `app-gui`; composed app/generated probe IDs, generated metadata aggregation, generic devtools/probe panel state, reducer/effect handling, window spec, injection store, and key/name lookup stay in this crate.
- Keep proc-macro declarations in `nive-runtime-derive`; runtime owns the generated target contracts. The derive crate generates paths against `nive_runtime::devtools` by default; app code may use `#[devtools_path("crate::dev::devtools")]` to redirect to an app-owned trait adapter.
- Keep app-domain fixture registration in `app-gui` through an app-owned fixture source trait (`DevtoolFixtureSource`); direct impls for `Vec<ProjectInfo>` and `Vec<Tag>` must stay with an app-owned trait because runtime-owned trait impls for those types violate the orphan rule. The derive attribute `devtools_path` allows app-gui to route generated code through its local adapter.
- `DevtoolValue` — generic fixture source trait for async resource devtools values; app-domain types (like `Vec<ProjectInfo>`) cannot implement this directly due to the orphan rule and use an app-owned fixture source adapter instead
- `DevtoolStateField` — generic state field collection/application trait for `AsyncState<T>` and `OperationState<C>`, used by `DevtoolStateCatalog` derive expansions
- Keep product brand assets (icon PNG bytes, brand theme tokens) in `app-gui`; the installer pattern in `nive-runtime::platform::app_icon` accepts generic icon bytes passed from the app.
- Keep widget-layer focus and overlay behavior in `nive-ui`; runtime may re-export stable helper APIs while lifecycle and shell code still consumes them.
- Keep concrete feedback types such as app toasts in `app-gui`; `ScreenUpdate` remains generic over the feedback payload.
- `AppPhase` and `SplashConfig` are generic runtime types; the concrete `AppReadiness` type alias (`AppPhase<UserFacingError, AppBootstrap, Clients>`) and `SPLASH_CONFIG` constant live in `app-gui` where the domain types are in scope.
- Prefer behavior-preserving moves from `app-gui` into this crate, with tests moved alongside the extracted types.
