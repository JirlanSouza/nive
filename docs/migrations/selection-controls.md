# Selection controls migration

This release standardizes application-facing selection controls around controlled,
typed values. Applications remain the durable state owner; callbacks request the
next value and callback absence produces a display-only control, not disabled
styling.

## Checkbox

`Checkbox::new` still accepts `bool`, but `on_toggle` now publishes
`CheckboxState`:

```rust
let checkbox = Checkbox::new("Include archived records", state)
    .on_toggle(Message::ArchivedChanged);
```

Replace boolean callback payloads with `CheckboxState::{Unchecked, Checked,
Mixed}`. User activation maps `Unchecked` to `Checked`, `Checked` to `Unchecked`,
and `Mixed` to `Checked`; users do not cycle into `Mixed`. The constructor is the
only state input. Checkbox owns its inline label, optional description, and
optional error. Whitespace-only errors are treated as absent.

## RadioGroup

Use one `RadioGroup` with typed `RadioOption` values instead of independent radio
widgets or grouped buttons. The group owns the visible legend, requirement,
description, error, selection, and callback:

```rust
let group = RadioGroup::new(
    "Delivery speed",
    selected,
    [
        RadioOption::new(Speed::Standard, "Standard"),
        RadioOption::new(Speed::Express, "Express"),
    ],
)
.required("Required")
.on_select(Message::SpeedChanged);
```

The outer `Option<T>::None` means no current selection. If “None” is selectable,
model it as an ordinary domain value, for example `Some(Preference::None)`.
Values must be unique. Duplicate values render a finite display-only fallback;
an absent selected value remains keyboard-recoverable.

## Switch

Replace `Switch::new(value).label(label)` with the composition matching the
interaction:

- `Switch::inline(label, value)` for an intrinsic inline choice.
- `Switch::setting(title, value).description(...)` for a fill-width setting row.

The old `label` forwarder remains deprecated in the first published release
containing this change and is removed in the immediately following published
release. Switch remains binary and immediate. Async persistence, pending,
failure, and retry behavior belong to the host. Motion remains immediate until
`establish-motion-preference-plumbing` and then
`adopt-motion-preference-in-selection-controls` supply the shared preference and
concrete transition.

## SegmentedControl

Replace per-item selection and messages with a typed group value:

```rust
let control = SegmentedControl::new(
    "Result layout",
    layout,
    [
        SegmentedOption::new(Layout::List, "List"),
        SegmentedOption::new(Layout::Grid, "Grid"),
    ],
)
.on_select(Message::LayoutChanged);
```

Canonical controls contain two through five unique options and exactly one
selected value. Options support a nonempty label and an optional leading icon.
Use `.linked()` instead of `.flat()`. More than five or open-ended choices move
to `Select`; longer visible one-of-many questions normally use `RadioGroup`.

`LegacySegmentedControl` and its status/badge-capable `SegmentedItem` remain a
deprecated bridge only in the first published release containing this change.
Both names, together with the deprecated `flat` forwarder, are removed in the
immediately following published release. Do not introduce new app-facing use of
the bridge.

## Semantics and accessibility

Visible names, controlled values, keyboard behavior, and preparatory semantic
metadata are retained. This release does not claim native accessibility-tree
roles, names, states, or relationships; those require the dedicated platform
accessibility foundation.
