# document-parser-worker

## Role

`crates/document-parser-worker` manages the Python document parser child process. It bridges `app-core` parse jobs/events to a parser process over NDJSON stdio.

## Internal Modules

- `worker`: public worker implementation that satisfies the `app-core` parse worker boundary.
- `manager`: process lifecycle state machine, job queue handling, command dispatch, and parser event handling.
- `child_process`: runtime process abstraction for shell/stdin/stdout behavior.
- `protocol`: parser command and event protocol types.
- `reader`: stderr/stdout reader behavior and parser event parsing.
- `error`: parser worker error type.

## Architectural Dependencies

Depends on:

- `app-core` for parse worker traits and parse events.
- `app-models` for parse configuration and job data.
- Tauri shell runtime APIs for process management.

Used by:

- app service construction as the concrete parse worker.

Must not depend on:

- `app-gui`
- `app-database`
- `docling-payload-py`

## Workflow

Parser worker changes should usually:

1. update protocol types if parser messages changed
2. update manager state transitions
3. keep process IO isolated behind runtime/process abstractions
4. publish parse events through `app-core` event contracts
5. add worker/manager tests for process lifecycle behavior

## Testing

- Use worker tests for spawn, queue, capabilities, restart, and event behavior.
- Parser integration should be verified through parser workflows when protocol changes.
- Focused command: `cargo test -p document-parser-worker`.

## Rules

Do:

- keep NDJSON protocol handling explicit
- log parser lifecycle failures with enough IDs to debug
- preserve queue and restart semantics when changing lifecycle code

Do not:

- put UI behavior in the worker
- call database repositories from this crate
- bypass `app-core` parse events for domain updates
