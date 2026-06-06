# Architecture Guidelines

Use this file as the architecture index. Read only the sub-docs relevant to the task:

- Workspace and active crates: `docs/agents/architecture/workspace.md`
- Rust/Iced UI architecture: `docs/agents/architecture/app-gui.md`
- Domain services and orchestration: `docs/agents/architecture/app-core.md`
- Persistence and repository boundaries: `docs/agents/architecture/app-database.md`
- Parser worker and Docling payloads: `docs/agents/architecture/parser-and-payload.md`
- Events, async tasks, and UI/service flow: `docs/agents/architecture/events-and-async.md`
- Active crate deep dives: `docs/agents/architecture/crates/README.md`
- Deprecated or historical areas: `docs/agents/architecture/legacy-areas.md`

Prefer existing ownership boundaries. Add a new abstraction only when it removes real duplication, clarifies a boundary, or matches an established local pattern.
