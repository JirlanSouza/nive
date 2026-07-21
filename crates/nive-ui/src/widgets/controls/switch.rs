use std::borrow::Cow;

use iced::{widget, Length};

use crate::theme::{choice::ChoicePersistentState, ControlSize};
use crate::Element;

use super::single_choice::{SingleChoice, SingleChoiceKind, SingleChoiceLayout};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SwitchComposition {
    TrackOnly,
    Inline,
    Setting,
}

/// A controlled immediate binary setting.
///
/// Use [`Switch::inline`] for intrinsic label-and-track composition and
/// [`Switch::setting`] for a fill-width title/description row with a protected
/// trailing track. Missing callbacks are display-only; explicit disabled state
/// is visually distinct. State and thumb position change immediately—async
/// persistence, failure, retry, and future motion preferences are host-owned.
/// Retained semantic metadata does not yet emit a native accessibility node.
pub struct Switch<'a, Message> {
    checked: bool,
    label: Option<Cow<'a, str>>,
    description: Option<Cow<'a, str>>,
    semantic_name: Option<Cow<'a, str>>,
    composition: SwitchComposition,
    size: ControlSize,
    disabled: bool,
    id: Option<widget::Id>,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
}

impl<'a, Message> Switch<'a, Message>
where
    Message: Clone + 'a,
{
    /// Creates an intrinsic inline label-and-track composition.
    pub fn inline(label: impl Into<Cow<'a, str>>, checked: bool) -> Self {
        Self::canonical(label.into(), checked, SwitchComposition::Inline)
    }

    /// Creates a fill-width setting row with a protected trailing track.
    pub fn setting(title: impl Into<Cow<'a, str>>, checked: bool) -> Self {
        Self::canonical(title.into(), checked, SwitchComposition::Setting)
    }

    /// Creates an advanced track-only compatibility composition.
    ///
    /// Supply a nonempty [`Switch::semantic_name`] before rendering. Normal app
    /// UI should prefer [`Switch::inline`] or [`Switch::setting`].
    pub fn new(checked: bool) -> Self {
        Self {
            checked,
            label: None,
            description: None,
            semantic_name: None,
            composition: SwitchComposition::TrackOnly,
            size: ControlSize::Sm,
            disabled: false,
            id: None,
            on_toggle: None,
        }
    }

    fn canonical(label: Cow<'a, str>, checked: bool, composition: SwitchComposition) -> Self {
        debug_assert!(
            !label.trim().is_empty(),
            "Switch requires a nonempty visible label or title"
        );

        Self {
            checked,
            label: Some(label),
            description: None,
            semantic_name: None,
            composition,
            size: ControlSize::Sm,
            disabled: false,
            id: None,
            on_toggle: None,
        }
    }

    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn description_maybe<T>(mut self, description: Option<T>) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.description = description.map(Into::into);
        self
    }

    pub fn semantic_name(mut self, semantic_name: impl Into<Cow<'a, str>>) -> Self {
        self.semantic_name = Some(semantic_name.into());
        self
    }

    pub fn id(mut self, id: widget::Id) -> Self {
        self.id = Some(id);
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

    pub fn on_toggle(mut self, on_toggle: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(on_toggle));
        self
    }

    pub fn on_toggle_maybe(mut self, on_toggle: Option<impl Fn(bool) -> Message + 'a>) -> Self {
        self.on_toggle = on_toggle.map(|on_toggle| Box::new(on_toggle) as _);
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let label = self.label.unwrap_or_else(|| {
            debug_assert!(
                self.semantic_name
                    .as_deref()
                    .is_some_and(|name| !name.trim().is_empty()),
                "track-only Switch requires nonempty semantic_name metadata"
            );
            Cow::Borrowed("")
        });
        let persistent = if self.checked {
            ChoicePersistentState::Selected
        } else {
            ChoicePersistentState::Unselected
        };
        let message = self
            .on_toggle
            .as_ref()
            .map(|on_toggle| on_toggle(!self.checked));
        let (layout, width) = match self.composition {
            SwitchComposition::Setting => (SingleChoiceLayout::Setting, Length::Fill),
            SwitchComposition::Inline | SwitchComposition::TrackOnly => {
                (SingleChoiceLayout::Leading, Length::Shrink)
            }
        };

        SingleChoice::new(SingleChoiceKind::Switch, layout, label, persistent)
            .description(self.description)
            .size(self.size)
            .width(width)
            .disabled(self.disabled)
            .id(self.id)
            .on_activate(message)
            .into()
    }
}

