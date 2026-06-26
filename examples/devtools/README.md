# Devtools Example

Demonstrates the Nive devtools panel for runtime state inspection.

## What it demonstrates

Launching the app with devtools enabled and inspecting `Resource` and `Operation` fields in the devtools panel.

## Concepts exercised

- `nive::run_with_devtools::<App>()` entry point
- `#[derive(Inspect)]` on the state struct
- `#[inspect(sample = sample_projects)]` for Resource payload simulation
- `#[inspect(input = sample_save_input)]` for Operation start/fail simulation
- Disabled controls with tooltips when a capability is not declared
- Feature-gated via `nive = { features = ["devtools"] }`

## How to run

```bash
cd examples/devtools
cargo run
```

The devtools panel opens as a separate window showing the app's inspectable state fields with loading/error/idle controls and capability-gated payload controls.
