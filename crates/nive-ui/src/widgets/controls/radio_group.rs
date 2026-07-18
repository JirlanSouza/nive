use std::borrow::Cow;

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    keyboard::{self, key},
    touch, widget, Event, Length, Point, Rectangle, Size, Vector,
};

use crate::advanced::focus::FocusState;
use crate::theme::{choice::ChoiceMetrics, ControlSize, FieldValidation, Theme};
use crate::Element;

use super::field::{normalized_error, FieldError};
use super::single_choice::{SingleChoice, SingleChoiceKind, SingleChoiceLayout};
use super::{FieldHint, FieldLabel, FieldRequirement};
use crate::theme::choice::ChoicePersistentState;

/// A typed, non-renderable option owned by a [`RadioGroup`].
///
/// Values must be unique within their group. The visible label is required and
/// may be supplemented by a wrapping description.
pub struct RadioOption<'a, T> {
    value: T,
    label: Cow<'a, str>,
    description: Option<Cow<'a, str>>,
    disabled: bool,
}

impl<'a, T> RadioOption<'a, T> {
    pub fn new(value: T, label: impl Into<Cow<'a, str>>) -> Self {
        let label = label.into();
        debug_assert!(
            !label.trim().is_empty(),
            "RadioOption requires a nonempty visible label"
        );

        Self {
            value,
            label,
            description: None,
            disabled: false,
        }
    }

    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn description_maybe<D>(mut self, description: Option<D>) -> Self
    where
        D: Into<Cow<'a, str>>,
    {
        self.description = description.map(Into::into);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Layout policy for complete radio option rows.
pub enum RadioGroupLayout {
    /// Stack options vertically.
    #[default]
    Vertical,
    /// Wrap between complete options when finite width is exhausted.
    HorizontalWrap,
}

/// A controlled, typed one-of-many choice with one composite focus entry.
///
/// The group owns its legend, requirement, description, error, selection, and
/// callback. `None` means no selected value; model a user-selectable “None” as
/// an ordinary inner `T` value. Duplicate option values produce a finite
/// display-only fallback. Physical LTR arrows navigate enabled options
/// circularly and Space activates the focused value. Native accessibility-tree
/// roles and relationships are not emitted yet.
pub struct RadioGroup<'a, T, Message> {
    legend: Cow<'a, str>,
    selected: Option<T>,
    options: Vec<RadioOption<'a, T>>,
    requirement: Option<FieldRequirement<'a>>,
    description: Option<Cow<'a, str>>,
    error: Option<Cow<'a, str>>,
    layout: RadioGroupLayout,
    size: ControlSize,
    width: Length,
    disabled: bool,
    id: Option<widget::Id>,
    on_select: Option<Box<dyn Fn(T) -> Message + 'a>>,
}

impl<'a, T, Message> RadioGroup<'a, T, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    pub fn new(
        legend: impl Into<Cow<'a, str>>,
        selected: Option<T>,
        options: impl IntoIterator<Item = RadioOption<'a, T>>,
    ) -> Self {
        let legend = legend.into();
        debug_assert!(
            !legend.trim().is_empty(),
            "RadioGroup requires a nonempty visible legend"
        );

        Self {
            legend,
            selected,
            options: options.into_iter().collect(),
            requirement: None,
            description: None,
            error: None,
            layout: RadioGroupLayout::Vertical,
            size: ControlSize::Sm,
            width: Length::Fill,
            disabled: false,
            id: None,
            on_select: None,
        }
    }

    pub fn requirement(mut self, requirement: FieldRequirement<'a>) -> Self {
        self.requirement = Some(requirement);
        self
    }

    pub fn required(self, text: impl Into<Cow<'a, str>>) -> Self {
        self.requirement(FieldRequirement::Required(text.into()))
    }

    pub fn optional(self, text: impl Into<Cow<'a, str>>) -> Self {
        self.requirement(FieldRequirement::Optional(text.into()))
    }

    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn description_maybe<D>(mut self, description: Option<D>) -> Self
    where
        D: Into<Cow<'a, str>>,
    {
        self.description = description.map(Into::into);
        self
    }

