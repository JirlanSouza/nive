# Legacy Areas

## Deprecated Areas

These areas remain in the repository for future reference, migration, compatibility checks, or salvage work, but they are not normal design sources:

- `app-ui`
- `crates/app-tauri`

Do not inspect or modify them unless the user explicitly asks for migration, compatibility, historical comparison, or reference extraction.

## Workspace Status

`crates/app-tauri` is intentionally not an active Cargo workspace member. Keep it in the repository unless the user explicitly asks to remove the legacy code.

`app-ui` is a legacy React/Vite frontend. Do not use its patterns for current Rust/Iced UI work.
