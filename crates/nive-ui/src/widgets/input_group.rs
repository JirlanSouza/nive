mod frame;
mod style;

use iced::{
    widget::{container, text, Row},
    Alignment, Length, Padding,
};

use crate::theme::ControlSize;
use crate::Element;

use self::frame::InputGroupFrame;
use self::style as theme_input_group;
use super::{
    button::{Button, GroupedItemKind, GroupedItemSpec},
    control_group::{position_for_index, radius_for_position, SlotPosition},
    icon as icon_widget,
    input::Input,
    IconName,
};

pub use style::InputGroupVariant;

pub struct InputGroup<'a, Message> {
    input: Input<'a, Message>,
    leading_slots: Vec<InputGroupSlot<'a, Message>>,
    trailing_slots: Vec<InputGroupSlot<'a, Message>>,
    size: ControlSize,
    fill: bool,
    variant: InputGroupVariant,
}

enum InputGroupSlot<'a, Message> {
    Text(&'a str),
    Icon(IconName),
    Action(Button<'a, Message>),
    Visual(Element<'a, Message>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputGroupSlotSide {
    Leading,
    Trailing,
}

impl<'a, Message> InputGroup<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(input: Input<'a, Message>) -> Self {
        Self {
            input,
            leading_slots: Vec::new(),
            trailing_slots: Vec::new(),
            size: ControlSize::Sm,
            fill: true,
            variant: InputGroupVariant::Default,
        }
    }

    pub fn leading_text(self, leading: &'a str) -> Self {
        self.push_slot(InputGroupSlotSide::Leading, InputGroupSlot::Text(leading))
    }

    pub fn leading_icon(self, leading: IconName) -> Self {
        self.push_slot(InputGroupSlotSide::Leading, InputGroupSlot::Icon(leading))
    }

    pub fn leading_action(self, leading: Button<'a, Message>) -> Self {
        self.push_slot(InputGroupSlotSide::Leading, InputGroupSlot::Action(leading))
    }

    pub fn leading_slot(self, leading: impl Into<Element<'a, Message>>) -> Self {
        self.push_slot(
            InputGroupSlotSide::Leading,
            InputGroupSlot::Visual(leading.into()),
        )
    }

    pub fn trailing_text(self, trailing: &'a str) -> Self {
        self.push_slot(InputGroupSlotSide::Trailing, InputGroupSlot::Text(trailing))
    }

    pub fn trailing_icon(self, trailing: IconName) -> Self {
        self.push_slot(InputGroupSlotSide::Trailing, InputGroupSlot::Icon(trailing))
    }

    pub fn trailing_action(self, trailing: Button<'a, Message>) -> Self {
        self.push_slot(
            InputGroupSlotSide::Trailing,
            InputGroupSlot::Action(trailing),
        )
    }

    pub fn trailing_slot(self, trailing: impl Into<Element<'a, Message>>) -> Self {
        self.push_slot(
            InputGroupSlotSide::Trailing,
            InputGroupSlot::Visual(trailing.into()),
        )
    }

    pub fn xs(mut self) -> Self {
        self.size = ControlSize::Xs;
        self
    }

    pub fn sm(mut self) -> Self {
        self.size = ControlSize::Sm;
        self
    }

    pub fn md(mut self) -> Self {
        self.size = ControlSize::Md;
        self
    }

    pub fn lg(mut self) -> Self {
        self.size = ControlSize::Lg;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn fill(mut self) -> Self {
        self.fill = true;
        self
    }

    pub fn shrink(mut self) -> Self {
        self.fill = false;
        self
    }

    pub fn variant(mut self, variant: InputGroupVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn ghost(self) -> Self {
        self.variant(InputGroupVariant::Ghost)
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = theme_input_group::metrics(self.size);
        let input_disabled = self.input.is_disabled();
        let input_validation = self.input.field_validation();
        let slot_count = self.leading_slots.len() + 1 + self.trailing_slots.len();
        let mut row = Row::new()
            .spacing(0)
            .align_y(Alignment::Center)
            .height(Length::Fill);

        let mut index = 0;

        for slot in self.leading_slots {
            row = row.push(slot.into_element(
                self.size,
                self.fill,
                input_disabled,
                metrics,
                position_for_index(index, slot_count),
            ));
            index += 1;
        }

        let input_radius =
            radius_for_position(position_for_index(index, slot_count), metrics.radius);
        row = row.push(self.input.into_group_element(
            self.fill,
            input_radius,
            metrics.font_size,
            metrics.input_padding_v,
            metrics.input_padding_h,
        ));
        index += 1;

        for slot in self.trailing_slots {
            row = row.push(slot.into_element(
                self.size,
                self.fill,
                input_disabled,
                metrics,
                position_for_index(index, slot_count),
            ));
            index += 1;
        }

        let mut group = container(row).height(Length::Fixed(metrics.height));

        if self.fill {
            group = group.width(Length::Fill);
        }

        Element::new(InputGroupFrame {
            content: group.into(),
            variant: self.variant,
            input_validation,
            radius: metrics.radius,
            disabled: input_disabled,
        })
    }

    fn push_slot(mut self, side: InputGroupSlotSide, slot: InputGroupSlot<'a, Message>) -> Self {
        match side {
            InputGroupSlotSide::Leading => self.leading_slots.push(slot),
            InputGroupSlotSide::Trailing => self.trailing_slots.push(slot),
        }

        self
    }
}

impl<'a, Message> InputGroupSlot<'a, Message>
where
    Message: Clone + 'a,
{
    fn into_element(
        self,
        size: ControlSize,
        _fill: bool,
        disabled: bool,
        metrics: theme_input_group::InputGroupMetrics,
        position: SlotPosition,
    ) -> Element<'a, Message> {
        let radius = radius_for_position(position, metrics.radius);

        match self {
            InputGroupSlot::Text(label) => container(
                text(label)
                    .size(metrics.font_size)
                    .shaping(text::Shaping::Auto),
            )
            .style(theme_input_group::slot_style(radius, disabled))
            .padding(Padding::ZERO.horizontal(metrics.slot_padding_h))
            .height(Length::Fill)
            .align_y(Alignment::Center)
            .into(),
            InputGroupSlot::Icon(icon) => container(icon_widget::new(icon).size(metrics.icon_size))
                .style(theme_input_group::slot_style(radius, disabled))
                .padding(Padding::ZERO.horizontal(metrics.slot_padding_h))
                .height(Length::Fill)
                .align_y(Alignment::Center)
                .into(),
            InputGroupSlot::Action(action) => action.into_grouped_item(GroupedItemSpec {
                size,
                radius,
                height: metrics.height,
                padding_h: metrics.slot_padding_h,
                selected: false,
                kind: GroupedItemKind::Embedded,
            }),
            InputGroupSlot::Visual(content) => container(content)
                .style(theme_input_group::slot_style(radius, disabled))
                .padding(Padding::ZERO.horizontal(metrics.slot_padding_h))
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into(),
        }
    }
}

impl<'a, Message> From<InputGroup<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(group: InputGroup<'a, Message>) -> Self {
        group.into_element()
    }
}
