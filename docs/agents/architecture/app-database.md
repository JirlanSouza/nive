# App Database Architecture

## Role

`crates/app-database` owns SQLx persistence, migrations, database managers, repositories, and unit-of-work boundaries.

## Database Scopes

There are two database scopes:

- `GlobalDb`: catalog-level data such as projects, tags, project/tag assignments, settings, and global rule library tables.
- `ProjectDb`: project-local data such as project rows, versions, documents, chunks, parse jobs, and project rule groups.

Use `GlobalDbManager` and `ProjectDbManager` through the `DbManager` trait. Pass a `UnitOfWork` into repositories.

## Repositories

Repositories should:

- require a `UnitOfWork`
- keep SQL inside `app-database`
- return database models or persistence errors, not UI-facing types
- require `project_id` for project-owned tables that include a `project_id` column
- scope tables without `project_id` through parent tables or parent identifiers in SQL

Do not rely on service-side filtering as the only project isolation boundary.

## Migrations

Migrations live under:

- `crates/app-database/migrations/global`
- `crates/app-database/migrations/project`

Call out schema changes explicitly in PR notes and handoff summaries.

## Tests

Repository behavior should be tested in `app-database` when SQL semantics, joins, transactions, migrations, or project isolation change.
