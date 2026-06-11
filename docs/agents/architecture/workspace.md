# Workspace Architecture

## Active Workspace

`rag-studio` is centered on the Rust/Iced application stack. Active Cargo workspace members are:

- `crates/app-gui`
- `crates/app-core`
- `crates/app-models`
- `crates/app-database`
- `crates/document-parser-worker`
- `crates/docling-payload-core`
- `crates/docling-payload-py`
- `crates/nive-runtime`
- `crates/nive-ui`
- `crates/app-gui-devtools-derive`
- `crates/xtask`

Use the root `justfile` as the source of truth for active development commands. Its `rust-packages` list defines the Rust crates used by active checks, tests, builds, and formatting.

## Command Boundary

Prefer these root commands for active work:

- `just fmt`
- `just fmt-check`
- `just check`
- `just lint`
- `just rust-test`
- `just test`
- `just dev`

Run package-specific `cargo` commands only for focused verification while iterating.

## Package Roles

- `app-gui`: desktop UI, user interaction, Iced tasks, screens, product-aware widgets, and app-specific client/probe wiring.
- `app-gui-devtools-derive`: proc macros for app devtools state, operation, probe catalog, and runtime client probe declarations.
- `nive-runtime`: shared app runtime foundation for UI state, operation state, request IDs, user-facing errors, devtools model contracts, and generic probe runtime behavior.
- `nive-ui`: shared visual design system for tokens, semantic theme contracts, and reusable UI primitives as they are extracted.
- `app-core`: domain services, app context, event bus integration, file system behavior, and parser orchestration.
- `app-models`: shared domain models passed between app layers.
- `app-database`: SQLx repositories, database managers, migrations, and unit-of-work boundaries.
- `document-parser-worker`: Rust manager for the Python parser child process.
- `docling-payload-core`: Rkyv payload construction, validation, read/write, and Docling document views.
- `docling-payload-py`: Python binding for `docling-payload-core`.
- `xtask`: project automation.
