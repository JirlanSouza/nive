# app-gui

## Role

`crates/app-gui` owns the Rust/Iced desktop application. It handles screen composition, platform UI bridges, app-specific devtools wiring, product-aware widgets, and conversion of user actions into `app-core` service calls.

## Internal Modules

- `app`: implements the Nive `Application` contract, configures product bootstrap assets/task, receives the successful bootstrap value in `init`, and owns product routing and screen updates.
- `app_shell`: app-specific logical window kinds, product window dimensions/icon adapters, and transitional visual toast composition.
- `welcome_screen`: welcome flow, project catalog UI, project creation, selection, reducers, actions, and view composition.
- `workspace_screen`: active workspace screen boundary.
- `client`: GUI-facing async clients that wrap `app-core` services in `iced::Task` and declare dev probe keys plus product-owned probe metadata through `devtools_derive::runtime_client`.
- `platform`: app icon bytes plus thin calls/re-exports over `nive-runtime::platform` for app icon installation and the currently used folder picker.
- `dev`: app-specific devtools host adapter for app-domain fixture sources, probe env ownership, probe store application, and UI composition over runtime devtools host/panel state. Implements `DevtoolsApp` for `RagStudioApp` to provide snapshot and command application hooks through the runtime trait contract.
- `theme`: compatibility re-export facade for `nive-ui::theme`.
- `widgets`: product-aware composite widgets plus a thin compatibility surface over extracted `nive-ui` primitives. Private primitive facades have been collapsed into direct `nive_ui` re-exports; only product assets such as `brand_mark` remain under local primitives.
- `ui_state`: compatibility facade for runtime-owned async state, operation state, request IDs, and user-facing errors.

## Architectural Dependencies

Depends on:

- `app-core` for domain services.
- `app-models` for shared UI/domain data.
- `nive-runtime` for the application runner, runtime updates, theme and window lifecycle, shared UI state, dialog dismissal and keyboard routing, devtools model/panel contracts, and generic probe runtime behavior.
- `nive-ui` for design tokens, theme contracts, dialog hosting, focus trapping, and reusable visual primitives.
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
- keep reducers deterministic when possible
- preserve cached async content where existing `AsyncState` patterns do so
- follow `docs/agents/architecture/app-gui-feedback.md` for new resource and operation feedback
- map service errors into user-facing errors at client boundaries
- keep product window assets, logical window enum, window titles, and app-specific dimensions local while declaring windows through `ApplicationConfig` and requesting transitions through `WindowCommand`
- provide dialog content and reducer messages through `ScreenView`; use Nive's
  dialog decoration and keyboard navigation instead of local modal or focus
  infrastructure
- keep visual toast composition local while using runtime `ToastState`, `ToastRequest`, `ToastMessage`, and toast expiration/tick behavior
- keep devtools host state, panel reducers/effect handling, command result recording, probe-effect branching, sidecar window opening/title/spec policy, generic state-field mutation helpers, injected failure errors, and generic window specs in `nive-runtime`; app-gui should only provide app icon adaptation, view adapter, app-domain fixture source adapter, probe env ownership, Iced view composition, and the `DevtoolsApp` impl for snapshot/command routing until lifecycle extraction
- keep client probe declarations on client impls with `devtools_derive::runtime_client`; `UiErrorProbe` should preserve app-owned metadata such as bootstrap and wrap runtime `ComposedProbeId` values rather than manually modeling client probe variants

Do not:

- query databases directly
- move domain rules into UI code
- use legacy React/Tauri UI code as a design source
