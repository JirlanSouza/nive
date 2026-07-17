use std::borrow::Cow;

use iced::{widget, Length};

use crate::theme::{ControlSize, FieldValidation};
use crate::Element;

use super::field::{normalized_error, FieldError};
use super::single_choice::{SingleChoice, SingleChoiceKind, SingleChoiceLayout};
use crate::theme::choice::ChoicePersistentState;

/// Controlled Checkbox value.
///
/// User activation requests `Unchecked → Checked`, `Checked → Unchecked`, and
/// `Mixed → Checked`. The application remains the durable state owner.
///
/// ```compile_fail
/// use nive_ui::theme::choice::ChoiceMetrics;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CheckboxState {
    /// The choice is not selected.
    #[default]
    Unchecked,
    /// The choice is selected.
    Checked,
    /// An app-owned aggregate contains both checked and unchecked values.
    Mixed,
}

impl CheckboxState {
    /// Returns the value requested by one user activation.
    ///
    /// Mixed activates to Checked; users do not cycle into Mixed.
    pub const fn next(self) -> Self {
        match self {
            Self::Unchecked | Self::Mixed => Self::Checked,
            Self::Checked => Self::Unchecked,
        }
    }
}

impl From<bool> for CheckboxState {
    fn from(checked: bool) -> Self {
        if checked {
            Self::Checked
        } else {
            Self::Unchecked
        }
    }
}

/// A controlled submitted choice with an inline visible label.
///
/// The constructor is the sole durable-state input. The optional description is
/// inside the complete target, while a normalized nonempty error is rendered
/// outside it and owns invalid presentation. With no callback the checkbox is
/// display-only; [`Checkbox::disabled`] additionally applies disabled styling.
/// Retained text and keyboard behavior do not yet imply native accessibility-tree
/// emission.
///
/// ```compile_fail
/// use nive_ui::prelude::*;
///
/// // State belongs only in the constructor; there is no competing builder.
/// let _ = Checkbox::<()>::new("Choice", false).checked(true);
/// ```
pub struct Checkbox<'a, Message> {
    label: Cow<'a, str>,
    state: CheckboxState,
    description: Option<Cow<'a, str>>,
    error: Option<Cow<'a, str>>,
    size: ControlSize,
    width: Length,
    disabled: bool,
    id: Option<widget::Id>,
    on_toggle: Option<Box<dyn Fn(CheckboxState) -> Message + 'a>>,
}

impl<'a, Message> Checkbox<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(label: impl Into<Cow<'a, str>>, state: impl Into<CheckboxState>) -> Self {
        let label = label.into();
        debug_assert!(
            !label.trim().is_empty(),
            "Checkbox requires a nonempty visible label"
        );

        Self {
            label,
            state: state.into(),
            description: None,
            error: None,
            size: ControlSize::Sm,
            width: Length::Shrink,
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

    pub fn error(mut self, error: impl Into<Cow<'a, str>>) -> Self {
        self.error = Some(error.into());
        self
    }

    pub fn error_maybe<T>(mut self, error: Option<T>) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.error = error.map(Into::into);
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

    crate::impl_layout_builders!(width_direct, fill_width_direct, shrink_width_direct);

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_toggle(mut self, on_toggle: impl Fn(CheckboxState) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(on_toggle));
        self
    }

    pub fn on_toggle_maybe(
        mut self,
        on_toggle: Option<impl Fn(CheckboxState) -> Message + 'a>,
    ) -> Self {
        self.on_toggle = on_toggle.map(|on_toggle| Box::new(on_toggle) as _);
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let error = normalized_error(self.error);
        let validation = if error.is_some() {
            FieldValidation::Invalid
        } else {
            FieldValidation::Valid
        };
        let persistent = match self.state {
            CheckboxState::Unchecked => ChoicePersistentState::Unselected,
            CheckboxState::Checked => ChoicePersistentState::Selected,
            CheckboxState::Mixed => ChoicePersistentState::Mixed,
        };
        let message = self
            .on_toggle
            .as_ref()
            .map(|on_toggle| on_toggle(self.state.next()));
        let target: Element<'a, Message> = SingleChoice::new(
            SingleChoiceKind::Checkbox,
            SingleChoiceLayout::Leading,
            self.label,
            persistent,
        )
        .description(self.description)
        .validation(validation)
        .size(self.size)
        .width(self.width)
        .disabled(self.disabled)
        .id(self.id)
        .on_activate(message)
        .into();

        if let Some(error) = error {
            widget::Column::new()
                .push(target)
                .push(FieldError::new(error).into_element())
                .spacing(crate::theme::space(crate::theme::SpaceStep::Xs))
                .width(self.width)
                .into()
        } else {
            target
        }
    }
}

