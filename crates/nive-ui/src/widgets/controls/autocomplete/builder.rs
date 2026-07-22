use std::borrow::Cow;
use std::rc::Rc;

use iced::{widget::Id, Length};

use super::render::results_content;
use super::widget::{AutocompleteCallbacks, AutocompleteHandles, AutocompleteWidget};
use super::{Autocomplete, AutocompleteHighlight, AutocompleteResults};
use crate::widgets::controls::{button, Input, InputGroup};
use crate::widgets::overlays::anchored_overlay::scroll::EnsureVisibleHandle;
use crate::{
    theme::{ControlSize, FieldValidation},
    widgets::{
        overlays::{Popover, PopoverCollision, PopoverInset, PopoverPlacement},
        primitives::IconRole,
    },
    Element,
};

impl<'a, T, Message> Autocomplete<'a, T, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    pub fn new(
        query: impl Into<Cow<'a, str>>,
        selected: Option<T>,
        results: AutocompleteResults<'a, T>,
    ) -> Self {
        Self {
            query: query.into(),
            selected,
            results,
            placeholder: Cow::Borrowed("Search"),
            semantic_name: None,
            size: ControlSize::Sm,
            width: Length::Fill,
            validation: FieldValidation::Valid,
            disabled: false,
            id: None,
            open: false,
            highlight: AutocompleteHighlight::None,
            on_change: None,
            on_select: None,
            on_clear: None,
            on_submit: None,
            on_blur: None,
            on_dismiss: None,
        }
    }

    pub fn query(&self) -> &str {
        self.query.as_ref()
    }

    pub fn selected(&self) -> Option<&T> {
        self.selected.as_ref()
    }

    pub fn results(&self) -> &AutocompleteResults<'a, T> {
        &self.results
    }

    pub fn placeholder(mut self, placeholder: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn semantic_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.semantic_name = Some(name.into());
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn highlight(mut self, highlight: AutocompleteHighlight) -> Self {
        self.highlight = highlight;
        self
    }

    pub fn on_change(mut self, callback: impl Fn(String) -> Message + 'a) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    pub fn on_change_maybe(mut self, callback: Option<impl Fn(String) -> Message + 'a>) -> Self {
        self.on_change = callback.map(|callback| Box::new(callback) as _);
        self
    }

    pub fn on_select(mut self, callback: impl Fn(T) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(callback));
        self
    }

    pub fn on_select_maybe(mut self, callback: Option<impl Fn(T) -> Message + 'a>) -> Self {
        self.on_select = callback.map(|callback| Box::new(callback) as _);
        self
    }

    pub fn on_clear(mut self, message: Message) -> Self {
        self.on_clear = Some(message);
        self
    }

    pub fn on_clear_maybe(mut self, message: Option<Message>) -> Self {
        self.on_clear = message;
        self
    }

    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    pub fn on_submit_maybe(mut self, message: Option<Message>) -> Self {
        self.on_submit = message;
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

    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.on_dismiss = Some(message);
        self
    }

    pub fn on_dismiss_maybe(mut self, message: Option<Message>) -> Self {
        self.on_dismiss = message;
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

    pub fn widget_id(&self) -> Option<&Id> {
        self.id.as_ref()
    }

    pub fn highlight_policy(&self) -> AutocompleteHighlight {
        self.highlight
    }

    pub fn is_open(&self) -> bool {
        self.open
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

    pub(super) fn into_element(self) -> Element<'a, Message>
    where
        T: 'static,
    {
        let Self {
            query,
            selected: _selected,
            results,
            placeholder,
            semantic_name,
            size,
            width,
            validation,
            disabled,
            id,
            open,
            highlight,
            on_change,
            on_select,
            on_clear,
            on_submit,
            on_blur,
            on_dismiss,
        } = self;
        let loading = matches!(&results, AutocompleteResults::Loading);
        let show_clear = !loading && !query.is_empty() && on_clear.is_some();
        let match_query = query.clone();
        let widget_query = query.to_string();
        let widget_results = results.clone();
        let handles = AutocompleteHandles::new();
        let input_focused = handles.input_focused();
        let highlighted = handles.highlighted_index();
        let ensure_pending = handles.ensure_pending();
        let local_closed = handles.local_closed();
        let ensure_visible = EnsureVisibleHandle::new();
        let on_select: Option<Rc<dyn Fn(T) -> Message + 'a>> = widget_results
            .has_unique_values()
            .then(|| on_select.map(Rc::from))
            .flatten();
        let mut input = Input::new(placeholder, query)
            .size(size)
            .validation(validation)
            .disabled(disabled)
            .track_focus(Rc::clone(&input_focused));
        if let Some(id) = id {
            input = input.id(id);
        }
        if let Some(name) = semantic_name {
            input = input.semantic_name(name);
        }
        if let Some(on_change) = on_change {
            input = input.on_change(on_change);
        }
        if let Some(message) = on_submit {
            input = input.on_submit(message);
        }
        if let Some(message) = on_blur {
            input = input.on_blur(message);
        }
        let mut anchor = InputGroup::new(input)
            .size(size)
            .width(width)
            .validation(validation)
            .disabled(disabled)
            .activity(loading);
        if show_clear {
            anchor = anchor.clear_action(
                button::icon(IconRole::WindowClose, "Clear")
                    .on_press(on_clear.expect("clear capability checked above")),
            );
        }

        let popover_dismiss = on_dismiss.clone();
        let callbacks = AutocompleteCallbacks::new(on_select.clone(), on_dismiss);
        let popover: Element<'a, Message> = Popover::new(anchor)
            .content(results_content(
                results,
                match_query,
                Rc::clone(&highlighted),
                Rc::clone(&ensure_pending),
                ensure_visible.clone(),
                Rc::clone(&local_closed),
                on_select.clone(),
            ))
            .open(open && !disabled)
            .placement(PopoverPlacement::BottomStart)
            .capped_anchor_width(360.0)
            .max_height(280.0)
            .collision(PopoverCollision::FlipAndShift)
            .inset(PopoverInset::EdgeToEdge)
            .on_dismiss_maybe(popover_dismiss)
            .ensure_visible(ensure_visible)
            .into();

        AutocompleteWidget::new(
            popover,
            widget_query,
            widget_results,
            open && !disabled,
            highlight,
            handles,
            callbacks,
        )
        .into()
    }
}
