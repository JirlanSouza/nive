# Theming Example

Demonstrates runtime theme switching with `Application::theme` override.

## What it demonstrates

Switching between System, Light, and Dark themes at runtime.

## Concepts exercised

- `Application::theme` override to return stored `ThemePreference`
- `AppUpdate::theme(ThemePreference::Dark)` to trigger runtime theme change
- `ThemePreference` enum (`System`, `Light`, `Dark`)
- `ThemeCatalog::NIVE` built-in catalog (implicit via default config)

## How to run

```bash
cd examples/theming
cargo run
```
