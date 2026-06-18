# app-gui

## Role

`crates/app-gui` owns the Rust/Iced desktop application. It handles screen composition, platform UI bridges, app-specific devtools wiring, product-aware widgets, and conversion of user actions into `app-core` service calls.

## Internal Modules

- `app`: implements the Nive `Application` contract, configures product bootstrap assets/task, receives the successful bootstrap value in `init`, and owns product routing and screen updates.
- `app_shell`: app-specific logical window kinds, product window dimensions, shell messages, and icon adapters.
- `welcome_screen`: welcome flow, project catalog UI, project creation, selection, reducers, actions, and view composition.
- `workspace_screen`: active workspace screen boundary.
- `client`: GUI-facing async clients that wrap `app-core` services in Nive runtime tasks and declare dev probe keys plus product-owned probe metadata through `nive_runtime::runtime_client`.
- `platform`: product app-icon installation and bytes.
- `dev`: app-domain fixture adapters, probe env/store ownership and the `DevtoolsApp` hooks that provide state snapshots plus command/probe application to the runtime.
- `widgets`: product-aware composite widgets and brand assets only; generic primitives are imported directly from `nive-ui`.
- `clock`: product relative-time formatting and current Unix time.

## Architectural Dependencies

Depends on:

- `app-core` for domain services.
- `app-models` for shared UI/domain data.
- `nive-runtime` for the application runner, runtime updates, theme and window lifecycle, shared UI state, dialog dismissal and keyboard routing, devtools model/panel contracts, and generic probe runtime behavior.
- `nive-ui` for design tokens, theme contracts, dialog hosting, focus trapping, reusable visual primitives, renderer types, and low-level widget APIs.

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

Startup readiness is owned by Nive. `app` supplies
`BootstrapSpec<AppBootstrap>` with a repeatable task factory, brand assets and
copy. `RagStudioApp::init` receives the successful bootstrap value, creates
`Clients` and starts the Welcome load; no product state or product window exists
before success.

## Testing

- Use reducer tests for observable state transitions and request correlation.
- Use widget/theme tests for deterministic style or layout policy behavior.
- Focused command: `cargo test -p app-gui`.
- Active workspace command: `just rust-test`.

## Rules

Do:

- use existing widgets and theme tokens
- import generic runtime and visual contracts directly from `nive-runtime` and `nive-ui`
- keep reducers deterministic when possible
- preserve cached async content where existing `AsyncState` patterns do so
- follow `docs/agents/architecture/app-gui-feedback.md` for new resource and operation feedback
- map service errors into user-facing errors at client boundaries
- keep product window assets, logical window enum, window titles, and app-specific dimensions local while declaring windows through `ApplicationConfig` and requesting transitions through `WindowCommand`
- provide dialog content and reducer messages through `ScreenView`; use Nive's
  dialog decoration and keyboard navigation instead of local modal or focus
  infrastructure
- emit toasts through `AppUpdate::toast`; Nive owns toast state, expiration, hover pause, dismiss, and applies the `ToastHost` overlay automatically to app-role windows
- keep devtools host state, generic view, panel reducers/effect handling, command result recording, auxiliary window lifecycle/title/spec/shortcut, generic state-field mutation helpers, injected failure errors, and generic window specs in `nive-runtime`; app-gui provides only app-domain fixture adapters, probe metadata/env/store ownership and the `DevtoolsApp` snapshot/command/probe hooks
- keep client probe declarations on client impls with `nive_runtime::runtime_client`; `UiErrorProbe` should preserve app-owned metadata such as bootstrap and wrap runtime `ComposedProbeId` values rather than manually modeling client probe variants

Do not:

- query databases directly
- move domain rules into UI code
- use legacy React/Tauri UI code as a design source
