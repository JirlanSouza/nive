# Form Control Composition Migration

Breaking alpha migration notes for the typed form-control foundation.

## Field and FieldGroup

| Before | After |
| --- | --- |
| `Field::new(control).label(label)` | `Field::new(label, control)` |
| `FieldGroup::new(content)` | `FieldGroup::new(visible_legend, fields)` |
| arbitrary canonical Field content | `Input`/`InputGroup`, or explicit `Field::custom(label, element)` |

`FieldControl` is opaque and accepts `Input` and `InputGroup`. Custom controls
remain possible through `Field::custom`, but the caller owns focus, size,
disabled/validation propagation, semantics and clipping. Invisible name-only
groups are deferred until Nive can emit a native accessible group relation;
provide a nonempty visible legend now.

A nonempty Field error is the only canonical Invalid source and replaces its
hint. Empty or whitespace-only errors normalize to no error. Use
`reserve_support_line(true)` when adjacent fields must retain one support-line
track while hint/error content changes.

## Input and InputGroup

An Input without `on_change` is read-only, not disabled. It retains focus,
selection, navigation and copy while mutation is blocked; use
`disabled(true)` to suppress focus and all activation. Store and reuse a
caller-owned `iced::widget::Id` when programmatic focus must survive rebuilds.

Direct public `Input::into_group_element` composition was removed. Construct
`InputGroup::new(input)` and use its typed builders. `leading_text` and
`trailing_text` remain deprecated for one release as aliases of `prefix` and
`unit`; migrate to the semantic spellings. Arbitrary slots are escape hatches
whose masking, paint, semantics and state propagation are caller-owned.

## Button

`secondary` now means `Neutral + Outline`; code that intentionally needs the
old contextual treatment must request `ButtonVariant::Subtle` explicitly.
`tertiary` is the high-level `Neutral + Ghost` action. Icon-only construction
now requires independent semantic text:

```rust
button::icon(IconRole::Close, "Close dialog")
```

Tooltips remain optional disclosure and do not replace the semantic name.
`ButtonIntent` and `ButtonVariant` remain public for advanced combinations.

## Exhaustive contracts and icon catalogs

Downstream exhaustive matches and scale literals must add
`TypographyRole::Control`, `TypographyRole::ControlStrong`, their typography
scale fields, and `IconRole::ValidationError`. Custom `icons.toml` catalogs
must map the canonical `validation-error` role while retaining `identity` and
all other required roles. Regenerate and verify offline:

```sh
nive icons sync
nive icons check
```

Nive retains semantic names, labels, requirements and support metadata, but
does not yet claim native AccessKit name/description/error/group relations or
an independently themed caret color under Iced 0.14.
