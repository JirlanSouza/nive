# Nive UI Contracts

`nive_ui::prelude` exposes the Nive `Element`, renderer, theme types, common
layout primitives and reusable widgets.

The app-facing public UI API is the crate root, `nive_ui::prelude`,
`nive_ui::theme`, and `nive_ui::widgets`. Screens should prefer those facades
for common layout primitives, shared `Element`/`Renderer` aliases, theme
builders/catalogs, and reusable widgets. Lower-level submodules under
`theme::*` and `widgets::*` remain available for advanced composition, styling
helpers and focused tests, but they are not the default integration surface for
product code.

`nive_ui::widgets` is organized as both a flat app-facing facade and a
taxonomy of category facades:

- `widgets::primitives` — text helpers, icons, color swatches, separators,
  Iced `space` and SVG helpers.
- `widgets::controls` — buttons, checkbox, switch, input, select, segmented
  control, autocomplete and color controls.
- `widgets::display` — badges, avatars, metadata, metric cards, trees, empty
  states and version badges.
- `widgets::containers` — cards, action/selectable cards, panels and split
  panes.
- `widgets::navigation` — tabs, toolbars, dropdown menus and command palette
  helpers.
- `widgets::overlays` — dialogs, popovers, tooltips, `DialogHost` and
  `ToastHost`.
- `widgets::feedback` — alerts, callouts, loading indicators, progress,
  skeletons, error/resource/operation status surfaces.

The crate root also exposes focused `layout`, `graphics` and `accessibility`
facades for code that wants narrower imports than the full widget catalog.

Theme definitions remain in `nive-ui`. `Theme::Light` and `Theme::Dark` are the
framework defaults. `ThemeBuilder` creates product-specific themes from a
semantic palette plus optional typography, shape, spacing and control metric
overrides. `ThemeCatalog` stores the light/dark pair the runtime should resolve
from `ThemePreference`.

`ThemeDensity` controls global UI compactness (spacing, paddings, gaps, control
heights, icon sizes). Apps configure density through `ThemeBuilder::density()`.
The default is `ThemeDensity::Standard`, which preserves current visual metrics.

`theme::active()` provides the current snapshot used by view helpers. The
active-theme storage is private to `nive-ui`; runtime synchronization is exposed
only through the framework integration module.

```rust
use nive_ui::prelude::*;

let light = Theme::builder("Acme Light", theme::ThemeMode::Light)
    .density(ThemeDensity::Compact)
    .accent(color::hex(0x0EA5E9))
    .build();
let dark = Theme::builder("Acme Dark", theme::ThemeMode::Dark)
    .density(ThemeDensity::Compact)
    .accent(color::hex(0x38BDF8))
    .build();
let catalog = ThemeCatalog::new(light, dark);
```

Build product theme catalogs once during application configuration; they are
intended to live for the process lifetime.

Tests that change the global snapshot must hold
`theme::testing::ThemeTestGuard`, which restores the previous theme when
dropped.

## Dialog Infrastructure

`DialogHost` owns modal composition, backdrop rendering, pointer blocking and
focus trapping. Runtime integrations provide optional backdrop and Escape
messages; the host publishes those messages without changing product state.

Dialog content remains composed with `Dialog`, `DialogHeader`,
`DialogFooter` and `DialogActionFooter`.

## Feedback And Status

`nive-ui` owns the reusable feedback and status components:

- `ErrorFeedback`, `ErrorEmptyState`, `ErrorStatusLine` and `ErrorDetailsDialog`
- `ResourceStatusLine`, `OperationStatusLine` and `OperationActionGroup`
- `InitialAvatar`, `MetricCard` and `VersionBadge`

Presentation contracts keep runtime types out of the UI crate. They are
defined in `nive-core` (zero dependencies) and reexported here:

- `ErrorPresentation`
- `ResourceStatusPresentation`
- `OperationStatusPresentation`

`nive-runtime::UserFacingError`, `Resource<T>` and `Operation<C>`
implement these contracts by depending on `nive-core` directly, not on
`nive-ui`. Applications supply product copy and messages while Nive owns the
reusable visual composition.

## Data, Indicators, And Identity

| Contract | Canonical API | Fixed semantics |
| --- | --- | --- |
| Count badge | `Badge::count(u64)` | 20 px high, 20 px minimum, `0..=99`, then `99+` |
| Status badge | `Badge::status(Cow<str>)` | compact one-line status, 96 px content bound |
| Labelled state | `StatusIndicator::new(tone, label)` | complete visible text plus a 6 px Xs/Sm or 8 px Md/Lg dot |
| Definition list | `KeyValueList::label_width(f32)` | surface-neutral, 96 px default shared column, 14 px text |
| Static row | `DataRow::reserve_indicator()` | clustered principal/secondary text; Shrink/Fixed peer slots protected |
| Identity | `InitialAvatar::person()` / `entity()` | Xs/Sm/Md/Lg = 24/32/40/56 px |
| Technical value | `MetadataTag::code(Cow<str>)` | 20 px high, 168 px maximum, middle ellipsis |

