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
- `widgets::navigation` — tabs, toolbars, canonical menus and command palette
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

`Dialog` owns one canonical anatomy: a fixed `DialogHeader`, a body that owns
the only vertical scroll region, and an optional fixed footer, painted as one
borderless `SurfaceRole::Dialog` frame (`ShapeSize::Lg`). `DialogSize::{Sm,
Md, Lg}` (420/560/720 px, `Sm` default) is the only width control — there is
no generic `Length`, raw width/height, raw padding, or raw radius builder.

```rust
Dialog::new(body)
    .size(DialogSize::Md)
    .header(DialogHeader::new("Title").description("Supporting copy"))
    .footer(DialogActionFooter::with_one(
        DialogAction::cancel("Cancel", Message::Cancel),
        DialogTerminalAction::primary("Save", Message::Save),
    ))
```

`DialogHeader` accepts owned/borrowed `Cow` title and description text, an
optional semantic `IconRole`, and an optional safe icon-only close action
(`.close(name, message)`) that is never destructive. `DialogFooter` hosts
arbitrary non-action footer content; `DialogActionFooter` is the canonical
action row — one required terminal `Primary`/`Destructive`
`DialogTerminalAction` plus at most two preceding `Cancel`/`Secondary`
`DialogAction`s, with measured responsive reflow (single row → status above
actions → stacked actions) and non-repeated-Enter activation of an enabled
Primary action (never Destructive).

`DialogHost` owns modal composition: it resolves the backdrop from the
theme's `SurfaceRole::Scrim` (no raw alpha), makes base content externally
inert (pointer/touch/keyboard/wheel/focus operations) while still drawing it,
gives Dialog-owned nested overlays (Select/Popover/Menu) first event
priority, and captures every remaining modal event so nothing clicks or
types through. Backdrop dismissal recognizes only a primary mouse/touch
press with a concrete position outside the frame; Escape recognizes only a
non-repeated keypress. `DialogInitialFocus::{First, Target(Id)}` resolves
initial focus (body content, then footer Cancel/Secondary, then the header
close affordance — never the terminal action), and a still-valid invoker is
restored as an inactive logical anchor on close. `.dialog_id(Id)` names a
declarative session so a rebuild with the same id doesn't recapture or
re-resolve focus, while a changed id replaces the workflow step.

At the runtime layer, `DialogRequest` (private fields; `dismiss(policy)`,
`dismiss_on_backdrop(...)`, `dismiss_on_escape(...)`,
`dismiss_on_backdrop_or_escape(...)`, `initial_focus(...)`, `id(...)`) and
`DialogDismiss` (non-exhaustive, composable backdrop/Escape routes) declare
what `ScreenView` hosts. `dismiss_on_backdrop`/`dismiss_on_escape` each
replace only their own route and preserve the other — chaining both no
longer silently drops the first.

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

## Tree

| Contract | Canonical API | Fixed semantics |
| --- | --- | --- |
| Controlled hierarchy | `Tree::new(nodes).state(&TreeState).on_event(...)` | app-owned `TreeState`, rebuilt `TreeNode`s, intent-only `TreeEvent` |
| Loaded branch | `TreeNode::branch(id, label, children)` | empty `Vec` renders one canonical empty-affordance row |
| Deferred branch | `TreeNode::branch_deferred(id, label)` | emits `ExpandRequested`, renders one loading placeholder row |
| Failed branch | `TreeNode::branch_failed(id, label, &error)` | `error: &impl ErrorPresentation`; renders one error row with retry (re-emits `ExpandRequested`) |
| Stateless row | `TreeItem::new(label)` | indentation, expander, hover/selection/disabled/focus styling, drag affordances; owns no state |

Node IDs are app-domain values, stable and unique within one rendered tree;
call `TreeState::retain_ids` when domain data changes. `TreeChildren` is
`Loaded`/`Deferred`/`Failed` and non-exhaustive. Loading, failed, and empty
rows are chrome: they never become selected, focused, type-ahead matched,
copied, or dragged, but they do count in `visible_index_of`/`scroll_offset_to`
rendered order.

