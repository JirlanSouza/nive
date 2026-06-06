# ADR 0002: Split Global And Project Database Scopes

## Status

Accepted

## Context

RAG Studio stores both catalog-level data and project-local data.

Catalog-level data includes projects, tags, project/tag relationships, settings, and global rule library data.

Project-local data includes project rows, versions, documents, chunks, parse jobs, and project rule groups.

## Decision

Use two database scopes:

- `GlobalDb` for catalog-level data.
- `ProjectDb` for project-local data.

Repositories must make the database scope explicit through their `UnitOfWork` state. Project-owned queries must enforce isolation in SQL, either with `project_id` directly or through parent tables such as `documents`, `versions`, or `projects`.

## Consequences

- Repository methods should not rely on UI or service filtering as the only isolation boundary.
- Methods operating on project-owned rows should include `project_id` when the table has that column.
- Tables without `project_id` must be scoped through parent identifiers or joins.
- Schema changes must be added under the correct migration scope.
