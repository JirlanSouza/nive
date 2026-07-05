# nive-runtime

Reusable runtime foundation for Rust/Iced desktop applications.

`nive-runtime` owns the application/update contracts, window lifecycle, reusable
state machines, user-facing feedback, and an optional devtools layer that are
reused by Rust/Iced apps without depending on app-domain services. It sits above
[`nive-ui`](../nive-ui) and re-exports stable helper APIs from it. It also
depends on [`nive-core`](../nive-core) directly, implementing its neutral
presentation contracts (`ErrorPresentation`, `ResourceStatusPresentation`,
`OperationStatusPresentation`, `ToastPresentation`) for `UserFacingError`,
`Resource<T>`, `Operation<C>`, and `ToastItem`.

## What's inside

- `Application`, `ApplicationConfig`, `Context`, and `run` — the stable product
  contract and the private Iced program runner.
- `Action`, `ActionId`, `ActionMap` — product action catalogs for shortcuts and
  future command surfaces.
- `Effect` — ordered task and runtime-effect composition for application
  hooks, plus `MessageContext`/`MessageSource` for message-origin routing.
- `BootstrapSpec` — repeatable startup task attempts, stale-result rejection,
  minimum splash duration, retry, and cancellation.
- `WindowSpec`, `WindowCommand`, `WindowRegistry` — generic window contracts,
  cardinality, and open/close/exit handshakes.
- `Resource` and `Operation` — reusable async resource and operation state
  machines.
- `UserFacingError` and toast state (`ToastState`, `ToastItem`) — user-facing
  feedback.
- `ScreenView` and `ScreenEffect` — screen composition contracts.
- `platform` — cross-platform app icon installer and optional file picker.
- `SettingsConfig`, `RuntimeSession`, `WindowSession` — opt-in runtime
  settings/session persistence for framework-owned preferences and keyed window
  geometry.
- `keyboard_navigation_subscription` and `ShortcutMap` — lower-level input
  helpers.
- `Theme`, `ThemeBuilder`, `ThemeCatalog`, and `ThemeMode` reexports — runtime theme
  configuration for apps that need product-specific light/dark themes.

## Feature flags

| Feature      | Default | Description                                                              |
| ------------ | ------- | ------------------------------------------------------------------------ |
| `devtools`   | off     | Enables the optional devtools layer, `run_with_devtools`, and `#[derive(Inspect)]` traversal from `nive-runtime-derive`. The most experimental part of Nive. |
| `file-picker`| off     | Enables `pick_file`, `pick_files`, `pick_folder`, and `save_file` backed by `rfd`. |

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

See `docs/` for contract details on the application, lifecycle, settings and
devtools layers.

## Public API

Use `nive_runtime::prelude::*` for application integration. Larger apps can
import directly from the public area modules (`application`, `actions`,
`feedback`, `input`, `lifecycle`, `screen`, `settings`, `state`, and
`support`) when a narrower layer-specific path is clearer. The app-facing
surface includes:

- `Application`, `ApplicationConfig`, `Context`, `WindowContext`, and `run`.
- `Action`, `ActionId`, `ActionMap`, and duplicate-ID validation.
- `Effect`, `MessageContext`, `MessageSource`, and `perform`.
- Lifecycle/window contracts such as `WindowSpec`, `WindowCommand`,
  `CloseDecision`, `ExitDecision`, `BootstrapSpec`, and `RuntimeEvent`.
- Opt-in settings/session contracts such as `SettingsConfig`,
  `RuntimeSession`, `WindowSession`, `WindowSessionSize`, and
  `WindowSessionPosition`.
- Reusable state and feedback types such as `Resource`, `Operation`,
  `Settled`, `Toast`, `UserFacingError`, `RequestId`, and clock helpers.
- Runtime task/subscription aliases and theme configuration reexports.
- Feature-gated platform/devtools APIs.

Runner internals remain private behind those modules. App code emits
`WindowCommand` through `Effect`; it should not call lower-level window-opening
helpers directly.

## Status

Part of Nive **v0.1.0**, a beta release. Public APIs may change before 1.0.

The `devtools` feature is the most experimental part of Nive. The full
`missing_docs` long tail (per-field and per-method docs) is not yet complete and
is tracked as post-v0.1 work.

## License

Licensed under the Apache License, Version 2.0. See [`LICENSE`](LICENSE).