`KeyValueList` hosts own fill, border, radius, shadow, and outer padding.
`MetadataItem::new` selects framework-styled Text, `code_value` selects Code,
and `custom_value` is the caller-styled escape hatch. Status remains
orthogonal through `status(tone)` and requires complete visible meaning in the
value. `DataRow` is never a row-level target; wrap a complete interactive row
in `SelectableItem`, and compose a peer action as a one-item `ActionGroup`
containing `ContentAction`.

Migration mappings:

| Previous API | Current API |
| --- | --- |
| `KeyValueList::role(...)` | remove it; put the list in the owning Card/Panel/Dialog |
| `MetadataItem::label_width(...)` | `KeyValueList::label_width(f32)` |
| `MetadataItem::value(element)` | `custom_value(element)` (`value` is deprecated for one release) |
| `MetadataItem::tone(...)` | `status(...)` (`tone` and tonal shortcuts are deprecated) |
| `Badge::new(text)` | `Badge::status(text)` |
| Badge size methods | remove them; badge geometry is fixed |
| bare `ToneRole` compact status | `StatusIndicator::new(tone, visible_label)` |
| `VersionBadge::new(value)` | `MetadataTag::code(value)` |

Downstream custom icon catalogs must add an `identity` mapping to
`icons.toml`, run `nive icons sync`, commit the regenerated catalog and asset,
then pass `nive icons check`. Renderer limits remain explicit: Nive does not
claim native definition-list/accessibility nodes or enforce OpenType `tnum`;
tooltips supplement, but never replace, complete visible identity/status text.

## Action Surfaces

`Card`, `ActionCard`, and `SelectableCard` share this frame:

| Variant | Fill | Perimeter | Shadow |
| --- | --- | --- | --- |
| `Filled` | Panel | none | none |
| `Outlined` | transparent | one default border | none |
| `Elevated` | Elevated | none | elevated |
| `Ghost` | transparent | none | none |

The default is `Filled` with `ShapeSize::Md` and
`PaddingRole::Content` (8/12/14 px in Compact/Standard/Comfortable). Raw
shape, radius, and padding remain escape hatches; `padding(0)` is flush.
`Card` is passive. `ActionCard` is one immediate target. `SelectableCard` is
controlled persistent selection and may reserve a display-only check slot.
The interactive cards have a 48 px minimum height, inset focus, and no nested
buttons, links, menus, or inputs. Recommended titles use complete
`BodyStrong` 14 px semibold typography; descriptions use complete 14 px Body.

`MetricCard` owns no surface or padding. It renders a secondary label before a
20 px semibold value, an optional muted baseline unit, and separate status and
trend content. An external `Card` owns chrome.

`ActionGroup` is a transparent inline content composition. It accepts
`ContentAction`, defaults to `ControlSize::Sm`, and follows
`theme::control_metrics(size).height` while its label typography stays at
14 px. Loading reserves width and is inert without impersonating explicit
disabled styling. `fill_width()` does not stretch items or enable wrapping;
`.wrap()` opts into whole-control wrapping and suppresses orphaned separators.

`Toolbar` is a surface bar for application chrome. Its `size` configures the
`ToolbarAction` values inside `ToolbarGroup`; the toolbar itself may add
surrounding chrome padding. Toolbar items are not accepted by content
`ActionGroup`.

### Card/content-action migration

| Previous spelling | Current spelling |
| --- | --- |
| `card.role(SurfaceRole::Panel)` | default `filled()` |
| `card.role(SurfaceRole::Elevated)` | `card.elevated()` |
| `card.bordered()` | `card.outlined()` (`bordered` is deprecated) |
| default Xl/Lg card radius plus raw padding | default Md plus semantic content padding |
| Body geometry plus a local semibold font | `ntext::body_strong(...)` |
| `widgets::navigation::ActionGroup` | `widgets::controls::ActionGroup` or flat facade |
| `ActionGroup::action(ToolbarAction::...)` | `ActionGroup::action(ContentAction::...)` |

Downstream exhaustive matches on `TypographyRole` and literals of
`TypographyScale` must include `BodyStrong`/`body_strong`.

`TabBar`, `VerticalRail`, `SectionHeader`, flat `SegmentedControl`, and toolbar
actions derive their primary extent from the active theme's `ControlSize`
metrics. A workbench shell applies one shared size to those managed regions
rather than requiring callers to compensate with different per-widget sizes.

