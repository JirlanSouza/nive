use std::{borrow::Cow, cell::Cell, rc::Rc, time::Duration};

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Renderer as _, Shell, Widget,
    },
    keyboard::{self, key::Named},
    touch,
    widget::{column, container, row, text, Id},
    window, Alignment, Background, Border, Color, Event, Length, Padding, Point, Rectangle, Shadow,
    Size, Vector,
};

use crate::{
    advanced::focus::FocusState,
    theme::{
        self,
        choice::{self, ChoicePersistentState, ChoiceStateInput},
        BorderRole, ControlRole, FieldValidation, FormControlMetrics, TextRole, TypographyRole,
    },
    widgets::{
        display::measured_text::{EllipsisStrategy, MeasuredText},
        navigation::menu::{
            self, MENU_COLUMN_GAP, MENU_ICON_SIZE, MENU_LIST_INSET, MENU_ROW_HEIGHT,
            MENU_ROW_PADDING_H, MENU_ROW_RADIUS,
        },
        overlays::{
            anchored_overlay::{scroll::EnsureVisibleHandle, translated_bounds, AnchoredOverlay},
            popover, PopoverCollision, PopoverInset, PopoverPlacement, PopoverWidth,
        },
        primitives::{icon, IconRole},
    },
    Element,
};

use super::SelectOption;

const TYPEAHEAD_TIMEOUT: Duration = Duration::from_millis(700);

#[derive(Debug, Clone)]
enum SelectEvent<T> {
    Commit(T),
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TriggerPress {
    Pointer,
    Touch(touch::Finger),
}

#[derive(Debug, Default)]
pub(super) struct SelectState {
    focus: FocusState,
    open: bool,
    pressed: Option<TriggerPress>,
}

pub(super) struct SelectWidget<'a, T, Message>
where
    T: Clone + Eq,
{
    closed_trigger: Element<'a, Message>,
    open_trigger: Element<'a, Message>,
    popup: Element<'a, SelectEvent<T>>,
    options: Vec<SelectOption<'a, T>>,
    selected: Option<T>,
    width: Length,
    height: f32,
    disabled: bool,
    model_valid: bool,
    id: Option<Id>,
    on_select: Option<Box<dyn Fn(T) -> Message + 'a>>,
    on_open: Option<Message>,
    on_close: Option<Message>,
    ensure_visible: EnsureVisibleHandle,
    focus_visible: Rc<Cell<bool>>,
}

impl<'a, T, Message> SelectWidget<'a, T, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
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

    fn interactive(&self) -> bool {
        !self.disabled && self.on_select.is_some()
    }

