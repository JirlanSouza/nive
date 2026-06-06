# Repository Guidelines

## Active Stack

`rag-studio` is centered on the Rust/Iced application stack:

- `crates/app-gui`
- `crates/app-core`
- `crates/app-models`
- `crates/app-database`
- parser and payload crates when a task touches document parsing

Deprecated UI shell areas are out of scope unless the user explicitly asks for migration, compatibility, or historical comparison.

## Context Docs

Read only the context needed for the task:

- Architecture and module boundaries: `docs/agents/architecture.md`
- Rust style and naming conventions: `docs/agents/rust-style.md`
- Database isolation and migrations: `docs/agents/database.md`
- Errors and logging: `docs/agents/errors-and-logging.md`
- Testing rules and commands: `docs/agents/testing.md`
- Commits and PRs: `docs/agents/commits-and-prs.md`

## Agent Working Rules

- Clarify before acting when the request is ambiguous. Do not guess hidden intent.
- Convert vague requests into a verifiable target before execution.
- Edit surgically. Change only the lines or sections needed for the verified target.
- Prefer existing project patterns over new abstractions.
- Add or update tests for behavior changes.
- Keep output minimal and task-focused.
