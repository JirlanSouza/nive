use std::borrow::Cow;

use iced::{widget::column, Length};

use super::layout::FieldGrid;
use super::style::{normalized_error, sanitize_minimum};
use super::{Field, FieldError, FieldGroup, FieldGroupLayout, FieldHint, FieldLabel};
use crate::theme::{self, ControlSize, SpaceStep};
use crate::Element;

impl<'a, Message> FieldGroup<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(
        legend: impl Into<Cow<'a, str>>,
        fields: impl IntoIterator<Item = Field<'a, Message>>,
    ) -> Self {
        let legend = legend.into();
        assert!(
            !legend.trim().is_empty(),
            "FieldGroup requires a nonempty visible legend"
        );

        Self {
            legend,
            fields: fields.into_iter().collect(),
            description: None,
            error: None,
            layout: FieldGroupLayout::Vertical,
            size: ControlSize::Sm,
            disabled: false,
            width: Length::Fill,
        }
    }

    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn error(mut self, error: impl Into<Cow<'a, str>>) -> Self {
        self.error = Some(error.into());
        self
    }

    pub fn layout(mut self, layout: FieldGroupLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn vertical(self) -> Self {
        self.layout(FieldGroupLayout::Vertical)
    }

    pub fn wrap(self, min_field_width: f32) -> Self {
        self.layout(FieldGroupLayout::Wrap { min_field_width })
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

    crate::impl_layout_builders!(width_direct, fill_width_direct, shrink_width_direct);

    pub(super) fn into_element(self) -> Element<'a, Message> {
        let field_gap = match self.size {
            ControlSize::Xs | ControlSize::Sm => theme::space(SpaceStep::Lg),
            ControlSize::Md | ControlSize::Lg => theme::space(SpaceStep::Xl),
        };
        let fields = self
            .fields
            .into_iter()
            .map(|field| field.apply_group_context(self.size, self.disabled).into())
            .collect();
        let minimum = match self.layout {
            FieldGroupLayout::Vertical => None,
            FieldGroupLayout::Wrap { min_field_width } => Some(sanitize_minimum(min_field_width)),
        };
        let fields: Element<'a, Message> = FieldGrid::new(fields, field_gap, minimum).into();
        let mut heading = column![FieldLabel::new(self.legend)]
            .spacing(theme::space(SpaceStep::Xs))
            .width(Length::Fill);
        if let Some(description) = self.description {
            heading = heading.push(FieldHint::new(description));
        }
        if let Some(error) = normalized_error(self.error) {
            heading = heading.push(FieldError::new(error));
        }

        column![heading, fields]
            .spacing(theme::space(SpaceStep::Lg))
            .width(self.width)
            .into()
    }
}
