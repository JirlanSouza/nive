# app-models

## Role

`crates/app-models` owns shared data structures used across UI, core services, database repositories, parser work, and payload-facing flows.

## Internal Modules

- `project`: project, project layout, version, project info, and project summary types.
- `document`: document status, document model, parse config, app settings, and outline items.
- `chunk`: chunk, chunk metadata, sync status, and index job models.
- `parse_job`: parse job model and status.
- `rule`: hierarchy rule, rule group, rule scope, pattern tokens, and reorder direction.
- `search`: search-facing shared models.

## Architectural Dependencies

Depends on:

- serialization/time/value crates only.

Used by:

- `app-gui`
- `app-core`
- `app-database`
- `document-parser-worker`

Must not depend on:

- `app-core`
- `app-database`
- `app-gui`
- parser implementation crates

## Workflow

Model changes should start from the shared contract:

1. identify which layers serialize, persist, or display the model
2. update constructors or defaults when required
3. update repository mappings and service logic in downstream crates
4. add tests in the layer where behavior changes

## Testing

- Add model-level tests only for parsing, defaults, serialization, or invariants owned by the model.
- Most model changes need downstream tests in `app-core`, `app-database`, or `app-gui`.
- Focused command: `cargo test -p app-models`.

## Rules

Do:

- keep models UI-agnostic and persistence-agnostic
- use serde naming intentionally for external/UI-facing shape
- keep constructors simple and deterministic

Do not:

- put service orchestration or SQL behavior here
- add dependencies on application crates
- hide project isolation logic inside shared models
