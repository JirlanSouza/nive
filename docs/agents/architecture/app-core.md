# App Core Architecture

## Role

`crates/app-core` owns domain services and orchestration. It coordinates app context, file system behavior, database repositories, events, and parser work.

## Main Areas

- `app_context`: active project state, global database manager, app directories, and shared app context.
- `project`: project catalog, project opening, project context, and project lifecycle services.
- `document`: document import, document listing, markdown preview, and document domain errors.
- `parsing`: parse service, parser worker trait, parse events, parse errors, and worker job DTOs.
- `rule`: hierarchy rule service, rule events, and rule domain errors.
- `tag`: tag service.
- `file_system`: project layout and file copying behavior.
- `app_event_bus` and `event_bus`: event publishing and handler contracts.

## Service Boundaries

- Services should accept IDs by reference (`&str`) when ownership is not needed.
- Services own orchestration, validation, transactions, and event publication.
- Repositories own SQL and persistence details.
- Models should remain in `app-models` when shared across layers.
- Use domain-specific errors and integrate them into `AppError` with `#[from]`.

## Transactions

Open database transactions in services when multiple repository operations must commit atomically or when an event should only publish after persistence succeeds.

Commit before publishing follow-up events unless the event is intentionally part of the transaction boundary.