    fn initial_highlight(&self) -> Option<usize> {
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

    fn open(&self, popup_tree: &mut Tree, state: &mut SelectState, shell: &mut Shell<'_, Message>) {
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

impl<'a, T, Message> From<SelectWidget<'a, T, Message>> for Element<'a, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    fn from(select: SelectWidget<'a, T, Message>) -> Self {
        Element::new(select)
    }
}

#[derive(Debug, Default)]
pub(super) struct SelectListState {
    highlight: Option<usize>,
    highlighted_label: Option<String>,
    pressed: Option<usize>,
    typeahead: String,
    typeahead_deadline: Option<iced::time::Instant>,
    now: Option<iced::time::Instant>,
    ensure_pending: bool,
}

impl SelectListState {
    fn reset(&mut self, highlight: Option<usize>, highlighted_label: Option<String>) {
        self.highlight = highlight;
        self.highlighted_label = highlighted_label;
        self.pressed = None;
        self.typeahead.clear();
        self.typeahead_deadline = None;
        self.ensure_pending = highlight.is_some();
    }
}

struct SelectList<'a, T>
where
    T: Clone + Eq,
{
    content: Element<'a, SelectEvent<T>>,
    options: Vec<SelectOption<'a, T>>,
    selected: Option<T>,
    ensure_visible: EnsureVisibleHandle,
    focus_visible: Rc<Cell<bool>>,
    selection_capable: bool,
}

impl<'a, T> SelectList<'a, T>
where
    T: Clone + Eq + 'a,
{
    fn new(
        options: Vec<SelectOption<'a, T>>,
        selected: Option<T>,
        ensure_visible: EnsureVisibleHandle,
        focus_visible: Rc<Cell<bool>>,
        selection_capable: bool,
    ) -> Self {
        let mut content = column![].padding(MENU_LIST_INSET).width(Length::Fill);
        if options.is_empty() {
            content = content.push(
                container(MeasuredText::new_inherited(
                    "No options available",
                    EllipsisStrategy::End,
                    TypographyRole::Control,
                ))
                .style(menu::style::row_style(false, false, false, MENU_ROW_RADIUS))
                .padding(Padding::ZERO.horizontal(MENU_ROW_PADDING_H))
                .center_y(Length::Fixed(MENU_ROW_HEIGHT))
                .width(Length::Fill),
            );
        }
        for option in &options {
            let selected_row = selected.as_ref() == Some(option.value());
            let mark = if selected_row { "✓" } else { "" };
            let row = row![
                container(text(mark))
                    .width(Length::Fixed(MENU_ICON_SIZE))
                    .center_y(Length::Fill),
                container(MeasuredText::new_inherited(
                    option.label.clone(),
                    EllipsisStrategy::End,
                    TypographyRole::Control,
                ))
                .width(Length::Fill)
                .clip(true),
            ]
            .spacing(MENU_COLUMN_GAP)
            .align_y(Alignment::Center)
            .height(Length::Fill)
            .width(Length::Fill);
            content = content.push(
                container(row)
                    .style(menu::style::row_style(
                        selected_row,
                        false,
                        option.is_disabled(),
                        MENU_ROW_RADIUS,
                    ))
                    .padding(Padding::ZERO.horizontal(MENU_ROW_PADDING_H))
                    .height(Length::Fixed(MENU_ROW_HEIGHT))
                    .width(Length::Fill),
            );
        }

        Self {
            content: container(content).width(Length::Fill).into(),
            options,
            selected,
            ensure_visible,
            focus_visible,
            selection_capable,
        }
    }

    fn request_highlight_visible(&self, state: &mut SelectListState, layout: Layout<'_>) {
        if !state.ensure_pending {
            return;
        }
        if let Some(bounds) = state
            .highlight
            .and_then(|index| option_bounds(layout.bounds(), index, self.options.len()))
        {
            self.ensure_visible.request(bounds);
        }
        state.ensure_pending = false;
    }
}

impl<'a, T> Widget<SelectEvent<T>, crate::theme::Theme, iced::Renderer> for SelectList<'a, T>
where
    T: Clone + Eq + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SelectListState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(SelectListState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
        let state = tree.state.downcast_mut::<SelectListState>();
        let reconciled = state
            .highlighted_label
            .as_deref()
            .and_then(|label| {
                self.options.iter().enumerate().find_map(|(index, option)| {
                    (option.label() == label
                        && is_enabled(&self.options, index, self.selection_capable))
                    .then_some(index)
                })
            })
            .or_else(|| {
                state
                    .highlight
                    .filter(|index| is_enabled(&self.options, *index, self.selection_capable))
            })
            .or_else(|| {
                state
                    .highlighted_label
                    .is_some()
                    .then(|| first_enabled(&self.options, self.selection_capable))
                    .flatten()
            });
        set_highlight(&self.options, state, reconciled);
        state.pressed = None;
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, SelectEvent<T>>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let state = tree.state.downcast_mut::<SelectListState>();
        self.request_highlight_visible(state, layout);
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            state.now = Some(*now);
            if state
                .typeahead_deadline
                .is_some_and(|deadline| *now > deadline)
            {
                state.typeahead.clear();
                state.typeahead_deadline = None;
            }
        }

        if let Some(position) = pointer_position(event, cursor) {
            let highlight = position
                .and_then(|point| option_at(layout.bounds(), point, self.options.len()))
                .filter(|index| is_enabled(&self.options, *index, self.selection_capable));
            if state.highlight != highlight {
                set_highlight(&self.options, state, highlight);
                state.ensure_pending = highlight.is_some();
                shell.request_redraw();
            }
        }

        if is_primary_press(event) {
            state.pressed = primary_position(event, cursor)
                .and_then(|point| option_at(layout.bounds(), point, self.options.len()))
                .filter(|index| is_enabled(&self.options, *index, self.selection_capable));
            if state.pressed.is_some() {
                shell.capture_event();
            }
            shell.request_redraw();
        }

        if let Some(index) = release_position(event, cursor)
            .and_then(|point| option_at(layout.bounds(), point, self.options.len()))
            .filter(|index| Some(*index) == state.pressed)
        {
            shell.publish(SelectEvent::Commit(self.options[index].value().clone()));
            shell.capture_event();
        }
        if is_release(event) {
            state.pressed = None;
            shell.request_redraw();
        }

