# Nive

[![CI](https://github.com/JirlanSouza/nive/actions/workflows/ci.yml/badge.svg)](https://github.com/JirlanSouza/nive/actions/workflows/ci.yml)

A Rust/Iced framework for building desktop applications.

## Features

- **Design System**: Semantic theme contracts, tokens, and reusable widgets
- **Application Lifecycle**: Window management, bootstrap, feedback, and devtools
- **Workbench Shell**: Fixed-region document, panel, diagnostics, command, and status surfaces
- **Anchored Popup Controls**: Collision-safe Tooltip/Popover, typed Menu,
  Select, and Autocomplete composition with one shared logical-focus root
- **Icon Management**: Theme-owned icon roles, app symbols, and provider-neutral `nive icons` CLI
- **Scaffolding**: `nive new` to start a project, `nive init` to add Nive to one
  you already have

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

Creating the app inside an existing Cargo workspace is fine — it is registered as
a member for you, so the first `cargo build` works.

To add Nive to a crate you already have, run `nive init` in it instead:

```bash
cd my-existing-crate
nive init --git https://github.com/JirlanSouza/nive --tag v0.1.0-alpha.1
```

It adds the dependency, sets up the icon workflow, and fills in whatever
boilerplate is missing. It never overwrites a file you wrote — anything it
skips is listed in the output.

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

## Examples

- [Counter](examples/counter/README.md) — Minimal app with `Application`, `Effect`, and `ScreenView`
- [Forms](examples/forms/README.md) — Typed form composition with submitted
  Checkbox/RadioGroup/Select, atomic Autocomplete results, immediate Switch
  settings, validation, and dialogs
- [Async Data](examples/async-data/README.md) — `Resource` with guarded `begin`/`settle` loading and app-owned operations
- [Multi Window](examples/multi-window/README.md) — Multiple windows with explicit `Window` enum
- [Theming](examples/theming/README.md) — Runtime theme switching with `Application::theme` override
- [Icons](examples/icons/README.md) — Roles, symbols, custom SVGs, and theme icon catalog overrides
- [Widget Gallery](examples/widget-gallery/README.md) — Deterministic public
  Tooltip, Popover, Menu, Select, and Autocomplete matrices alongside the full
  widget catalog
- [Workbench Monitor](examples/workbench-monitor/README.md) — Fixed-region
  monitor validating shared Tooltip/Menu consumers and a genuine typed service
  filter without replacing navigation-owned controls
- [File Picker](examples/file-picker/README.md) — Native file picker dialogs (feature-gated)
- [Devtools](examples/devtools/README.md) — Runtime state inspection panel (feature-gated)

### Popup-control reference runs

From the repository root:

```bash
rtk just widget-gallery-dev
rtk just example-dev forms
rtk just example-dev workbench-monitor

rtk cargo test --manifest-path examples/widget-gallery/Cargo.toml
rtk cargo test --manifest-path examples/forms/Cargo.toml
rtk cargo test --manifest-path examples/workbench-monitor/Cargo.toml
rtk just examples-check
```

The current anchored-overlay contract uses physical LTR Start/End alignment and
submenu arrows, immediate terminal visual states without interpolated popup
motion, and Iced 0.14 rectangular clipping for arbitrary EdgeToEdge Popover
descendants. Semantic names and popup state are retained for future
accessibility integration, but native accessibility-tree roles, names,
expanded state, active-descendant relations, and announcements are not yet
claimed.

available. The user captures and attaches the named Light/Dark, density,
open/focus, nested, narrow, and low-viewport screenshots. The agent reviews
only those supplied images, lands corrections, and requests replacement images;
manual sign-off remains incomplete until the user confirms the final evidence.
The agent does not capture manual-validation screenshots.

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
