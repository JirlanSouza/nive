# nive-runtime

Reusable runtime foundation for Rust/Iced desktop applications.

`nive-runtime` owns the application/update contracts, window lifecycle, reusable
state machines, user-facing feedback, and an optional devtools layer that are
reused by Rust/Iced apps without depending on app-domain services. It sits above
[`nive-ui`](../nive-ui) and re-exports stable helper APIs from it. It also
depends on [`nive-core`](../nive-core) directly, implementing its neutral
presentation contracts for runtime state and adapting Iced keyboard events to
its core-owned action and shortcut contracts.

## What's inside

- `Application`, `ApplicationConfig`, `Context`, and `run` — the stable product
  contract and the private Iced program runner.
- `Action`, `ActionId`, `ActionMap` — re-exported core-owned product action
  catalogs shared with command surfaces.
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
- `ShortcutBinding`, `ShortcutModifiers`, `NamedShortcutKey`, `ShortcutMap`,
  and `keyboard_navigation_subscription` — neutral shortcut vocabulary plus
  lower-level runtime input integration.
- `Theme`, `ThemeBuilder`, `ThemeCatalog`, and `ThemeMode` reexports — runtime theme
  configuration for apps that need product-specific light/dark themes.

## Feature flags

| Feature      | Default | Description                                                              |
| ------------ | ------- | ------------------------------------------------------------------------ |
| `devtools`   | off     | Enables the optional devtools layer, `run_with_devtools`, and `#[derive(Inspect)]` traversal from `nive-runtime-derive`. The most experimental part of Nive. |
| `file-picker`| off     | Enables `pick_file`, `pick_files`, `pick_folder`, and `save_file` backed by `rfd`. |

## Usage

Applications depend on the `nive` umbrella crate, which forwards these features:

```toml
[dependencies]
nive = { git = "https://github.com/JirlanSouza/nive", tag = "v0.1.0-alpha.1" }
# optional capabilities:
# nive = { git = "...", tag = "v0.1.0-alpha.1", features = ["devtools", "file-picker"] }
```

```rust
use nive::prelude::*;
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
- Core-owned `Action`, `ActionId`, `ActionMap`, neutral shortcut types, and
  duplicate-ID validation re-exported through stable runtime paths.
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

Part of Nive **v0.1.0-alpha.1**, a pre-crates.io alpha. Public APIs break between alphas.

The `devtools` feature is the most experimental part of Nive. The full
`missing_docs` long tail (per-field and per-method docs) is not yet complete and
is tracked as post-v0.1 work.

## License

Licensed under the Apache License, Version 2.0. See [`LICENSE`](LICENSE).
