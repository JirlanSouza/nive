# Multi-Window Example

Demonstrates Nive's multi-window support with an explicit `Window` enum.

## What it demonstrates

A main window that opens a detail window, with shared state between windows.

## Concepts exercised

- `enum Window { Main, Detail }` as the window kind type
- `ApplicationConfig::window(Window::Main, WindowSpec::app())` to register windows
- `ApplicationConfig::initial_window(Window::Main)` to set the startup window
- `Effect::window(WindowCommand::Open(Window::Detail))` to open a second window
- `WindowContext<Window>` matching to render different views per window
- Shared mutable state across windows

## How to run

```bash
cd examples/multi-window
cargo run
```
