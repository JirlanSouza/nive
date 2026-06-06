# xtask

## Role

`crates/xtask` owns project automation that is easier to maintain in Rust than shell. It should support build/dev workflows without becoming application runtime code.

## Internal Modules

- `main.rs`: command dispatch and automation behavior.

## Architectural Dependencies

Depends on:

- lightweight configuration/serialization crates needed by automation.

Used by:

- root automation workflows when a Rust helper is preferable to shell logic.

Must not depend on:

- application runtime crates unless a specific automation task truly requires inspecting their metadata
- UI or database runtime behavior

## Workflow

Automation changes should:

1. keep commands deterministic and script-friendly
2. return useful exit codes
3. avoid hidden writes outside the intended generated artifacts
4. be wired through `just` when intended for regular developer use

## Testing

- Add tests when command parsing or generated output behavior is nontrivial.
- Focused command: `cargo test -p xtask`.

## Rules

Do:

- keep automation explicit and reproducible
- prefer `just` as the discoverable command surface

Do not:

- put application behavior in `xtask`
- make regular dev workflows depend on undocumented direct binary invocation
