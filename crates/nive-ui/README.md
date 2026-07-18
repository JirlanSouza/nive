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

### Managed logical focus

`nive-runtime` wraps the final content of every window in exactly one managed
focus root. A standalone `nive-ui` application opts in explicitly; `FocusRoot`
is intentionally absent from the ordinary prelude:

```rust
use nive_ui::accessibility::FocusRoot;

let window_content = FocusRoot::new(content);
```

The root is layout- and paint-neutral. It preserves one logical sequential
anchor across pointer blur, base content, and nested overlays while leaving
next/previous ordering to native Iced focus operations. Without a root, Nive
widgets retain compatible local focus behavior, but cross-widget anchor
uniqueness and retention are not guaranteed.

External custom widgets use `nive_ui::advanced::focus::{FocusState,
FocusVisibility}`. Store one `FocusState` in persistent widget Tree state,
call `register(operation, id, bounds)` from `Widget::operate`, call
`focus_from_pointer()` only after the widget claims a primary pointer/touch
press, and paint the ring from `is_focus_visible()` without changing layout.
`Auto` hides pointer-origin rings and shows keyboard-origin rings;
`AlwaysWhileActive` is for controls whose active frame communicates editing.
On disablement or identity replacement call `clear()`, and on real blur call
`deactivate()`. Keep durable selection and composite highlight/roving state
separate from the outer logical focus state.

### Selection controls

Use `Checkbox` for submitted independent choices, including controlled
`CheckboxState::Mixed`; `RadioGroup` for one visible choice among labelled
options; `Switch::inline` or `Switch::setting` for an immediate binary setting;
and typed `SegmentedControl` for two through five fixed modes or filters. Use
popup-backed `Select` for longer or open-ended option sets and `TabBar` for
documents or views with their own lifecycle.

All four controls are controlled: callbacks request the next typed value and the
application supplies the next view state. Missing callbacks mean display-only,
while `disabled(true)` suppresses interaction and applies disabled presentation.
Switch state changes are immediate; async persistence and failure UI remain
host-owned. Visible labels and preparatory semantic metadata do not yet imply
native accessibility-tree emission.

See [the selection-controls migration guide](../../docs/migrations/selection-controls.md)
for callback and compatibility changes.

## Structural widget contracts

### Form controls and composition

`Input`, `InputGroup`, `Field`, `FieldGroup`, and `Button` share
`theme::FormControlMetrics`. Built-in outer heights by density are:

| Density | Xs | Sm | Md | Lg |
| --- | ---: | ---: | ---: | ---: |
| Compact | 20 | 24 | 28 | 32 |
| Standard | 24 | 28 | 32 | 36 |
| Comfortable | 28 | 32 | 36 | 40 |

Form value text uses `TypographyRole::Control` (Inter Regular 14 px) and
button labels use `ControlStrong` (Inter Semibold 14 px), both with 1.25 line
height. `Field::new(label, Input/InputGroup)` is the canonical typed boundary:
the Field owns validation from its nonempty error, Required/Optional text,
label focus, and the shared hint/error slot. `Field::custom` is an explicit
escape hatch whose focus, state, size, semantics, and clipping remain
caller-owned.

`FieldGroup::new(visible_legend, fields)` owns typed Fields, stays
surface-neutral, and offers Vertical or equal-track Wrap layout. `InputGroup`
owns one frame around its typed prefix/unit/icon/status/actions; arbitrary
slots retain caller-owned masking and semantics. Inputs without `on_change`
are read-only, not disabled.

Button hierarchy is primary (Suggested+Solid), secondary (Neutral+Outline),
tertiary (Neutral+Ghost), and destructive confirmation (Destructive+Solid).
Label buttons are intrinsic unless `fill_width` is requested; icon buttons
require `button::icon(icon, semantic_name)`. Retained semantic metadata does
not yet imply native AccessKit name/relationship emission.

- `TabBar` is the controlled document/view collection: a borderless Chrome
  strip with Canvas-connected active tabs, bounded one-line labels, stable
  dirty/close geometry, pinned-first overflow menu, horizontal or mapped
  vertical-wheel navigation, manual-activation roving focus, and id-based drag
  intents. Use `SegmentedControl` for fixed equal-choice sets instead.
- `VerticalRail` is compact edge navigation with a panel-facing seam and local
  selected indicator; left/right currently map to physical window edges.
  `SelectableItem` is the form-compatible list row with whole-row selection,
  inset focus, and operational `trailing_text`; caller-styled `trailing(...)`
  retains semantic tone.

- `SectionHeader` is a transparent 12 px semibold section heading. Its
  single-line title fills and clips before protected status/actions; use
  `title_tooltip` to expose the full title. Principal workbench document titles
  belong to `nive-workbench::DocumentHeader`.
- `Card`, `ActionCard`, and `SelectableCard` share `ShapeSize::Md`,
  density-resolved `PaddingRole::Content`, and filled, outlined, elevated, and
  ghost variants. Use `Card` for passive grouping, `ActionCard` for one
  immediate whole-surface action, and `SelectableCard` for app-controlled
  persistent selection. Actionable cards keep a 48 px minimum target and must
  not contain nested interaction targets.
- `MetricCard` is surface-free and label-first. Its 20 px value may share a
  baseline with a muted unit; optional status and trend remain separate. Wrap
  it in `Card` when chrome is required.
- `KeyValueList` and `DataRow` are static, surface-neutral metadata
  compositions. `KeyValueList` owns one 96 px label column and typed
  Text/Code/Custom values; `DataRow` protects only Shrink/Fixed peer content.
  Their host owns chrome and whole-row interaction.
- `Badge::count` and `Badge::status` distinguish numeric and semantic content.
  `StatusIndicator` pairs a 6/8 px `ToneDot` with complete visible neutral
  text; `Spinner` is reserved for activity. A nonempty Status badge suppresses
  a duplicate status indicator, while Count may coexist.
- `InitialAvatar` is a fixed person/entity identity fallback with Unicode-safe
  initials and the provider-neutral `IconRole::Identity` fallback.
  `MetadataTag::code` owns literal technical metadata; `VersionBadge` is a
  deprecated one-release migration wrapper.
- `Toolbar` owns its surface, inset, bottom seam, and horizontal overflow.
  `ToolbarGroup` accepts navigation-owned `ToolbarAction`. Content-owned
  `ActionGroup` lives under `widgets::controls`, accepts `ContentAction`, keeps
  14 px labels at every `ControlSize`, and wraps complete controls only after
  explicit `.wrap()`.
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
