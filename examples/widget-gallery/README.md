# Widget Gallery

Runnable gallery for inspecting the current Nive widget baseline.

It demonstrates the public widget families exported by `nive-ui`: actions,
inputs, display, layout/navigation, overlays, feedback/state, theme roles,
icons, and motion primitives. The examples intentionally use current widget
APIs and theme behavior without redesigning or compensating for visual issues.

The gallery exercises:

- variant matrices for size, tone, disabled, loading, long-label, and compact
  states
- deterministic Input states, typed InputGroup slots, Field support/error
  ownership, multi-field Vertical/Wrap groups, and primary/secondary/tertiary/
  destructive Button hierarchy plus advanced axes
- controlled Checkbox Unchecked/Checked/Mixed/error states, typed RadioGroup
  layouts, exact inline/setting Switch compositions, and typed Default/Linked
  SegmentedControl fixtures
- **density switching** between Comfortable, Standard, and Compact through
  theme/catalog selection
- app-owned state for inputs, selections, overlays, feedback controls, and
  theme preference
- runtime-owned one-root-per-window logical focus across real Nive controls:
  pointer/touch entry hides `Auto` rings, keyboard traversal shows them, empty
  presses retain the sequential anchor, and composite highlight/selection stay
  independent
- semantic icon roles, a fixture app symbol, and a theme catalog override on
  the icons page
- Tooltip matrices for isolated/scoped timing, pointer/focus/Escape,
  Top/Right/Bottom/Left collision, long text, disabled explanation, and
  independent semantic names
- Popover matrices for all inset, width, collision, focus, nesting, lifecycle,
  low-height scrolling, and one-surface ownership policies
- canonical typed Menu matrices for commands, checkboxes, radio groups,
  submenus, separators, shortcuts, destructive/display-only/disabled state,
  long labels/lists, typeahead, scrolling, and nested Escape
- typed Select matrices across density and ControlSize with Field integration,
  selected/placeholder/open/invalid/display-only/disabled, empty/duplicate/
  missing models, collision, scrolling, pointer, and bounded keyboard behavior
- atomic typed Autocomplete matrices for Suggestions/Loading/Empty/Error,
  None/First highlight, Unicode emphasis, clear/Spinner, callback absence,
  retrieval-versus-validation error, disabled/duplicate rows, narrow/low
  layout, Enter pass-through, and pointer-before-blur ordering
- real modal overlay behavior for dialogs and the canonical `CommandPalette`
  (Cmd+K / Ctrl+K or the "Open command palette" trigger), including
  controlled query, filtered/empty results, and keyboard navigation
- the canonical runtime-hosted `ToastHost` driven only through `Effect::toast`:
  each tone, a five-toast burst that queues past the three-visible cap and
  promotes on dismiss, an optional persistent action, hover/keyboard-focus/
  modal pause and resume, the non-default `TopEnd` logical position, and a
  narrow-viewport width clamp; a summary-only `Toast::error` with full
  diagnostics reserved for `ErrorDetailsDialog`
- structural stress cases for long/narrow `SectionHeader`, transparent Toolbar
  groups and contained overflow, adjacent Panel header/body anatomy, 12/6
  overlay scrollbars, the complete semantic Separator matrix, and interactive,
  locked, display-only, invalid-minimum, and constrained SplitPane states;
  SplitStack fixed/fill sizing, divider independence, stop-at-minimum
  saturation, collapse by dragging past a minimum with the pre-drag width
  restored, locked, display-only, and degraded-input states
- controlled TabBar states for active, inactive, dirty, pinned, closable,
  disabled, long-label, overflow/menu, keyboard, context, reorder, and tear-off
  behavior; both physical SideRail sides with labels, icons, and overflow; and
  SelectableItem size/selection/disabled/trailing-content comparisons
- Card/ActionCard/SelectableCard filled, outlined, elevated, and ghost
  comparisons on Canvas, including 48 px targets, callback absence,
  controlled selected/disabled combinations, selection indicator, keyboard
  focus, and long leading/title/description/trailing content
- surface-free and externally framed MetricCard values with baseline units,
  status, and trend; ContentAction label/icon/loading/destructive/disabled
  states plus whole-control wrapping and the oversized-action clipping fallback
