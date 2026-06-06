# app-database

## Role

`crates/app-database` owns SQLite persistence. It defines database managers, unit-of-work boundaries, SQLx repositories, migrations, and database errors.

## Internal Modules

- `manager`: `DbManager`, `UnitOfWork`, DB state markers, and executor abstraction.
- `pool`: global/project pool builders and migration runners.
- `repositories`: SQL access for projects, tags, documents, chunks, parse jobs, rules, and project catalog data.
- `error`: `DatabaseError` and SQLx error mapping.
- `migrations/global`: global catalog schema.
- `migrations/project`: project-local schema.

## Architectural Dependencies

Depends on:

- `app-models` for persisted domain models.
- `sqlx` for SQLite access.

Used by:

- `app-core` services.

Must not depend on:

- `app-core`
- `app-gui`
- parser worker crates

## Workflow

Repository changes should:

1. choose `GlobalDb` or `ProjectDb`
2. accept a `UnitOfWork`
3. enforce project isolation in SQL for project-owned data
4. map rows into `app-models`
5. return `DatabaseError`

Schema changes should add migrations under the correct scope and be called out explicitly.

## Testing

- Add repository tests for query semantics, joins, project isolation, transactions, and migrations.
- Focused command: `cargo test -p app-database`.
- Active workspace command: `just rust-test`.

## Rules

Do:

- keep SQL in repositories
- pass `project_id` for tables that have a `project_id` column
- scope tables without `project_id` through parent tables or parent IDs in SQL
- prefer structured row mapping over ad hoc caller-side filtering

Do not:

- rely on UI or service filtering as the only project isolation boundary
- expose SQLx row types outside `app-database`
- add repository methods that silently cross project boundaries