impl<'a, Message> From<Checkbox<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(checkbox: Checkbox<'a, Message>) -> Self {
        checkbox.into_element()
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use iced::{keyboard::key, Point, Size};

    use super::*;
    use crate::test_support::WidgetHarness;
    use crate::widgets::controls::choice_test_support::{
        key_pressed, key_released, pointer_click, touch_tap,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Message {
        Toggled(CheckboxState),
    }

    #[test]
    fn state_model_and_bool_conversion_are_deterministic() {
        assert_eq!(CheckboxState::default(), CheckboxState::Unchecked);
        assert_eq!(CheckboxState::from(false), CheckboxState::Unchecked);
        assert_eq!(CheckboxState::from(true), CheckboxState::Checked);
        assert_eq!(CheckboxState::Unchecked.next(), CheckboxState::Checked);
        assert_eq!(CheckboxState::Checked.next(), CheckboxState::Unchecked);
        assert_eq!(CheckboxState::Mixed.next(), CheckboxState::Checked);
    }

    #[test]
    fn owned_and_borrowed_data_and_error_normalization_render() {
        let borrowed: Element<'_, Message> = Checkbox::new("Borrowed", false)
            .description("Description")
            .error("   ")
            .into();
        let owned: Element<'_, Message> =
            Checkbox::new(String::from("Owned"), CheckboxState::Mixed)
                .description(String::from("Description"))
                .error(String::from("Required"))
                .into();

        assert!(
            WidgetHarness::new(borrowed, Size::new(240.0, 120.0))
                .bounds()
                .height
                > 0.0
        );
        assert!(
            WidgetHarness::new(owned, Size::new(240.0, 120.0))
                .bounds()
                .height
                > 28.0
        );
        assert_eq!(normalized_error(Some(Cow::Borrowed(" \n"))), None);
    }

    #[test]
    fn every_modality_publishes_the_next_state_once() {
        let checkbox = || -> Element<'static, Message> {
            Checkbox::new("Mixed", CheckboxState::Mixed)
                .id(widget::Id::new("checkbox"))
                .on_toggle(Message::Toggled)
                .into()
        };

        let mut pointer = WidgetHarness::new(checkbox(), Size::new(240.0, 80.0));
        assert_eq!(
            pointer_click(&mut pointer, Point::new(8.0, 8.0)),
            [Message::Toggled(CheckboxState::Checked)]
        );

        let mut touch = WidgetHarness::new(checkbox(), Size::new(240.0, 80.0));
        assert_eq!(
            touch_tap(&mut touch, 1, Point::new(8.0, 8.0)),
            [Message::Toggled(CheckboxState::Checked)]
        );

        let id = widget::Id::new("checkbox");
        let keyboard_checkbox: Element<'_, Message> = Checkbox::new("Mixed", CheckboxState::Mixed)
            .id(id.clone())
            .on_toggle(Message::Toggled)
            .into();
        let mut keyboard = WidgetHarness::new(keyboard_checkbox, Size::new(240.0, 80.0));
        keyboard.focus(id);
        assert!(keyboard
            .update(key_pressed(key::Named::Space, key::Code::Space))
            .messages
            .is_empty());
        assert_eq!(
            keyboard
                .update(key_released(key::Named::Space, key::Code::Space))
                .messages,
            [Message::Toggled(CheckboxState::Checked)]
        );
    }

    #[test]
    fn callback_absence_and_disabled_remove_focus_and_activation() {
        let display: Element<'_, Message> = Checkbox::new("Display", true).into();
        let disabled: Element<'_, Message> = Checkbox::new("Disabled", CheckboxState::Mixed)
            .disabled(true)
            .on_toggle(Message::Toggled)
            .into();
        let mut display = WidgetHarness::new(display, Size::new(240.0, 80.0));
        let mut disabled = WidgetHarness::new(disabled, Size::new(240.0, 80.0));

        assert_eq!(display.focusable_ids().len(), 0);
        assert_eq!(disabled.focusable_ids().len(), 0);
        assert!(pointer_click(&mut display, Point::new(8.0, 8.0)).is_empty());
        assert!(pointer_click(&mut disabled, Point::new(8.0, 8.0)).is_empty());
    }

    #[test]
    fn every_size_uses_the_form_height_floor() {
        for (size, height) in [
            (ControlSize::Xs, 24.0),
            (ControlSize::Sm, 28.0),
            (ControlSize::Md, 32.0),
            (ControlSize::Lg, 36.0),
        ] {
            let checkbox: Element<'_, Message> = Checkbox::new("Choice", false).size(size).into();
            let harness = WidgetHarness::new(checkbox, Size::new(240.0, 80.0));

            assert_eq!(harness.bounds().height, height);
        }
    }
}