        if let Event::Keyboard(keyboard::Event::KeyPressed {
            text: Some(text), ..
        }) = event
        {
            if !text.is_empty() && text.chars().all(|character| !character.is_control()) {
                let now = state.now.unwrap_or_else(iced::time::Instant::now);
                if state
                    .typeahead_deadline
                    .is_none_or(|deadline| now > deadline)
                {
                    state.typeahead.clear();
                }
                state.typeahead.push_str(text);
                state.typeahead_deadline = Some(now + TYPEAHEAD_TIMEOUT);
                if let Some(index) = typeahead_match(
                    &self.options,
                    state.highlight,
                    state.typeahead.as_str(),
                    self.selection_capable,
                ) {
                    set_highlight(&self.options, state, Some(index));
                    state.ensure_pending = true;
                }
                shell.capture_event();
                shell.request_redraw();
                return;
            }
        }

        let moved = match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::ArrowDown),
                ..
            }) => move_highlight(&self.options, state.highlight, 1, self.selection_capable),
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::ArrowUp),
                ..
            }) => move_highlight(&self.options, state.highlight, -1, self.selection_capable),
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::Home),
                ..
            }) => first_enabled(&self.options, self.selection_capable),
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::End),
                ..
            }) => last_enabled(&self.options, self.selection_capable),
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::Enter | Named::Space),
                ..
            }) => {
                if let Some(index) = state.highlight {
                    shell.publish(SelectEvent::Commit(self.options[index].value().clone()));
                    shell.capture_event();
                }
                return;
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::Tab),
                ..
            }) => {
                shell.publish(SelectEvent::Close);
                return;
            }
            _ => return,
        };
        if moved != state.highlight {
            set_highlight(&self.options, state, moved);
            state.ensure_pending = moved.is_some();
            shell.request_redraw();
        }
        shell.capture_event();
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        cursor
            .position()
            .and_then(|point| option_at(layout.bounds(), point, self.options.len()))
            .filter(|index| is_enabled(&self.options, *index, self.selection_capable))
            .map_or(mouse::Interaction::None, |_| mouse::Interaction::Pointer)
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
        let state = tree.state.downcast_ref::<SelectListState>();
        for (index, option) in self.options.iter().enumerate() {
            let Some(bounds) = option_bounds(layout.bounds(), index, self.options.len()) else {
                continue;
            };
            let resolved = choice::resolve_state(ChoiceStateInput {
                persistent: if self.selected.as_ref() == Some(option.value()) {
                    ChoicePersistentState::Selected
                } else {
                    ChoicePersistentState::Unselected
                },
                validation: FieldValidation::Valid,
                callback_present: self.selection_capable && !option.is_disabled(),
                disabled: option.is_disabled(),
                hovered: state.highlight == Some(index),
                pressed: state.pressed == Some(index),
                focused: self.focus_visible.get() && state.highlight == Some(index),
            });
            let control = theme.control(ControlRole::Selectable, resolved.control);
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border::default().rounded(MENU_ROW_RADIUS),
                    shadow: Shadow::default(),
                    snap: true,
                },
                control.background,
            );
            if resolved.control.interaction.focused {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + 1.0,
                            y: bounds.y + 1.0,
                            width: (bounds.width - 2.0).max(0.0),
                            height: (bounds.height - 2.0).max(0.0),
                        },
                        border: Border {
                            color: theme.border(BorderRole::Focus).color,
                            width: 1.0,
                            radius: MENU_ROW_RADIUS.into(),
                        },
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    Background::Color(Color::TRANSPARENT),
                );
            }
        }
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, SelectEvent<T>, crate::theme::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, T> From<SelectList<'a, T>> for Element<'a, SelectEvent<T>>
where
    T: Clone + Eq + 'a,
{
    fn from(list: SelectList<'a, T>) -> Self {
        Element::new(list)
    }
}

fn trigger<'a, Message: 'a>(
    label: Cow<'a, str>,
    placeholder: bool,
    open: bool,
    width: Length,
    metrics: FormControlMetrics,
) -> Element<'a, Message> {
    let label = text(label)
        .font(metrics.text_style.font)
        .size(metrics.text_style.size)
        .line_height(text::LineHeight::Relative(metrics.text_style.line_height))
        .shaping(text::Shaping::Auto)
        .style(theme::text::style(if placeholder {
            TextRole::Muted
        } else {
            TextRole::Primary
        }));
    let chevron = icon::role(if open {
        IconRole::NiveDisclosureUp
    } else {
        IconRole::NiveDisclosureDown
    })
    .custom_size(MENU_ICON_SIZE);

    container(
        row![
            container(label).width(Length::Fill).clip(true),
            container(chevron)
                .width(Length::Fixed(MENU_ICON_SIZE))
                .height(Length::Fixed(MENU_ICON_SIZE)),
        ]
        .align_y(Alignment::Center),
    )
    .padding(metrics.padding)
    .width(width)
    .height(Length::Fixed(metrics.height))
    .into()
}

