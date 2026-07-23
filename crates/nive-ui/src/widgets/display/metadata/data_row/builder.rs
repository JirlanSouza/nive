use std::borrow::Cow;

use iced::Length;

use crate::widgets::display::measured_text::{EllipsisStrategy, MeasuredText};
use crate::widgets::display::metadata::data_row::{DataRow, DataRowLayout};
use crate::widgets::display::metadata::style as theme_metadata;
use crate::{
    theme::{ControlSize, TextRole, ToneRole, TypographyRole},
    widgets::ToneDot,
    Element,
};

impl<'a, Message> DataRow<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            value: None,
            tone: None,
            reserve_indicator: false,
            leading: None,
            trailing: None,
            size: ControlSize::Sm,
            width: None,
        }
    }

    /// Adds secondary text clustered beside the principal label.
    pub fn value(mut self, value: impl Into<Cow<'a, str>>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Adds a status dot when the visible label/value cluster states the status in text.
    pub fn tone(mut self, tone: ToneRole) -> Self {
        self.tone = Some(tone);
        self
    }

    /// Reserves the same status slot used by toned sibling rows.
    pub fn reserve_indicator(mut self) -> Self {
        self.reserve_indicator = true;
        self
    }

    pub fn neutral(self) -> Self {
        self.tone(ToneRole::Neutral)
    }
    pub fn accent(self) -> Self {
        self.tone(ToneRole::Accent)
    }
    pub fn info(self) -> Self {
        self.tone(ToneRole::Info)
    }
    pub fn success(self) -> Self {
        self.tone(ToneRole::Success)
    }
    pub fn warning(self) -> Self {
        self.tone(ToneRole::Warning)
    }
    pub fn danger(self) -> Self {
        self.tone(ToneRole::Danger)
    }

    pub fn leading(mut self, leading: impl Into<Element<'a, Message>>) -> Self {
        self.leading = Some(leading.into());
        self
    }

    pub fn trailing(mut self, trailing: impl Into<Element<'a, Message>>) -> Self {
        self.trailing = Some(trailing.into());
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

    crate::impl_layout_builders!(width_opt, fill_width_opt, shrink_width_opt);

    pub(super) fn into_element(self) -> Element<'a, Message> {
        let metrics = theme_metadata::metrics(self.size);
        let cluster_has_text = !self.label.trim().is_empty()
            || self
                .value
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty());
        let has_indicator_slot = self.reserve_indicator || self.tone.is_some();
        let indicator: Element<'a, Message> = self.tone.filter(|_| cluster_has_text).map_or_else(
            || {
                if has_indicator_slot {
                    let diameter = crate::widgets::primitives::tone_dot::dot_size(self.size);
                    iced::widget::Space::new()
                        .width(Length::Fixed(diameter))
                        .height(Length::Fixed(diameter))
                        .into()
                } else {
                    iced::widget::Space::new().into()
                }
            },
            |tone| ToneDot::new(tone).size(self.size).into(),
        );
        let leading_present = self.leading.is_some();
        let trailing_present = self.trailing.is_some();
        let secondary_present = self.value.is_some();
        let leading = self
            .leading
            .unwrap_or_else(|| iced::widget::Space::new().into());
        let trailing = self
            .trailing
            .unwrap_or_else(|| iced::widget::Space::new().into());
        let principal_text = self.label;
        let secondary_text = self.value.unwrap_or_default();
        let principal = MeasuredText::new(
            principal_text.clone(),
            EllipsisStrategy::End,
            TypographyRole::Body,
            TextRole::Primary,
        )
        .into();
        let secondary = MeasuredText::new(
            secondary_text.clone(),
            EllipsisStrategy::End,
            TypographyRole::Body,
            TextRole::Secondary,
        )
        .into();

        DataRowLayout {
            children: [indicator, leading, principal, secondary, trailing],
            principal_text,
            secondary_text,
            has_indicator_slot,
            leading_present,
            secondary_present,
            trailing_present,
            indicator_width: crate::widgets::primitives::tone_dot::dot_size(self.size),
            gap: metrics.slot_gap,
            minimum_height: metrics.minimum_height,
            width: self.width.unwrap_or(Length::Shrink),
        }
        .into()
    }
}