- surface-neutral KeyValueList/DataRow in standalone and hosted forms, typed
  text/code/custom values, shared/invalid label columns, mixed status slots,
  protected peer actions, selectable-row ownership, and constrained overflow
- Count/Status Badge, labelled StatusIndicator, 6/8 px ToneDot, disabled and
  clipped-host stress, empty status omission, and separate Spinner activity
- InitialAvatar person/entity matrices across 24/32/40/56 px, Unicode and
  identity-icon fallback, status-outline contexts, plus exact and constrained
  MetadataTag values
- opt-in devtools inspection for sample `Resource` and `Operation` fields
- module-public helper coverage for `nive::ui::widgets::skeleton`, which is not
  re-exported from `crates/nive-ui/src/widgets.rs`

Composite or utility exports are covered through their owning widgets:

- `Button`, `ButtonVariant`, `TextInputAppearance`, `InputGroupVariant`,
  `SplitPaneConstraints`, `SplitSizing`, and `SplitResize` are exercised through
  button, input group, split pane, and split stack variants.
- `IconSource`, `ErrorPresentation`, `ResourceStatusPresentation`, and
  `OperationStatusPresentation` are support contracts for the demonstrated
  widgets rather than standalone visuals. Autocomplete uses only typed
  suggestions/results and exposes no message adapter or arbitrary row content.
- `ErrorFeedbackAction`, `ErrorFeedbackActionRow`, and
  `ErrorFeedbackCommandRole` are covered by error feedback/status action rows.
- `RgbHexColor` is shown next to `ColorPicker` as the current color's normalized
  hex representation.

Run it with:

```bash
cargo run --manifest-path examples/widget-gallery/Cargo.toml
```

From the repository root, run it with terminal-triggered rebuild/reload:

```bash
just widget-gallery-dev
```

Run devtools explicitly when inspecting simulator integration:

```bash
just widget-gallery-devtools
```

Check it with:

```bash
cargo test --manifest-path examples/widget-gallery/Cargo.toml
cargo check --manifest-path examples/widget-gallery/Cargo.toml
just examples-check
```

For visual review, inspect light and dark themes at wide, narrow, and low
window sizes. Exercise toolbar overflow, long-title tooltips, scrollbar
hover/drag, TabBar wheel/menu/keyboard/drag/cancellation, both rail sides,
SelectableItem focus, and SplitPane hover, drag, focus, locked, and display-only
states. On the Actions page, traverse cards and content actions with Tab,
Enter, and Space; compare all three densities and constrain the window until
ActionGroup wraps without splitting a control.

For managed-focus sign-off, enter representative Input, Button, selection,
TabBar, Tree, SplitPane, and Popover paths by pointer/touch and keyboard. Check
that only keyboard-origin focus paints an `Auto` ring, inputs retain native
caret/blur behavior, empty presses do not reset the following Tab position,
composites keep one outer Tab stop, and overlay dismissal restores only when
its cause permits. These optical assertions require the user-supplied review
below; automated focus tests do not replace it.

On Inputs, also exercise Checkbox pointer/Space transitions and error wrapping,
RadioGroup arrows/Space/disabled skipping and horizontal wrapping, immediate
Switch endpoints, and SegmentedControl bounded keyboard navigation/truncation.

For the popup-control review, inspect Standard Light/Dark, Compact Xs, and
Comfortable Lg at wide, narrow, and low window sizes. Exercise Tooltip timing,
scope and collision; Popover insets, chrome, focus and nested priority; Menu
columns, durable state, keyboard, submenu and scrolling; Select open/invalid/
empty/Field behavior; and Autocomplete result, Unicode, focus, Enter, clear,
blur, and pointer-selection flows.

For the CommandPalette review, on the Overlays page open the palette
(Cmd+K/Ctrl+K or its trigger button) and exercise, grouped by area: (a)
open/close and the controlled query as you type; (b) filtered results versus
the distinct empty-query and no-match empty states; (c) `ArrowUp`/`ArrowDown`
navigation over eligible rows (the disabled "Delete project" row is skipped),
`Enter` activation, and that `Home`/`End`/`Left`/`Right`/text still reach the
search `Input`; (d) long content and a constrained/narrow/low viewport, where
only the result list scrolls while the input and frame stay fixed. Confirm
`Escape` and an outside press each dismiss exactly once, and that opening a
Dialog while the palette is open replaces it (one modal session per window).

