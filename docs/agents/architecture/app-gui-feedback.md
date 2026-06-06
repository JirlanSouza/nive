# App GUI State And Operation Feedback

Use this guide when adding or refactoring UI feedback in `crates/app-gui`.

## State Model

- Use `AsyncState<T>` for async resources: lists, summaries, details, configuration, cached data, refreshes, stale data, and request-ID-based stale response handling.
- Use `OperationState<C>` for user-triggered commands: create, delete, open, save, update, import, export, submit, or any action started by a button, menu item, dialog confirmation, or inline control.
- Keep feedback scoped to the owning surface. Resource failures stay in the loading panel/section/field group. Operation failures stay in the dialog/form/row/control that launched the command.
- Use global notices only when no local surface can explain, retry, dismiss, or show diagnostic details for the failure.
- Do not use skeletons for command operations. Do not use button-level loading as the only feedback for cold-loading resources.

## Resources: `AsyncState<T>`

Pass full `AsyncState<T>` to subviews that need to distinguish loading, loaded, cached refresh, stale failure, and no-cache failure. Do not collapse nuanced resource props to only `value()` plus a separate error accessor.

| State | UI behavior |
| --- | --- |
| `Idle` | Empty or no-selection state when natural. |
| `Loading { value: None }` | Cold-load skeletons shaped like final content. |
| `Loading { value: Some(value) }` | Keep cached content and show a small neutral refresh indicator locally. |
| `Loaded(value)` | Normal content; empty values render empty states. |
| `Failed { value: Some(value), error }` | Keep stale content and show compact local stale feedback with retry/details. |
| `Failed { value: None, error }` | Show local load failure feedback with retry/details. |

Use `widgets::skeleton` for cold loading:

- `block()` for rectangular placeholders and avatars
- `rounded()` for small leading dots or icon placeholders
- `text_row()` for text placeholders
- `control(row![...])` for button, tag, or filter-like placeholders
- `card(content)` for card-shaped placeholders with a `SurfaceRole`

Skeletons should approximate final layout and reuse existing widget metrics, theme tokens, shape, padding, and gap values. Cached refreshes keep real content visible and add `LoadingIndicator::new().neutral().xs()` in the local header or command area.

## Operations: `OperationState<C>`

Model user-triggered commands with `OperationState<C>`. The context `C` should contain the minimum data needed to retry or scope the failure, such as a target ID. Empty context is fine when the surrounding draft/state already preserves the retry payload.

| State | UI behavior |
| --- | --- |
| `Idle` | Normal controls. |
| `Running { request_id, context }` | Show loading on the initiating control and disable conflicting controls. |
| `Failed { error, context }` | Keep the initiating surface open, show local error feedback, and preserve retry context. |

Use request IDs for operations that can receive stale async completions. `finish(request_id)` and `fail(request_id, error)` should mutate state only for the matching running request.

Operation loading should be targeted:

- primary command buttons use `.loading(is_running)` plus disabled guards
- cancel/close controls are disabled only when canceling conflicts with the running command
- inputs feeding the command are disabled while it runs
- nested operations disable only their related subtree
- user input is preserved while running or failed unless success intentionally resets the surface

## Error, Retry, And Dismiss

Choose feedback chrome by weight:

- full panel/section feedback for resource failures that replace missing content
- compact stale feedback above cached content when stale data remains usable
- `InlineAlert::new(error.summary()).danger()` for dialog or form command failures
- `ErrorStatusLine` for footer or inline form failures where full alert chrome is too heavy
- extra-small `InlineAlert` with `ErrorFeedbackActionRow::xs()` for nested compact operations

Show diagnostic details only when `UserFacingError::has_diagnostic_detail()` is true. Avoid duplicating error text: if a surface already has title/description plus a details action, do not repeat diagnostic detail as visible body text.

Retry re-runs the same resource load or command with preserved context. Dismiss clears only the owning error. Canceling a dialog clears operation errors owned by that dialog. Changing the failing input should clear stale operation errors when the old failure no longer applies. Success clears operation state and resets only the successful command's surface.

## Busy State

Use global or screen-level busy only for work that genuinely blocks unrelated actions. Prefer local running state for cached refreshes, secondary panel loads, nested form operations, and commands scoped to one item. Add separate readiness checks when a specific action needs stricter guards than the general busy state.

## Testing

Use reducer tests for:

- resource cold load, cached refresh, stale failure, no-cache failure, empty loaded values, and stale response handling
- operation success, failure, retry, dismiss, clearing behavior, and stale request handling
- failures staying local instead of becoming global notices
- busy/readiness rules when loading should or should not block unrelated actions

Use widget or primitive tests only for reusable visual policy, such as skeleton metrics, colors, surface roles, or deterministic style rules.

Focused checks for `app-gui` feedback changes: `just fmt` and `cargo test -p app-gui`.

## Checklist

For a resource: store `AsyncState<T>`, return request IDs when needed, pass full state to feedback views, render shaped skeletons for cold load, preserve cached content during refresh/failure, keep errors local, and test reducer transitions/request correlation.

For an operation: store `OperationState<C>`, keep minimal retry context in `C`, show loading on the initiating control, disable only conflicting controls, preserve input/context for retry, keep errors local, and test success, failure, stale request handling, retry, dismiss, and clearing behavior.
