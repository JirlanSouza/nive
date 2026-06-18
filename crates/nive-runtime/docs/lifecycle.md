# Lifecycle Contract

The contract checkpoint defines:

- `WindowSpec`, `WindowRole` and `WindowCardinality`
- `WindowCommand` and typed command rejection
- `CoreEvent`
- `CloseDecision` and `ExitDecision`
- read-only `WindowQuery`
- declarative `BootstrapSpec`

`WindowSpec::app()` and `WindowSpec::auxiliary()` provide the approved default
sizes and single-window cardinality. Runtime registry enforcement, opening
cleanup, close/exit handshakes and bootstrap state transitions are implemented
in later slices.
