# app-gui

## Role

`crates/app-gui` owns the Rust/Iced desktop application. It handles UI state, screen composition, platform UI bridges, visual primitives, and conversion of user actions into `app-core` service calls.

## Internal Modules

- `app`: top-level Iced application, routing, shell integration, and screen updates.
- `app_shell`: window policy, dialogs, toasts, keyboard navigation, and `ScreenUpdate`/`ScreenView`.
- `welcome_screen`: welcome flow, project catalog UI, project creation, selection, reducers, actions, and view composition.
- `workspace_screen`: active workspace screen boundary.
- `client`: GUI-facing async clients that wrap `app-core` services in `iced::Task`.
- `platform`: file picker and platform UI integrations.
- `theme`, `tokens`, `widgets`: visual system, reusable primitives, and composite widgets.
- `async_state`, `operation_state`, `request`: reusable UI state helpers for async resources and request correlation.

## Architectural Dependencies

Depends on:

- `app-core` for domain services.
- `app-models` for shared UI/domain data.
- `iced` for UI runtime and tasks.
- `rfd` for platform file picking.

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
- map service errors into user-facing errors at client boundaries

Do not:

- query databases directly
- move domain rules into UI code
- use legacy React/Tauri UI code as a design source
