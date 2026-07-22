use std::{borrow::Cow, cell::Cell, rc::Rc};

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    keyboard::{self, key::Named},
    touch,
    widget::Id,
    window, Event, Length, Rectangle, Size, Vector,
};

use super::helpers::{find_list_state, first_enabled, trigger, unique_values};
use super::{SelectEvent, SelectList, SelectState, SelectWidget, TriggerPress};
use crate::{
    theme::FormControlMetrics,
    widgets::{
        controls::select::SelectOption,
        overlays::{
            anchored_overlay::{scroll::EnsureVisibleHandle, translated_bounds, AnchoredOverlay},
            popover, PopoverCollision, PopoverInset, PopoverPlacement, PopoverWidth,
        },
    },
    Element,
};

impl<'a, T, Message> SelectWidget<'a, T, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    #[allow(clippy::too_many_arguments)]
    pub(in crate::widgets::controls::select) fn new(
        options: Vec<SelectOption<'a, T>>,
        selected: Option<T>,
        placeholder: Option<Cow<'a, str>>,
        width: Length,
        metrics: FormControlMetrics,
        disabled: bool,
        id: Option<Id>,
        on_select: Option<Box<dyn Fn(T) -> Message + 'a>>,
        on_open: Option<Message>,
        on_close: Option<Message>,
    ) -> Self {
        let selected_option = selected
            .as_ref()
            .and_then(|selected| options.iter().find(|option| option.value() == selected));
        let model_valid = unique_values(&options);
        let is_placeholder = selected_option.is_none();
        let label = selected_option
            .map(|option| Cow::Owned(option.label().to_owned()))
            .or(placeholder)
            .unwrap_or(Cow::Borrowed("Select"));
        let closed_trigger = trigger(label.clone(), is_placeholder, false, width, metrics);
        let open_trigger = trigger(label, is_placeholder, true, width, metrics);
        let ensure_visible = EnsureVisibleHandle::new();
        let focus_visible = Rc::new(Cell::new(false));
        let list: Element<'a, SelectEvent<T>> = SelectList::new(
            options.clone(),
            selected.clone(),
            ensure_visible.clone(),
            Rc::clone(&focus_visible),
            model_valid,
        )
        .into();
        let popup = popover::surface_with_ensure_visible(
            list,
            PopoverInset::EdgeToEdge,
            Some(&ensure_visible),
            PopoverWidth::AtLeastAnchor,
        );

        Self {
            closed_trigger,
            open_trigger,
            popup,
            options,
            selected,
            width,
            height: metrics.height,
            disabled,
            model_valid,
            id,
            on_select,
            on_open,
            on_close,
            ensure_visible,
            focus_visible,
        }
    }

    pub(in crate::widgets::controls::select) fn interactive(&self) -> bool {
        !self.disabled && self.on_select.is_some()
    }

    pub(in crate::widgets::controls::select) fn initial_highlight(&self) -> Option<usize> {
        if !self.model_valid {
            return None;
        }
        self.selected
            .as_ref()
            .and_then(|selected| {
                self.options
                    .iter()
                    .position(|option| option.value() == selected && !option.is_disabled())
            })
            .or_else(|| first_enabled(&self.options, true))
    }

    pub(in crate::widgets::controls::select) fn open(
        &self,
        popup_tree: &mut Tree,
        state: &mut SelectState,
        shell: &mut Shell<'_, Message>,
    ) {
        if state.open {
            return;
        }

        state.open = true;
        state.pressed = None;
        if let Some(list) = find_list_state(popup_tree) {
            let highlight = self.initial_highlight();
            let label = highlight.map(|index| self.options[index].label().to_owned());
            list.reset(highlight, label);
        }
        if let Some(message) = self.on_open.clone() {
            shell.publish(message);
        }
        shell.invalidate_layout();
        shell.request_redraw();
    }
}

