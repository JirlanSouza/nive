# Workspace Architecture

## Active Workspace

Nive is a Rust/Iced desktop application framework. Active Cargo workspace members are:

- `crates/nive-core`: neutral presentation and interaction contracts shared by nive-ui and nive-runtime (zero dependencies)
- `crates/nive-ui`: visual design system (tokens, theme, widgets, icons)
- `crates/nive-runtime`: application lifecycle, window management, feedback, devtools
- `crates/nive-runtime-derive`: proc macros for devtools
- `crates/nive-workbench`: fixed-region professional desktop shell built from nive-ui primitives, with optional runtime adapters
- `crates/nive`: umbrella crate that re-exports nive-ui and nive-runtime, and exposes workbench APIs through `nive::workbench` plus prelude tiers
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
- `just doc-check`
- `just examples-check`
- `just example-dev <example>`
- `just widget-gallery-dev`
- `just widget-gallery-devtools`
- `just scaffold-smoke`
- `just package-check`
- `just readiness`
- `just build`
- `just release`

Icon management commands:

- `just icons-list`
- `just icons-sync`
- `just icons-check`
- `just icons-add-symbol <variant> <provider-ref>`
- `just icons-set-role <role-name> <provider-ref>`

App scaffolding:

- `just create-app <name>`

Standalone example development:

- `just example-dev <example>` runs `examples/<example>` with terminal-triggered reload.
- `just widget-gallery-dev` runs the widget gallery without devtools.
- `just widget-gallery-devtools` runs the widget gallery with devtools explicitly enabled.

Run package-specific `cargo` commands only for focused verification while iterating.

## Package Roles

- `nive-core`: neutral presentation contracts (`ErrorPresentation`, `ResourceStatusPresentation`, `OperationStatusPresentation`, `ToastPresentation`, `ToastTone`) plus immutable application actions and toolkit-neutral shortcuts shared by `nive-ui` and `nive-runtime`. Zero dependencies — no `iced`, no widgets, no runtime lifecycle types. Add only cross-layer neutral contracts; concrete runtime state, UI vocabulary such as icons/menu hierarchy, and opinionated rendering helpers stay in their owning layers.
- `nive-ui`: shared visual design system for tokens, semantic theme contracts, reusable UI primitives, bundled Inter/Geist Mono assets, and icon management. Control widgets share combined-state precedence through `ControlState`, while each widget category owns its final paint. `surface::style()` supplies fill and shadow only; composing regions own borders and structural seams.
- `nive-runtime`: shared app runtime foundation for application/update contracts, `Resource`/`Operation` async state, request IDs, user-facing errors, lifecycle contracts, and optional devtools simulator (feature `devtools`). It auto-registers `nive-ui` bundled fonts and defaults applications to Inter unless configured otherwise. The `Inspect` trait + derive walk app state to discover simulatable fields; `SimulableState` exposes snapshots, explicit capabilities, and simulator actions.
- `nive-runtime-derive`: proc macro `#[derive(Inspect)]` that generates recursive `Inspect::inspect` implementations for app state structs.
- `nive-workbench`: shell composition layer for document tabs, generic panel hosts, compact side rails, bottom header tabs, layout/session state, diagnostics/status surfaces, command palette hosting, and monitor-app visual validation. Default features depend on `nive-ui` only; the optional `runtime` feature adds adapters for supported `nive-runtime` concepts.
- `nive`: umbrella crate that re-exports `nive-ui` and `nive-runtime`, exposes `nive-workbench` as `nive::workbench`, and includes workbench APIs in `nive::prelude::*` for convenient app development.
- `nive-cli`: CLI binary (`nive`) for scaffolding new apps and managing provider-neutral icon manifests/generated modules.
