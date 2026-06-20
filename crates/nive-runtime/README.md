# nive-runtime

Reusable runtime foundation for Rust/Iced desktop applications.

`nive-runtime` owns the application/update contracts, window lifecycle, reusable
state machines, user-facing feedback, and an optional devtools layer that are
reused by Rust/Iced apps without depending on app-domain services. It sits above
[`nive-ui`](../nive-ui) and re-exports stable helper APIs from it.

## What's inside

- `Application`, `ApplicationConfig`, `Context`, and `run` — the stable product
  contract and the private Iced program runner.
- `Update`, `AppUpdate`, `RuntimeCommand` — ordered task and runtime-effect
  composition.
- `BootstrapSpec` — repeatable startup task attempts, stale-result rejection,
  minimum splash duration, retry, and cancellation.
- `WindowSpec`, `WindowCommand`, `WindowRegistry` — generic window contracts,
  cardinality, and open/close/exit handshakes.
- `AsyncState` and `OperationState` — reusable resource and operation state
  machines.
- `UserFacingError` and toast state (`ToastState`, `ToastItem`) — user-facing
  feedback.
- `ScreenView` and `ScreenUpdate` — screen composition contracts.
- `platform` — cross-platform app icon installer and optional file picker.
- `keyboard_navigation_subscription` and `ShortcutMap` — input helpers.
- `Theme`, `ThemeBuilder`, `ThemeCatalog`, and `ThemeMode` reexports — runtime theme
  configuration for apps that need product-specific light/dark themes.

## Feature flags

| Feature      | Default | Description                                                              |
| ------------ | ------- | ------------------------------------------------------------------------ |
| `devtools`   | off     | Enables the optional devtools layer, `run_with_devtools`, and the derive macros from `nive-runtime-derive`. The most experimental part of Nive. |
| `file-picker`| off     | Enables `pick_file`, `pick_files`, and `pick_folder` backed by `rfd`.    |

## Usage (monorepo path dependency)

```toml
[dependencies]
nive-runtime = { path = "../nive-runtime" }
# optional capabilities:
# nive-runtime = { path = "../nive-runtime", features = ["devtools", "file-picker"] }
```

```rust
use nive_runtime::prelude::*;
```

See `docs/` for contract details on the application, lifecycle, and devtools
layers.

## Public API

Use `nive_runtime::prelude::*` for application integration. The stable public
surface is the crate root/prelude, `Application` and `ApplicationConfig`,
`Update`/`AppUpdate`, lifecycle/window contracts, reusable state and feedback
types, runtime task/subscription aliases, theme configuration reexports, and
feature-gated platform/devtools APIs.

Implementation modules under `application`, `lifecycle`, `feedback`, `screen`
and `state` are private; app code should not depend on runner internals.

## Status

Part of Nive **v0.1.0**, a beta release. Public APIs may change before 1.0.

The `devtools` feature is the most experimental part of Nive. The full
`missing_docs` long tail (per-field and per-method docs) is not yet complete and
is tracked as post-v0.1 work.

## License

Licensed under the Apache License, Version 2.0. See [`LICENSE`](LICENSE).