impl<'a, T, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for SelectWidget<'a, T, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SelectState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(SelectState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![
            Tree::new(&self.closed_trigger),
            Tree::new(&self.open_trigger),
            Tree::new(&self.popup),
        ]
    }

    fn diff(&self, tree: &mut Tree) {
        if tree.children.len() != 3 {
            tree.children = self.children();
        } else {
            tree.children[0].diff(self.closed_trigger.as_widget());
            tree.children[1].diff(self.open_trigger.as_widget());
            tree.children[2].diff(self.popup.as_widget());
        }
        let state = tree.state.downcast_mut::<SelectState>();
        if !self.interactive() {
            state.focus.clear();
            state.open = false;
            state.pressed = None;
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Fixed(self.height))
    }

    fn size_hint(&self) -> Size<Length> {
        self.size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        let closed =
            self.closed_trigger
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &limits);
        let open =
            self.open_trigger
                .as_widget_mut()
                .layout(&mut tree.children[1], renderer, &limits);
        layout::Node::with_children(closed.size(), vec![closed, open])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<SelectState>();
        if self.interactive() {
            state
                .focus
                .register(operation, self.id.as_ref(), layout.bounds());
            self.focus_visible.set(state.focus.is_focus_visible());
        } else {
            state.focus.clear();
            self.focus_visible.set(false);
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let interactive = self.interactive();
        let bounds = layout.bounds();
        let (state_storage, children) = (&mut tree.state, &mut tree.children);
        let state = state_storage.downcast_mut::<SelectState>();

        if !interactive {
            if state.focus.is_active() || state.open || state.pressed.is_some() {
                state.focus.clear();
                state.open = false;
                state.pressed = None;
                self.focus_visible.set(false);
                shell.request_redraw();
            }
            return;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(bounds) =>
            {
                state.focus.focus_from_pointer();
                self.focus_visible.set(state.focus.is_focus_visible());
                state.pressed = Some(TriggerPress::Pointer);
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.focus.deactivate();
                self.focus_visible.set(false);
                state.pressed = None;
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let opens = state.pressed == Some(TriggerPress::Pointer) && cursor.is_over(bounds);
                state.pressed = None;
                if opens {
                    self.open(&mut children[2], state, shell);
                    shell.capture_event();
                }
                shell.request_redraw();
            }
            Event::Touch(touch::Event::FingerPressed { id, position })
                if bounds.contains(*position) =>
            {
                state.focus.focus_from_pointer();
                self.focus_visible.set(state.focus.is_focus_visible());
                state.pressed = Some(TriggerPress::Touch(*id));
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Touch(touch::Event::FingerLifted { id, position }) => {
                let opens =
                    state.pressed == Some(TriggerPress::Touch(*id)) && bounds.contains(*position);
                if state.pressed == Some(TriggerPress::Touch(*id)) {
                    state.pressed = None;
                }
                if opens {
                    self.open(&mut children[2], state, shell);
                    shell.capture_event();
                }
                shell.request_redraw();
            }
            Event::Touch(touch::Event::FingerLost { id, .. })
                if state.pressed == Some(TriggerPress::Touch(*id)) =>
            {
                state.pressed = None;
                shell.request_redraw();
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key:
                    keyboard::Key::Named(
                        Named::Enter | Named::Space | Named::ArrowDown | Named::ArrowUp,
                    ),
                ..
            }) if state.focus.is_active() && !state.open => {
                state.focus.focus_from_keyboard();
                self.focus_visible.set(true);
                self.open(&mut children[2], state, shell);
                shell.capture_event();
            }
            Event::Window(window::Event::Unfocused) => {
                state.pressed = None;
                state.open = false;
                state.focus.deactivate();
                self.focus_visible.set(false);
                shell.request_redraw();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if self.interactive() && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<SelectState>();
        self.focus_visible.set(state.focus.is_focus_visible());
        let index = usize::from(state.open);
        let trigger = if state.open {
            &self.open_trigger
        } else {
            &self.closed_trigger
        };
        if let Some(trigger_layout) = layout.children().nth(index) {
            trigger.as_widget().draw(
                &tree.children[index],
                renderer,
                theme,
                inherited_style,
                trigger_layout,
                cursor,
                viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        _renderer: &iced::Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        let (state_storage, children) = (&mut tree.state, &mut tree.children);
        let state = state_storage.downcast_mut::<SelectState>();
        if !state.open || !self.interactive() {
            return None;
        }

        let selected = self.selected.as_ref();
        let on_select = self.on_select.as_ref();
        let on_close = self.on_close.clone();
        let open = &mut state.open;
        let pressed = &mut state.pressed;
        let popup_tree = &mut children[2];
        Some(overlay::Element::new(Box::new(
            AnchoredOverlay::new(
                translated_bounds(layout.bounds(), translation),
                &mut self.popup,
                popup_tree,
                PopoverPlacement::BottomStart,
                PopoverWidth::AtLeastAnchor,
                PopoverCollision::FlipAndShift,
                4.0,
                Some(SelectEvent::Close),
                move |event, shell: &mut Shell<'_, Message>| {
                    if let SelectEvent::Commit(value) = event {
                        if selected != Some(&value) {
                            if let Some(on_select) = on_select {
                                shell.publish(on_select(value));
                            }
                        }
                    }
                    if *open {
                        *open = false;
                        *pressed = None;
                        if let Some(message) = on_close.clone() {
                            shell.publish(message);
                        }
                        shell.invalidate_layout();
                        shell.request_redraw();
                    }
                },
            )
            .ensure_visible(self.ensure_visible.clone()),
        )))
    }
}
