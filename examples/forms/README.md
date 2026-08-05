# Forms Example

Demonstrates Nive form widgets and dialog patterns.

## What it demonstrates

A responsive typed contact form with validation feedback and a confirmation dialog.

## Concepts exercised

- labelled `FieldGroup` composition with deterministic wrapping and propagated sizing
- typed `Field` controls with Required/Optional metadata, reserved hint/error support, and label focus
- editable, explicit read-only, and disabled `Input` states using `on_change`
- `InputGroup` semantic adornments and a named clear action
- primary/secondary action hierarchy and error-owned validation chrome
- submitted tri-state `Checkbox`, required typed `RadioGroup`, and an immediate
  `Switch::setting` alongside display-only/disabled selection states
- typed `Select<AccountTier>` inside `Field`, with app-owned selection,
  placeholder, disabled Enterprise option, open/close messages, empty fixture,
  invalid correction, and submission
- typed organization `Autocomplete<Organization>` inside `Field`, with
  app-owned query/filter/order/selection and atomic Suggestions, Loading,
  Empty, and retrieval Error fixtures; retrieval failure remains separate from
  Field validation
- distinct Autocomplete query change, clear, selection, Input submit, blur,
  and controlled dismissal paths, including Enter without highlight and
  pointer selection before blur
- `DialogRequest` with `dismiss_on_backdrop` and `dismiss_on_escape`
- `Toast::success` for submission feedback
- `ScreenView::dialog` for modal presentation

## How to run

```sh
just example-dev forms
```

Equivalent standalone run from the repository root:

```sh
cargo run --manifest-path examples/forms/Cargo.toml
```

Check it from the repository root:

```sh
cargo test --manifest-path examples/forms/Cargo.toml
cargo check --manifest-path examples/forms/Cargo.toml
just examples-check
```

Select and Autocomplete use the public Nive popup contracts without a second
Panel, Scrollable, field frame, local focus coordinator, or styling repair.
Popup visuals are immediate; Start/End placement is physical LTR; and retained
name/open/value/highlight metadata does not yet emit native accessibility-tree
roles, names, expanded state, active-descendant relations, or announcements.

For manual sign-off, keep the dev command running while the reviewer captures
and attaches initial, Select open, Autocomplete
Suggestions/Loading/Empty/retrieval-error, invalid submit, corrected typed
choices, clear, Enter-without-highlight submit, pointer-before-blur, disabled/
display-only, submitted, narrow, and low-viewport screenshots in representative
Light/Dark densities. Review those images, apply corrections, and request
replacements when needed. Sign-off remains open until the reviewer confirms the
final supplied evidence.
