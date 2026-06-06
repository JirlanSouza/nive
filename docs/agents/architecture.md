# Architecture Guidelines

## Active Crates

`rag-studio` is a Cargo workspace organized around the current Rust/Iced app stack:

- `crates/app-gui`: Iced desktop UI, screens, state reducers, tasks, theme, tokens, and widgets.
- `crates/app-core`: Domain services, event bus, file system behavior, and parsing orchestration.
- `crates/app-models`: Shared domain models and types.
- `crates/app-database`: SQLite persistence layer, SQLx repositories, and migrations.
- `crates/document-parser-worker`: Child-process manager for the Python document parser.
- `crates/docling-payload-core`: Rkyv payload read/write for Docling documents.
- `crates/docling-payload-py`: PyO3 binding for `docling-payload-core`.
- `crates/xtask`: Project automation tasks.

## Module Boundaries

Map UI-facing changes through the current `app-gui` architecture:

- `Message`: user input or async event entering a screen.
- `State/reducer`: pure state transition and validation.
- `Action`: effect requested by state.
- `Screen`: converts actions into `iced::Task`.
- `tasks/*`: async bridge to services and IO.
- `app-core`: domain behavior and orchestration.
- `app-models`: shared domain types.
- `app-database`: persistence, migrations, and repository behavior.
- `theme`, `tokens`, `widgets`: visual system and reusable UI primitives.

Prefer existing ownership boundaries. Add a new abstraction only when it removes real duplication, clarifies a boundary, or matches an established local pattern.

## Historical Areas

Deprecated UI shell areas should not be used as normal design sources. Read them only when the user explicitly asks for migration, compatibility, historical comparison, or salvage work.

Treat `.kiro/specs` as historical unless the user asks to work from a specific spec.