    pub fn error(mut self, error: impl Into<Cow<'a, str>>) -> Self {
        self.error = Some(error.into());
        self
    }

    pub fn error_maybe<E>(mut self, error: Option<E>) -> Self
    where
        E: Into<Cow<'a, str>>,
    {
        self.error = error.map(Into::into);
        self
    }

    pub fn layout(mut self, layout: RadioGroupLayout) -> Self {
        self.layout = layout;
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

    pub fn id(mut self, id: widget::Id) -> Self {
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

    fn into_element(self) -> Element<'a, Message> {
        let error = normalized_error(self.error);
        let metrics = ChoiceMetrics::for_theme(crate::theme::active(), self.size);
        let mut legend = widget::Row::new()
            .push(FieldLabel::new(self.legend))
            .spacing(metrics.support_gap)
            .align_y(iced::Alignment::Center)
            .width(Length::Fill);

        if let Some(requirement) = self.requirement {
            let text = match requirement {
                FieldRequirement::Required(text) | FieldRequirement::Optional(text) => text,
            };
            legend = legend.push(FieldHint::new(text));
        }

        let mut heading = widget::Column::new()
            .push(legend.wrap())
            .spacing(metrics.support_gap)
            .width(Length::Fill);

        if let Some(description) = self.description {
            heading = heading.push(FieldHint::new(description));
        }
        if let Some(error) = &error {
            heading = heading.push(FieldError::new(error.clone()).into_element());
        }

        let choices: Element<'a, Message> = RadioGroupWidget {
            selected: self.selected,
            options: self.options,
            layout: self.layout,
            size: self.size,
            width: self.width,
            validation: if error.is_some() {
                FieldValidation::Invalid
            } else {
                FieldValidation::Valid
            },
            disabled: self.disabled,
            id: self.id,
            on_select: self.on_select,
        }
        .into();

        widget::Column::new()
            .push(heading)
            .push(choices)
            .spacing(metrics.group_gap)
            .width(self.width)
            .into()
    }
}

impl<'a, T, Message> From<RadioGroup<'a, T, Message>> for Element<'a, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    fn from(group: RadioGroup<'a, T, Message>) -> Self {
        group.into_element()
    }
}

struct RadioGroupWidget<'a, T, Message> {
    selected: Option<T>,
    options: Vec<RadioOption<'a, T>>,
    layout: RadioGroupLayout,
    size: ControlSize,
    width: Length,
    validation: FieldValidation,
    disabled: bool,
    id: Option<widget::Id>,
    on_select: Option<Box<dyn Fn(T) -> Message + 'a>>,
}

#[derive(Debug, Default)]
struct RadioGroupState {
    focus: FocusState,
    focused_index: Option<usize>,
}

