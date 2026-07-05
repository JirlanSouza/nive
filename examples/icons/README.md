# Icons Example

Demonstrates Nive's provider-neutral icon model.

## What it demonstrates

- Semantic `IconRole` rendering through the active theme catalog.
- App-owned `IconSymbol` rendering through the same `Icon` widget.
- A custom SVG registered in `icons.toml`.
- A theme catalog override for the `window-close` framework role.

## How to run

```bash
cd examples/icons
cargo run
```

Use `nive icons add-symbol Shield lucide:shield-check` to add another app
symbol to this example manifest.
