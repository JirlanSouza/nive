# Forms Example

Demonstrates Nive form widgets and dialog patterns.

## What it demonstrates

A contact form with input fields, validation feedback, and a confirmation dialog.

## Concepts exercised

- `Field` with labels and error messages
- `Input` with `on_input` and `FieldValidation`
- `DialogRequest` with `dismiss_on_backdrop` and `dismiss_on_escape`
- `Toast::success` for submission feedback
- `ScreenView::dialog` for modal presentation

## How to run

```bash
cd examples/forms
cargo run
```
