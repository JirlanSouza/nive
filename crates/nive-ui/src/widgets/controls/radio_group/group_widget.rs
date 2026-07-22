use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    keyboard::{self, key},
    touch, Event, Length, Point, Rectangle, Size, Vector,
};

use super::{RadioGroupFocus, RadioGroupLayout, RadioGroupState, RadioGroupWidget};
use crate::theme::choice::ChoicePersistentState;
use crate::theme::{choice::ChoiceMetrics, Theme};
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
