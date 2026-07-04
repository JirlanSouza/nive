# nive-core

Neutral presentation contracts shared by `nive-ui` and `nive-runtime`.

`nive-core` is the lowest layer of the [Nive](../) framework. It has **zero
dependencies** — no `iced`, no widgets, no runtime lifecycle, no platform
APIs — and defines read-only contracts for rendering error, toast, and
async-state status without depending on any concrete type from another layer.

## What's inside

- `ErrorPresentation` — summary/detail view over a presentable error.
- `ResourceStatusPresentation` — status view over an async resource
  (`nive-runtime::Resource<T>`).
- `OperationStatusPresentation` — status view over an async operation
  (`nive-runtime::Operation<C>`).
- `ToastPresentation` — view over a presentable toast.
- `ToastTone` — the semantic tone shared by runtime toast state and UI
  rendering.

## Why this crate exists

`nive-ui` renders these contracts through widgets like `ResourceStatusLine`
and `ToastHost`; `nive-runtime` implements them for `UserFacingError`,
`Resource<T>`, `Operation<C>`, and `ToastItem`. Before this crate existed,
`nive-ui` defined the traits and `nive-runtime` implemented them — which
worked because `nive-runtime` already depends on `nive-ui`, but inverted
ownership: a UI crate should not be the one defining headless contracts for
runtime state. `nive-core` gives both layers a neutral base to depend on.

## Charter

This crate exists to fix that inverted boundary, not to become a general
shared-types dumping ground. A type belongs here only if it is a read-only
presentation contract consumed by more than one layer **and** free of any
`iced`, widget, runtime lifecycle, or platform dependency. Concrete runtime
types, UI vocabulary, and opinionated helpers (tone-to-color mapping, error
formatting) stay in the layer that owns them.

## Public API

`nive-core` re-exports everything at the crate root; there is no internal
taxonomy to navigate for a crate this small.

```rust
use nive_core::{ErrorPresentation, ToastPresentation, ToastTone};
```

## Status

Part of Nive **v0.1.0**, a pre-publication alpha. Public APIs may change
before 1.0.

## License

Licensed under the Apache License, Version 2.0. See [`LICENSE`](LICENSE).
