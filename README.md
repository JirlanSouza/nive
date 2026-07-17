# Nive

[![CI](https://github.com/JirlanSouza/nive/actions/workflows/ci.yml/badge.svg)](https://github.com/JirlanSouza/nive/actions/workflows/ci.yml)

A Rust/Iced framework for building desktop applications.

## Features

- **Design System**: Semantic theme contracts, tokens, and reusable widgets
- **Application Lifecycle**: Window management, bootstrap, feedback, and devtools
- **Workbench Shell**: Fixed-region document, panel, diagnostics, command, and status surfaces
- **Icon Management**: Theme-owned icon roles, app symbols, and provider-neutral `nive icons` CLI
- **Scaffolding**: `nive new` CLI for quick project setup

## Quick Start

> **Alpha channel (pre-crates.io):** Install from GitHub using an alpha tag.

```bash
# Install the CLI from the GitHub alpha channel
cargo install --git https://github.com/JirlanSouza/nive --tag v0.1.0-alpha.1 --locked nive-cli

# Create a new app (uses the same alpha tag as the dependency)
nive new my-app --git https://github.com/JirlanSouza/nive --tag v0.1.0-alpha.1

# Build and run
cd my-app
cargo build
cargo run
```

> **crates.io (final release):** `cargo install nive-cli` and `nive = "0.1"` will be
> the install path after the v0.1.0 crates.io publication.

## Crates

- [`nive`](crates/nive): Umbrella crate that re-exports everything
- [`nive-ui`](crates/nive-ui): Visual design system (tokens, theme, widgets, icons)
- [`nive-runtime`](crates/nive-runtime): Application lifecycle, window management, feedback
- [`nive-runtime-derive`](crates/nive-runtime-derive): Proc macros for devtools
- [`nive-workbench`](crates/nive-workbench): Fixed-region professional desktop shell
- [`nive-cli`](crates/nive-cli): CLI for scaffolding and icon management

## Platform Notes

### macOS
The app icon is set at runtime via `objc2-app-kit`. No build-time setup is needed — the icon is embedded automatically.

### Windows
The app icon is embedded into the `.exe` at build time via `winres`. Add a `build.rs` to your app:

```rust
fn main() {
    #[cfg(target_os = "windows")]
    nive_runtime::platform::app_icon::install_app_icon_at_build(
        "assets/icons/app.ico",
    );
}
```

The icon path is relative to your crate's `Cargo.toml`. If the file is missing, the build emits a warning and the executable uses the default OS icon.

### Linux
On first launch, Nive installs a `.desktop` entry and icon PNG to `~/.local/share/` so GNOME/KDE show your app's branded icon in the launcher. Subsequent launches are idempotent. Upgrading the app version triggers a one-time overwrite of the `.desktop` file.

## Documentation

- [Getting Started Guide](docs/guides/getting-started.md)
- [Adding Icons](docs/guides/adding-icons.md)
- [Development Guide](docs/development.md)
- [Architecture](docs/agents/architecture.md)

## Examples

- [Counter](examples/counter/README.md) — Minimal app with `Application`, `Effect`, and `ScreenView`
- [Forms](examples/forms/README.md) — Typed form composition with submitted Checkbox/RadioGroup, immediate Switch settings, validation, and dialogs
- [Async Data](examples/async-data/README.md) — `Resource` with guarded `begin`/`settle` loading and app-owned operations
- [Multi Window](examples/multi-window/README.md) — Multiple windows with explicit `Window` enum
- [Theming](examples/theming/README.md) — Runtime theme switching with `Application::theme` override
- [Icons](examples/icons/README.md) — Roles, symbols, custom SVGs, and theme icon catalog overrides
- [Widget Gallery](examples/widget-gallery/README.md) — Deterministic visual matrices for public widgets, including typed selection controls
- [Workbench Monitor](examples/workbench-monitor/README.md) — Fixed-region monitor validating typed setting/filter selection without replacing navigation-owned controls
- [File Picker](examples/file-picker/README.md) — Native file picker dialogs (feature-gated)
- [Devtools](examples/devtools/README.md) — Runtime state inspection panel (feature-gated)

## Development

```bash
# Format code
just fmt

# Check all crates
just check

# Run tests
just test

# Build documentation
just doc

# Run CI-like readiness checks
just readiness

# Sync framework icons
just icons-sync
```

## License

Apache-2.0