`Tree` never depends on `nive-runtime`; `branch_failed` consumes the neutral
`nive_core::ErrorPresentation` contract (summary plus diagnostic detail), and
apps pass their own `Resource<T>`/`UserFacingError` failure without
conversion since `UserFacingError` already implements it. Context requests,
rename, clipboard, paste, and drop are intent only — Tree performs no domain
mutation, touches no system clipboard, and hosts no menu or inline editor.
Context requests carry a `SelectionSnapshot` and honor
`ContextSelectionBehavior`; the application hosts the canonical `Menu` at the
request position. Selection follows `SelectionMode::{None, Single, Multiple}`;
`Multiple` supports additive and Shift-range selection with a Tree-owned
anchor. Row focus renders independently from durable selection in both `Tree`
and `TreeItem`. Tree renders every expanded-visible row — it does not
virtualize the viewport, though the uniform-row geometry stays
virtualization-ready for a dedicated later change. Visible-traversal metadata
(depth, level, parentage, order, expanded/selected/disabled state, placeholder
rows) is recorded in the widget layer, but no native accessibility-tree role,
`aria-*` value, or active-descendant relation is claimed yet.

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
`TypographyScale` must include `BodyStrong`/`body_strong`, plus form-specific
`Control`/`control` and `ControlStrong`/`control_strong`.

## Form Controls And Composition

`theme::FormControlMetrics` projects the active theme's concrete
`ControlSize` into form geometry without changing finite custom heights.

| Density | Xs | Sm | Md | Lg |
| --- | ---: | ---: | ---: | ---: |
| Compact | 20 | 24 | 28 | 32 |
| Standard | 24 | 28 | 32 | 36 |
| Comfortable | 28 | 32 | 36 | 40 |

The projection supplies Control/ControlStrong 14 px typography, horizontal
padding, radius, icon size, gap, a 1 px field perimeter, and a layout-neutral
2 px focus stroke on a rectangle inset 1 px. Standard Input and InputGroup
reuse the same private frame; Embedded Input paints no duplicate chrome.

Input capability is independent from appearance:

| State | Focus/select/copy | Mutate/paste/IME | Submit | Actions |
| --- | --- | --- | --- | --- |
| Editable (`on_change`) | yes | yes | configured | configured |
| Explicit/callback-less read-only | yes | no | non-mutating configured submit | explicitly enabled group actions remain enabled |
| Disabled | no | no | no | no |

Canonical composition is label-first:

```rust
use nive_ui::prelude::*;

let fields = [
    Field::new("Name", Input::new("Enter a name", "").on_change(|_| ()))
        .required("Required")
        .hint("Public display name")
        .reserve_support_line(true),
    Field::new(
        "Amount",
        InputGroup::new(Input::new("Amount", "42"))
            .prefix("USD")
            .unit("monthly"),
    )
    .optional("Optional"),
];
let form: Element<'_, ()> = FieldGroup::new("Profile", fields)
    .description("Account details")
    .layout(FieldGroupLayout::Wrap { min_field_width: 240.0 })
    .into();
```

A nonempty Field error replaces its hint and is the sole Invalid source;
empty/whitespace errors normalize to absence. Errors combine 12 px Danger text
with the required 14 px `IconRole::ValidationError`. Field spacing resolves
from the active density (`Sm` label-to-control, `Xs` control-to-support).

Wrap uses `max(1, floor((W + G) / (M + G)))` columns and equal tracks
`max(0, (W - (columns - 1)G) / columns)`. Invalid minima normalize to 240 px;
unbounded hosts fall back to Vertical. FieldGroup paints no surface—Card or a
section owns visual boundaries.

InputGroup typed APIs are `prefix`, `unit`, `semantic_icon`, `status`,
leading/trailing actions, named `clear_action`, and `activity`. The deprecated
`leading_text`/`trailing_text` aliases remain for one release. Arbitrary slots
are rectangular escape hatches with caller-owned paint, masking, semantics,
and propagation.

Button shortcuts map to primary Suggested+Solid, secondary Neutral+Outline,
tertiary Neutral+Ghost, and destructive solid Danger. The public intent and
variant axes remain the advanced surface. Loading retains width and label,
uses a foreground-inheriting metric-sized Spinner, and suppresses activation.
Explicit-width labels ellipsize and disclose the complete Cow only while
truncated. Icon-only construction requires semantic text independently from a
tooltip.