impl<'a, Message> From<Switch<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(switch: Switch<'a, Message>) -> Self {
        switch.into_element()
    }
}

#[cfg(test)]
mod tests {
    use iced::{keyboard::key, Point, Size};

    use super::*;
    use crate::test_support::WidgetHarness;
    use crate::widgets::controls::choice_test_support::{
        key_pressed, key_released, pointer_click, touch_tap,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Message {
        Toggled(bool),
    }

    #[test]
    fn inline_and_setting_have_distinct_width_grammar() {
        let inline: Element<'_, Message> = Switch::inline("Inline", false).into();
        let setting: Element<'_, Message> = Switch::setting("Setting", false)
            .description("Takes effect immediately")
            .into();
        let inline = WidgetHarness::new(inline, Size::new(320.0, 120.0));
        let setting = WidgetHarness::new(setting, Size::new(320.0, 120.0));

        assert!(inline.bounds().width < 320.0);
        assert_eq!(setting.bounds().width, 320.0);
        assert!(setting.bounds().height > 28.0);
    }

    #[test]
    fn pointer_touch_and_space_publish_next_bool_once() {
        let switch = || -> Element<'static, Message> {
            Switch::inline("Immediate", false)
                .id(widget::Id::new("switch"))
                .on_toggle(Message::Toggled)
                .into()
        };

        let mut pointer = WidgetHarness::new(switch(), Size::new(240.0, 80.0));
        assert_eq!(
            pointer_click(&mut pointer, Point::new(8.0, 8.0)),
            [Message::Toggled(true)]
        );

        let mut touch = WidgetHarness::new(switch(), Size::new(240.0, 80.0));
        assert_eq!(
            touch_tap(&mut touch, 1, Point::new(8.0, 8.0)),
            [Message::Toggled(true)]
        );

        let id = widget::Id::new("switch");
        let keyboard_switch: Element<'_, Message> = Switch::inline("Immediate", false)
            .id(id.clone())
            .on_toggle(Message::Toggled)
            .into();
        let mut keyboard = WidgetHarness::new(keyboard_switch, Size::new(240.0, 80.0));
        keyboard.focus(id);
        assert!(keyboard
            .update(key_pressed(key::Named::Space, key::Code::Space))
            .messages
            .is_empty());
        assert_eq!(
            keyboard
                .update(key_released(key::Named::Space, key::Code::Space))
                .messages,
            [Message::Toggled(true)]
        );
    }

    #[test]
    fn callback_absence_and_disabled_are_inert() {
        let display: Element<'_, Message> = Switch::inline("Display", true).into();
        let disabled: Element<'_, Message> = Switch::setting("Disabled", true)
            .disabled(true)
            .on_toggle(Message::Toggled)
            .into();
        let mut display = WidgetHarness::new(display, Size::new(240.0, 80.0));
        let mut disabled = WidgetHarness::new(disabled, Size::new(240.0, 80.0));

        assert!(display.focusable_ids().is_empty());
        assert!(disabled.focusable_ids().is_empty());
        assert!(pointer_click(&mut display, Point::new(8.0, 8.0)).is_empty());
        assert!(pointer_click(&mut disabled, Point::new(8.0, 8.0)).is_empty());
    }

    #[test]
    fn exact_track_geometry_is_owned_by_choice_metrics() {
        for (size, track) in [
            (ControlSize::Xs, iced::Size::new(28.0, 16.0)),
            (ControlSize::Sm, iced::Size::new(32.0, 18.0)),
            (ControlSize::Md, iced::Size::new(36.0, 20.0)),
            (ControlSize::Lg, iced::Size::new(40.0, 22.0)),
        ] {
            let metrics =
                crate::theme::choice::ChoiceMetrics::for_theme(crate::theme::Theme::Light, size);

            assert_eq!(metrics.switch_track, track);
            assert_eq!(metrics.switch_thumb_size, track.height - 4.0);
        }
    }
}
