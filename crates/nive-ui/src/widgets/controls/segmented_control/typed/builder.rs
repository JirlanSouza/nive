use std::borrow::Cow;

use iced::Length;

use super::{SegmentedControl, SegmentedControlVariant, SegmentedOption};
use crate::theme::ControlSize;
use crate::IconRef;

impl<'a, T> SegmentedOption<'a, T> {
    pub fn new(value: T, label: impl Into<Cow<'a, str>>) -> Self {
        let label = label.into();
        debug_assert!(
            !label.trim().is_empty(),
            "SegmentedOption requires a nonempty visible label"
        );

        Self {
            value,
            label,
            icon: None,
            disabled: false,
        }
    }

    pub fn icon(mut self, icon: impl Into<IconRef>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<'a, T, Message> SegmentedControl<'a, T, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    pub fn new(
        semantic_name: impl Into<Cow<'a, str>>,
        selected: T,
        options: impl IntoIterator<Item = SegmentedOption<'a, T>>,
    ) -> Self {
        let semantic_name = semantic_name.into();
        debug_assert!(
            !semantic_name.trim().is_empty(),
            "SegmentedControl requires nonempty semantic-name metadata"
        );

        let mut control = Self {
            semantic_name,
            selected,
            options: options.into_iter().collect(),
            size: ControlSize::Sm,
            width: Length::Shrink,
            variant: SegmentedControlVariant::Default,
            disabled: false,
            id: None,
            on_select: None,
            contents: Vec::new(),
        };
        control.contents = (0..control.options.len())
            .map(|index| control.item_content(index, f32::INFINITY))
            .collect();
        control
    }

    pub fn linked(mut self) -> Self {
        self.variant = SegmentedControlVariant::Linked;
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

    pub fn id(mut self, id: iced::widget::Id) -> Self {
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
}
