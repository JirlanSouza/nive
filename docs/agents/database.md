# Database Guidelines

## Data Isolation

All database repository methods must explicitly require a `project_id` parameter.

SQL queries must enforce project isolation directly at the query level:

```sql
WHERE project_id = ?
```

Do not rely on callers, UI state, or service-level filtering as the only isolation boundary.

## Migrations

SQLite migrations live under `crates/app-database/migrations/`.

Call out schema changes and new migrations explicitly in PR notes or handoff summaries.