impl<T, Message> RadioGroupWidget<'_, T, Message>
where
    T: Clone + Eq,
    Message: Clone,
{
    fn values_are_unique(&self) -> bool {
        self.options.iter().enumerate().all(|(index, option)| {
            self.options[index + 1..]
                .iter()
                .all(|peer| peer.value != option.value)
        })
    }

    fn interactive(&self) -> bool {
        !self.disabled
            && self.values_are_unique()
            && self.on_select.is_some()
            && self.options.iter().any(|option| !option.disabled)
    }

    fn selected_index(&self) -> Option<usize> {
        self.selected.as_ref().and_then(|selected| {
            self.options
                .iter()
                .position(|option| &option.value == selected)
        })
    }

    fn reconciled_focus(&self, state: &RadioGroupState) -> Option<usize> {
        state
            .focused_index
            .filter(|index| {
                self.options
                    .get(*index)
                    .is_some_and(|option| !option.disabled)
            })
            .or_else(|| {
                self.selected_index()
                    .filter(|index| !self.options[*index].disabled)
            })
            .or_else(|| self.options.iter().position(|option| !option.disabled))
    }

    fn option_element<'a>(
        &'a self,
        index: usize,
        state: &RadioGroupState,
        width: Length,
    ) -> Element<'a, Message>
    where
        T: 'a,
        Message: 'a,
    {
        let option = &self.options[index];
        let selected = self.selected.as_ref() == Some(&option.value);
        let message = (self.interactive() && !option.disabled && !selected).then(|| {
            self.on_select.as_ref().expect("interactive radio group")(option.value.clone())
        });

        SingleChoice::new(
            SingleChoiceKind::Radio,
            SingleChoiceLayout::Leading,
            option.label.clone(),
            if selected {
                ChoicePersistentState::Selected
            } else {
                ChoicePersistentState::Unselected
            },
        )
        .description(option.description.clone())
        .validation(self.validation)
        .size(self.size)
        .width(width)
        .disabled(self.disabled || option.disabled)
        .on_activate(message)
        .register_focus(false)
        .focused(state.focus.is_focus_visible() && self.reconciled_focus(state) == Some(index))
        .into()
    }

    fn focus_target(&self, state: &RadioGroupState) -> Option<usize> {
        self.reconciled_focus(state)
    }

    fn move_focus(&self, state: &mut RadioGroupState, delta: isize) -> Option<usize> {
        let enabled = self
            .options
            .iter()
            .enumerate()
            .filter_map(|(index, option)| (!option.disabled).then_some(index))
            .collect::<Vec<_>>();
        let current = self.focus_target(state)?;
        let position = enabled
            .iter()
            .position(|index| *index == current)
            .unwrap_or(0);
        let next = (position as isize + delta).rem_euclid(enabled.len() as isize) as usize;
        let index = enabled[next];
        state.focused_index = Some(index);
        Some(index)
    }

    fn publish_if_changed(&self, index: usize, shell: &mut Shell<'_, Message>) {
        let Some(option) = self.options.get(index) else {
            return;
        };
        if option.disabled || self.selected.as_ref() == Some(&option.value) {
            return;
        }
        if let Some(on_select) = &self.on_select {
            shell.publish(on_select(option.value.clone()));
        }
    }
}