## Selection Controls

| Intent | Canonical control | Value/callback |
| --- | --- | --- |
| Submitted independent choice | `Checkbox` | `CheckboxState` / `on_toggle` |
| One visible choice among options | `RadioGroup<T>` | `Option<T>` / `on_select` |
| Immediate binary setting | `Switch::inline` or `Switch::setting` | `bool` / `on_toggle` |
| Two through five fixed modes or filters | `SegmentedControl<T>` | `T` / `on_select` |
| Longer or open-ended option set | `Select<T>` | popup-backed selection |
| Documents/views with lifecycle | `TabBar` | navigation-owned active id |

`CheckboxState::Mixed` is an app-supplied aggregate value. Activation requests
`Mixed -> Checked`; there is no user cycle into Mixed. Checkbox owns its inline
label and optional error, while RadioGroup owns its legend and one group error.
Radio and segmented option values must be unique.

Choice outer heights follow the complete density-by-size form table above.
Checkbox/Radio indicators are 14/16/18/20 px for Xs/Sm/Md/Lg. Switch tracks are
28x16, 32x18, 36x20, and 40x22 px with a 2 px thumb inset. Focus is a
layout-neutral 2 px overlay and can coexist with selected or invalid state.

RadioGroup is one tab entry with circular physical LTR arrow navigation.
SegmentedControl is one tab entry with bounded Left/Right and Home/End. Callback
absence removes focus and hover/pressed behavior without applying disabled
colors. Native accessibility-tree emission is not claimed yet.

`Switch::new(value).label(...)`, `SegmentedControl::flat`,
`LegacySegmentedControl`, and `SegmentedItem` are bounded one-release migration
bridges. See `docs/migrations/selection-controls.md` from the repository root.

Input semantic names, Field labels/requirements/support, FieldGroup headings,
and icon action names are retained for a future accessibility bridge. Iced
0.14 does not currently let Nive emit the required native AccessKit
name/description/error/group relationships or independently paint caret color.

`TabBar`, `VerticalRail`, `SectionHeader`, flat `SegmentedControl`, and toolbar
actions derive their primary extent from the active theme's `ControlSize`
metrics. A workbench shell applies one shared size to those managed regions
rather than requiring callers to compensate with different per-widget sizes.

`SplitPane` defaults to `ControlSize::Sm` and exposes the standard
`size`/`xs`/`sm`/`md`/`lg` vocabulary. Its visual and layout divider remains one
logical pixel while its centered resize target is derived from the selected
control size, so interaction ergonomics do not change pane-ratio geometry.

## Anchored Overlays And Popup Controls

Anchored composition follows one ownership chain:

```text
private anchored geometry/lifecycle
        ├── Tooltip
        └── Popover
              └── Menu
                    ├── Select
                    └── Autocomplete
```

The private kernel owns translated anchor measurement, collision, an 8 px safe
viewport, a 4 px default gap, bounded overflow, nested event priority, message
relay, and integration with the shared logical-focus root. Application code
uses only the high-level types and policies.

### Tooltip and Popover

`Tooltip::new(anchor, text)` accepts passive supplementary text. Isolated
disclosure waits 500 ms. A `TooltipScope` gives different neighbors a 100 ms
delay while a shown Tooltip is visible or during its 600 ms warm window; the
same neighbor still waits 500 ms, and unshown candidates do not warm the scope.
Pointer intent wins over retained focus, nested scopes/windows remain isolated,
and at most one Tooltip appears per scope. Tooltip uses 12 px BodySmall text,
4x8 px padding, a 4 px radius and gap, a 280 px wrapping cap, and shared
flip-and-shift collision. Tooltip text never supplies the anchor's semantic
name. Disabled anchors may remain pointer-explainable but are not made
keyboard-focusable.

`Popover` is controlled through `open(bool)` and owns exactly one
`SurfaceRole::Popover` frame: fill, subtle 1 px perimeter, 8 px radius, compact
shadow, rectangular outer clip, semantic inset, and one bounded vertical
Scrollable. Insets are Standard 12 px, Compact 8 px, and EdgeToEdge 0 px.
Callers supply surface-free content without another `Panel`, radius, or
Scrollable. Arbitrary EdgeToEdge descendants do not receive a generic rounded
mask under Iced 0.14; Nive-owned lists add a 4 px inset for corner containment.

