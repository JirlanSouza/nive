# ADR 0001: Use Rust/Iced As The Active Desktop Stack

## Status

Accepted

## Context

The repository still contains legacy React/Tauri code, but current development targets the Rust/Iced desktop application in `crates/app-gui`.

The active Rust crates are coordinated by the root `justfile`. `crates/app-tauri` is kept in the repository for reference, but it is not an active Cargo workspace member.

## Decision

Use `crates/app-gui` as the active desktop UI stack.

Use `app-core`, `app-models`, and `app-database` as the active application, domain, model, and persistence layers.

Treat `app-ui` and `crates/app-tauri` as legacy or historical reference areas unless a task explicitly targets migration, compatibility, or historical comparison.

## Consequences

- New UI work targets `crates/app-gui`.
- Active development commands use `just` recipes.
- The repository may keep legacy code for reference without making it part of normal checks.
- Architectural guidance should use the Rust/Iced stack as the source of truth.
