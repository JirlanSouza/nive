# Repository Guidelines

## Active Stack

Nive is centered on the Rust/Iced framework stack:

- `crates/nive-core`: shared presentation and interaction contracts (zero dependencies)
- `crates/nive-ui`: visual design system
- `crates/nive-runtime`: application lifecycle
- `crates/nive-runtime-derive`: proc macros
- `crates/nive`: umbrella crate
- `crates/nive-cli`: scaffolding and icon-management CLI

## Context Docs

Read only the context needed for the task:

- Architecture and module boundaries: `docs/agents/architecture.md`
- Rust style and naming conventions: `docs/agents/rust-style.md`
- Testing rules and commands: `docs/agents/testing.md`
- Commits and PRs: `docs/agents/commits-and-prs.md`

## Agent Working Rules

- Clarify before acting when the request is ambiguous. Do not guess hidden intent.
- Convert vague requests into a verifiable target before execution.
- Edit surgically. Change only the lines or sections needed for the verified target.
- Prefer existing project patterns over new abstractions.
- Add or update tests for behavior changes.
- Keep output minimal and task-focused.