Default geometry is BottomStart, FlipAndShift, Content width, a 4 px gap, and
an 8 px safe viewport. Width is domain-specific:

| Policy | Contract before chosen-side clamp |
| --- | --- |
| `Content` | intrinsic content capped at 360 px |
| `MatchAnchor` | safe-clamped anchor width |
| `AtLeastAnchor` | anchor floor plus content growth capped at 360 px |
| `Fixed(x)` | finite nonnegative requested width |

`PopoverFocusPolicy::RetainAnchor` is the default. `FocusFirst` enters the
first enabled descendant and permits ordinary Tab exit; `Trap` cycles Tab
inside. Escape, outside primary press, or owned activation requests exactly
one dismissal when `on_dismiss` exists and conditionally restores the still-
valid opaque anchor after controlled closure. Callback absence creates no
hidden close and does not capture an outside press merely to simulate one;
programmatic close is silent.

```rust
use nive_ui::prelude::*;

let help = Tooltip::new(
    button::icon(IconRole::DialogInformation, "Deployment details"),
    "Inspect deployment health",
)
.placement(TooltipPlacement::Right);

let details = Popover::new(button::secondary("Details").on_press(()))
    .content(text("Surface-free content"))
    .open(true)
    .focus_policy(PopoverFocusPolicy::RetainAnchor)
    .on_dismiss(());
```

### Menu

`Menu::new(trigger)` internally owns the canonical EdgeToEdge FocusFirst
Popover. It uses fixed 28 px rows across density, stable independent choice and
icon tracks, one shortcut/annotation/submenu track, renderer-measured
truncation Tooltip, one root-composite focus target, bounded arrows, Home/End,
700 ms prefix typeahead, and Popover-owned scrolling. Physical Right/Left opens
and closes submenus under the current LTR contract.

```rust
use nive_core::Action;
use nive_ui::prelude::*;

let action = Action::new("project.rename", "Rename", ());
let menu = Menu::new(button::secondary("Actions").on_press(()))
    .open(true)
    .on_dismiss(())
    .command(MenuCommand::from_action(&action))
    .checkbox(
        MenuCheckbox::new("Pinned", CheckboxState::Checked)
            .on_toggle(|_| ())
            .dismiss_policy(MenuDismissPolicy::KeepOpen),
    )
    .separator()
    .command(MenuCommand::new("Delete").destructive().on_press(()));
```

Checkbox/radio state is app-controlled and distinct from transient highlight.
`DismissAll` publishes the leaf message first and then one dismiss message when
that capability exists; otherwise it publishes only the leaf and stays open.
`KeepOpen` never dismisses. Callback-less leaves are normal display-only rows,
not disabled rows; explicit disabled wins and preserves durable marks and
geometry.

### Select and Autocomplete

`SelectOption<T>` separates a unique application value from its Cow-compatible
label and disabled state. `Select<T>` accepts `Option<T>` committed selection,
defaults to fill width and Sm, converts to typed `FieldControl`, and renders
Menu-derived choices in one AtLeastAnchor Popover. Opening and bounded
navigation never commit. Enter/Space publishes a different highlighted value;
Escape/Tab cancel, and Tab continues traversal. Empty, duplicate, and missing-
selection models remain finite and never invent app state.

`AutocompleteSuggestion<T>` carries a unique value, label, optional leading
icon, optional trailing Secondary text, and disabled state.
`AutocompleteResults::{Suggestions, Loading, Empty, Error}` is atomic. The app
owns query, filtering/order, async retrieval, results, and committed
`Option<T>`. Input focus/caret remains in the field while logical highlight is
bounded and preserved by value. Enter without highlight passes to Input submit;
selection publishes only `on_select(T)` before blur and not `on_dismiss`.
Retrieval Error remains popup content and does not imply Field invalid state.

