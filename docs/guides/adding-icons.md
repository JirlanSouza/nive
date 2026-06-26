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

### External Apps (cargo install)

If you installed Nive via `cargo install nive-cli`, use the `nive` CLI directly:

```bash
# Initialize icon manifest
nive icons init

# Add icons
nive icons add User user
nive icons add Settings settings
nive icons add Home home

# Sync icons (downloads SVGs and generates enum)
nive icons sync
```

### Monorepo Development

If you're working within the Nive monorepo, use the `just` recipes:

```bash
# Initialize icon manifest
just icons-init

# Add icons
just icons-add User user
just icons-add Settings settings
just icons-add Home home

# Sync icons
just icons-sync
```

## Icon Management Commands

- `nive icons init` - Initialize icon manifest in your app
- `nive icons sync` - Download SVGs and generate enum
- `nive icons check` - Verify icons are up to date
- `nive icons add <Variant> <lucide-name>` - Add a new icon
- `nive icons list` - List all icons in manifest

## Using App Icons

```rust
use crate::widgets::icon_generated::IconName;

fn view(&self) -> Element<Message> {
    Icon::new(IconName::User).md().into()
}
```

## Finding Lucide Icons

Browse available icons at [lucide.dev](https://lucide.dev/icons/).

Use the kebab-case name (e.g., `arrow-right`, `check-circle`) when adding icons.
