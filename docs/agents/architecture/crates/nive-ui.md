# nive-ui

## Role

`nive-ui` owns the shared visual design system for desktop applications built on this framework: design tokens, semantic theme contracts, and reusable primitive widgets that are independent of product-specific domain logic.

Current scope:

- `tokens::color` — color constants, hex parsing/formatting, and color helpers (`hex`, `parse_hex_color`, `format_hex_color`, `format_rgb_hex_color`)
- `tokens::spacing` — spacing scale constants (`SPACE_0` through `SPACE_12`)
- `tokens::radius` — border-radius scale constants (`XS` through `XXXXL`)
- `tokens::shadow` — shadow presets (`NONE`, `POPOVER`) using `iced::Shadow`
- `tokens::typography` — font family helpers (`FontFamily`), font constants (`UI`, `MONO`), text size scale, and line-height constants
- `theme` — semantic role enums, framework-owned theme names (`Nive Light`/`Nive Dark`), theme data, active theme accessors, helper style functions, and iced `Catalog` implementations for custom widget styling
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
- `widgets::Popover` — reusable anchored overlay placement, collision, and dismissal behavior
- `widgets::DropdownMenu` — themed dropdown menu primitive
- `widgets::Dialog` and `DialogHeader`/`DialogFooter`/`DialogActionFooter` — reusable dialog surface primitives
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

Remaining widget primitives still in `app-gui.widgets.primitives` (product-aware or not yet generalized):

## Boundaries

- Keep product-specific composite widgets (tag input, error feedback, project stat, etc.) in `app-gui.widgets.composite`; `nive-ui` owns only reusable primitives.
- Keep product branding and brand assets out of `nive-ui`; framework theme labels should stay app-agnostic unless app-configurable naming is introduced.
- Keep visual feedback composition in `app-gui`; shared feedback state/contracts such as toasts and `ScreenUpdate` remain in `nive-runtime`.
- Tokens must remain pure constants and pure functions with no side effects and no dependency on `app-core`, `app-models`, or any other domain crate.
- Token modules may depend on `iced` for color, shadow, and font types only.
- Theme modules may depend on `iced` for theme, widget catalog, and style types, but must stay domain-agnostic.
- Widget modules may depend on `nive-ui` theme/tokens and `iced`, but not on app screens, app shell, app-core, or app-models.
- `nive-ui` should not depend on `nive-runtime` or `app-gui`; it is a lower layer.

## Workflow

- `cargo check -p nive-ui`
- `cargo test -p nive-ui`
