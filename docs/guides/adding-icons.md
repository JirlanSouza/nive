# Adding Icons to Your Nive App

Nive provides an integrated icon management system using Lucide icons.

## Framework Icons

The framework includes essential icons available via `nive::ui::widgets::Icon`:

```rust
use nive::prelude::*;
use nive_ui::widgets::icon::Icon;

fn view(&self) -> Element<Message> {
    Icon::new(IconName::Search).md().into()
}
```

## Adding App-Specific Icons

Your app can have its own set of icons beyond the framework essentials.

### Initialize Icon Manifest

```bash
cargo run --package xtask --manifest-path ../nive/Cargo.toml -- icons init .
```

This creates:
- `icons/lucide.toml` - manifest for your app's icons
- `assets/icons/lucide/` - directory for SVG files
- `src/widgets/icon.generated.rs` - generated enum

### Add Icons

```bash
# Add a single icon
cargo run --package xtask --manifest-path ../nive/Cargo.toml -- icons add User user

# Add multiple icons
cargo run --package xtask --manifest-path ../nive/Cargo.toml -- icons add Settings settings
cargo run --package xtask --manifest-path ../nive/Cargo.toml -- icons add Home home
```

### Sync Icons

```bash
cargo run --package xtask --manifest-path ../nive/Cargo.toml -- icons sync
```

This downloads the SVGs from Lucide and generates the enum.

### Using App Icons

```rust
use crate::widgets::icon_generated::AppIcon;

fn view(&self) -> Element<Message> {
    Icon::new(AppIcon::User).md().into()
}
```

## Icon Management Commands

- `icons init <path>` - Initialize icon manifest in your app
- `icons sync` - Download SVGs and generate enum
- `icons check` - Verify icons are up to date
- `icons add <Variant> <lucide-name>` - Add a new icon

## Finding Lucide Icons

Browse available icons at [lucide.dev](https://lucide.dev/icons/).

Use the kebab-case name (e.g., `arrow-right`, `check-circle`) when adding icons.
