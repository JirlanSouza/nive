# nive-ui

## Role

`nive-ui` owns the shared visual design system for desktop applications built on this framework: design tokens, semantic theme contracts, and reusable primitive widgets that are independent of product-specific domain logic.

Current scope:

- `tokens::color` — color constants, hex parsing/formatting, and color helpers (`hex`, `parse_hex_color`, `format_hex_color`, `format_rgb_hex_color`)
- `tokens::spacing` — spacing scale constants (`SPACE_0` through `SPACE_12`)
- `tokens::radius` — border-radius scale constants (`XS` through `XXXXL`)
- `tokens::shadow` — shadow presets (`NONE`, `POPOVER`) using `iced::Shadow`
- `tokens::typography` — font family helpers (`FontFamily`), font constants (`UI`, `MONO`), text size scale, and line-height constants
- `theme` — semantic role enums, framework-owned theme names (`Nive Light`/`Nive Dark`), `ThemeBuilder`, `ThemeCatalog`, theme data, active theme accessors, helper style functions, and iced `Catalog` implementations for custom widget styling
- `Renderer` and `Element` aliases for the shared Iced renderer/theme pair
- `widgets::text` — typography-aware text constructors and text color style helpers
- `widgets::Badge` — tone-aware label badge primitive
- `widgets::Button` and `ButtonVariant` — themed button with primary, secondary, outline, ghost, destructive, link, and embedded chrome
- `widgets::Card` and `widgets::Panel` — generic themed surface containers
- `widgets::Checkbox` — themed checkbox primitive
- `widgets::Switch` — themed toggle switch primitive
- `widgets::SegmentedControl` and `SegmentedItem` — segmented control primitive
- `widgets::Field` — field grouping, label, hint, and error text primitives
- `widgets::Separator` — reusable horizontal/vertical rule primitive
- `widgets::ColorSwatch` — reusable themed color preview and selectable swatch primitive
- `widgets::ColorPicker` and `widgets::ColorInput` — reusable RGB/alpha color picker controls, popover trigger, validation, and hex parsing helpers
- `widgets::Autocomplete` — reusable input-anchored autocomplete overlay and keyboard navigation behavior
- `widgets::CommandPalette` — reusable action-driven search palette (`CommandPaletteRow`, `command_palette_filter`, `command_palette_view`) for desktop-style command surfaces
- `widgets::Popover` — reusable anchored overlay placement, collision, and dismissal behavior
- `widgets::DropdownMenu` — themed dropdown menu primitive
- `widgets::Dialog` and `DialogHeader`/`DialogFooter`/`DialogActionFooter` — reusable dialog surface primitives
- Native menu bar, system tray, and global shortcut support are intentionally deferred. Iced 0.14 does not provide a native menu API; only `show_system_menu` (window context menu) exists. In-app menus use `DropdownMenu` and consume the same action catalog as shortcuts, toolbars, and the command palette. Revisit when Iced ships an upstream menu bar, tray, or global shortcut API.
- `widgets::ActionCard` — themed card with action area primitive
- `widgets::Toolbar`, `ToolbarGroup`, and `ToolbarAction` — themed toolbar primitives
- `widgets::Tabs`, `TabBar`, and `TabItem` — themed tab navigation primitives
- `widgets::SectionHeader` — reusable section header with optional action and status
- `widgets::EmptyState` — reusable empty state primitive
- `widgets::Select` — themed select dropdown primitive
- `widgets::SelectableCard` and `SelectableItem` — themed selectable card primitives
- `widgets::skeleton` — loading placeholder block, rounded, text row, control, and card primitives
- `widgets::tooltip` — themed tooltip placement/style helpers
- `widgets::animation` — animation frame, timeline, stagger, and runner primitives
- `widgets::metadata` — `DataRow`, `KeyValueList`, and `MetadataItem` for structured metadata display
- `focus_trap` — reusable Tab/Shift+Tab focus cycling helpers for overlay and modal widgets
- `BootstrapView` — generic startup loading/failure template with brand assets, animated status, retry/details actions and error-details content

Remaining widget primitives still in `app-gui.widgets.primitives` (product-aware or not yet generalized):

Internal layout:

- `theme::catalog` keeps public style class types separate from Iced catalog integration modules.
- `theme::color_scheme` keeps the public module path stable while hiding scheme construction details in submodules.
- `dialog_host` keeps host composition separate from overlay event, layout, and backdrop handling.
- `widgets::autocomplete` keeps keyboard navigation and state helpers separate from widget composition.
- `widgets::feedback` owns reusable feedback/status components, including `OperationActionGroup`.

## Public API Contract

Application and screen code should consume `nive-ui` through the crate root,
`nive_ui::prelude`, `nive_ui::theme` and `nive_ui::widgets`. This facade owns:

- the shared `Element` and `Renderer` aliases
- common Iced layout primitives reexported by the prelude
- semantic theme roles, `ThemeBuilder`, `ThemeCatalog` and active theme helpers
- reusable primitive widget contracts and hosts
- token constants and pure token helpers

Lower-level `theme::*` and `widgets::*` submodules may remain public for
advanced composition, styling helpers and focused tests. Product code should
prefer the facades and avoid depending on private host internals.

## Accessibility Contract

Every new widget must meet the accessibility checklist documented in
`crates/nive-ui/docs/components.md` under the "Accessibility Contract"
section. The minimum bar is: icon-only interactive widgets accept a label
or tooltip, disabled and loading states are explicit, and overlay-like
widgets honor Escape and Tab through `is_escape_key_press` and
`focus_trap::direction_from_event`. Full platform accessibility is
deferred until Iced adds upstream support.

## Boundaries

- Keep product-specific composite widgets (tag input, error feedback, project stat, etc.) in `app-gui.widgets.composite`; `nive-ui` owns only reusable primitives.
- Keep embedded product branding and brand assets out of `nive-ui`; bootstrap
  templates accept app-provided assets and copy without depending on product
  crates.
- Keep visual toast composition in `nive-ui` (`ToastHost` and the `ToastPresentation` contract); shared toast state/identity, expiration, pause/resume and tick handling remain in `nive-runtime`, which implements `ToastPresentation` for `ToastItem` and applies the host automatically.
- Tokens must remain pure constants and pure functions with no side effects and no dependency on `app-core`, `app-models`, or any other domain crate.
- Token modules may depend on `iced` for color, shadow, and font types only.
- Theme modules may depend on `iced` for theme, widget catalog, and style types, but must stay domain-agnostic.
- Custom themes should be built through `ThemeBuilder` and passed to runtime as
  a `ThemeCatalog`; do not hardcode product brand colors or product theme names
  in `nive-ui`.
- Widget modules may depend on `nive-ui` theme/tokens and `iced`, but not on app screens, app shell, app-core, or app-models.
- `nive-ui` should not depend on `nive-runtime` or `app-gui`; it is a lower layer.

## Workflow

- `cargo check -p nive-ui`
- `cargo test -p nive-ui`