```rust
use nive_ui::prelude::*;

let tier = Field::new(
    "Account tier",
    Select::new(
        vec![SelectOption::new("starter", "Starter"),
             SelectOption::new("team", "Team")],
        Some("team"),
    )
    .on_select(|_| ()),
);

let organization = Field::new(
    "Organization",
    Autocomplete::new(
        "niv",
        None,
        AutocompleteResults::suggestions(vec![
            AutocompleteSuggestion::new(1_u64, "Nive Labs"),
        ]),
    )
    .open(true)
    .on_change(|_| ())
    .on_select(|_| ())
    .on_dismiss(()),
);
```

For both controls, Field owns the visible label, hint/error, validation,
ControlSize, disabled context, and focus association. Missing callbacks remove
only their capabilities and do not request disabled colors. Explicit disabled
has stronger precedence and suppresses interaction without changing geometry.

All Popover/Menu/Select/Autocomplete state changes are immediate in this wave;
visual interpolation belongs to `adopt-motion-preference-in-anchored-overlays`
after shared motion-preference plumbing. Start/End and submenu arrows are
physical LTR. Names, open/expanded state, values, result state, and logical
active rows are preparatory metadata only; no native accessibility-tree role,
name, expanded state, active-descendant relation, or announcement is claimed.

Keep category ownership explicit: TabBar owns documents, Toolbar owns chrome,
VerticalRail owns edge navigation, Dialog owns modal interaction,
CommandPalette owns command search, and specialized inputs retain their domain
contracts. Popup mechanics do not justify replacing those categories with
Select, Menu, or generic Popover.

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

`CommandPalette` is a self-contained typed composite, not a render helper: it
owns its viewport-centered top placement, single search-input focus target,
filtered keyboard navigation, highlight, ensure-visible scrolling, empty
state, and visual frame. It shares a private modal-hosting kernel with
`DialogHost` — base content stays drawn but externally inert while the
palette is open, `Escape` and an outside primary press each publish
`on_dismiss` exactly once, and a window hosts at most one canonical modal
session (a later `CommandPalette` or `Dialog` mount replaces rather than
stacks).

Hosts supply only `open(bool)`, the controlled query, filtered typed
`CommandPaletteItem`s, `on_query_change`, and `on_dismiss` — mirroring
`Menu::open`:

- `CommandPaletteItem` carries `id`, an optional leading `IconRole`, `label`,
  optional `description`, optional `shortcut_label`, an `enabled` flag, and
  the message emitted on activation. `CommandPaletteItem::activated()`
  returns `None` for disabled items so a disabled item renders but never
  publishes.
- `command_palette_filter` performs a case-insensitive substring match on the
  label and description. An empty query returns every index in input order;
  it stays the provided default matcher and can be swapped by the
  application.
- The search query is application-controlled: the application owns
  `query: String`, supplies `on_query_change`, and runs its own filtering
  (with `command_palette_filter` or its own). The widget owns only transient
  state — the highlight index, `ArrowUp`/`ArrowDown`/`Enter` result
  navigation, and ensure-visible scrolling. The single focus target is the
  native search `Input`, so `Home`/`End`/`Left`/`Right` and text stay its own
  caret and editing controls; `Escape` is never delegated to the Input (which
  would otherwise blur itself) and instead reaches the shared kernel's own
  dismissal handling.

Apps project an `ActionMap<M>` by iterating its actions and calling
`CommandPaletteItem::from_action(&Action<M>)`. The types are owned by
zero-dependency `nive-core`, so this projection needs no runtime dependency and
preserves identity, label, description, shortcut, enabled state, and activation
semantics. `nive-workbench`'s `runtime` feature provides
`action_palette_items(&ActionMap<M>)` as a ready-made projection helper;
`nive-workbench` itself hosts no bespoke command-palette type.

`ToolbarAction::from_action(&Action<M>)` projects the same command as a text
toolbar action. `ToolbarAction::from_action_with_icon(&Action<M>, IconRole)`
adds UI-owned icon decoration; icons, selection, destructive tone, loading,
and menu hierarchy do not become core action data. This follows Qt's useful
single-command-source idea without adopting mutable action objects or signals:
the application still rebuilds immutable actions from current state.

## Accessibility Contract

Nive's accessibility contract is the minimum expectation every new widget
must meet. The contract focuses on the affordances the framework can
enforce today; full platform accessibility will land when Iced ships the
upstream APIs.

