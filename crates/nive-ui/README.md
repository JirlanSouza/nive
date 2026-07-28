# nive-ui

Reusable visual design system for Rust/Iced desktop applications.

`nive-ui` is the design system layer of the [Nive](../) framework. It owns the
design tokens, semantic theme contracts, and reusable primitive widgets that
are independent of product-specific domain logic. It depends on `iced` and
[`nive-core`](../nive-core) (zero-dependency presentation and interaction
contracts) and does **not** depend on `nive-runtime` or any application crate.

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

Presentation contracts such as `ToastPresentation` and immutable application
actions are defined in `nive-core`. UI controls project those shared contracts
while keeping icons, hierarchy, and visual state in this crate; no runtime
dependency is needed. See `docs/components.md` for details.

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

`FocusRoot::on_modal_change(|active| ...)` publishes a message whenever the
aggregate modal activity below it changes — every open session of the shared
modal-hosting kernel (`Dialog`, `CommandPalette`, and any future consumer)
reports itself automatically, so no host wiring is needed. Use it to suspend
ambient timed behavior, such as notification expiry, while the user is held in
a modal step.

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
popup-backed `Select<T>` with labelled `SelectOption<T>` values for longer or
open-ended option sets and `TabBar` for documents or views with their own
lifecycle.

All four controls are controlled: callbacks request the next typed value and the
application supplies the next view state. Missing callbacks mean display-only,
while `disabled(true)` suppresses interaction and applies disabled presentation.
Switch state changes are immediate; async persistence and failure UI remain
host-owned. Visible labels and preparatory semantic metadata do not yet imply
native accessibility-tree emission.

See [the selection-controls migration guide](../../docs/migrations/selection-controls.md)
for callback and compatibility changes.

### Anchored overlays and popup controls

The composition direction is intentionally one-way:

```text
anchored geometry/lifecycle
        ├── Tooltip
        └── Popover
              └── Menu
                    ├── Select
                    └── Autocomplete
```

- `Tooltip` is passive supplementary disclosure. It reveals from pointer or
  keyboard focus after scoped timing, flips/shifts inside the viewport, and
  never replaces an icon-only anchor's independent semantic name.
- `Popover` owns the only floating fill, perimeter, 8 px radius, shadow,
  rectangular outer clip, semantic inset, and bounded vertical Scrollable.
  Supply surface-free content; do not add another `Panel`, radius, or
  `Scrollable`.
- `Menu` owns its EdgeToEdge FocusFirst Popover, fixed 28 px desktop rows, one
  composite focus target, typed command/checkbox/radio/submenu entries, and
  dismissal policy. `MenuCommand::from_action(&Action<M>)` projects the shared
  `nive-core` action without creating another command catalog.
- `Select<T>` is the typed bounded-choice form control. `Autocomplete<T>` is
  the typed query input whose app owns query, filtering, retrieval state,
  ordering, and committed value. Both integrate with `Field`; retrieval Error
  is popup content, not Field invalid state.

```rust
use nive_ui::prelude::*;

let tier = Field::new(
    "Account tier",
    Select::new(
        vec![
            SelectOption::new("starter", "Starter"),
            SelectOption::new("team", "Team"),
        ],
        Some("team"),
    )
    .on_select(|_| ()),
);

let results = AutocompleteResults::suggestions(vec![
    AutocompleteSuggestion::new(1_u64, "Nive Labs"),
]);
let organization = Field::new(
    "Organization",
    Autocomplete::new("niv", None, results)
        .open(true)
        .on_change(|_| ())
        .on_select(|_| ())
        .on_dismiss(()),
);
```

Callback absence removes only that capability and does not apply disabled
colors. Explicit `disabled(true)` has stronger precedence and suppresses
interaction without changing geometry. Overlay, highlight, result, and
chevron visuals move immediately to their terminal state; interpolated motion
belongs to the later `adopt-motion-preference-in-anchored-overlays` work.
`Start`/`End` and submenu arrows currently use physical LTR semantics. Retained
names, open state, values, and logical highlight are preparatory metadata only:
Nive does not yet emit native accessibility-tree roles, names, expanded state,
active-descendant relations, or announcements.

Category-specific controllers keep their ownership. Use `TabBar` for document
navigation, `Toolbar` for application chrome, `SideRail` for edge
navigation, `Dialog` for modal interaction, `CommandPalette` for command
search, and specialized inputs such as `ColorInput` for their domains; do not
replace them with `Select`, `Menu`, or a generic Popover merely because they
also open floating content.

## Structural widget contracts

### Form controls and composition

`Input`, `InputGroup`, `Select`, `Autocomplete`, `Field`, `FieldGroup`, and
`Button` share
`theme::FormControlMetrics`. Built-in outer heights by density are:

| Density | Xs | Sm | Md | Lg |
| --- | ---: | ---: | ---: | ---: |
| Compact | 20 | 24 | 28 | 32 |
| Standard | 24 | 28 | 32 | 36 |
| Comfortable | 28 | 32 | 36 | 40 |

Form value text uses `TypographyRole::Control` (Inter Regular 14 px) and
button labels use `ControlStrong` (Inter Semibold 14 px), both with 1.25 line
height. `Field::new(label, Input/InputGroup/Select/Autocomplete)` is the
canonical typed boundary:
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
- `SideRail` is compact edge navigation: a rotated label beside an upright
  icon, with a panel-facing seam and a full-height selected indicator on the
  opposite, window-facing edge. It carries no count or status marker, because a
  rail one chrome height wide has no room beside its label; that belongs to the
  panel an item selects. Left/right currently map to physical window edges.
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
  `MetadataTag::code` owns literal technical metadata.
