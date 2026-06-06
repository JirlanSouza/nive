# Events And Async Architecture

## GUI Async Flow

`app-gui` async work should follow this flow:

1. A user action emits a screen `Message`.
2. The reducer updates state and returns an `Action`.
3. `action_runner` converts the action into `ScreenUpdate` or `iced::Task`.
4. `client/*` performs async service calls.
5. Results return as messages with request IDs when stale responses are possible.

Keep request correlation in state/reducer code, not in view composition.

## Core Events

`app-core` events decouple domain services. Examples include parse job lifecycle events and rule-change notifications.

Event handlers should:

- do the smallest useful amount of work
- log recoverable handler failures
- avoid panicking on normal failure paths
- publish follow-up events only after required persistence succeeds

## Long-Running Work

Use service-level orchestration for long-running or external work. UI code should observe state and display progress rather than owning backend processes.

## Error Handling

Async boundaries should convert domain errors into user-facing errors only at UI-facing client boundaries. Keep domain errors domain-specific in `app-core` and persistence errors in `app-database`.