fn find_list_state(tree: &mut Tree) -> Option<&mut SelectListState> {
    if tree.tag == tree::Tag::of::<SelectListState>() {
        return Some(tree.state.downcast_mut::<SelectListState>());
    }
    tree.children.iter_mut().find_map(find_list_state)
}

fn unique_values<T: Eq>(options: &[SelectOption<'_, T>]) -> bool {
    options.iter().enumerate().all(|(index, option)| {
        options[..index]
            .iter()
            .all(|previous| previous.value() != option.value())
    })
}

fn set_highlight<T>(
    options: &[SelectOption<'_, T>],
    state: &mut SelectListState,
    highlight: Option<usize>,
) {
    state.highlight = highlight;
    state.highlighted_label = highlight.map(|index| options[index].label().to_owned());
}

fn first_enabled<T>(options: &[SelectOption<'_, T>], model_valid: bool) -> Option<usize> {
    model_valid
        .then(|| options.iter().position(|option| !option.is_disabled()))
        .flatten()
}

fn last_enabled<T>(options: &[SelectOption<'_, T>], model_valid: bool) -> Option<usize> {
    model_valid
        .then(|| options.iter().rposition(|option| !option.is_disabled()))
        .flatten()
}

fn is_enabled<T>(options: &[SelectOption<'_, T>], index: usize, model_valid: bool) -> bool {
    model_valid
        && options
            .get(index)
            .is_some_and(|option| !option.is_disabled())
}

fn move_highlight<T>(
    options: &[SelectOption<'_, T>],
    current: Option<usize>,
    direction: isize,
    model_valid: bool,
) -> Option<usize> {
    if direction > 0 {
        ((current.map_or(0, |index| index.saturating_add(1)))..options.len())
            .find(|index| is_enabled(options, *index, model_valid))
            .or(current.filter(|index| is_enabled(options, *index, model_valid)))
    } else {
        let start = current.unwrap_or(options.len());
        (0..start)
            .rev()
            .find(|index| is_enabled(options, *index, model_valid))
            .or(current.filter(|index| is_enabled(options, *index, model_valid)))
    }
}

fn typeahead_match<T>(
    options: &[SelectOption<'_, T>],
    current: Option<usize>,
    prefix: &str,
    model_valid: bool,
) -> Option<usize> {
    if options.is_empty() {
        return None;
    }
    let prefix = prefix.to_lowercase();
    let start = current.map_or(0, |index| (index + 1) % options.len());
    (0..options.len())
        .map(|offset| (start + offset) % options.len())
        .find(|index| {
            is_enabled(options, *index, model_valid)
                && options[*index].label().to_lowercase().starts_with(&prefix)
        })
}

fn option_bounds(bounds: Rectangle, index: usize, count: usize) -> Option<Rectangle> {
    (index < count).then(|| Rectangle {
        x: bounds.x + MENU_LIST_INSET,
        y: bounds.y + MENU_LIST_INSET + index as f32 * MENU_ROW_HEIGHT,
        width: (bounds.width - MENU_LIST_INSET * 2.0).max(0.0),
        height: MENU_ROW_HEIGHT,
    })
}

fn option_at(bounds: Rectangle, point: Point, count: usize) -> Option<usize> {
    if point.x < bounds.x + MENU_LIST_INSET
        || point.x > bounds.x + bounds.width - MENU_LIST_INSET
        || point.y < bounds.y + MENU_LIST_INSET
    {
        return None;
    }
    let index = ((point.y - bounds.y - MENU_LIST_INSET) / MENU_ROW_HEIGHT).floor() as usize;
    option_bounds(bounds, index, count)
        .is_some_and(|bounds| bounds.contains(point))
        .then_some(index)
}

fn pointer_position(event: &Event, _cursor: mouse::Cursor) -> Option<Option<Point>> {
    match event {
        Event::Mouse(mouse::Event::CursorMoved { position }) => Some(Some(*position)),
        Event::Mouse(mouse::Event::CursorLeft) => Some(None),
        Event::Touch(touch::Event::FingerMoved { position, .. }) => Some(Some(*position)),
        _ => None,
    }
}

fn is_primary_press(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
    )
}

fn primary_position(event: &Event, cursor: mouse::Cursor) -> Option<Point> {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => cursor.position(),
        Event::Touch(touch::Event::FingerPressed { position, .. }) => Some(*position),
        _ => None,
    }
}

fn release_position(event: &Event, cursor: mouse::Cursor) -> Option<Point> {
    match event {
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => cursor.position(),
        Event::Touch(touch::Event::FingerLifted { position, .. }) => Some(*position),
        _ => None,
    }
}

fn is_release(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. })
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test_support::WidgetHarness,
        widgets::controls::{choice_test_support::key_pressed, Select},
    };
    use iced::{keyboard::key, widget::Space, Size};

    fn options() -> Vec<SelectOption<'static, u8>> {
        vec![
            SelectOption::new(1, "Alpha"),
            SelectOption::new(2, "Beta").disabled(true),
            SelectOption::new(3, "Bravo"),
        ]
    }

    #[test]
    fn bounded_navigation_skips_disabled_options() {
        let options = options();

        assert_eq!(move_highlight(&options, Some(0), 1, true), Some(2));
        assert_eq!(move_highlight(&options, Some(2), 1, true), Some(2));
        assert_eq!(move_highlight(&options, Some(2), -1, true), Some(0));
        assert_eq!(move_highlight(&options, Some(0), -1, true), Some(0));
    }

    #[test]
    fn typeahead_wraps_only_its_search_pass() {
        let options = options();

        assert_eq!(typeahead_match(&options, Some(2), "a", true), Some(0));
        assert_eq!(typeahead_match(&options, Some(0), "br", true), Some(2));
        assert_eq!(typeahead_match(&options, Some(0), "be", true), None);
    }

    #[test]
    fn row_geometry_has_one_four_pixel_list_inset() {
        let bounds = Rectangle::new(Point::new(10.0, 20.0), Size::new(200.0, 92.0));

        assert_eq!(
            option_bounds(bounds, 1, 3),
            Some(Rectangle::new(
                Point::new(14.0, 52.0),
                Size::new(192.0, 28.0)
            ))
        );
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Message {
        Opened,
        Selected(u8),
        Closed,
    }

    fn interactive_select(selected: Option<u8>, id: Id) -> Element<'static, Message> {
        Select::new(options(), selected)
            .id(id)
            .on_select(Message::Selected)
            .on_open(Message::Opened)
            .on_close(Message::Closed)
            .into()
    }

    #[test]
    fn closed_keyboard_activation_opens_without_committing_and_has_one_focus_target() {
        for (named, code) in [
            (Named::Enter, key::Code::Enter),
            (Named::Space, key::Code::Space),
            (Named::ArrowDown, key::Code::ArrowDown),
            (Named::ArrowUp, key::Code::ArrowUp),
        ] {
            let id = Id::unique();
            let mut harness = WidgetHarness::new(
                interactive_select(Some(1), id.clone()),
                Size::new(240.0, 160.0),
            );
            assert_eq!(harness.focusable_ids(), vec![id.clone()]);
            harness.focus(id);

            let result = harness.update(key_pressed(named, code));

            assert_eq!(result.messages, vec![Message::Opened]);
            assert!(result.captured);
            assert!(harness.state_at::<SelectState>(&[0]).open);
            assert!(harness.has_overlay());
        }
    }

    #[test]
    fn popup_navigation_is_bounded_skips_disabled_and_commits_before_close() {
        let id = Id::unique();
        let mut harness = WidgetHarness::new(
            interactive_select(Some(1), id.clone()),
            Size::new(240.0, 160.0),
        );
        harness.focus(id);
        harness.update(key_pressed(Named::Enter, key::Code::Enter));

        let moved = harness
            .update_overlay(key_pressed(Named::ArrowDown, key::Code::ArrowDown))
            .expect("open Select overlay");
        assert!(moved.messages.is_empty());
        let committed = harness
            .update_overlay(key_pressed(Named::Enter, key::Code::Enter))
            .expect("open Select overlay");

        assert_eq!(
            committed.messages,
            vec![Message::Selected(3), Message::Closed]
        );
        assert!(!harness.state_at::<SelectState>(&[0]).open);
    }

    #[test]
    fn committing_the_current_value_closes_without_republishing_selection() {
        let id = Id::unique();
        let mut harness = WidgetHarness::new(
            interactive_select(Some(1), id.clone()),
            Size::new(240.0, 160.0),
        );
        harness.focus(id);
        harness.update(key_pressed(Named::Enter, key::Code::Enter));

        let result = harness
            .update_overlay(key_pressed(Named::Enter, key::Code::Enter))
            .expect("open Select overlay");

        assert_eq!(result.messages, vec![Message::Closed]);
    }

    #[test]
    fn escape_closes_and_tab_closes_without_capturing_traversal() {
        for (named, code, captured) in [
            (Named::Escape, key::Code::Escape, true),
            (Named::Tab, key::Code::Tab, false),
        ] {
            let id = Id::unique();
            let mut harness = WidgetHarness::new(
                interactive_select(Some(1), id.clone()),
                Size::new(240.0, 160.0),
            );
            harness.focus(id);
            harness.update(key_pressed(Named::Enter, key::Code::Enter));

            let result = harness
                .update_overlay(key_pressed(named, code))
                .expect("open Select overlay");

            assert_eq!(result.messages, vec![Message::Closed]);
            assert_eq!(result.captured, captured);
            assert!(!harness.state_at::<SelectState>(&[0]).open);
        }
    }

    #[test]
    fn popup_is_at_least_as_wide_as_its_trigger() {
        let id = Id::unique();
        let select: Element<'static, Message> = Select::new(options(), Some(1))
            .id(id.clone())
            .width(Length::Fixed(180.0))
            .on_select(Message::Selected)
            .on_open(Message::Opened)
            .on_close(Message::Closed)
            .into();
        let mut harness = WidgetHarness::new(select, Size::new(240.0, 160.0));
        harness.focus(id);
        harness.update(key_pressed(Named::Enter, key::Code::Enter));

        let popup = harness.overlay_bounds().expect("open Select overlay");

        assert!(popup.width >= harness.bounds().width);
        assert!(popup.x >= 8.0);
        assert!(popup.x + popup.width <= 232.0);
    }

    #[test]
    fn disabling_an_existing_select_clears_focus_and_open_ownership() {
        let id = Id::unique();
        let mut harness = WidgetHarness::new(
            interactive_select(Some(1), id.clone()),
            Size::new(240.0, 160.0),
        );
        harness.focus(id.clone());
        harness.update(key_pressed(Named::Enter, key::Code::Enter));
        assert!(harness.state_at::<SelectState>(&[0]).open);

        let disabled: Element<'static, Message> = Select::new(options(), Some(1))
            .id(id)
            .disabled(true)
            .on_select(Message::Selected)
            .into();
        harness.replace(disabled);

        assert!(!harness.state_at::<SelectState>(&[0]).open);
        assert_eq!(harness.focused_widgets(), 0);
        assert!(harness.focusable_ids().is_empty());
    }

    #[test]
    fn pointer_and_touch_commit_the_option_before_the_close_notification() {
        for touch_input in [false, true] {
            let id = Id::unique();
            let mut harness = WidgetHarness::new(
                interactive_select(Some(1), id.clone()),
                Size::new(240.0, 160.0),
            );
            harness.focus(id);
            harness.update(key_pressed(Named::Enter, key::Code::Enter));
            let popup = harness.overlay_bounds().expect("open Select overlay");
            let point = Point::new(
                popup.x + MENU_LIST_INSET + 12.0,
                popup.y + MENU_LIST_INSET + MENU_ROW_HEIGHT * 2.0 + 14.0,
            );

            let (pressed, released) = if touch_input {
                (
                    Event::Touch(touch::Event::FingerPressed {
                        id: touch::Finger(7),
                        position: point,
                    }),
                    Event::Touch(touch::Event::FingerLifted {
                        id: touch::Finger(7),
                        position: point,
                    }),
                )
            } else {
                harness.set_cursor(point);
                (
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                )
            };
            let press = harness
                .update_overlay(pressed)
                .expect("open Select overlay");
            assert!(press.messages.is_empty());
            let release = harness
                .update_overlay(released)
                .expect("open Select overlay");

            assert_eq!(
                release.messages,
                vec![Message::Selected(3), Message::Closed]
            );
        }
    }

    #[test]
    fn programmatic_rebuild_is_silent_and_preserves_the_open_session() {
        let id = Id::unique();
        let mut harness = WidgetHarness::new(
            interactive_select(Some(1), id.clone()),
            Size::new(240.0, 160.0),
        );
        harness.focus(id.clone());
        assert_eq!(
            harness
                .update(key_pressed(Named::Enter, key::Code::Enter))
                .messages,
            vec![Message::Opened]
        );

        harness.replace(interactive_select(Some(3), id));

        assert!(harness.state_at::<SelectState>(&[0]).open);
        assert!(harness.has_overlay());
    }

    #[test]
    fn callback_absence_is_display_only_without_disabled_focus_or_hover_capability() {
        let select: Element<'static, Message> = Select::new(options(), Some(1)).into();
        let mut harness = WidgetHarness::new(select, Size::new(240.0, 160.0));
        harness.set_cursor(Point::new(40.0, 16.0));

        assert!(harness.focusable_ids().is_empty());
        assert_eq!(harness.mouse_interaction(), mouse::Interaction::None);
        assert!(harness
            .update(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left
            )))
            .messages
            .is_empty());
        assert!(!harness.has_overlay());
    }

    #[test]
    fn empty_model_opens_with_finite_explanatory_content_and_cannot_commit() {
        let id = Id::unique();
        let select: Element<'static, Message> = Select::new(Vec::new(), None::<u8>)
            .id(id.clone())
            .on_select(Message::Selected)
            .on_open(Message::Opened)
            .on_close(Message::Closed)
            .into();
        let mut harness = WidgetHarness::new(select, Size::new(240.0, 120.0));
        harness.focus(id);
        assert_eq!(
            harness
                .update(key_pressed(Named::Enter, key::Code::Enter))
                .messages,
            vec![Message::Opened]
        );

        let bounds = harness.overlay_bounds().expect("empty Select overlay");
        assert!(bounds.x.is_finite());
        assert!(bounds.y.is_finite());
        assert!(bounds.width.is_finite() && bounds.width > 0.0);
        assert!(bounds.height.is_finite() && bounds.height >= MENU_ROW_HEIGHT);
        let commit = harness
            .update_overlay(key_pressed(Named::Enter, key::Code::Enter))
            .expect("empty Select overlay");
        assert!(commit.messages.is_empty());
        assert!(harness.state_at::<SelectState>(&[0]).open);
    }

    #[test]
    fn duplicate_values_are_diagnosable_and_every_row_is_nonactivating() {
        let id = Id::unique();
        let duplicate_options = vec![
            SelectOption::new(1_u8, "First"),
            SelectOption::new(1_u8, "Duplicate"),
        ];
        let model = Select::<_, Message>::new(duplicate_options.clone(), None);
        assert!(!model.has_unique_values());
        let select: Element<'static, Message> = Select::new(duplicate_options, None)
            .id(id.clone())
            .on_select(Message::Selected)
            .on_open(Message::Opened)
            .on_close(Message::Closed)
            .into();
        let mut harness = WidgetHarness::new(select, Size::new(240.0, 120.0));
        harness.focus(id);
        harness.update(key_pressed(Named::Enter, key::Code::Enter));

        let bounds = harness.overlay_bounds().expect("duplicate Select overlay");
        assert!(bounds.width.is_finite() && bounds.height.is_finite());
        let activation = harness
            .update_overlay(key_pressed(Named::Enter, key::Code::Enter))
            .expect("duplicate Select overlay");
        assert!(activation.messages.is_empty());
        assert!(harness.state_at::<SelectState>(&[0]).open);
    }

    #[test]
    fn missing_selected_value_recovers_through_first_enabled_without_inventing_state() {
        let id = Id::unique();
        let mut harness = WidgetHarness::new(
            interactive_select(Some(99), id.clone()),
            Size::new(240.0, 160.0),
        );
        harness.focus(id);
        harness.update(key_pressed(Named::Enter, key::Code::Enter));

        let result = harness
            .update_overlay(key_pressed(Named::Enter, key::Code::Enter))
            .expect("open Select overlay");

        assert_eq!(result.messages, vec![Message::Selected(1), Message::Closed]);
    }

    #[test]
    fn home_end_and_typeahead_drive_the_persistent_open_overlay() {
        for (selected, navigation, expected) in [
            (
                1,
                key_pressed(Named::End, key::Code::End),
                Message::Selected(3),
            ),
            (
                3,
                key_pressed(Named::Home, key::Code::Home),
                Message::Selected(1),
            ),
            (1, text_key("br"), Message::Selected(3)),
        ] {
            let id = Id::unique();
            let mut harness = WidgetHarness::new(
                interactive_select(Some(selected), id.clone()),
                Size::new(240.0, 160.0),
            );
            harness.focus(id);
            harness.update(key_pressed(Named::Enter, key::Code::Enter));
            harness
                .update_overlay(navigation)
                .expect("open Select overlay");

            let result = harness
                .update_overlay(key_pressed(Named::Enter, key::Code::Enter))
                .expect("open Select overlay");

            assert_eq!(result.messages, vec![expected, Message::Closed]);
        }
    }

    #[test]
    fn initial_highlight_is_ensured_visible_in_a_low_viewport() {
        let options = (0_u8..24)
            .map(|value| SelectOption::new(value, format!("Option {value}")))
            .collect::<Vec<_>>();
        let id = Id::unique();
        let select: Element<'static, Message> = Select::new(options, Some(23))
            .id(id.clone())
            .width(Length::Fixed(180.0))
            .on_select(Message::Selected)
            .into();
        let mut harness = WidgetHarness::new(select, Size::new(220.0, 90.0));
        harness.focus(id);
        harness.update(key_pressed(Named::Enter, key::Code::Enter));

        harness
            .update_overlay(Event::Window(window::Event::RedrawRequested(
                iced::time::Instant::now(),
            )))
            .expect("open Select overlay");

        assert!(harness
            .overlay_scroll_offsets()
            .iter()
            .any(|offset| offset.y.abs() > f32::EPSILON));
    }

    #[test]
    fn popup_flips_and_remains_safe_when_the_trigger_is_near_the_bottom() {
        let id = Id::unique();
        let content: Element<'static, Message> = iced::widget::column![
            Space::new().height(Length::Fixed(58.0)),
            Select::new(options(), Some(1))
                .id(id.clone())
                .on_select(Message::Selected),
        ]
        .into();
        let mut harness = WidgetHarness::new(content, Size::new(240.0, 100.0));
        let anchor = harness.focusable_bounds(&id).expect("Select trigger");
        harness.focus(id);
        harness.update(key_pressed(Named::Enter, key::Code::Enter));

        let popup = harness.overlay_bounds().expect("flipped Select overlay");

        assert!(popup.y < anchor.y);
        assert!(popup.y >= 8.0);
        assert!(popup.y + popup.height <= 92.0);
    }

    #[test]
    fn diff_reorder_preserves_the_highlighted_option_by_visible_identity() {
        let id = Id::unique();
        let mut harness = WidgetHarness::new(
            interactive_select(Some(1), id.clone()),
            Size::new(240.0, 160.0),
        );
        harness.focus(id.clone());
        harness.update(key_pressed(Named::Enter, key::Code::Enter));
        harness
            .update_overlay(key_pressed(Named::ArrowDown, key::Code::ArrowDown))
            .expect("open Select overlay");

        let reordered: Element<'static, Message> = Select::new(
            vec![
                SelectOption::new(1, "Alpha"),
                SelectOption::new(3, "Bravo"),
                SelectOption::new(2, "Beta").disabled(true),
            ],
            Some(1),
        )
        .id(id)
        .on_select(Message::Selected)
        .on_open(Message::Opened)
        .on_close(Message::Closed)
        .into();
        harness.replace(reordered);

        let result = harness
            .update_overlay(key_pressed(Named::Enter, key::Code::Enter))
            .expect("reconciled Select overlay");
        assert_eq!(result.messages, vec![Message::Selected(3), Message::Closed]);
    }

    #[test]
    fn closed_wheel_and_command_wheel_never_mutate_or_open_selection() {
        let id = Id::unique();
        let mut harness = WidgetHarness::new(
            interactive_select(Some(1), id.clone()),
            Size::new(240.0, 160.0),
        );
        harness.focus(id);

        assert!(harness
            .update(Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
            }))
            .messages
            .is_empty());
        harness.update(Event::Keyboard(keyboard::Event::ModifiersChanged(
            keyboard::Modifiers::COMMAND,
        )));
        assert!(harness
            .update(Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Pixels { x: 0.0, y: 40.0 },
            }))
            .messages
            .is_empty());
        assert!(!harness.state_at::<SelectState>(&[0]).open);
    }

    #[test]
    fn wide_narrow_wide_relayout_keeps_open_geometry_finite_and_safe() {
        let id = Id::unique();
        let select: Element<'static, Message> = Select::new(options(), Some(1))
            .id(id.clone())
            .width(Length::Fixed(300.0))
            .on_select(Message::Selected)
            .into();
        let mut harness = WidgetHarness::new(select, Size::new(400.0, 180.0));
        harness.focus(id);
        harness.update(key_pressed(Named::Enter, key::Code::Enter));

        for viewport in [
            Size::new(400.0, 180.0),
            Size::new(220.0, 100.0),
            Size::new(400.0, 180.0),
        ] {
            harness.relayout(viewport);
            let popup = harness.overlay_bounds().expect("open Select overlay");
            assert!(popup.x.is_finite() && popup.y.is_finite());
            assert!(popup.width.is_finite() && popup.height.is_finite());
            assert!(popup.x >= 8.0);
            assert!(popup.x + popup.width <= viewport.width - 8.0);
        }
    }

    fn text_key(value: &str) -> Event {
        let key = keyboard::Key::Character(value.into());
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: keyboard::key::Physical::Code(key::Code::KeyB),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::NONE,
            text: Some(value.into()),
            repeat: false,
        })
    }
}