impl<T, Message> Widget<Message, Theme, iced::Renderer> for RadioGroupWidget<'_, T, Message>
where
    T: Clone + Eq,
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<RadioGroupState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(RadioGroupState::default())
    }

    fn children(&self) -> Vec<Tree> {
        let state = RadioGroupState::default();
        (0..self.options.len())
            .map(|index| Tree::new(self.option_element(index, &state, self.option_width())))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let elements = {
            let state = tree.state.downcast_ref::<RadioGroupState>();
            (0..self.options.len())
                .map(|index| self.option_element(index, state, self.option_width()))
                .collect::<Vec<_>>()
        };
        tree.diff_children(&elements.iter().map(Element::as_widget).collect::<Vec<_>>());

        if tree
            .state
            .downcast_ref::<RadioGroupState>()
            .focus
            .is_active()
        {
            tree.state.downcast_mut::<RadioGroupState>().focused_index = self
                .selected_index()
                .filter(|index| !self.options[*index].disabled)
                .or_else(|| self.options.iter().position(|option| !option.disabled));
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width);
        let state = tree.state.downcast_ref::<RadioGroupState>();
        let metrics = ChoiceMetrics::for_theme(crate::theme::active(), self.size);
        let maximum = limits.max().width;
        let finite = maximum.is_finite();
        let mut nodes = Vec::with_capacity(self.options.len());

        match self.layout {
            RadioGroupLayout::Vertical => {
                let mut y = 0.0;
                let mut intrinsic_width: f32 = 0.0;
                for index in 0..self.options.len() {
                    let mut option = self.option_element(index, state, self.option_width());
                    let node = option.as_widget_mut().layout(
                        &mut tree.children[index],
                        renderer,
                        &limits.width(self.width),
                    );
                    intrinsic_width = intrinsic_width.max(node.size().width);
                    let height = node.size().height;
                    nodes.push(node.move_to(Point::new(0.0, y)));
                    y += height + metrics.option_gap;
                }
                let height = (y - metrics.option_gap).max(0.0);
                let size = limits.resolve(
                    self.width,
                    Length::Shrink,
                    Size::new(intrinsic_width, height),
                );
                layout::Node::with_children(size, nodes)
            }
            RadioGroupLayout::HorizontalWrap => {
                let mut x = 0.0;
                let mut y = 0.0;
                let mut row_height: f32 = 0.0;
                let mut intrinsic_width: f32 = 0.0;
                for index in 0..self.options.len() {
                    let mut option = self.option_element(index, state, Length::Shrink);
                    let mut node = option.as_widget_mut().layout(
                        &mut tree.children[index],
                        renderer,
                        &layout::Limits::NONE,
                    );
                    if finite && node.size().width > maximum {
                        let mut option = self.option_element(index, state, Length::Fill);
                        node = option.as_widget_mut().layout(
                            &mut tree.children[index],
                            renderer,
                            &layout::Limits::new(Size::ZERO, Size::new(maximum, f32::INFINITY)),
                        );
                    }
                    if finite && x > 0.0 && x + node.size().width > maximum {
                        intrinsic_width = intrinsic_width.max((x - metrics.option_gap).max(0.0));
                        y += row_height + metrics.option_gap;
                        x = 0.0;
                        row_height = 0.0;
                    }
                    row_height = row_height.max(node.size().height);
                    let width = node.size().width;
                    nodes.push(node.move_to(Point::new(x, y)));
                    x += width + metrics.option_gap;
                }
                intrinsic_width = intrinsic_width.max((x - metrics.option_gap).max(0.0));
                let intrinsic = Size::new(intrinsic_width, y + row_height);
                let size = limits.resolve(self.width, Length::Shrink, intrinsic);
                layout::Node::with_children(size, nodes)
            }
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<RadioGroupState>();
        if self.interactive() {
            let RadioGroupState {
                focus,
                focused_index,
            } = state;
            focus.expose(operation, self.id.as_ref(), layout.bounds());
            operation.focusable(
                self.id.as_ref(),
                layout.bounds(),
                &mut RadioGroupFocus {
                    focus,
                    focused_index,
                },
            );
        } else {
            state.focus.clear();
        }
        let state = tree.state.downcast_ref::<RadioGroupState>();
        for (index, (tree, child_layout)) in
            tree.children.iter_mut().zip(layout.children()).enumerate()
        {
            self.option_element(index, state, self.option_width())
                .as_widget_mut()
                .operate(tree, child_layout, renderer, operation);
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        {
            let Tree {
                state, children, ..
            } = tree;
            let state = state.downcast_ref::<RadioGroupState>();
            for (index, (child_tree, child_layout)) in
                children.iter_mut().zip(layout.children()).enumerate()
            {
                self.option_element(index, state, self.option_width())
                    .as_widget_mut()
                    .update(
                        child_tree,
                        event,
                        child_layout,
                        cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
            }
        }

        let interactive = self.interactive();
        let hit = event_position(event, cursor).and_then(|position| {
            layout
                .children()
                .enumerate()
                .find_map(|(index, child)| child.bounds().contains(position).then_some(index))
        });
        let state = tree.state.downcast_mut::<RadioGroupState>();
        if !interactive {
            state.focus.clear();
            state.focused_index = None;
            return;
        }

        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. })
        ) {
            if let Some(index) = hit.filter(|index| !self.options[*index].disabled) {
                state.focus.focus_from_pointer();
                state.focused_index = Some(index);
                shell.request_redraw();
            } else {
                state.focus.deactivate();
            }
        }

        if !state.focus.is_active() {
            return;
        }

        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(named),
            repeat: false,
            ..
        }) = event
        {
            let focus_key = matches!(
                named,
                key::Named::ArrowUp
                    | key::Named::ArrowLeft
                    | key::Named::ArrowDown
                    | key::Named::ArrowRight
                    | key::Named::Home
                    | key::Named::End
                    | key::Named::Space
            );
            if focus_key {
                state.focus.focus_from_keyboard();
            }
            let target = match named {
                key::Named::ArrowUp | key::Named::ArrowLeft => self.move_focus(state, -1),
                key::Named::ArrowDown | key::Named::ArrowRight => self.move_focus(state, 1),
                key::Named::Home => self.options.iter().position(|option| !option.disabled),
                key::Named::End => self.options.iter().rposition(|option| !option.disabled),
                key::Named::Space => self.focus_target(state),
                _ => None,
            };
            if let Some(index) = target {
                state.focused_index = Some(index);
                self.publish_if_changed(index, shell);
                shell.capture_event();
                shell.request_redraw();
            } else if focus_key {
                shell.request_redraw();
            }
        }

        if matches!(event, Event::Window(iced::window::Event::Unfocused)) {
            state.focus.deactivate();
            state.focused_index = None;
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<RadioGroupState>();
        for (index, (tree, child_layout)) in tree.children.iter().zip(layout.children()).enumerate()
        {
            let interaction = self
                .option_element(index, state, self.option_width())
                .as_widget()
                .mouse_interaction(tree, child_layout, cursor, viewport, renderer);
            if interaction != mouse::Interaction::None {
                return interaction;
            }
        }
        mouse::Interaction::None
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<RadioGroupState>();
        for (index, (tree, child_layout)) in tree.children.iter().zip(layout.children()).enumerate()
        {
            self.option_element(index, state, self.option_width())
                .as_widget()
                .draw(
                    tree,
                    renderer,
                    theme,
                    inherited_style,
                    child_layout,
                    cursor,
                    viewport,
                );
        }
    }

    fn overlay<'a>(
        &'a mut self,
        _tree: &'a mut Tree,
        _layout: Layout<'a>,
        _renderer: &iced::Renderer,
        _viewport: &Rectangle,
        _translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, iced::Renderer>> {
        None
    }
}

