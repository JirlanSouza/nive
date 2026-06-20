# nive-runtime-derive

Proc-macro support for the Nive runtime devtools layer.

`nive-runtime-derive` provides the derive and attribute macros used by
`nive-runtime`'s optional `devtools` feature. End users normally depend on
`nive-runtime` (which re-exports these macros when `devtools` is enabled); this
crate is published separately for transparency.

## Macros

- `Devtools` — implements the `nive_runtime::devtools::Devtools` marker.
- `UiErrorProbeCatalog` — generates a probe catalog from an error enum.
- `runtime_client` — generates dev probe metadata and key injection for a
  client struct.
- `DevtoolStateCatalog` — collects devtool state fields on a state struct.
- `DevtoolStateHost` — marks the root state snapshot host.
- `DevtoolOperationContext` — generates an operation input schema/builder.

## Attributes

- `#[devtool(fixtures = "path")]` — names a fixture-provider function for a
  state field (used with `DevtoolStateCatalog`).
- `#[devtool(nested)]` — marks a field whose devtool state is nested.
- `#[devtools_path("path::to::devtools")]` — overrides the default
  `nive_runtime::devtools` target path for `DevtoolStateCatalog`,
  `DevtoolStateHost`, and `DevtoolOperationContext`.

See `nive-runtime/docs/devtools.md` for usage.

## Status

Part of Nive **v0.1.0**, a beta release. Generated APIs may change before 1.0.
This crate enforces `#![warn(missing_docs)]` and is fully documented.

## License

Licensed under the Apache License, Version 2.0. See [`LICENSE`](LICENSE).