For the Toast review, on the Feedback page exercise, grouped by area: (a)
each tone button, then "Push 5 toasts" — confirm at most three are visible
with the remainder queued, and that a queued toast starts its own duration
only once dismissal promotes it into the visible stack; (b) hover a visible
toast and confirm it stops counting down, move the pointer away and confirm
it resumes from the remaining duration rather than resetting, then repeat by
Tab-focusing a toast's dismiss button instead of hovering; (c) "With action" —
confirm it stays visible until you press "Restart now" or dismiss it, and
that pressing the action also dismisses it; (d) "Push error toast" — confirm
the card shows only the safe summary, never the diagnostic detail; (e) open a
Dialog or the CommandPalette while toasts are visible — confirm existing
toasts render beneath its scrim and stop counting down, and that a toast
pushed while the modal is open stays queued until it closes; (f) resize the
window narrow and confirm the stack keeps its clearance from the viewport
edge. Toasts render top-right (`ToastPosition::TopEnd`) rather than the
`BottomEnd` default, to prove the corner is actually configurable and not
hardcoded.

For the Tree review, use the Trees section on the Layout & Navigation page.
It exercises the full contract through public APIs only:

| Scenario | How to reach it |
| --- | --- |
| Expansion, indentation, guides | expand/collapse `examples` → `widget-gallery` → `src` → `pages` |
| Deferred loading | expand `remote-packages`; observe the loading placeholder, then the loaded `schema.json`/`cache.bin` children |
| Failed branch with retry | expand `remote-config`; it always fails after a short delay — observe the canonical error row, then press its retry affordance to re-trigger the same failure |
| Empty branch affordance | expand `archived`; observe the canonical empty row, distinct from a collapsed branch |
| Selection modes | toggle Single/Multiple; in Multiple, primary-modifier-click for additive selection and Shift-click for a range |
| Keyboard navigation and type-ahead | Tab into the Tree, then Up/Down/Left/Right/Home/End/PageUp/PageDown and type a label's first letters |
| Disabled row skipping | `target` is disabled — confirm navigation, selection, and drag/drop skip it |
| Context-menu-via-Menu | right-click a row; Tree emits `ContextRequested` only, and the app hosts the canonical `Menu` at the pointer position with Rename/Copy/Delete commands |
| Drag/drop affordances | drag a row onto another; observe the dragging row dim and the Before/After/Into drop-target indicator on the row under the pointer |
| `TreeItem` primitive | the adjacent "TreeItem primitive" panel composes rows directly, with no owned hierarchy/selection/focus state |

Current platform limits are explicit: Start/End and submenu Right/Left are
physical LTR; popup and chevron visuals change immediately without interpolated
motion; arbitrary EdgeToEdge Popover descendants receive rectangular Iced 0.14
clipping rather than a generic rounded mask; retained semantic metadata does
not yet emit native accessibility-tree roles, names, expanded state,
active-descendant relations, or announcements; and Toast's tone-to-politeness
mapping and "announce only the newest toast" semantics are preparatory only,
since no native AccessKit live-region emission exists in this Iced version.

Run `just widget-gallery-dev` and keep the review app available while the
reviewer captures the named Standard Light/Dark,
Compact Xs, Comfortable Lg, wide/narrow/low, hover/focus/open, nested,
truncation, result-state, keyboard, Tree (expansion, loading/failed-with-
retry/empty child states, selection modes, keyboard focus versus selection,
context-menu-via-Menu, drag/drop), and Toast (tone, queue/promotion, hover/
focus pause and resume, action, error-summary, beneath-scrim/modal-pause,
narrow-width) screenshots. Review the supplied images, apply corrections, and
request replacements when needed. Sign-off remains open until the reviewer
confirms the final supplied evidence.

With devtools enabled, open the panel with Cmd+Option+I on macOS or Ctrl+Alt+I
on other platforms to force the inspected feedback sample states.
