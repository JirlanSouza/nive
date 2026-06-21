# Runtime Settings And Session Contract

This document records the implemented contract for the first runtime-owned
settings and session persistence slice.

## Scope

`nive-runtime` owns only framework session state:

- effective user theme preference
- per-window size and position keyed by stable window keys
- future runtime session fields such as most-recent app window

Product settings remain app-owned. RAG Studio provider configuration,
project/database settings, indexing/chunking preferences and domain-specific
recent data must not move into `nive-runtime`.

## Public API Direction

Settings are opt-in. Apps that want runtime persistence configure an explicit
settings file path:

```rust
ApplicationConfig::new("acme-app")
    .settings(SettingsConfig::file(config_path))
```

The first implementation adds:

- `nive_runtime::settings::{SettingsConfig, RuntimeSession, WindowSession}`
- `WindowSessionSize` and `WindowSessionPosition` for explicit geometry data
- crate root and prelude reexports for the settings contract
- `ApplicationConfig::settings(SettingsConfig)`
- `ApplicationConfig::configured_settings() -> Option<&SettingsConfig>`

The runtime should not choose a platform config directory in the first slice.
Apps supply the path so Nive stays independent from product policy. A later
helper can wrap `dirs` if repeated apps need a framework default.

## Window Session Keys

Use `WindowSpec::session_key("workspace")` for stable restore keys:

```rust
ApplicationConfig::new("acme-app")
    .window(WindowKind::Workspace, WindowSpec::app().session_key("workspace"));
```

Rationale:

- it preserves the existing `ApplicationConfig::window(kind, spec)` shape
- it keeps the key close to the window behavior it persists
- `Option<&'static str>` keeps `WindowSpec` copyable
- apps can opt in per window instead of making every window persistent

Rules:

- Session keys are product-owned, stable and user-invisible.
- Windows without a key are never restored from or written to session state.
- Window size and position are restored before opening a keyed window and are
  updated from Iced `window::Event::Resized` and `window::Event::Moved` events.
- If duplicate keys are configured, the runtime should keep startup resilient,
  log the duplicate and let the later implementation define deterministic
  first-match behavior.
- `WindowCardinality::Multiple` can share one default key initially; per-window
  instance session keys are explicitly deferred.

## File Format

The runtime uses a small versioned JSON file backed by workspace `serde` and
`serde_json`.

```rust
struct RuntimeSessionFile {
    version: u32,
    session: RuntimeSession,
}
```

Decision:

- JSON is already a workspace dependency and is easy to inspect/debug.
- No TOML/bincode/RON dependency should be introduced for this slice.
- `nive-runtime` depends on `serde = { workspace = true, features = ["derive"] }`
  and `serde_json.workspace = true`.

Versioning:

- Version `1` is the initial format.
- Missing files produce the default runtime session.
- Unknown versions are recoverable and fall back to defaults.
- Corrupt JSON is recoverable and falls back to defaults.
- Read/write errors must not block startup.

## Startup And Load Timing

Load runtime settings synchronously during `Program::new`, after
`Application::config()` is available and before `NiveCore::new` builds
`ThemeController`.

Rationale:

- persisted theme preference can affect the splash/bootstrap windows too
- initial product windows open with the configured/persisted session state
- simple file reads are small enough to keep before Iced daemon startup

Precedence:

1. `ApplicationConfig` values are the fallback defaults.
2. A successfully loaded runtime session overrides persisted fields such as
   theme preference.
3. Runtime system-theme detection still resolves `ThemePreference::System`.

If settings fail to load, startup continues with `ApplicationConfig` defaults.
The runtime logs the error. User-facing surfacing can be added later once there
is a broader diagnostics model; the first implementation must not block or
fail app startup because of settings corruption.

## Save Timing

The implementation persists theme preference changes after
`RuntimeCommand::Theme` changes the configured preference, even if the effective
theme does not change because of the current system mode.

Window geometry persistence updates the in-memory runtime session from reliable
Iced window move/resize events and writes the same session file. Saving runs as
a runtime task; the user-visible update does not wait on disk I/O.

Window display mode persistence is deferred. Iced 0.14 exposes move and resize
events, but this slice does not model a portable user-driven maximized or
fullscreen mode change event.

## Error And Logging Model

Add a typed settings error only if the implementation needs more than the
existing platform-error wrapper.

Expected logs:

- `settings.load_missing path=...`
- `settings.load_failed path=... error=...`
- `settings.unsupported_version path=... version=...`
- `settings.save_failed path=... error=...`
- `settings.duplicate_window_key key=...`

Missing settings files are normal and should be logged at debug level at most.
Corrupt files and save failures should be warning-level logs.

## Test Names

Focused tests for this slice:

- `runtime_settings_disabled_by_default`
- `runtime_session_loads_theme_preference_before_theme_controller`
- `missing_session_file_falls_back_to_config_defaults`
- `corrupt_session_file_falls_back_to_config_defaults`
- `unknown_session_version_falls_back_to_config_defaults`
- `theme_preference_change_schedules_session_save`
- `window_spec_exposes_session_key`
- `windows_without_session_key_do_not_restore_session_state`
- `runtime_session_restores_window_size_and_position`
- `runtime_session_clamps_restored_window_size_to_spec_bounds`
- `runtime_session_ignores_window_state_without_session_key`
- `window_resize_updates_runtime_session`
- `window_move_updates_runtime_session`
- `duplicate_window_session_keys_are_reported_without_panicking`

Final verification for the settings implementation:

```text
rtk cargo fmt --package nive-runtime
rtk cargo test -p nive-runtime
rtk cargo check -p app-gui --all-targets
rtk cargo test -p app-gui --test nive_contract
rtk cargo test -p app-gui --features dev --test nive_contract
```
