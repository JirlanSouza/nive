# App GUI Architecture

## Role

`crates/app-gui` owns the Rust/Iced desktop UI. It should translate user intent into messages, reducer updates, actions, and service calls without embedding domain persistence logic.

## Screen Flow

For UI-facing changes, map behavior through the current screen architecture:

- `Message`: user input or async event entering the screen.
- `State/reducer`: pure state transition and validation.
- `Action`: effect requested by state.
- `Screen`: converts messages and actions into `ScreenUpdate` or `iced::Task`.
- `action_runner`: converts reducer actions into tasks, toasts, dialogs, or screen outcomes.
- `client/*`: async bridge from GUI actions to `app-core` services and IO.
- `ui/*`: view composition from state.

Prefer reducer coverage for observable state transitions and request correlation behavior.

## UI Structure

Use existing modules before adding new ones:

- `app_shell`: shell state, window policy, dialogs, toasts, keyboard navigation, and screen update/view helpers.
- `welcome_screen`: welcome flow, project catalog UI, new-project flow, selection, and resource loading states.
- `workspace_screen`: active workspace flow.
- `client`: GUI-facing service clients that return `iced::Task`.
- `platform`: platform-specific UI integrations such as file picking.
- `theme`, `tokens`, `widgets`: visual system and reusable primitives.

## Boundaries

- Keep business behavior in `app-core`.
- Keep database access out of `app-gui`.
- Keep `State/reducer` logic deterministic when possible.
- Use `AsyncState`, `OperationState`, and request IDs for resource loading and stale-response handling.
- Prefer existing widgets and theme tokens over ad hoc styling.