### Interactive Widget Expectations

- **Icon-only Button widgets** require retained semantic text at construction;
  tooltip disclosure is independent and never substitutes for that metadata.
  Native accessible-name emission is still deferred. Other compact actions
  retain their app-owned visible/semantic labels according to their APIs.
- **Disabled and loading states** MUST be exposed through the widget API.
  `disabled()`, `loading()`, and visual variants keep the state explicit
  and prevent apps from hiding state behind a single boolean.
- **Typed Field error state** comes from one nonempty Field error, which drives
  both Invalid chrome and visible icon-plus-text support. Standalone controls
  may still use `FieldValidation`; silent or color-only failures are not
  acceptable.

### Managed Logical Focus

These state domains are related but not interchangeable:

- **Active focus** identifies the one target currently receiving focused
  interaction. Window deactivation or a press on empty content may remove it.
- **Visible focus** is only the paint decision for that active target. With
  `FocusVisibility::Auto`, keyboard origin shows the ring and pointer/touch
  origin hides it; it never changes geometry.
- **Logical anchor** is the unique retained sequential position owned by one
  `FocusRoot`. It may survive pointer blur so the next native Iced Tab
  operation continues from the last interacted control.
- **Composite-internal focus** is the highlighted row, roving tab, Tree row,
  range anchor, or active split handle owned by a composite. The composite has
  one outer `FocusState`; its internal item is not another window Tab stop.
- **Durable selection** is controlled application/domain state and persists
  independently of hover, focus, highlight, or ring visibility.
- **Overlay focus policy** decides entry, containment, dismissal-cause, and
  conditional restoration. It shares the root coordinator and never creates
  a popup-local focus manager.

`nive-runtime` installs one `FocusRoot` outside all content and overlay hosts
for each window. Standalone `nive-ui` applications wrap their final content
with `nive_ui::accessibility::FocusRoot`; unrooted widgets retain local focus
behavior without the cross-widget uniqueness/retention guarantee. External
custom focusables store `nive_ui::advanced::focus::FocusState` in persistent
Tree state, register it during `operate`, notify it after claimed pointer/touch
focus, and derive focus paint from `is_focus_visible()`.

### Overlay Keyboard Contract

- **Escape** dismisses modal dialogs through their configured policy. The framework helper
  `nive_runtime::is_escape_key_press(&Event)` detects the key, and
  `DialogRequest::dismiss_on_escape` / `dismiss_on_backdrop_or_escape`
  routes the dismiss message. Tooltip closes on Escape. Popover and the
  innermost Menu level publish one controlled dismissal only when that
  capability exists; Select/Autocomplete follow their documented cancel paths.
- **Tab and Shift+Tab** follow the overlay category. RetainAnchor keeps focus
  on the anchor, FocusFirst permits ordinary traversal exit, and Trap cycles
  inside the Popover. Menu contributes one composite Tab stop; Select and
  Autocomplete keep focus on their trigger/Input while popup highlight remains
  logical. Dialog trapping uses
  `nive_ui::focus_trap::direction_from_event`, `FocusDirection::Next`, and
  `FocusDirection::Previous` over the same root coordinator.
  Modifier-only Tab (Ctrl/Alt/Cmd+Tab) is left to the application so
  platform shortcuts still work.
- **Enter** activates the focused button. Custom widgets that accept
  Enter (autocomplete, command palette) handle the key internally and
  surface the action through their message API.
- **Backdrop clicks** dismiss modal dialogs only when the request configures
  a backdrop route (`DialogDismiss::backdrop(...)`,
  `DialogRequest::dismiss_on_backdrop(...)`, or
  `dismiss_on_backdrop_or_escape(...)`). Backdrop and Escape are independent
  routes on `DialogDismiss`: an absent route is still captured (never clicks
  or types through) but publishes nothing, and configuring one route never
  erases the other. `DialogDismiss::none()`/`Default` configures neither.
- **Dialog focus return** captures the managed target that preceded the modal.
  Closing from a dialog button, Escape, backdrop, or controlled state restores
  a still-valid target only as the logical anchor: no target remains actively
  focused and no ring is painted. A newer programmatic external target wins;
  a removed or disabled target falls back to native Iced traversal.

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
