# Counter Example

Minimal Nive application demonstrating the v0.1 application contract.

## What it demonstrates

A simple counter app that exercises the minimal surface of the `Application` trait.

## Concepts exercised

- `Application` trait with `type Window = ()` and `type Bootstrap = ()` defaults
- `ApplicationConfig::new` with `.name()`
- `init`, `update`, `view` lifecycle methods
- `AppUpdate` builder (returning `()` for no-op)
- `ScreenView` with `column!`, `row!`, `button`, `text` widgets
- `nive::Result` entry point

## How to run

```bash
cd examples/counter
cargo run
```
