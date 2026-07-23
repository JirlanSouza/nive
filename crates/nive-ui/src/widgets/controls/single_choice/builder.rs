use std::borrow::Cow;

use iced::{
    advanced::{mouse, Layout},
    widget, Length, Padding, Rectangle, Size,
};

use crate::theme::{
    self,
    choice::{self, ChoiceMetrics, ChoicePersistentState, ChoiceStateInput, ResolvedChoiceState},
    ControlSize, FieldValidation, TextRole, TypographyRole,
};
use crate::widgets::controls::single_choice::{
    SingleChoice, SingleChoiceKind, SingleChoiceLayout, SingleChoiceState,
};
use crate::widgets::text;
use crate::Element;

impl<'a, Message> SingleChoice<'a, Message> {
    pub(in crate::widgets::controls) fn new(
        kind: SingleChoiceKind,
        layout: SingleChoiceLayout,
        label: Cow<'a, str>,
        persistent: ChoicePersistentState,
    ) -> Self {
        Self {
            kind,
            layout,
            label,
            description: None,
            persistent,
            validation: FieldValidation::Valid,
            size: ControlSize::Sm,
            width: Length::Shrink,
            disabled: false,
            id: None,
            on_activate: None,
            register_focus: true,
            focused_override: false,
        }
    }

    pub(in crate::widgets::controls) fn description(
        mut self,
        description: Option<Cow<'a, str>>,
    ) -> Self {
        self.description = description;
        self
    }

    pub(in crate::widgets::controls) fn validation(mut self, validation: FieldValidation) -> Self {
        self.validation = validation;
        self
    }

    pub(in crate::widgets::controls) fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub(in crate::widgets::controls) fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub(in crate::widgets::controls) fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub(in crate::widgets::controls) fn id(mut self, id: Option<widget::Id>) -> Self {
        self.id = id;
        self
    }

    pub(in crate::widgets::controls) fn on_activate(
        mut self,
        on_activate: Option<Message>,
    ) -> Self {
        self.on_activate = on_activate;
        self
    }

    pub(in crate::widgets::controls) fn register_focus(mut self, register_focus: bool) -> Self {
        self.register_focus = register_focus;
        self
    }

    pub(in crate::widgets::controls) fn focused(mut self, focused: bool) -> Self {
        self.focused_override = focused;
        self
    }

    pub(super) fn metrics(&self, theme: crate::theme::Theme) -> ChoiceMetrics {
        ChoiceMetrics::for_theme(theme, self.size)
    }

    pub(super) fn content(&self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let metrics = self.metrics(theme::active());
        let visual_width = match self.kind {
            SingleChoiceKind::Checkbox | SingleChoiceKind::Radio => metrics.indicator_size,
            SingleChoiceKind::Switch => metrics.switch_track.width,
        };
        let anchor = widget::Space::new()
            .width(Length::Fixed(
                visual_width + metrics.focus_stroke_width * 2.0,
            ))
            .height(Length::Fixed(metrics.form.height));
        let label_role = if self.disabled {
            TextRole::Disabled
        } else {
            TextRole::Primary
        };
        let description_role = if self.disabled {
            TextRole::Disabled
        } else {
            TextRole::Secondary
        };
        let mut copy = widget::Column::new()
            .push(
                text::with_role(self.label.clone(), TypographyRole::Control, label_role)
                    .wrapping(widget::text::Wrapping::WordOrGlyph),
            )
            .spacing(metrics.support_gap);

        if let Some(description) = &self.description {
            copy = copy.push(
                text::with_role(
                    description.clone(),
                    TypographyRole::BodySmall,
                    description_role,
                )
                .wrapping(widget::text::Wrapping::WordOrGlyph),
            );
        }

        let copy_width = if matches!(self.width, Length::Shrink) {
            Length::Shrink
        } else {
            Length::Fill
        };
        let copy = widget::Container::new(copy.width(copy_width))
            .padding(Padding {
                top: metrics.form.content_inset,
                right: 0.0,
                bottom: metrics.form.content_inset,
                left: 0.0,
            })
            .width(copy_width)
            .height(Length::Shrink);
        let row = match self.layout {
            SingleChoiceLayout::Leading => widget::Row::new()
                .push(anchor)
                .push(copy)
                .spacing(metrics.form.gap),
            SingleChoiceLayout::Setting => widget::Row::new()
                .push(copy)
                .push(anchor)
                .spacing(metrics.form.gap),
        };

        row.align_y(iced::Alignment::Start)
            .width(self.width)
            .height(Length::Shrink)
            .into()
    }

    pub(super) fn resolved_state(
        &self,
        state: &SingleChoiceState,
        cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> ResolvedChoiceState {
        choice::resolve_state(ChoiceStateInput {
            persistent: self.persistent,
            validation: self.validation,
            callback_present: self.on_activate.is_some(),
            disabled: self.disabled,
            hovered: cursor.is_over(bounds),
            pressed: state.press.is_some(),
            focused: state.focus.is_focus_visible() || self.focused_override,
        })
    }

    pub(super) fn anchor_bounds(&self, layout: Layout<'_>, metrics: ChoiceMetrics) -> Rectangle {
        let anchor_slot = match self.layout {
            SingleChoiceLayout::Leading => layout.children().next(),
            SingleChoiceLayout::Setting => layout.children().nth(1),
        }
        .map_or(layout.bounds(), |layout| layout.bounds());
        let size = match self.kind {
            SingleChoiceKind::Checkbox | SingleChoiceKind::Radio => {
                Size::new(metrics.indicator_size, metrics.indicator_size)
            }
            SingleChoiceKind::Switch => metrics.switch_track,
        };

        Rectangle {
            x: anchor_slot.x + (anchor_slot.width - size.width) / 2.0,
            y: anchor_slot.y + (metrics.form.height - size.height) / 2.0,
            width: size.width,
            height: size.height,
        }
    }
}
