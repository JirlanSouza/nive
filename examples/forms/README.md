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
- `DialogRequest` with `dismiss_on_backdrop` and `dismiss_on_escape`
- `Toast::success` for submission feedback
- `ScreenView::dialog` for modal presentation

## How to run

```sh
rtk just example-dev forms
```

Check it from the repository root:

```sh
rtk cargo test --manifest-path examples/forms/Cargo.toml
rtk cargo check --manifest-path examples/forms/Cargo.toml
```

For manual sign-off, the agent launches the dev command and keeps the app
available. The user captures and attaches initial, invalid-submit,
Mixed-to-Checked, required RadioGroup correction, immediate Switch,
corrected/valid-submit, disabled, label-focus, and narrow-wrap screenshots in
representative Light/Dark densities. The agent
reviews only those supplied images and requests replacements after visual
corrections; it does not capture screenshots itself.