struct RadioGroupFocus<'a> {
    focus: &'a mut FocusState,
    focused_index: &'a mut Option<usize>,
}

impl operation::Focusable for RadioGroupFocus<'_> {
    fn is_focused(&self) -> bool {
        operation::Focusable::is_focused(self.focus)
    }

    fn focus(&mut self) {
        operation::Focusable::focus(self.focus);
        *self.focused_index = None;
    }

    fn unfocus(&mut self) {
        operation::Focusable::unfocus(self.focus);
        *self.focused_index = None;
    }
}

impl<T, Message> RadioGroupWidget<'_, T, Message> {
    fn option_width(&self) -> Length {
        match self.layout {
            RadioGroupLayout::Vertical => self.width,
            RadioGroupLayout::HorizontalWrap => Length::Shrink,
        }
    }
}

impl<'a, T, Message> From<RadioGroupWidget<'a, T, Message>> for Element<'a, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    fn from(group: RadioGroupWidget<'a, T, Message>) -> Self {
        Element::new(group)
    }
}

fn event_position(event: &Event, cursor: mouse::Cursor) -> Option<Point> {
    match event {
        Event::Touch(touch::Event::FingerPressed { position, .. })
        | Event::Touch(touch::Event::FingerLifted { position, .. }) => Some(*position),
        Event::Mouse(_) => cursor.position(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use iced::{keyboard::key, Point, Size};

    use super::*;
    use crate::test_support::WidgetHarness;
    use crate::widgets::controls::choice_test_support::{key_pressed, pointer_click};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Choice {
        First,
        Second,
        Third,
        None,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Message {
        Selected(Choice),
    }

    fn options() -> [RadioOption<'static, Choice>; 3] {
        [
            RadioOption::new(Choice::First, "First"),
            RadioOption::new(Choice::Second, "Second").description("A longer description"),
            RadioOption::new(Choice::Third, "Third").disabled(true),
        ]
    }

    #[test]
    fn generic_values_none_and_explicit_none_render() {
        let unselected: Element<'_, Message> = RadioGroup::new("Choice", None, options()).into();
        let explicit_none: Element<'_, Message> = RadioGroup::new(
            "Choice",
            Some(Choice::None),
            [RadioOption::new(Choice::None, "No preference")],
        )
        .into();

        assert!(
            WidgetHarness::new(unselected, Size::new(320.0, 240.0))
                .bounds()
                .height
                > 0.0
        );
        assert!(
            WidgetHarness::new(explicit_none, Size::new(320.0, 240.0))
                .bounds()
                .height
                > 0.0
        );
    }

    #[test]
    fn vertical_and_horizontal_wrap_are_finite() {
        let vertical: Element<'_, Message> = RadioGroup::new("Choice", None, options()).into();
        let wrapped: Element<'_, Message> = RadioGroup::new("Choice", None, options())
            .layout(RadioGroupLayout::HorizontalWrap)
            .into();
        let vertical = WidgetHarness::new(vertical, Size::new(160.0, 400.0));
        let wrapped = WidgetHarness::new(wrapped, Size::new(160.0, 400.0));

        assert!(vertical.bounds().size().width.is_finite());
        assert!(wrapped.bounds().size().height.is_finite());
    }

    #[test]
    fn duplicate_values_fall_back_to_display_only() {
        let duplicate: Element<'_, Message> = RadioGroup::new(
            "Duplicate",
            None,
            [
                RadioOption::new(Choice::First, "First"),
                RadioOption::new(Choice::First, "Again"),
            ],
        )
        .on_select(Message::Selected)
        .into();
        let mut harness = WidgetHarness::new(duplicate, Size::new(320.0, 240.0));

        assert!(harness.focusable_ids().is_empty());
        assert!(pointer_click(&mut harness, Point::new(8.0, 60.0)).is_empty());
    }

    #[test]
    fn group_has_one_focus_entry_and_arrows_skip_disabled_options() {
        let id = widget::Id::new("radio-group");
        let group: Element<'_, Message> = RadioGroup::new("Choice", Some(Choice::First), options())
            .id(id.clone())
            .on_select(Message::Selected)
            .into();
        let mut harness = WidgetHarness::new(group, Size::new(320.0, 240.0));

        assert_eq!(harness.focusable_ids(), std::slice::from_ref(&id));
        harness.focus(id);
        assert!(harness
            .state_at::<RadioGroupState>(&[1])
            .focus
            .is_focus_visible());
        assert_eq!(
            harness
                .update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight))
                .messages,
            [Message::Selected(Choice::Second)]
        );
        assert!(harness
            .update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight))
            .messages
            .is_empty());
    }

    #[test]
    fn selected_option_activation_is_a_no_op() {
        let group: Element<'_, Message> = RadioGroup::new("Choice", Some(Choice::First), options())
            .on_select(Message::Selected)
            .into();
        let mut harness = WidgetHarness::new(group, Size::new(320.0, 240.0));

        assert!(pointer_click(&mut harness, Point::new(8.0, 60.0)).is_empty());
        assert!(harness.state_at::<RadioGroupState>(&[1]).focus.is_active());
        assert!(!harness
            .state_at::<RadioGroupState>(&[1])
            .focus
            .is_focus_visible());
    }

    #[test]
    fn focused_value_reconciles_after_option_reorder() {
        let id = widget::Id::new("reordered-radio-group");
        let initial: Element<'static, Message> =
            RadioGroup::new("Choice", Some(Choice::First), options())
                .id(id.clone())
                .on_select(Message::Selected)
                .into();
        let mut harness = WidgetHarness::new(initial, Size::new(320.0, 240.0));
        harness.focus(id.clone());
        assert_eq!(
            harness
                .update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight))
                .messages,
            [Message::Selected(Choice::Second)]
        );

        let reordered: Element<'static, Message> = RadioGroup::new(
            "Choice",
            Some(Choice::Second),
            [
                RadioOption::new(Choice::Second, "Second"),
                RadioOption::new(Choice::First, "First"),
                RadioOption::new(Choice::Third, "Third").disabled(true),
            ],
        )
        .id(id)
        .on_select(Message::Selected)
        .into();
        harness.replace(reordered);

        assert_eq!(
            harness
                .update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight))
                .messages,
            [Message::Selected(Choice::First)]
        );
    }
}