- `Toolbar` owns its surface, inset, bottom seam, and horizontal overflow.
  `Toolbar::spacer()` anchors following groups to the trailing edge at finite
  fill widths. `ToolbarGroup` accepts navigation-owned `ToolbarAction`.
  Content-owned `ActionGroup` lives under `widgets::controls`, accepts
  `ContentAction`, keeps 14 px labels at every `ControlSize`, and wraps complete
  controls only after explicit `.wrap()`.
- `Panel` is square and borderless by default. Header and body are adjacent,
  the header/body seam is overlaid by `Panel`, and `body_padding` affects only
  body content. Rounded, bordered, or elevated standalone treatment is opt-in.
- `overlay_scrollbar()` supplies native floating 12 px lanes with a fixed 6 px
  thumb and no reserved content width. Rails are transparent; hover strengthens
  neutral color and drag uses accent. Iced does not expose state-dependent
  native thumb width, so all states remain 6 px.
- Every widget icon slot takes `impl Into<IconRef>`, so a framework `IconRole`
  and an application's own generated `IconSymbol` share one call shape:
  `.icon(IconRole::ViewRefresh)` and `.icon(IconSymbol::Server)` both compile.
  Use roles for UI vocabulary the framework owns and app symbols for domain
  nouns; `nive icons add-symbol` generates the `From<IconSymbol> for IconRef`
  that makes the second form work.
- A pointer drag survives the cursor leaving and re-entering the window while
  its button is held, so a splitter, tab reorder, or tree drag resumes from the
  next move back inside rather than needing a release and re-press. Window focus
  loss is what cancels a drag.
- `Separator` accepts only `Subtle` or `Section` strength and full or logical
  inset extent. Until direction plumbing lands, leading/trailing map to
  left/right for horizontal rules and top/bottom for vertical rules.
- `SplitPane` is the two-pane proportional splitter: its `ratio` is a share of
  its own container, so both panes scale with it. It keeps a one-pixel seam and
  a `ControlSize`-derived hit target. Hover/focus/drag presentation is
  geometry-neutral; locked and callback-free panes are fully inert. Invalid
  minima normalize to zero and impossible minima use deterministic proportional
  allocation.
- `SplitStack` is the N-pane splitter that owns one axis. Each pane is
  `SplitSizing::Fixed` at a logical pixel length or `SplitSizing::Fill`, and
  exactly one pane fills. Dragging a divider moves the two panes bordering it
  and nothing else, stopping once an adjacent pane reaches its minimum rather
  than pushing the pane beyond it; growing the container grows the filling pane
  alone. It registers one logical-focus target and roves a divider index with
  the cross-axis arrows, `Home`, and `End`. Dragging past a neighbour's minimum
  can propose collapsing that pane, opt-in through `SplitStackPane::collapsible`
  plus `SplitStack::on_collapse`; each drag proposes at most one collapse and
  reports the pane's pre-drag length so the app can restore it there. Both
  splitters derive their seam, grip, and hit target from one shared
  implementation. Reach for `SplitStack` when sibling dividers must not disturb
  each other — chaining two `SplitPane`s along one axis always couples them.
- `Tree` is the controlled hierarchy widget: the app owns `TreeState`,
  rebuilds `TreeNode`s every view pass, and applies each intent-only
  `TreeEvent`. `TreeChildren` models `Loaded`, `Deferred`, and `Failed`
  branches; build a failed branch with `TreeNode::branch_failed(id, label,
  &error)` from a value implementing the core `ErrorPresentation` contract
  (`nive-runtime`'s `UserFacingError` already does). Deferred, failed, and
  empty branches each render one canonical chrome row — loading placeholder,
  error row with retry, or empty affordance — excluded from selection, focus,
  navigation, type-ahead, clipboard, and drag/drop. Context requests are
  intent only: Tree emits `ContextRequested` and hosts no menu, so the
  application hosts the canonical `Menu` at the request position. Rename is
  also intent only (`RenameRequested`); Tree hosts no inline editor. `TreeItem`
  is the stateless primitive row for custom hierarchies; both render row focus
  independently from durable selection. Tree renders every expanded-visible
  row and does not virtualize the viewport.

## Usage

Applications depend on the `nive` umbrella crate, not on `nive-ui` directly:

```toml
[dependencies]
nive = { git = "https://github.com/JirlanSouza/nive", tag = "v0.1.0-alpha.1" }
```

```rust
use nive::prelude::*;
```

Depending on `nive-ui` alone is for crates that build widgets rather than
applications, and gives up the runtime, window management, and feedback layers.

## Status

Part of Nive **v0.1.0-alpha.1**, a pre-crates.io alpha. Public APIs break
between alphas.

The full `missing_docs` long tail (per-field and per-method docs across the
widget catalog) is not yet complete and is tracked as post-v0.1 work.

## License

Licensed under the Apache License, Version 2.0. See [`LICENSE`](LICENSE).

Bundled icons are sourced from [Lucide](https://lucide.dev) (ISC License); see
[`THIRD_PARTY_LICENSES.md`](THIRD_PARTY_LICENSES.md) for the notice.
