# Lifecycle Contract

The contract checkpoint defines:

- `WindowSpec`, `WindowRole` and `WindowCardinality`
- `WindowCommand` and typed command rejection
- `CoreEvent`
- `CloseDecision` and `ExitDecision`
- read-only `WindowQuery`
- declarative `BootstrapSpec`

`WindowSpec::app()` and `WindowSpec::auxiliary()` provide the approved default
sizes and single-window cardinality.

`WindowRegistry` is keyed by `window::Id`, keeps multiple instances of the same
product window kind, and tracks the internal `Opening -> Open -> removed`
lifecycle. Kind-based lookup selects the most recently opened or focused
instance. Removing an interrupted opening also removes its handle, preventing
ghost windows from affecting cardinality and lifecycle decisions.

The application runner enforces `WindowCommand` against the registry and emits
typed `CommandRejected` events for missing specs, missing windows and opens
requested during exit. Single-cardinality opens focus the current
representative; multiple-cardinality specs create distinct instances.

Close requests for non-final app windows use `CloseDecision`. The runtime tracks
pending app-window closes so batched close requests cannot bypass the
final-window `ExitDecision`. Closing the last effective app window and explicit
exit requests use `ExitDecision`. Deferred decisions deliver their task messages
to the app and confirm automatically when the task finishes. Auxiliary windows
do not keep the process alive and are closed before accepted exit.

Bootstrap is an internal lifecycle rather than a product window. The runtime
owns attempt correlation, stale-result rejection, minimum duration, pending
success, retry, failure details and cancellation. Product code supplies a task
factory, result type, brand assets and copy through `BootstrapSpec`.

The app is created only after bootstrap succeeds. Closing the splash cancels
the controller, ignores late results and exits without calling application
hooks.
