# Widget Gallery

Runnable gallery for inspecting the current Nive widget baseline.

It demonstrates the public widget families exported by `nive-ui`: actions,
inputs, display, layout/navigation, overlays, feedback/state, theme roles,
icons, and motion primitives. The examples intentionally use current widget
APIs and theme behavior without redesigning or compensating for visual issues.

The gallery exercises:

- variant matrices for size, tone, disabled, loading, long-label, and compact
  states
- app-owned state for inputs, selections, overlays, feedback controls, and
  theme preference
- real overlay behavior for popovers, dialogs, autocomplete, and command
  palette content
- devtools inspection for sample `Resource` and `Operation` fields
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

Check it with:

```bash
cargo check
```

Open devtools with Cmd+Option+I on macOS or Ctrl+Alt+I on other platforms to
force the inspected feedback sample states.
