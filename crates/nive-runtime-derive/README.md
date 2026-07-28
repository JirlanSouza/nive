# nive-runtime-derive

Proc-macro support for the Nive runtime devtools layer.

`nive-runtime-derive` provides `#[derive(Inspect)]` for `nive-runtime`'s
optional `devtools` simulator. End users normally depend on `nive-runtime` or
`nive` (which re-export the derive); this crate is published separately for
transparency.

## Macros

- `Inspect` — generates recursive `Inspect::inspect` traversal for structs
  when the derive crate's `devtools` feature is enabled. With the feature off,
  it expands to nothing so apps can leave `#[derive(Inspect)]` in production
  source.

## Attributes

- `#[inspect(skip)]` — excludes a field from traversal.
- `#[inspect(default)]` — enables Resource default-payload simulation and
  requires the payload to implement `Default`.
- `#[inspect(sample = path)]` — enables Resource sample-payload simulation
  through a `fn() -> T` factory.
- `#[inspect(input = path)]` — enables Operation start/fail simulation through
  a `fn() -> C` input factory.

See `nive-runtime/docs/devtools.md` for usage.

## Status

Part of Nive **v0.1.0-alpha.1**, a pre-crates.io alpha. Generated APIs break between alphas.
This crate enforces `#![warn(missing_docs)]` and is fully documented.

## License

Licensed under the Apache License, Version 2.0. See [`LICENSE`](LICENSE).
