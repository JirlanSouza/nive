use std::{borrow::Cow, cell::Cell, rc::Rc};

use iced::widget::Id;
use iced::{
    border::Radius,
    widget::{container, text_input},
    Length, Padding,
};

use super::adapter::{InputEvent, TextInputAdapter};
use super::style as theme_text_input;
use super::style::TextInputAppearance;
use super::{Input, InputRender};
use crate::theme::{ControlSize, FieldValidation};
use crate::widgets::controls::form_frame::{FormControlFrame, FormFrameAppearance};
use crate::Element;

impl<'a, Message> Input<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(placeholder: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) -> Self {
        Self {
            placeholder: placeholder.into(),
            value: value.into(),
            appearance: TextInputAppearance::Standard,
            validation: FieldValidation::Valid,
            size: ControlSize::Sm,
            width: Length::Fill,
            id: None,
            generated_field_id: false,
            semantic_name: None,
            disabled: false,
            read_only: false,
            secure: false,
            on_change: None,
            on_submit: None,
            on_blur: None,
            focus_tracker: None,
        }
    }

    pub fn appearance(mut self, appearance: TextInputAppearance) -> Self {
        self.appearance = appearance;
        self
    }

    pub fn standard(self) -> Self {
        self.appearance(TextInputAppearance::Standard)
    }

    pub fn embedded(self) -> Self {
        self.appearance(TextInputAppearance::Embedded)
    }

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

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
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

    crate::impl_layout_builders!(width_direct, fill_width_direct, shrink_width_direct);

    /// Assigns the stable widget id used for focus operations.
    ///
    /// Callers that focus an Input programmatically should retain and reuse
    /// the same id across view rebuilds.
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self.generated_field_id = false;
        self
    }

    /// Retains a semantic name independently from placeholder content.
    ///
    /// Native accessibility emission is intentionally deferred until Iced
    /// exposes the required relationship hooks.
    pub fn semantic_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.semantic_name = Some(name.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Maps edited text values into app messages.
    pub fn on_change(mut self, f: impl Fn(String) -> Message + 'a) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Conditionally maps edited text values into app messages.
    pub fn on_change_maybe(mut self, f: Option<impl Fn(String) -> Message + 'a>) -> Self {
        self.on_change = f.map(|f| Box::new(f) as _);
        self
    }

    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    pub fn on_blur(mut self, message: Message) -> Self {
        self.on_blur = Some(message);
        self
    }

    pub fn on_blur_maybe(mut self, message: Option<Message>) -> Self {
        self.on_blur = message;
        self
    }

    pub fn value(&self) -> &str {
        self.value.as_ref()
    }

    pub fn track_focus(mut self, focused: Rc<Cell<bool>>) -> Self {
        self.focus_tracker = Some(focused);
        self
    }

    pub fn map<NewMessage: Clone + 'a>(
        self,
        f: impl Fn(Message) -> NewMessage + 'a,
    ) -> Input<'a, NewMessage> {
        let f: Rc<dyn Fn(Message) -> NewMessage + 'a> = Rc::new(f);
        let on_change = self.on_change.map({
            let f = Rc::clone(&f);
            move |on_change| {
                Box::new(move |value| f(on_change(value))) as Box<dyn Fn(String) -> NewMessage + 'a>
            }
        });

        Input {
            placeholder: self.placeholder,
            value: self.value,
            appearance: self.appearance,
            validation: self.validation,
            size: self.size,
            width: self.width,
            id: self.id,
            generated_field_id: self.generated_field_id,
            semantic_name: self.semantic_name,
            disabled: self.disabled,
            read_only: self.read_only,
            secure: self.secure,
            on_change,
            on_submit: self.on_submit.map(|message| f(message)),
            on_blur: self.on_blur.map(|message| f(message)),
            focus_tracker: self.focus_tracker,
        }
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only || self.on_change.is_none()
    }

    pub fn semantic_name_value(&self) -> Option<&str> {
        self.semantic_name.as_deref()
    }

    pub fn widget_id(&self) -> Option<&Id> {
        self.id.as_ref()
    }

    pub fn control_size(&self) -> ControlSize {
        self.size
    }

    pub fn field_validation(&self) -> FieldValidation {
        self.validation
    }

    pub(crate) fn apply_field_context(
        mut self,
        label: Cow<'a, str>,
        size: ControlSize,
        validation: FieldValidation,
        disabled: bool,
    ) -> (Self, Id) {
        let id = match self.id.clone() {
            Some(id) => id,
            None => {
                self.generated_field_id = true;
                let id = Id::unique();
                self.id = Some(id.clone());
                id
            }
        };
        self.semantic_name = Some(label);
        self.size = size;
        self.validation = validation;
        self.disabled |= disabled;
        (self, id)
    }

    pub(in crate::widgets::controls) fn into_group_element(
        self,
        fill: bool,
        radius: Radius,
        font_size: f32,
        padding_v: f32,
        padding_h: f32,
    ) -> Element<'a, Message> {
        self.into_text_input_with(InputRender {
            width: if fill { Length::Fill } else { Length::Shrink },
            font: crate::theme::typography(crate::theme::TypographyRole::Control).font,
            font_size,
            line_height: crate::theme::typography(crate::theme::TypographyRole::Control)
                .line_height,
            padding: Padding::ZERO.vertical(padding_v).horizontal(padding_h),
            radius,
            appearance: TextInputAppearance::Embedded,
        })
    }

    pub(super) fn into_text_input(self) -> Element<'a, Message> {
        let frame_metrics = crate::theme::form_control_metrics(self.size);
        let width = self.width;
        let appearance = self.appearance;
        let validation = self.validation;
        let disabled = self.disabled;
        let content = self.into_text_input_with(InputRender {
            width,
            font: frame_metrics.text_style.font,
            font_size: frame_metrics.text_style.size,
            line_height: frame_metrics.text_style.line_height,
            padding: frame_metrics.padding,
            radius: frame_metrics.radius.into(),
            appearance: TextInputAppearance::Embedded,
        });

        match appearance {
            TextInputAppearance::Standard => Element::new(FormControlFrame {
                content: container(content)
                    .width(width)
                    .height(Length::Fixed(frame_metrics.height))
                    .into(),
                appearance: FormFrameAppearance::Default,
                validation,
                metrics: frame_metrics,
                disabled,
                interactive: true,
            }),
            TextInputAppearance::Embedded => content,
        }
    }

    pub(super) fn into_text_input_with(self, render: InputRender) -> Element<'a, Message> {
        let mut input = text_input::TextInput::new(self.placeholder.as_ref(), self.value.as_ref())
            .padding(render.padding)
            .font(render.font)
            .size(render.font_size)
            .line_height(iced::widget::text::LineHeight::Relative(render.line_height))
            .secure(self.secure);

        let id = self.id.clone();
        let focus_identity = (!self.generated_field_id).then(|| id.clone()).flatten();
        if let Some(id) = self.id {
            input = input.id(id);
        }

        input = input.style(theme_text_input::style(
            render.appearance,
            self.validation,
            render.radius,
        ));

        input = input.width(render.width);

        input = input.on_input_maybe(
            (!self.disabled).then_some(InputEvent::Changed as fn(String) -> InputEvent),
        );

        if self.on_submit.is_some() {
            input = input.on_submit(InputEvent::Submit);
        }

        let input: Element<'a, InputEvent> = input.into();
        let effective_read_only = self.read_only || self.on_change.is_none();
        let input: Element<'a, Message> = Element::new(TextInputAdapter {
            content: input,
            on_change: self.on_change,
            on_submit: self.on_submit,
            on_blur: self.on_blur,
            focus_tracker: self.focus_tracker,
            semantic_name: self.semantic_name,
            read_only: effective_read_only,
            disabled: self.disabled,
            focus_identity,
        });

        input
    }
}
