mod control;
mod group;
mod label_focus;
mod layout;
mod parts;
mod style;

#[cfg(test)]
mod field_tests;

use std::borrow::Cow;

use iced::{widget::Id, Length};

use crate::theme::{ControlSize, FieldValidation};
use crate::widgets::controls::{Autocomplete, Input, InputGroup, Select};
use crate::Element;

pub(super) use self::style::normalized_error;

#[derive(Debug, Clone, Copy, PartialEq)]
struct FieldMetrics {
    label_to_control_gap: f32,
    control_to_support_gap: f32,
    requirement_gap: f32,
    support_line_height: f32,
}

/// A visible label, typed form control, and shared hint/error support slot.
///
/// Labels and other semantic metadata are retained for a future accessibility
/// bridge. Nive does not currently claim native AccessKit label, description,
/// error, or group relationship emission.
pub struct Field<'a, Message> {
    label: Cow<'a, str>,
    control: FieldControl<'a, Message>,
    hint: Option<Cow<'a, str>>,
    error: Option<Cow<'a, str>>,
    requirement: Option<FieldRequirement<'a>>,
    reserve_support_line: bool,
    size: ControlSize,
    disabled: bool,
    width: Length,
    #[cfg(test)]
    probe_name: Option<&'static str>,
}

/// Opaque typed boundary for controls supported by [`Field`].
///
/// Its variants are private so future controls can be added without making
/// downstream code exhaustively match framework internals.
pub struct FieldControl<'a, Message> {
    kind: FieldControlKind<'a, Message>,
}

enum FieldControlKind<'a, Message> {
    Input(Input<'a, Message>),
    InputGroup(Box<InputGroup<'a, Message>>),
    Deferred(FieldControlFactory<'a, Message>),
    Custom(Element<'a, Message>),
}

type FieldControlFactory<'a, Message> = Box<
    dyn FnOnce(
            Cow<'a, str>,
            ControlSize,
            FieldValidation,
            bool,
        ) -> (Element<'a, Message>, Option<Id>)
        + 'a,
>;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
/// Visible, app-localized requirement metadata rendered beside a field label.
pub enum FieldRequirement<'a> {
    /// The app-provided text communicates that the value is required.
    Required(Cow<'a, str>),
    /// The app-provided text communicates that the value is optional.
    Optional(Cow<'a, str>),
}

impl<'a, Message> From<Input<'a, Message>> for FieldControl<'a, Message> {
    fn from(input: Input<'a, Message>) -> Self {
        Self {
            kind: FieldControlKind::Input(input),
        }
    }
}

impl<'a, Message> From<InputGroup<'a, Message>> for FieldControl<'a, Message> {
    fn from(group: InputGroup<'a, Message>) -> Self {
        Self {
            kind: FieldControlKind::InputGroup(Box::new(group)),
        }
    }
}

impl<'a, T, Message> From<Select<'a, T, Message>> for FieldControl<'a, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    fn from(select: Select<'a, T, Message>) -> Self {
        Self {
            kind: FieldControlKind::Deferred(Box::new(move |label, size, validation, disabled| {
                let (select, id) = select.apply_field_context(label, size, validation, disabled);
                (select.into(), Some(id))
            })),
        }
    }
}

impl<'a, T, Message> From<Autocomplete<'a, T, Message>> for FieldControl<'a, Message>
where
    T: Clone + Eq + 'a + 'static,
    Message: Clone + 'a,
{
    fn from(autocomplete: Autocomplete<'a, T, Message>) -> Self {
        Self {
            kind: FieldControlKind::Deferred(Box::new(move |label, size, validation, disabled| {
                let (autocomplete, id) =
                    autocomplete.apply_field_context(label, size, validation, disabled);
                (autocomplete.into(), Some(id))
            })),
        }
    }
}

impl<'a, Message> From<Field<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(field: Field<'a, Message>) -> Self {
        field.into_element()
    }
}

/// A surface-neutral labelled collection of typed [`Field`] values.
///
/// Construction requires a nonempty visible legend. The group propagates its
/// size and disabled state monotonically, while local Field errors remain next
/// to their controls. Description and group error are concise heading rows;
/// the group never paints a Card-like surface or aggregates descendant errors.
/// Native AccessKit group relationships are not emitted yet.
pub struct FieldGroup<'a, Message> {
    legend: Cow<'a, str>,
    fields: Vec<Field<'a, Message>>,
    description: Option<Cow<'a, str>>,
    error: Option<Cow<'a, str>>,
    layout: FieldGroupLayout,
    size: ControlSize,
    disabled: bool,
    width: Length,
}

#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
/// Responsive layout modes for a typed [`FieldGroup`].
pub enum FieldGroupLayout {
    /// Places each field on its own row.
    #[default]
    Vertical,
    /// Uses equal finite tracks while preserving the requested minimum width.
    ///
    /// Invalid minima normalize to 240 logical pixels. An unbounded host falls
    /// back to vertical layout.
    Wrap { min_field_width: f32 },
}

impl<'a, Message> From<FieldGroup<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(group: FieldGroup<'a, Message>) -> Self {
        group.into_element()
    }
}

pub struct FieldLabel<'a> {
    label: Cow<'a, str>,
}

pub struct FieldHint<'a> {
    hint: Cow<'a, str>,
}

pub struct FieldError<'a> {
    error: Cow<'a, str>,
}