`SplitPane` defaults to `ControlSize::Sm` and exposes the standard
`size`/`xs`/`sm`/`md`/`lg` vocabulary. Its visual and layout divider remains one
logical pixel while its centered resize target is derived from the selected
control size, so interaction ergonomics do not change pane-ratio geometry.

## Bootstrap Template

`BootstrapView` owns the generic loading and startup-failure composition,
including brand placement, animated status dots, retry/details actions and the
error-details dialog content. Applications supply product assets and copy;
`nive-runtime` supplies lifecycle state and internal messages.

## Toast Host

`ToastHost` owns the generic toast overlay: corner positioning, hover
pause/resume wiring and dismissible toast rows built from the
`ToastPresentation` contract (defined in `nive-core`, reexported here).
`nive-runtime::ToastItem` implements `ToastPresentation` by depending on
`nive-core` directly, so the runtime owns toast identity, visible/queued
state, promotion and timing while `nive-ui` owns only the visual composition.
The runtime applies the host automatically to app-role windows; applications
do not mount it themselves and toasts may remain visible alongside a modal
dialog.

## Command Palette

The `command_palette` widget provides a reusable action-driven search palette:

- `CommandPaletteRow` carries `id`, `label`, optional `description`, optional
  `shortcut_label`, an `enabled` flag, and the message emitted on activation.
  `CommandPaletteRow::activated()` returns `None` for disabled rows so the
  host submit handler can ignore them.
- `command_palette_filter` performs a case-insensitive substring match on the
  label and description. An empty query returns every index in input order.
- `command_palette_view` renders the search input, a scrollable list of rows
  with highlight, description and shortcut hint support, and an empty state.

The palette is intentionally a render helper. It does **not** own the open/
closed state, the query value, the highlighted row, or the keyboard navigation.
Apps wrap the result in a `DialogRequest` (or an app-owned overlay) and route
`ArrowUp`/`ArrowDown`/`Enter`/`Escape` themselves. The runtime shortcut
`Cmd+K` / `Ctrl+K` should activate the host wrapper.

`nive-runtime::command_palette_rows(&ActionMap<M>)` adapts an
`ActionMap<M>` into a `Vec<CommandPaletteRow<'_, M>>` so apps can drop the
palette directly on top of their existing action catalog without manually
mapping label/description/shortcut fields.

## Accessibility Contract

Nive's accessibility contract is the minimum expectation every new widget
must meet. The contract focuses on the affordances the framework can
enforce today; full platform accessibility will land when Iced ships the
upstream APIs.

### Interactive Widget Expectations

- **Icon-only interactive widgets** (`Button` with no label, `SelectableItem`
  used as an icon row, action rows in toolbars) MUST accept a label or
  tooltip string. `Button::tooltip`, `SelectableItem::tooltip`, and
  equivalent methods on every interactive widget provide the accessible
  name. Tests cover the construction paths.
- **Disabled and loading states** MUST be exposed through the widget API.
  `disabled()`, `loading()`, and visual variants keep the state explicit
  and prevent apps from hiding state behind a single boolean.
- **Error states** for fields MUST be reachable through
  `FieldValidation::Invalid` plus an error message. Silent failures are
  not acceptable.

### Overlay Keyboard Contract

- **Escape** dismisses modal dialogs and popovers. The framework helper
  `nive_runtime::is_escape_key_press(&Event)` detects the key, and
  `DialogRequest::dismiss_on_escape` / `dismiss_on_backdrop_or_escape`
  routes the dismiss message. The popover overlay also maps Escape to
  its own dismissal.
- **Tab and Shift+Tab** cycle focus through the overlay's focusable
  controls. `nive_ui::focus_trap::direction_from_event` resolves the
  direction from the keyboard event and `FocusDirection::Next` /
  `FocusDirection::Previous` execute the chained Iced focus operation.
  Modifier-only Tab (Ctrl/Alt/Cmd+Tab) is left to the application so
  platform shortcuts still work.
- **Enter** activates the focused button. Custom widgets that accept
  Enter (autocomplete, command palette) handle the key internally and
  surface the action through their message API.
- **Backdrop clicks** dismiss modal dialogs unless the dialog explicitly
  disables the behavior through `DialogDismiss::OnEscape` or
  `DialogDismiss::None`.

### New Widget Checklist

When adding or reviewing a Nive widget:

1. **Label or tooltip support** for icon-only or compact variants.
2. **Disabled and loading API** when the widget can be in those states.
3. **Escape and Tab behavior** for any overlay-like widget, or a clear
   reason the widget is not focus-trapped.
4. **Error or status API** for any widget that represents a resource,
   operation, or async state.
5. **Tests** for the pure keyboard helpers (`is_escape_key_press`,
   `direction_from_event`) and any state-driven accessibility affordance
   the widget provides.
