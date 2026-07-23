mod interaction;

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    touch, Event, Length, Point, Rectangle, Size, Vector,
};

use super::{RadioGroupFocus, RadioGroupLayout, RadioGroupState, RadioGroupWidget};
use crate::theme::choice::ChoicePersistentState;
use crate::theme::Theme;
use crate::widgets::controls::single_choice::{SingleChoice, SingleChoiceKind, SingleChoiceLayout};
use crate::Element;

impl<T, Message> RadioGroupWidget<'_, T, Message>
where
    T: Clone + Eq,
    Message: Clone,
{
    pub(super) fn values_are_unique(&self) -> bool {
        self.options.iter().enumerate().all(|(index, option)| {
            self.options[index + 1..]
                .iter()
                .all(|peer| peer.value != option.value)
        })
    }

    pub(super) fn interactive(&self) -> bool {
        !self.disabled
            && self.values_are_unique()
            && self.on_select.is_some()
            && self.options.iter().any(|option| !option.disabled)
    }

    pub(super) fn selected_index(&self) -> Option<usize> {
        self.selected.as_ref().and_then(|selected| {
            self.options
                .iter()
                .position(|option| &option.value == selected)
        })
    }

    pub(super) fn reconciled_focus(&self, state: &RadioGroupState) -> Option<usize> {
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

    pub(super) fn option_element<'a>(
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

    pub(super) fn focus_target(&self, state: &RadioGroupState) -> Option<usize> {
        self.reconciled_focus(state)
    }

    pub(super) fn move_focus(&self, state: &mut RadioGroupState, delta: isize) -> Option<usize> {
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

    pub(super) fn publish_if_changed(&self, index: usize, shell: &mut Shell<'_, Message>) {
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
        self.layout_impl(tree, renderer, limits)
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
        self.update_impl(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
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
    pub(super) fn option_width(&self) -> Length {
        match self.layout {
            RadioGroupLayout::Vertical => self.width,
            RadioGroupLayout::HorizontalWrap => Length::Shrink,
        }
    }
}

pub(super) fn event_position(event: &Event, cursor: mouse::Cursor) -> Option<Point> {
    match event {
        Event::Touch(touch::Event::FingerPressed { position, .. })
        | Event::Touch(touch::Event::FingerLifted { position, .. }) => Some(*position),
        Event::Mouse(_) => cursor.position(),
        _ => None,
    }
}
