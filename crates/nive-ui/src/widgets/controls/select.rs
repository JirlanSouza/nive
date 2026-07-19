mod widget;

use std::{borrow::Cow, fmt};

use iced::{widget::Id, Length};

use crate::theme::{self, ControlSize, FieldValidation};
use crate::Element;

use self::widget::SelectWidget;
use super::form_frame::{FormControlFrame, FormFrameAppearance};

#[derive(Debug, Clone)]
/// One labelled application value rendered by [`Select`].
///
/// The value is the durable identity and must be unique among peer options.
/// Disabled options remain visible with stable geometry but cannot be
/// highlighted or activated.
pub struct SelectOption<'a, T> {
    value: T,
    label: Cow<'a, str>,
    disabled: bool,
}

impl<'a, T> SelectOption<'a, T> {
    pub fn new(value: T, label: impl Into<Cow<'a, str>>) -> Self {
        let label = label.into();
        debug_assert!(
            !label.trim().is_empty(),
            "SelectOption requires a nonempty visible label"
        );

        Self {
            value,
            label,
            disabled: false,
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn label(&self) -> &str {
        self.label.as_ref()
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<T: Eq> PartialEq for SelectOption<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq> Eq for SelectOption<'_, T> {}

impl<T> fmt::Display for SelectOption<'_, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label.as_ref())
    }
}

/// A typed selection control backed by explicit [`SelectOption`] models.
///
/// The supplied `Option<T>` is application-owned committed state. Select keeps
/// only ephemeral open, highlight, typeahead, pressed, and scroll state;
/// opening or navigating never mutates the supplied value. Activation publishes
/// `T`, and the selection changes only after the application rebuilds the view.
///
/// Select is a fill-width, density-aware form control with Sm as its default
/// [`ControlSize`]. Converting it into a typed `Field` lets the Field propagate
/// size, disabled, validation, name, and focus context before element erasure;
/// the Field remains the sole visible label and support/error owner. Placeholder
/// text is not a semantic name.
///
/// `on_select` absence makes the control display-only without disabled colors.
/// Explicit `disabled(true)` has stronger precedence, suppresses interaction,
/// and preserves value, chevron, frame, and geometry. Open/close callbacks
/// report user transitions exactly once when configured; programmatic rebuilds
/// are silent. Popup and chevron state changes are immediate and use no local
/// motion preference or animator.
///
/// The retained name, open state, value, and logical highlight are preparatory
/// metadata. Select does not currently emit native combobox roles, names,
/// expanded state, active-descendant relations, or announcements.
///
/// The former PickList-shaped value list and the private popup row/Tree
/// adapters are intentionally unsupported:
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = Select::<_, ()>::new(vec![String::from("One")], None);
/// ```
///
/// ```compile_fail
/// use nive_ui::widgets::controls::select::widget::{
///     SelectListState, SelectState, SelectWidget,
/// };
/// ```
pub struct Select<'a, T, Message>
where
    T: Clone + Eq,
{
    options: Vec<SelectOption<'a, T>>,
    selected: Option<T>,
    placeholder: Option<Cow<'a, str>>,
    size: ControlSize,
    width: Length,
    validation: FieldValidation,
    semantic_name: Option<Cow<'a, str>>,
    disabled: bool,
    id: Option<Id>,
    on_select: Option<Box<dyn Fn(T) -> Message + 'a>>,
    on_open: Option<Message>,
    on_close: Option<Message>,
}

impl<'a, T, Message> Select<'a, T, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    pub fn new(options: impl Into<Vec<SelectOption<'a, T>>>, selected: Option<T>) -> Self {
        Self {
            options: options.into(),
            selected,
            placeholder: None,
            size: ControlSize::Sm,
            width: Length::Fill,
            validation: FieldValidation::Valid,
            semantic_name: None,
            disabled: false,
            id: None,
            on_select: None,
            on_open: None,
            on_close: None,
        }
    }

    pub fn from_values(options: impl Into<Vec<T>>, selected: Option<T>) -> Self
    where
        T: ToString,
    {
        Self::new(
            options
                .into()
                .into_iter()
                .map(|value| {
                    let label = value.to_string();
                    SelectOption::new(value, label)
                })
                .collect::<Vec<_>>(),
            selected,
        )
    }

    pub fn placeholder(mut self, placeholder: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = Some(placeholder.into());
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

    pub fn validation(mut self, validation: FieldValidation) -> Self {
        self.validation = validation;
        self
    }

    pub fn invalid(self, invalid: bool) -> Self {
        self.validation(if invalid {
            FieldValidation::Invalid
        } else {
            FieldValidation::Valid
        })
    }

    pub fn semantic_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.semantic_name = Some(name.into());
        self
    }

    /// Assigns the stable widget id used for focus operations.
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

    pub fn on_open(mut self, message: Message) -> Self {
        self.on_open = Some(message);
        self
    }

    pub fn on_open_maybe(mut self, message: Option<Message>) -> Self {
        self.on_open = message;
        self
    }

    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    pub fn on_close_maybe(mut self, message: Option<Message>) -> Self {
        self.on_close = message;
        self
    }

    pub fn control_size(&self) -> ControlSize {
        self.size
    }

    pub fn field_validation(&self) -> FieldValidation {
        self.validation
    }

    pub fn semantic_name_value(&self) -> Option<&str> {
        self.semantic_name.as_deref()
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Returns whether every option carries a unique application value.
    pub fn has_unique_values(&self) -> bool {
        self.options.iter().enumerate().all(|(index, option)| {
            self.options[..index]
                .iter()
                .all(|previous| previous.value() != option.value())
        })
    }

    pub fn widget_id(&self) -> Option<&Id> {
        self.id.as_ref()
    }

    pub(crate) fn apply_field_context(
        mut self,
        label: Cow<'a, str>,
        size: ControlSize,
        validation: FieldValidation,
        disabled: bool,
    ) -> (Self, Id) {
        let id = self.id.clone().unwrap_or_else(Id::unique);
        self.id = Some(id.clone());
        self.semantic_name = Some(label);
        self.size = size;
        self.validation = validation;
        self.disabled |= disabled;
        (self, id)
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = theme::form_control_metrics(self.size);
        let width = self.width;
        let validation = self.validation;
        let disabled = self.disabled;
        let interactive = self.on_select.is_some();
        let _semantic_name = self.semantic_name.as_deref();
        let select = SelectWidget::new(
            self.options,
            self.selected,
            self.placeholder,
            width,
            metrics,
            disabled,
            self.id,
            self.on_select,
            self.on_open,
            self.on_close,
        );

        Element::new(FormControlFrame {
            content: select.into(),
            appearance: FormFrameAppearance::Default,
            validation,
            metrics,
            disabled,
            interactive,
        })
    }
}

impl<'a, T, Message> From<Select<'a, T, Message>> for Element<'a, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    fn from(select: Select<'a, T, Message>) -> Self {
        select.into_element()
    }
}

