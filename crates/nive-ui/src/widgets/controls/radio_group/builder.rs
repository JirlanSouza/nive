use std::borrow::Cow;

use iced::{widget, Length};

use super::{RadioGroup, RadioGroupLayout, RadioGroupWidget, RadioOption};
use crate::theme::{choice::ChoiceMetrics, ControlSize, FieldValidation};
use crate::widgets::controls::field::{normalized_error, FieldError};
use crate::widgets::controls::{FieldHint, FieldLabel, FieldRequirement};
use crate::Element;

impl<'a, T> RadioOption<'a, T> {
    pub fn new(value: T, label: impl Into<Cow<'a, str>>) -> Self {
        let label = label.into();
        debug_assert!(
            !label.trim().is_empty(),
            "RadioOption requires a nonempty visible label"
        );

        Self {
            value,
            label,
            description: None,
            disabled: false,
        }
    }

    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn description_maybe<D>(mut self, description: Option<D>) -> Self
    where
        D: Into<Cow<'a, str>>,
    {
        self.description = description.map(Into::into);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<'a, T, Message> RadioGroup<'a, T, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    pub fn new(
        legend: impl Into<Cow<'a, str>>,
        selected: Option<T>,
        options: impl IntoIterator<Item = RadioOption<'a, T>>,
    ) -> Self {
        let legend = legend.into();
        debug_assert!(
            !legend.trim().is_empty(),
            "RadioGroup requires a nonempty visible legend"
        );

        Self {
            legend,
            selected,
            options: options.into_iter().collect(),
            requirement: None,
            description: None,
            error: None,
            layout: RadioGroupLayout::Vertical,
            size: ControlSize::Sm,
            width: Length::Fill,
            disabled: false,
            id: None,
            on_select: None,
        }
    }

    pub fn requirement(mut self, requirement: FieldRequirement<'a>) -> Self {
        self.requirement = Some(requirement);
        self
    }

    pub fn required(self, text: impl Into<Cow<'a, str>>) -> Self {
        self.requirement(FieldRequirement::Required(text.into()))
    }

    pub fn optional(self, text: impl Into<Cow<'a, str>>) -> Self {
        self.requirement(FieldRequirement::Optional(text.into()))
    }

    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn description_maybe<D>(mut self, description: Option<D>) -> Self
    where
        D: Into<Cow<'a, str>>,
    {
        self.description = description.map(Into::into);
        self
    }

    pub fn error(mut self, error: impl Into<Cow<'a, str>>) -> Self {
        self.error = Some(error.into());
        self
    }

    pub fn error_maybe<E>(mut self, error: Option<E>) -> Self
    where
        E: Into<Cow<'a, str>>,
    {
        self.error = error.map(Into::into);
        self
    }

    pub fn layout(mut self, layout: RadioGroupLayout) -> Self {
        self.layout = layout;
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

    crate::impl_layout_builders!(width_direct, fill_width_direct, shrink_width_direct);

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn id(mut self, id: widget::Id) -> Self {
        self.id = Some(id);
        self
    }

    pub fn on_select(mut self, on_select: impl Fn(T) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(on_select));
        self
    }

    pub fn on_select_maybe(mut self, on_select: Option<impl Fn(T) -> Message + 'a>) -> Self {
        self.on_select = on_select.map(|on_select| Box::new(on_select) as _);
        self
    }

    pub(super) fn into_element(self) -> Element<'a, Message> {
        let error = normalized_error(self.error);
        let metrics = ChoiceMetrics::for_theme(crate::theme::active(), self.size);
        let mut legend = widget::Row::new()
            .push(FieldLabel::new(self.legend))
            .spacing(metrics.support_gap)
            .align_y(iced::Alignment::Center)
            .width(Length::Fill);

        if let Some(requirement) = self.requirement {
            let text = match requirement {
                FieldRequirement::Required(text) | FieldRequirement::Optional(text) => text,
            };
            legend = legend.push(FieldHint::new(text));
        }

        let mut heading = widget::Column::new()
            .push(legend.wrap())
            .spacing(metrics.support_gap)
            .width(Length::Fill);

        if let Some(description) = self.description {
            heading = heading.push(FieldHint::new(description));
        }
        if let Some(error) = &error {
            heading = heading.push(FieldError::new(error.clone()).into_element());
        }

        let choices: Element<'a, Message> = RadioGroupWidget {
            selected: self.selected,
            options: self.options,
            layout: self.layout,
            size: self.size,
            width: self.width,
            validation: if error.is_some() {
                FieldValidation::Invalid
            } else {
                FieldValidation::Valid
            },
            disabled: self.disabled,
            id: self.id,
            on_select: self.on_select,
        }
        .into();

        widget::Column::new()
            .push(heading)
            .push(choices)
            .spacing(metrics.group_gap)
            .width(self.width)
            .into()
    }
}
