# Database Guidelines

## Data Isolation

`app-database` has two database scopes:

- `GlobalDb`: catalog-level data such as projects, tags, project/tag assignments, settings, and global rule library tables.
- `ProjectDb`: project-local data such as project rows, versions, documents, chunks, parse jobs, and project rule groups.

For `GlobalDb` repositories, require `project_id` only for methods that operate on project-scoped relations such as `project_tags`.

For `ProjectDb` repositories, methods that read or mutate project-owned rows should require `project_id` when the target table has a `project_id` column. Enforce isolation directly in SQL:

```sql
WHERE project_id = ?
```

When a project-owned table has no `project_id` column, scope through its parent table or parent identifier in SQL. Prefer joins or `EXISTS` checks against `documents`, `versions`, or `projects` over caller-side filtering.

Do not rely on callers, UI state, or service-level filtering as the only isolation boundary.

## Migrations

SQLite migrations live under `crates/app-database/migrations/`.

Call out schema changes and new migrations explicitly in PR notes or handoff summaries.
