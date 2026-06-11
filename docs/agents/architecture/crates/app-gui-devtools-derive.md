# app-gui-devtools-derive

## Role

`crates/app-gui-devtools-derive` owns proc macros that keep devtools integration declarative in `app-gui`.

Current scope:

- `DevtoolStateCatalog`
- `DevtoolStateHost`
- `DevtoolOperationContext`
- `UiErrorProbeCatalog`
- `runtime_client` for client impl probe-key declarations, generated client `DEV_PROBES` metadata, and probe key injection

## Boundaries

- Generate paths against `nive-runtime` for runtime-owned probe metadata and app-shell/devtools contracts where possible.
- Keep app-domain fixture/value traits in `app-gui` until those state bridges are abstracted.
- Keep app-owned probe composition in `app-gui` until the runtime catalog boundary is extracted; during the transition, `UiErrorProbe` should combine app-owned probes such as bootstrap with generated client metadata instead of manually listing client probe variants.
- Prefer attribute/derive integration for app feature code; avoid hand-written devtools routing in screens and clients.
