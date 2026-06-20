# nive-runtime-derive

## Role

`crates/nive-runtime-derive` owns proc macros that keep devtools integration and
probe metadata declarative. It is a framework crate: generated paths target
`nive-runtime` APIs by default. The `devtools_path` override remains only as an
explicit escape hatch for non-standard integration targets; it is not the normal
`app-gui` path.

Current scope:

- `DevtoolStateCatalog` — derive for explicitly annotated devtool state fields,
  with optional `#[devtools_path(...)]` override for non-standard targets
- `DevtoolStateHost` — derive for root state snapshot/application hosts, with
  optional `devtools_path` override for non-standard targets
- `DevtoolOperationContext` — derive for operation input schemas/builders, with
  optional `devtools_path` override for non-standard targets
- `UiErrorProbeCatalog` — derive for error probe catalog generation (targets `nive_runtime::devtools::probe::ProbeCatalogEntry`)
- `runtime_client` — attribute macro for client impl probe-key declarations, generated client `DEV_PROBES` metadata, explicit app-owned probe labels/scopes, and probe key injection

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
- `nive_runtime::devtools::probe::ProbeCatalogEntry`
- `nive_runtime::devtools::probe::ProbeMeta`
- `nive_runtime::devtools::probe::ProbeErrorScope`

## Boundaries

- Generate paths against `nive-runtime` APIs by default; do not route `app-gui`
  derives through a local state bridge.
- Keep app-specific probe summaries, short keys, and custom error scopes explicit in `runtime_client` attributes; the framework macro may provide generic name-derived defaults, but must not hardcode product semantics such as project catalog or tag behavior.
- Keep app-domain fixture data in explicit app-owned fixture-provider functions referenced by `#[devtool(fixtures = path)]`.
- Keep app-owned probe composition in `app-gui` until the runtime catalog boundary is extracted.
- Prefer derive/attribute integration for app feature code; avoid hand-written devtools routing in screens and clients.

## Workflow

- `cargo check -p nive-runtime-derive`
- `cargo test -p nive-runtime-derive`
