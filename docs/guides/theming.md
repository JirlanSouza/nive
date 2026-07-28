# Theming a Nive App

A Nive app supplies **six colors**. Every other color a widget paints — panel and
sidebar surfaces, border weights, hover and pressed fills, tone backgrounds — is
derived from those six.

That derivation is the point. It is what keeps text contrast floors, surface
adjacency, and the idle → hover → pressed ladder true for *your* theme and not
only for the framework defaults.

## Building a Theme

```rust
use nive::prelude::*;

fn brand_catalog() -> ThemeCatalog {
    let light = ThemeBuilder::new("Brand Light", ThemeMode::Light)
        .palette(ThemePalette {
            background: Color::from_rgb8(0xF7, 0xF5, 0xF2),
            text: Color::from_rgb8(0x24, 0x1E, 0x1A),
            primary: Color::from_rgb8(0xD9, 0x6B, 0x21),
            success: Color::from_rgb8(0x15, 0x80, 0x3D),
            warning: Color::from_rgb8(0xC5, 0x66, 0x05),
            danger: Color::from_rgb8(0xDC, 0x26, 0x26),
        })
        .build();

    let dark = ThemeBuilder::new("Brand Dark", ThemeMode::Dark)
        .accent(Color::from_rgb8(0xD9, 0x6B, 0x21))
        .build();

    ThemeCatalog::new(light, dark)
}
```

Install it on the app config:

```rust
fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
    ApplicationConfig::new("my-app").theme_catalog(brand_catalog())
}
```

A `ThemeCatalog` is always a light/dark pair. Which one is active follows the
user's `ThemePreference` (`System`, `Light`, or `Dark`), so both sides have to be
worth looking at.

## Starting From a Framework Theme

When the brand is one accent away from the defaults, change only that:

```rust
let dark = ThemeBuilder::from_theme(Theme::Dark)
    .name("Brand Dark")
    .accent(BRAND)
    .build();
```

`from_theme` copies the palette, scales, density, and icons from an existing
theme, so anything you do not touch keeps the framework's tuning.

## The Palette

`.palette(ThemePalette { .. })` sets all six at once. Individual setters exist
for changing one:

| setter | palette field |
|---|---|
| `.app_background(color)` | `background` |
| `.text(color)` | `text` |
| `.accent(color)` | `primary` — resolves as `ToneRole::Accent` |
| `.success(color)` | `success` |
| `.warning(color)` | `warning` |
| `.danger(color)` | `danger` |

## Beyond Color

The builder also takes the four metric scales, the density, and an icon
catalogue:

```rust
let theme = ThemeBuilder::new("Compact", ThemeMode::Light)
    .density(ThemeDensity::Compact)
    .icons(crate::icons::APP_ICON_CATALOG)
    .build();
```

`ThemeDensity` moves control heights and spacing together — `Compact`,
`Standard`, `Comfortable`. Prefer it over hand-editing the scales; it is what
keeps a dense screen internally consistent.

The scales themselves are available when density is not enough:

```rust
use nive::prelude::*;

let shapes = theme::shape::scale();
let typography = theme::typography::scale();
let spacing = theme::spacing::scale();

let theme = ThemeBuilder::new("Custom", ThemeMode::Light)
    .typography(typography)
    .shapes(shapes)
    .spacing(spacing)
    .controls(theme::component::scale(shapes, typography, spacing))
    .build();
```

For icons, see [Adding Icons](adding-icons.md) — the catalogue you install here
is the one `nive icons sync` generates.

## What You Cannot Set

There is no API for pinning a semantic color: `surface.panel`, `border.strong`,
`control.hover`, and the rest are computed from the palette, and the resolved
`ColorScheme` is not constructible from outside the framework.

This is deliberate. Nive asserts things about those colors — that muted text
clears a contrast floor on every surface it can land on, that adjacent surfaces
stay distinguishable, that pressing a control reads as *more* emphasis than
hovering it. An app that pinned `control.hover` directly would make those
guarantees false for itself, with nothing to catch it.

If a specific tone will not come out of the palette, move the palette.

## Switching at Runtime

The active theme follows `ThemePreference`, which the app returns from
`Application::theme` and can change with `Effect::theme(preference)`:

```rust
fn update(&mut self, ..) -> impl Into<Effect<Self::Message, Self::Window>> {
    self.preference = ThemePreference::Dark;
    Effect::theme(self.preference)
}
```

`ThemePreference::System` follows the OS.

Note that themes themselves are built once, not per frame: `Theme::custom` leaks
its data for the process lifetime, which suits a handful of build-time themes and
not a color picker that mints a new theme per keystroke.

## Example

[`examples/theming`](../../examples/theming) builds a branded pair through both
doors — a full palette on one side, `from_theme` on the other — and switches
between them at runtime.
