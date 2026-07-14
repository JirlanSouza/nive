# Visual Foundations Migration

Breaking alpha migration notes for the typography, surface, control-state,
font-delivery, and icon foundation changes.

## Fonts and typography

`nive-runtime` now registers the bundled Inter Regular/SemiBold and Geist Mono
Regular/Medium faces automatically and defaults the application font to Inter.
Applications can keep using the existing font override. Direct Iced consumers
register each byte slice returned by `nive_ui::fonts::bundled()` and can use
`nive_ui::fonts::default_font()` for the same default. Build `nive-ui` without
default features to opt out of the `bundled-fonts` assets.

`TypographyRole::SectionLabel` is now 12 px semibold with 1.25 line height.
`SectionHeader` owns this compact panel/content-section level and uses primary
text. Compose a principal document title separately with
`TypographyRole::Heading` (16 px semibold, 1.25 line height). Metadata uses
`Caption`; only dense code uses the 10 px `CodeSmall` tier.

## Surfaces and control state

`surface::style()` paints fill and shadow only. Add an explicit border where a
card or panel needs one; structural hairlines belong to the region that caps a
seam. `SurfaceRole::Elevated` is reserved for genuinely floating content:
selection uses the selectable/accent control state and avatars use a neutral
tone fill.

Widgets now pass the complete `ControlState` to the theme. Disabled precedence,
selected interaction feedback, and focus-visible borders are resolved once,
while each widget category still chooses how to project those semantic values.
Remove consumer-local disabled/selected alpha ladders.

## Icons

| Before | After |
| --- | --- |
| `Icon::size(16.0)` | `Icon::size(IconSize::Md)` or `Icon::md()` |
| `Icon::size(custom_pixels)` | `Icon::custom_size(custom_pixels)` |
| `Icon::rotation(radians)` for a static transform | `Icon::rotation(Rotation::Quarter)` (or another quarter turn) |
| continuous animated `Icon::rotation(radians)` | `Icon::animated_rotation(radians)` |

`IconSize` maps `Xs/Sm/Md/Lg/Xl` to 12/14/16/20/24 px and defaults to `Md`.
Controls use their resolved `control.icon_size` via `custom_size`. `Icon` is a
monochrome decorative primitive: it inherits the host foreground and the host
owns the accessible name. Custom `IconSource` SVGs are vetted by `nive icons`
for the 24×24, stroke-2, rounded, monochrome `currentColor` contract.
