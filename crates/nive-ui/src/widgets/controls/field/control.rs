use std::borrow::Cow;

use iced::{
    widget::{column, container, Id, Space},
    Length,
};

use super::label_focus::LabelFocus;
use super::style::{field_label, metrics, normalized_error, style};
use super::{Field, FieldControl, FieldControlKind, FieldError, FieldHint, FieldRequirement};
use crate::theme::{ControlSize, FieldValidation};
use crate::Element;

impl<'a, Message> Field<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(
        label: impl Into<Cow<'a, str>>,
        control: impl Into<FieldControl<'a, Message>>,
    ) -> Self {
        Self {
            label: label.into(),
            control: control.into(),
            hint: None,
            error: None,
            requirement: None,
            reserve_support_line: false,
            size: ControlSize::Sm,
            disabled: false,
            width: Length::Fill,
            #[cfg(test)]
            probe_name: None,
        }
    }

    /// Creates a labelled field around an unsupported arbitrary widget.
    ///
    /// The caller owns focus targeting, size, validation, disabled state, and
    /// semantic association for this deliberately limited escape hatch.
    pub fn custom(
        label: impl Into<Cow<'a, str>>,
        control: impl Into<Element<'a, Message>>,
    ) -> Self {
        Self::new(
            label,
            FieldControl {
                kind: FieldControlKind::Custom(control.into()),
            },
        )
    }

    pub fn hint(mut self, hint: impl Into<Cow<'a, str>>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn error(mut self, error: impl Into<Cow<'a, str>>) -> Self {
        self.error = Some(error.into());
        self
    }

    pub fn requirement(mut self, requirement: FieldRequirement<'a>) -> Self {
        self.requirement = Some(requirement);
        self
    }

    pub fn required(self, label: impl Into<Cow<'a, str>>) -> Self {
        self.requirement(FieldRequirement::Required(label.into()))
    }

    pub fn optional(self, label: impl Into<Cow<'a, str>>) -> Self {
        self.requirement(FieldRequirement::Optional(label.into()))
    }

    pub fn reserve_support_line(mut self, reserve: bool) -> Self {
        self.reserve_support_line = reserve;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn xs(self) -> Self {
        self.size(ControlSize::Xs)
    }

    pub fn sm(self) -> Self {
        self.size(ControlSize::Sm)
    }

    pub fn md(self) -> Self {
        self.size(ControlSize::Md)
    }

    pub fn lg(self) -> Self {
        self.size(ControlSize::Lg)
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub(super) fn apply_group_context(mut self, size: ControlSize, disabled: bool) -> Self {
        self.size = size;
        self.disabled |= disabled;
        self
    }

    #[cfg(test)]
    pub(super) fn probe_name(mut self, name: &'static str) -> Self {
        self.probe_name = Some(name);
        self
    }

    crate::impl_layout_builders!(width_direct, fill_width_direct, shrink_width_direct);

    pub(super) fn into_element(self) -> Element<'a, Message> {
        let metrics = metrics();
        let error = normalized_error(self.error);
        let validation = if error.is_some() {
            FieldValidation::Invalid
        } else {
            FieldValidation::Valid
        };
        let (control, focus_target) =
            self.control
                .into_element(self.label.clone(), self.size, validation, self.disabled);
        let label = field_label(self.label, self.requirement, metrics.requirement_gap);
        let support = match error {
            Some(error) => Some(FieldError::new(error).into_element()),
            None => self.hint.map(|hint| FieldHint::new(hint).into_element()),
        };
        let support = if self.reserve_support_line {
            Some(support.unwrap_or_else(|| {
                Space::new()
                    .height(Length::Fixed(metrics.support_line_height))
                    .into()
            }))
        } else {
            support
        };
        let mut control_and_support = column![control].width(Length::Fill);
        if let Some(support) = support {
            control_and_support = control_and_support
                .push(support)
                .spacing(metrics.control_to_support_gap);
        }
        let content = column![label, control_and_support]
            .spacing(metrics.label_to_control_gap)
            .width(Length::Fill);

        let content: Element<'a, Message> =
            container(content).style(style).width(self.width).into();

        let field: Element<'a, Message> =
            LabelFocus::new(content, focus_target, self.disabled).into();

        #[cfg(test)]
        if let Some(name) = self.probe_name {
            return crate::test_support::named_probe(name, field);
        }

        field
    }
}

impl<'a, Message> FieldControl<'a, Message>
where
    Message: Clone + 'a,
{
    pub(super) fn into_element(
        self,
        label: Cow<'a, str>,
        size: ControlSize,
        validation: FieldValidation,
        disabled: bool,
    ) -> (Element<'a, Message>, Option<Id>) {
        match self.kind {
            FieldControlKind::Input(input) => {
                let (input, id) = input.apply_field_context(label, size, validation, disabled);
                (input.into(), Some(id))
            }
            FieldControlKind::InputGroup(group) => {
                let (group, id) = group.apply_field_context(label, size, validation, disabled);
                (group.into(), Some(id))
            }
            FieldControlKind::Deferred(factory) => factory(label, size, validation, disabled),
            FieldControlKind::Custom(control) => (control, None),
        }
    }
}
