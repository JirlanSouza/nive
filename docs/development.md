# Development Guide

This guide describes the active development workflow for RAG Studio. It is intended for contributors working on the current Rust/Iced application stack.

## Active Stack

Current development targets:

- `crates/app-gui`: Rust/Iced desktop UI.
- `crates/app-core`: domain services and orchestration.
- `crates/app-models`: shared domain models.
- `crates/app-database`: SQLite persistence and migrations.
- `crates/document-parser-worker`: Rust manager for the Python parser process.
- `crates/docling-payload-core`: Docling payload read/write and compatibility.
- `crates/docling-payload-py`: Python binding for payload generation.
- `document-parser`: Python parser implementation.

Legacy areas remain in the repository for reference, but they are not part of the normal active workflow:

- `app-ui`
- `crates/app-tauri`

## Requirements

Install:

- Rust stable and Cargo.
- `just`.
- `uv` for the Python document parser environment.
- Python 3.11 or newer.

Node.js and pnpm are only needed for legacy `app-ui` workflows.

## Setup

Sync the parser environment:

```sh
just parser-sync
```

List available commands:

```sh
just
```

## Development

Run the active desktop app:

```sh
just dev
```

Run the app with an injected UI error scenario:

```sh
just dev-error create_project
```

List UI error scenarios:

```sh
just dev-error-list
```

## Verification

Use root `just` recipes for normal validation:

```sh
just fmt-check
just check
just rust-test
```

Run the broader active test suite, including parser checks:

```sh
just test
```

Run the CI-equivalent local entrypoint:

```sh
just ci
```

Focused checks are available for common work areas:

```sh
just check-gui
just check-core
just check-db
just test-gui
just test-core
just test-db
just test-parser-worker
just test-payload-core
just payload-py-test
```

## Change Workflow

For behavior changes:

1. Identify the affected layer.
2. Add or update the focused test closest to the behavior.
3. Implement the smallest scoped change.
4. Run focused verification.
5. Run broader active checks when the change crosses crate boundaries.

Use these default test surfaces:

- UI state and request behavior: `app-gui` reducer tests.
- Domain orchestration: `app-core` service tests.
- SQL behavior and isolation: `app-database` repository tests.
- Parser process behavior: `document-parser-worker` tests.
- Payload compatibility and views: `docling-payload-core` tests.
- Python binding behavior: `payload-py-test`.

## Architecture Decisions

Architecture Decision Records live in `docs/adr/`.

Read the relevant ADR before changing:

- active UI/application stack
- database scope or project isolation
- parser process protocol
- payload format or compatibility behavior
