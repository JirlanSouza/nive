# Widget Public API Semantics Migration

Breaking alpha migration notes for the `nive-ui` public API cleanup.

## Shape

| Before | After |
| --- | --- |
| `ShapeRole::None` | `ShapeSize::None` |
| `ShapeRole::ExtraSmall` | `ShapeSize::Xs` |
| `ShapeRole::Small` | `ShapeSize::Sm` |
| `ShapeRole::Medium` | `ShapeSize::Md` |
| `ShapeRole::Large` | `ShapeSize::Lg` |
| `ShapeRole::ExtraLarge` | `ShapeSize::Xl` |
| `ShapeRole::Full` | `ShapeSize::Full` |
| `Card::new(content).sm()` | `Card::new(content).shape_md()` |

`ShapeSize::Xxl` is new. `ShapeSize::Full` now resolves to
`radius::FULL = 9999.0`, meaning pill/circle semantics, not the largest numeric
radius.

## Tone

| Before | After |
| --- | --- |
| `ToneRole::Primary` | `ToneRole::Accent` |
| `ThemeBuilder::primary(color)` | `ThemeBuilder::accent(color)` |
| status widgets using primary-tone wording | `accent()` or `tone(ToneRole::Accent)` |

Use `danger()` for status tone and `destructive()` for destructive actions.
`Button`, `DropdownMenuItem`, and `ToolbarAction` use `destructive()`;
`Badge` and `InlineAlert` use `danger()`.

## Layout

| Before | After |
| --- | --- |
| `fill()` on `Checkbox` | `fill_width()` |
| `fill()` on `InputGroup` | `fill_width()` |
| `fill()` on `Select` | `fill_width()` |
| `fill()` on `SegmentedControl` | `fill_width()` |
| `fill()` on `DataRow` | `fill_width()` |
| `fill()` on `KeyValueList` | `fill_width()` |
| `fill()` on `DropdownMenu` | `fill_width()` |
| `fill()` on `Toolbar` | `fill_width()` |
| `fill()` on `ToolbarActionGroup` | `fill_width()` |
| `fill()` on `TabBar` | `fill_width()` |
| `shrink()` on `Button` | `shrink_width()` |
| `shrink()` on `Field` | `shrink_width()` |
| `shrink()` on `Input` | `shrink_width()` |
| `shrink()` on `InputGroup` | `shrink_width()` |
| `shrink()` on `Select` | `shrink_width()` |
| `shrink()` on `SelectableItem` | `shrink_width()` |

`Tree` keeps `fill()` and it now explicitly means both axes, equivalent to
`fill_width().fill_height()`.

## Button

| Before | After |
| --- | --- |
| `ButtonVariant::Primary` | `ButtonIntent::Suggested` + `ButtonVariant::Solid` |
| `ButtonVariant::Secondary` | `ButtonIntent::Neutral` + `ButtonVariant::Subtle` |
| `ButtonVariant::Outline` | `ButtonIntent::Neutral` + `ButtonVariant::Outline` |
| `ButtonVariant::Ghost` | `ButtonIntent::Neutral` + `ButtonVariant::Ghost` |
| `ButtonVariant::Destructive` | `ButtonIntent::Destructive` + `ButtonVariant::Solid` |
| `ButtonVariant::Link` | no button equivalent; use `ghost()` as an interim affordance |
| `button::link(label)` | `button::ghost(label)` until a dedicated link control exists |
| `Button::link()` | `Button::ghost()` until a dedicated link control exists |

High-level shortcuts keep working: `primary()`, `secondary()`, `outline()`,
`ghost()`, and `destructive()` map to the pairs above.

## Interaction

| Before | After |
| --- | --- |
| `Input::on_input(f)` | `Input::on_change(f)` |
| `Input::on_input_maybe(f)` | `Input::on_change_maybe(f)` |
| `PathInput::on_input(f)` | `PathInput::on_change(f)` |
| optional browse wiring by omission | `PathInput::on_browse_maybe(option)` |
| `Autocomplete::on_select(fn(usize) -> Message)` | `Autocomplete::suggestions(values).on_select(fn(value) -> Message)` |
| `CommandPaletteRow::enabled(false)` | `CommandPaletteRow::disabled(true)` |
| `CommandPaletteRow::disabled()` | `CommandPaletteRow::disabled(true)` |
| `TabBar::on_activate(fn(Id, ActivationTrigger) -> Message)` | `TabBar::on_select(fn(Id) -> Message)` |
| `TabBar::on_activate_maybe(option)` | `TabBar::on_select_maybe(option)` |

`disabled(true)` always suppresses interaction messages even when callbacks are
present. Passing `None` to a `_maybe` callback removes the callback without
changing the visual disabled state.
