# Workspace Architecture

## Active Workspace

Nive is a Rust/Iced desktop application framework. Active Cargo workspace members are:

- `crates/nive-ui`: visual design system (tokens, theme, widgets, icons)
- `crates/nive-runtime`: application lifecycle, window management, feedback, devtools
- `crates/nive-runtime-derive`: proc macros for devtools
- `crates/nive`: umbrella crate that re-exports nive-ui and nive-runtime
- `crates/nive-cli`: CLI for scaffolding and icon management (binary name: `nive`)

Use the root `justfile` as the source of truth for active development commands.

## Command Boundary

Prefer these root commands for active work:

- `just fmt`
- `just fmt-check`
- `just check`
- `just lint`
- `just test`
- `just doc`
- `just build`
- `just release`

Icon management commands:

- `just icons-list`
- `just icons-sync`
- `just icons-check`
- `just icons-add <variant> <lucide-name>`

App scaffolding:

- `just create-app <name>`

Run package-specific `cargo` commands only for focused verification while iterating.

## Package Roles

- `nive-ui`: shared visual design system for tokens, semantic theme contracts, reusable UI primitives, and icon management.
- `nive-runtime`: shared app runtime foundation for application/update contracts, `Resource`/`Operation` async state, request IDs, user-facing errors, lifecycle contracts, and optional devtools simulator (feature `devtools`). The `Inspect` trait + derive walk app state to discover simulatable fields; `SimulableState` exposes snapshots, explicit capabilities, and simulator actions.
- `nive-runtime-derive`: proc macro `#[derive(Inspect)]` that generates recursive `Inspect::inspect` implementations for app state structs.
- `nive`: umbrella crate that re-exports `nive-ui` and `nive-runtime` for convenient app development.
- `nive-cli`: CLI binary (`nive`) for scaffolding new apps and managing Lucide icons.
