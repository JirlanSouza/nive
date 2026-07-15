# Widget Gallery

Runnable gallery for inspecting the current Nive widget baseline.

It demonstrates the public widget families exported by `nive-ui`: actions,
inputs, display, layout/navigation, overlays, feedback/state, theme roles,
icons, and motion primitives. The examples intentionally use current widget
APIs and theme behavior without redesigning or compensating for visual issues.

The gallery exercises:

- variant matrices for size, tone, disabled, loading, long-label, and compact
  states
- **density switching** between Comfortable, Standard, and Compact through
  theme/catalog selection
- app-owned state for inputs, selections, overlays, feedback controls, and
  theme preference
- semantic icon roles, a fixture app symbol, and a theme catalog override on
  the icons page
- real overlay behavior for popovers, dialogs, autocomplete, and command
  palette content
- structural stress cases for long/narrow `SectionHeader`, transparent Toolbar
  groups and contained overflow, adjacent Panel header/body anatomy, 12/6
  overlay scrollbars, the complete semantic Separator matrix, and interactive,
  locked, display-only, invalid-minimum, and constrained SplitPane states
- controlled TabBar states for active, inactive, dirty, pinned, closable,
  disabled, long-label, overflow/menu, keyboard, context, reorder, and tear-off
  behavior; both physical VerticalRail sides with metadata and overflow; and
  SelectableItem size/selection/disabled/trailing-content comparisons
- Card/ActionCard/SelectableCard filled, outlined, elevated, and ghost
  comparisons on Canvas, including 48 px targets, callback absence,
  controlled selected/disabled combinations, selection indicator, keyboard
  focus, and long leading/title/description/trailing content
- surface-free and externally framed MetricCard values with baseline units,
  status, and trend; ContentAction label/icon/loading/destructive/disabled
  states plus whole-control wrapping and the oversized-action clipping fallback
- opt-in devtools inspection for sample `Resource` and `Operation` fields
- module-public helper coverage for `nive::ui::widgets::skeleton`, which is not
  re-exported from `crates/nive-ui/src/widgets.rs`

Composite or utility exports are covered through their owning widgets:

- `Button`, `ButtonVariant`, `TextInputAppearance`, `InputGroupVariant`,
  `SplitPaneConstraints`, and `SplitPaneDirection` are exercised through button,
  input group, and split pane variants.
- `AutocompleteMessage`, `IconSource`, `ErrorPresentation`,
  `ResourceStatusPresentation`, and `OperationStatusPresentation` are support
  traits/messages for the demonstrated widgets rather than standalone visuals.
- `ErrorFeedbackAction`, `ErrorFeedbackActionRow`, and
  `ErrorFeedbackCommandRole` are covered by error feedback/status action rows.
- `RgbHexColor` is shown next to `ColorPicker` as the current color's normalized
  hex representation.

Run it with:

```bash
cargo run
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
cargo check
```

For visual review, inspect light and dark themes at wide, narrow, and low
window sizes. Exercise toolbar overflow, long-title tooltips, scrollbar
hover/drag, TabBar wheel/menu/keyboard/drag/cancellation, both rail sides,
SelectableItem focus, and SplitPane hover, drag, focus, locked, and display-only
states. On the Actions page, traverse cards and content actions with Tab,
Enter, and Space; compare all three densities and constrain the window until
ActionGroup wraps without splitting a control.

With devtools enabled, open the panel with Cmd+Option+I on macOS or Ctrl+Alt+I
on other platforms to force the inspected feedback sample states.
