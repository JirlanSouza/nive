# nive-runtime-derive

## Role

`crates/nive-runtime-derive` owns proc macros that keep devtools integration and probe metadata declarative. It is a framework crate: generated paths target `nive-runtime` APIs by default, with an optional `devtools_path` attribute for app-owned trait adapters.

Current scope:

- `DevtoolStateCatalog` — derive with optional `#[devtools_path("crate::dev::devtools")]` attribute
- `DevtoolStateHost` — derive with optional `devtools_path` attribute
- `DevtoolOperationContext` — derive with optional `devtools_path` attribute
- `UiErrorProbeCatalog` — derive for error probe catalog generation (targets `nive_runtime::ProbeCatalogEntry`)
- `runtime_client` — attribute macro for client impl probe-key declarations, generated client `DEV_PROBES` metadata, and probe key injection

## Generated Paths

Default generated paths target `nive_runtime::devtools::*`:

- `nive_runtime::devtools::DevtoolStateCatalog`
- `nive_runtime::devtools::DevtoolStateHost`
- `nive_runtime::devtools::DevtoolStateField`
- `nive_runtime::devtools::DevtoolCommand`
- `nive_runtime::devtools::DevtoolCommandResult`
- `nive_runtime::devtools::DevtoolStateSnapshot`
- `nive_runtime::devtools::DevtoolFieldSchema`
- `nive_runtime::devtools::DevtoolInputField`
- `nive_runtime::devtools::DevtoolInputValues`
- `nive_runtime::devtools::join_path`
- `nive_runtime::ProbeCatalogEntry`
- `nive_runtime::ProbeMeta`
- `nive_runtime::ProbeErrorScope`

App code that uses an app-owned trait adapter (e.g. `app-gui`'s `AppDevtoolStateField`) passes `#[devtools_path("crate::dev::devtools")]` to redirect generated paths.

## Boundaries

- Generate paths against `nive-runtime` APIs by default; keep app-domain overrides explicit via the `devtools_path` attribute.
- Keep app-domain fixture/value trait adapters in `app-gui` to satisfy the orphan rule for domain types like `Vec<ProjectInfo>`.
- Keep app-owned probe composition in `app-gui` until the runtime catalog boundary is extracted.
- Prefer derive/attribute integration for app feature code; avoid hand-written devtools routing in screens and clients.

## Workflow

- `cargo check -p nive-runtime-derive`
- `cargo test -p nive-runtime-derive`