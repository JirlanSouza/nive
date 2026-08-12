# Adding Icons to Your Nive App

Nive icons are theme-owned design assets. Framework widgets request semantic
`IconRole` values, and the active `Theme` resolves those roles through an
`IconCatalog`. App-owned concrete icons are generated as `IconSymbol` values.

## Rendering Icons

```rust
use nive::prelude::*;

fn view(&self) -> Element<'_, Message> {
    Icon::role(IconRole::EditFind).md().into()
}
```

Generated app symbols render through the same widget:

```rust
use crate::icons::IconSymbol;
use nive::prelude::*;

fn view(&self) -> Element<'_, Message> {
    Icon::symbol(IconSymbol::User).md().into()
}
```

## Manifest

Apps use an app-root `icons.toml`:

```toml
[provider.lucide]
version = "0.460.0"
stroke_width = "2"
stroke_linecap = "round"
stroke_linejoin = "round"

[roles]
window-close = "lucide:x"
edit-find = "lucide:search"

[symbols]
User = "lucide:user"
BrandMark = "custom:brand-mark"

[custom]
brand-mark = "assets/icons/custom/brand-mark.svg"
```

Provider refs in the manifest are explicit: `lucide:<slug>` or
`custom:<name>`. CLI edit commands accept bare Lucide shorthand such as
`user`, but write `lucide:user` back to the manifest.

## Commands

```bash
nive icons init
nive icons add-symbol User user
nive icons set-role window-close lucide:x
nive icons add-custom brand-mark assets/icons/custom/brand-mark.svg
nive icons sync
nive icons check
```

Discovery commands are separate from build/check workflows:

```bash
nive icons search user --provider lucide
nive icons list --provider lucide --category users
nive icons show lucide:user
nive icons gallery --provider lucide
```

`nive icons check` is offline. It validates `icons.toml`, generated Rust
modules, generated SVG assets, custom SVG paths, and stale files without
fetching provider data.

Generated Rust is deterministic and compatible with the default `rustfmt`
configuration. Repeating `nive icons sync` does not change the generated
modules, and the supported verification sequence is:

```bash
nive icons sync
nive icons check
cargo fmt --check
```

Your manifest is *additive* over the framework catalog: declare only the roles
you override. A role you never mention resolves to Nive's default glyph, and a
new `IconRole` in a later Nive release will not invalidate your manifest.

The CLI checks that a role name is well-formed kebab-case, not that your Nive
version declares it — it generates code for whichever `nive` your `Cargo.toml`
points at, and only your compiler knows that version's roles. A name Nive does
not have therefore surfaces as `no variant named ... in IconRole`. Run
`nive icons list --provider lucide` for provider discovery, and see `IconRole`
in the `nive` API docs for the roles your version declares.

## Generated Modules

`nive icons init` and `nive icons sync` maintain:

```text
src/icons.rs
src/icons/
  generated.rs
  generated/
    catalog.rs
    symbols.rs
assets/icons/generated/
```

`src/icons.rs` re-exports:

```rust
pub use generated::{catalog::APP_ICON_CATALOG, symbols::IconSymbol};
```

Install the generated catalog into the app theme:

```rust
use crate::icons::APP_ICON_CATALOG;
use nive::prelude::*;

fn app_theme_catalog() -> ThemeCatalog {
    ThemeCatalog::new(
        Theme::builder("App Light", ThemeMode::Light)
            .icons(APP_ICON_CATALOG)
            .build(),
        Theme::builder("App Dark", ThemeMode::Dark)
            .icons(APP_ICON_CATALOG)
            .build(),
    )
}
```

## Migration

- Replace `IconName::<Variant>` with `IconRole::<Role>` for semantic framework
  intent or generated `IconSymbol::<Variant>` for app-owned concrete icons.
- Replace `Icon::new(IconName::Search)` with `Icon::role(IconRole::EditFind)`
  or `Icon::symbol(IconSymbol::User)`.
- Replace `icons/lucide.toml` with app-root `icons.toml`.
  `nive icons check` reports a migration error when only the legacy manifest
  exists.
