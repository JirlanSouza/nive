# nive-ui

Reusable visual design system for Rust/Iced desktop applications.

`nive-ui` is the design system layer of the [Nive](../) framework. It owns the
design tokens, semantic theme contracts, and reusable primitive widgets that
are independent of product-specific domain logic. It depends on `iced` and
[`nive-core`](../nive-core) (zero-dependency presentation contracts) and does
**not** depend on `nive-runtime` or any application crate.

## What's inside

- `tokens` — color, spacing, radius, shadow, and typography constants.
- `theme` — semantic role enums, framework themes (`Nive Light` / `Nive Dark`),
  custom theme builders/catalogs, active-theme accessors, and iced `Catalog`
  implementations.
- `widgets` — reusable widgets exposed through a flat facade and category
  facades: `primitives`, `controls`, `display`, `containers`, `navigation`,
  `overlays`, and `feedback`.
- `layout`, `graphics`, `accessibility` — focused facades for layout surfaces,
  visual assets, and keyboard/focus helpers.
- `focus_trap` — Tab/Shift+Tab focus cycling helpers for overlays.
- `BootstrapView` — generic startup loading/failure template.
- `DialogHost` / `ToastHost` — modal and toast overlay composition.

Presentation contracts such as `ToastPresentation` are defined in `nive-core`
and reexported here, keeping runtime types out of the UI crate. See
`docs/components.md` for contract details.

## Public API

Use `nive_ui::prelude::*` for app and screen code. The stable public surface is
the crate root, `nive_ui::prelude`, `nive_ui::theme`, and `nive_ui::widgets`.
These facades expose `Element`, `Renderer`, common Iced layout primitives,
theme role/builder/catalog types, reusable widget contracts, `BootstrapView`,
`DialogHost`, and `ToastHost`.

The individual `theme::*` and `widgets::*` submodules remain public for advanced
composition and tests, but apps should prefer the root/prelude/widget reexports
unless a lower-level style function or widget state helper is needed.

## Structural widget contracts

- `SectionHeader` is a transparent 12 px semibold section heading. Its
  single-line title fills and clips before protected status/actions; use
  `title_tooltip` to expose the full title. Principal workbench document titles
  belong to `nive-workbench::DocumentHeader`.
- `Toolbar` owns its surface, inset, bottom seam, and horizontal overflow.
  `ToolbarGroup` and toolbar-style `ActionGroup` are transparent spacing groups;
  use `Toolbar::separator()` only when spacing is not a sufficient boundary.
- `Panel` is square and borderless by default. Header and body are adjacent,
  the header/body seam is overlaid by `Panel`, and `body_padding` affects only
  body content. Rounded, bordered, or elevated standalone treatment is opt-in.
- `overlay_scrollbar()` supplies native floating 12 px lanes with a fixed 6 px
  thumb and no reserved content width. Rails are transparent; hover strengthens
  neutral color and drag uses accent. Iced does not expose state-dependent
  native thumb width, so all states remain 6 px.
- `Separator` accepts only `Subtle` or `Section` strength and full or logical
  inset extent. Until direction plumbing lands, leading/trailing map to
  left/right for horizontal rules and top/bottom for vertical rules.
- `SplitPane` keeps a one-pixel seam and a `ControlSize`-derived hit target.
  Hover/focus/drag presentation is geometry-neutral; locked and callback-free
  panes are fully inert. Invalid minima normalize to zero and impossible minima
  use deterministic proportional allocation.

## Usage (monorepo path dependency)

```toml
[dependencies]
nive-ui = { path = "../nive-ui" }
```

```rust
use nive_ui::prelude::*;
```

## Status

Part of Nive **v0.1.0**, a beta release. Public APIs may change before 1.0.

The full `missing_docs` long tail (per-field and per-method docs across the
widget catalog) is not yet complete and is tracked as post-v0.1 work.

## License

Licensed under the Apache License, Version 2.0. See [`LICENSE`](LICENSE).

Bundled icons are sourced from [Lucide](https://lucide.dev) (ISC License); see
[`THIRD_PARTY_LICENSES.md`](THIRD_PARTY_LICENSES.md) for the notice.