#[cfg(test)]
mod select_tests {
    use super::*;
    use crate::{
        test_support::WidgetHarness,
        theme::{testing::ThemeTestGuard, Theme, ThemeDensity, ThemeMode},
        widgets::controls::{Field, FieldControl},
    };
    use iced::Size;

    #[test]
    fn layout_builders_set_select_width() {
        let default = Select::<_, ()>::from_values(vec!["Free"], None::<&str>);
        let shrunk = Select::<_, ()>::from_values(vec!["Free"], None::<&str>).shrink_width();
        let filled = Select::<_, ()>::from_values(vec!["Free"], None::<&str>).fill_width();

        assert_eq!(default.width, Length::Fill);
        assert_eq!(shrunk.width, Length::Shrink);
        assert_eq!(filled.width, Length::Fill);
    }

    #[test]
    fn typed_options_separate_value_label_and_disabled_state() {
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct Plan(u8);

        let owned = String::from("Professional");
        let option = SelectOption::new(Plan(2), owned).disabled(true);

        assert_eq!(option.value(), &Plan(2));
        assert_eq!(option.label(), "Professional");
        assert!(option.is_disabled());
    }

    #[test]
    fn option_identity_depends_on_the_typed_value() {
        let first = SelectOption::new(7_u8, "Seven");
        let renamed = SelectOption::new(7_u8, String::from("Siete")).disabled(true);
        let different = SelectOption::new(8_u8, "Eight");

        assert_eq!(first, renamed);
        assert_ne!(first, different);
    }

    #[test]
    #[should_panic(expected = "SelectOption requires a nonempty visible label")]
    fn typed_option_rejects_an_empty_label() {
        let _ = SelectOption::new(1_u8, "   ");
    }

    #[test]
    fn field_context_overrides_size_validation_disabled_and_semantic_name() {
        let (select, _) = Select::<_, ()>::from_values(vec!["Free"], None::<&str>)
            .semantic_name("Standalone plan")
            .validation(FieldValidation::Invalid)
            .apply_field_context(
                Cow::Borrowed("Billing plan"),
                ControlSize::Lg,
                FieldValidation::Valid,
                true,
            );

        assert_eq!(select.semantic_name_value(), Some("Billing plan"));
        assert_eq!(select.control_size(), ControlSize::Lg);
        assert_eq!(select.field_validation(), FieldValidation::Valid);
        assert!(select.is_disabled());
    }

    #[test]
    fn select_converts_through_the_opaque_field_control_boundary() {
        let select = Select::<_, ()>::new(vec![SelectOption::new(1_u8, "One")], Some(1_u8));
        let _: FieldControl<'_, ()> = select.into();

        let field: Element<'_, ()> = Field::new(
            "Choice",
            Select::new(vec![SelectOption::new(1_u8, "One")], Some(1_u8)),
        )
        .into();
        let _ = WidgetHarness::new(field, Size::new(240.0, 120.0));
    }

    #[test]
    fn every_density_and_control_size_uses_exact_form_frame_height() {
        for density in ThemeDensity::ALL {
            let theme = Theme::builder("Select matrix", ThemeMode::Light)
                .density(density)
                .build();
            let _guard = ThemeTestGuard::activate(theme);

            for size in [
                ControlSize::Xs,
                ControlSize::Sm,
                ControlSize::Md,
                ControlSize::Lg,
            ] {
                let metrics = theme.form_control_metrics(size);
                let select: Element<'_, ()> =
                    Select::new(vec![SelectOption::new(1_u8, "One")], Some(1_u8))
                        .size(size)
                        .into();
                let harness = WidgetHarness::new(select, Size::new(240.0, 120.0));

                assert_eq!(harness.bounds().height, metrics.height);
                assert_eq!(metrics.text_style.size, 14.0);
            }
        }
    }
}
