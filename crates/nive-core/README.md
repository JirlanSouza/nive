# nive-core

Neutral presentation and interaction contracts shared by Nive layers.

`nive-core` is the lowest layer of the [Nive](../) framework. It has **zero
dependencies** — no `iced`, no widgets, no runtime lifecycle, no platform
APIs — and defines read-only presentation contracts plus immutable application
actions without depending on concrete types from another layer.

## What's inside

- `ErrorPresentation` — summary/detail view over a presentable error.
- `ResourceStatusPresentation` — status view over an async resource
  (`nive-runtime::Resource<T>`).
- `OperationStatusPresentation` — status view over an async operation
  (`nive-runtime::Operation<C>`).
- `ToastPresentation` — view over a presentable toast.
- `ToastTone` — the semantic tone shared by runtime toast state and UI
  rendering.
- `Action`, `ActionId`, `ActionMap`, `DuplicateActionId` — one immutable,
  ordered product-command catalog shared by runtime routing and UI controls.
- `ShortcutBinding`, `ShortcutKey`, `NamedShortcutKey`, `ShortcutModifiers`,
  `ShortcutMap` — toolkit-neutral keyboard contracts; runtime translates Iced
  events at its boundary.

## Why this crate exists

`nive-ui` renders these contracts through widgets like `ResourceStatusLine`
and `ToastHost` and projects shared actions into command surfaces;
`nive-runtime` implements the presentation traits and translates toolkit
events for the same actions. Before this crate existed,
`nive-ui` defined the traits and `nive-runtime` implemented them — which
worked because `nive-runtime` already depends on `nive-ui`, but inverted
ownership: a UI crate should not be the one defining headless contracts for
runtime state. `nive-core` gives both layers a neutral base to depend on.

## Charter

This crate exists to fix that inverted boundary, not to become a general
shared-types dumping ground. A type belongs here only if it is a neutral
presentation or interaction contract consumed by more than one layer **and**
free of any `iced`, widget, runtime lifecycle, or platform dependency.
Concrete runtime state, UI vocabulary (including icons and menu hierarchy),
and opinionated helpers stay in the layer that owns them.

## Public API

`nive-core` re-exports everything at the crate root; there is no internal
taxonomy to navigate for a crate this small.

```rust
use nive_core::{
    Action, ActionMap, ErrorPresentation, ShortcutBinding, ToastPresentation,
    ToastTone,
};

let actions = ActionMap::new().action(
    Action::new("file.save", "Save", ())
        .shortcut(ShortcutBinding::primary_character('s')),
);
# let _ = actions;
```

## Status

Part of Nive **v0.1.0-alpha.1**, a pre-crates.io alpha. Public APIs break
between alphas.

## License

Licensed under the Apache License, Version 2.0. See [`LICENSE`](LICENSE).
