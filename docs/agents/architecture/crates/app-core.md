# app-core

## Role

`crates/app-core` owns domain services and orchestration. It coordinates app context, project lifecycle, document work, parsing, rules, tags, filesystem behavior, event publication, and repository calls.

## Internal Modules

- `app_context`, `app_context_builder`: shared app state, app dirs, global DB manager, and active project context.
- `project`: project catalog, project opening, project context, project DTOs, and project errors.
- `document`: document import/listing, markdown preview, markdown rendering helpers, and document errors.
- `parsing`: parse service, parse worker trait, parse events, parse worker errors, and parse job DTOs.
- `rule`: rule service, rule events, and rule errors.
- `tag`: tag service.
- `file_system`: project directory layout and file copy behavior.
- `app_event_bus`, `event_bus`: event bus implementation and traits.
- `services_builder`: service graph construction.

## Architectural Dependencies

Depends on:

- `app-models` for shared domain data.
- `app-database` for repository and DB manager abstractions.
- `docling-payload-core` read support for parsed payloads.

Used by:

- `app-gui`
- `document-parser-worker` for core traits/events and parser job types

Must not depend on:

- `app-gui`
- legacy shell/UI crates

## Workflow

Service changes should usually:

1. validate inputs and active project context
2. open a pool or transaction through a DB manager
3. call repositories
4. commit if multiple changes are atomic
5. publish domain events after required persistence succeeds

Use `&str` for IDs when the service does not need ownership.

## Testing

- Use service tests for orchestration, event behavior, transactions, and domain errors.
- Keep focused tests near the module they cover.
- Focused command: `cargo test -p app-core`.
- Active workspace command: `just rust-test`.

## Rules

Do:

- use domain-specific errors and integrate them into `AppError` with `#[from]`
- keep SQL out of services
- commit persistence before publishing follow-up events unless a different boundary is intentional
- keep parser orchestration behind `ParseWorker`

Do not:

- expose database details to `app-gui`
- use generic `AppError` variants when a domain error communicates the failure better
- publish events before persistence required by event handlers exists
