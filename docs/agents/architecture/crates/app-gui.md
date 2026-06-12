# app-gui

## Role

`crates/app-gui` owns the Rust/Iced desktop application. It handles screen composition, platform UI bridges, app-specific devtools wiring, product-aware widgets, and conversion of user actions into `app-core` service calls.

## Internal Modules

- `app`: top-level Iced application, routing, shell integration, and screen updates. Uses `NiveApplication`, `AppPhase` (via local `AppReadiness` type alias), `SplashConfig`, and `minimum_splash_duration_task` from `nive-runtime` for the daemon facade and splash/boot lifecycle mechanics.
- `app_shell`: app-specific logical window kinds, product window dimensions/icon adapters, concrete dialog/toast hosting, and keyboard navigation over runtime app-shell contracts.
- `bootstrap_screen`: startup splash and startup failure feedback before app services are available.
- `welcome_screen`: welcome flow, project catalog UI, project creation, selection, reducers, actions, and view composition.
- `workspace_screen`: active workspace screen boundary.
- `client`: GUI-facing async clients that wrap `app-core` services in `iced::Task` and declare dev probe keys plus product-owned probe metadata through `devtools_derive::runtime_client`.
- `platform`: app icon bytes plus thin calls/re-exports over `nive-runtime::platform` for app icon installation and the currently used folder picker.
- `focus_trap` compatibility facade re-exported from `nive-runtime`, backed by `nive-ui::focus_trap`.
- `dev`: app-specific devtools host adapter for translating runtime window specs into app shell window policies, app-domain fixture sources, and UI composition over runtime devtools host/panel state. Implements `DevtoolsApp` for `RagStudioApp` to provide snapshot and command application hooks through the runtime trait contract.
- `theme`: compatibility re-export facade for `nive-ui::theme`.
- `widgets`: product-aware composite widgets plus a thin compatibility surface over extracted `nive-ui` primitives. Private primitive facades have been collapsed into direct `nive_ui` re-exports; only product assets such as `brand_mark` remain under local primitives.
- `ui_state`: compatibility facade for runtime-owned async state, operation state, request IDs, and user-facing errors.

## Architectural Dependencies

Depends on:

- `app-core` for domain services.
- `app-models` for shared UI/domain data.
- `nive-runtime` for shared UI state, app-shell/window contracts, devtools model/panel contracts, and generic probe runtime behavior.
- `nive-ui` for design tokens, theme contracts, focus helpers, and reusable visual primitives.
- `iced` for UI runtime and tasks.

Used by:

- the active desktop binary and `just dev` workflow.

Must not depend on:

- `app-database`
- parser implementation crates directly
- legacy `app-ui` or `crates/app-tauri`

## Workflow

UI behavior should usually flow through:

1. `Message`
2. `State/reducer`
3. `Action`
4. `action_runner`
5. `client/*`
6. `app-core` service
7. result message with request ID when stale responses are possible

Keep view files focused on composition from state. Keep service calls out of views.

Startup readiness is owned by `app` over runtime lifecycle primitives. The Welcome window may be opened as the initial host window, but its content must route through `AppPhase::Booting`, `AppPhase::BootFailed`, and `AppPhase::Ready`: render `bootstrap_screen` until bootstrap succeeds, create `Clients` only on success, and render/load `welcome_screen` only after `Ready`. The minimum splash duration is enforced through `SplashConfig`, `AppPhase`, and `minimum_splash_duration_task`, not through a local session struct. Fast successful bootstraps wait for the splash gate before transitioning; failures surface immediately.

## Testing

- Use reducer tests for observable state transitions and request correlation.
- Use widget/theme tests for deterministic style or layout policy behavior.
- Focused command: `cargo test -p app-gui`.
- Active workspace command: `just rust-test`.

## Rules

Do:

- use existing widgets and theme tokens
- keep reducers deterministic when possible
- preserve cached async content where existing `AsyncState` patterns do so
- follow `docs/agents/architecture/app-gui-feedback.md` for new resource and operation feedback
- map service errors into user-facing errors at client boundaries
- keep product window assets, logical window enum, window titles, and app-specific dimensions local while using runtime `WindowSpec`, `WindowRegistry`, `WindowHandle`, and `open_window` for generic mechanics
- keep devtools host state, panel reducers/effect handling, generic state-field mutation helpers, injected failure errors, and generic window specs in `nive-runtime`; app-gui should only provide the transitional shell policy translation, view adapter, app-domain fixture source adapter, probe env ownership, Iced view composition, and the `DevtoolsApp` impl for snapshot/command routing until lifecycle extraction
- keep client probe declarations on client impls with `devtools_derive::runtime_client`; `UiErrorProbe` should preserve app-owned metadata such as bootstrap and wrap runtime `ComposedProbeId` values rather than manually modeling client probe variants

Do not:

- query databases directly
- move domain rules into UI code
- use legacy React/Tauri UI code as a design source
