use std::{cell::Cell, rc::Rc};

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    keyboard::{self, key},
    Event, Length, Rectangle, Size, Vector,
};

use super::{
    input::Input,
    popover::{
        translated_bounds, PopoverCollision, PopoverOverlay, PopoverPlacement, PopoverWidth,
    },
    shell_relay,
};
use crate::Element;

type ContentBuilder<'a, Message> = Box<dyn Fn(Option<usize>) -> Element<'a, Message> + 'a>;

pub struct Autocomplete<'a, Message> {
    anchor: Element<'a, AutocompleteMessage<Message>>,
    content: Element<'a, AutocompleteMessage<Message>>,
    content_builder: Option<ContentBuilder<'a, Message>>,
    item_count: usize,
    input_value: String,
    input_focused: Rc<Cell<bool>>,
    open: bool,
    placement: PopoverPlacement,
    width: PopoverWidth,
    collision: PopoverCollision,
    gap: f32,
    on_select: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    on_dismiss: Option<Message>,
}

#[derive(Clone)]
pub enum AutocompleteMessage<Message> {
    Parent(Message),
    Dismiss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AutocompleteNavigation {
    Previous,
    Next,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AutocompleteKeyAction {
    Navigate(AutocompleteNavigation),
    SelectHighlighted,
    Dismiss,
}

#[derive(Debug, Default)]
struct AutocompleteState {
    highlighted: Option<usize>,
    dismissed: bool,
    input_focused: bool,
    open: bool,
    item_count: usize,
    input_value: String,
}

impl<'a, Message> Autocomplete<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(input: Input<'a, Message>) -> Self {
        Self::with_anchor(input, Into::into)
    }

    pub fn with_anchor(
        input: Input<'a, Message>,
        anchor: impl FnOnce(
            Input<'a, AutocompleteMessage<Message>>,
        ) -> Element<'a, AutocompleteMessage<Message>>,
    ) -> Self {
        let input_value = input.value().to_owned();
        let input_focused = Rc::new(Cell::new(false));
        let input = input
            .map(AutocompleteMessage::Parent)
            .track_focus(Rc::clone(&input_focused))
            .on_blur(AutocompleteMessage::Dismiss);

        Self {
            anchor: anchor(input),
            content: Element::from(iced::widget::Space::new()).map(AutocompleteMessage::Parent),
            content_builder: None,
            item_count: 0,
            input_value,
            input_focused,
            open: false,
            placement: PopoverPlacement::default(),
            width: PopoverWidth::default(),
            collision: PopoverCollision::default(),
            gap: 0.0,
            on_select: None,
            on_dismiss: None,
        }
    }
}

impl<'a, Message> Autocomplete<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn content(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.content = content.into().map(AutocompleteMessage::Parent);
        self.content_builder = None;
        self
    }

    pub fn content_with(mut self, f: impl Fn(Option<usize>) -> Element<'a, Message> + 'a) -> Self {
        self.content_builder = Some(Box::new(f));
        self.refresh_content(initial_highlight(self.open, self.item_count));
        self
    }

    pub fn item_count(mut self, item_count: usize) -> Self {
        self.item_count = item_count;
        self.refresh_content(initial_highlight(self.open, self.item_count));
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn placement(mut self, placement: PopoverPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn width(mut self, width: PopoverWidth) -> Self {
        self.width = width;
        self
    }

    pub fn collision(mut self, collision: PopoverCollision) -> Self {
        self.collision = collision;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn on_select(mut self, f: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(f));
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

    fn sync_state(&self, state: &mut AutocompleteState) {
        if !self.open || self.item_count == 0 {
            state.highlighted = None;
            state.dismissed = false;
        } else if state.input_value != self.input_value {
            state.highlighted = Some(0);
            state.dismissed = false;
        } else if !state.open || state.item_count != self.item_count {
            state.highlighted = Some(0);
        } else if state
            .highlighted
            .is_some_and(|index| index >= self.item_count)
        {
            state.highlighted = Some(self.item_count - 1);
        }

        state.open = self.open;
        state.item_count = self.item_count;
        state.input_value = self.input_value.clone();
    }

    fn refresh_content(&mut self, highlighted: Option<usize>) {
        if let Some(builder) = self.content_builder.as_ref() {
            self.content = builder(highlighted).map(AutocompleteMessage::Parent);
        }
    }

    fn is_open(&self, state: &AutocompleteState) -> bool {
        self.open && self.item_count > 0 && !state.dismissed
    }

    fn refresh_content_tree(&mut self, tree: &mut Tree, highlighted: Option<usize>) {
        self.refresh_content(highlighted);
        tree.diff(self.content.as_widget());
    }

    fn handle_keyboard(
        &mut self,
        state: &mut AutocompleteState,
        event: &Event,
        shell: &mut Shell<'_, Message>,
    ) -> bool {
        let Some(action) = key_action(event) else {
            return false;
        };

        match action {
            AutocompleteKeyAction::Navigate(direction) => {
                state.highlighted =
                    navigate_highlight(state.highlighted, self.item_count, direction);
                shell.capture_event();
                shell.invalidate_layout();
                shell.request_redraw();

                true
            }
            AutocompleteKeyAction::SelectHighlighted => {
                let message = state
                    .highlighted
                    .and_then(|index| self.on_select.as_ref().map(|on_select| on_select(index)));

                self.publish_optional(message, shell)
            }
            AutocompleteKeyAction::Dismiss => self.dismiss(state, shell),
        }
    }

    fn handle_message(
        &self,
        state: &mut AutocompleteState,
        message: AutocompleteMessage<Message>,
        shell: &mut Shell<'_, Message>,
    ) {
        match message {
            AutocompleteMessage::Parent(message) => shell.publish(message),
            AutocompleteMessage::Dismiss => {
                self.dismiss(state, shell);
            }
        }
    }

    fn dismiss(&self, state: &mut AutocompleteState, shell: &mut Shell<'_, Message>) -> bool {
        if !self.is_open(state) {
            return false;
        }

        state.dismissed = true;
        state.highlighted = None;
        self.publish_optional(self.on_dismiss.clone(), shell);
        shell.capture_event();
        shell.invalidate_layout();
        shell.request_redraw();

        true
    }

    fn dismiss_without_shell(&self, state: &mut AutocompleteState) {
        if !self.is_open(state) {
            return;
        }

        state.dismissed = true;
        state.highlighted = None;
    }

    fn sync_input_focus(&self, state: &mut AutocompleteState) {
        let was_focused = state.input_focused;
        let is_focused = self.input_focused.get();

        if was_focused && !is_focused {
            self.dismiss_without_shell(state);
        }

        state.input_focused = is_focused;
    }

    fn publish_optional(&self, message: Option<Message>, shell: &mut Shell<'_, Message>) -> bool {
        let Some(message) = message else {
            return false;
        };

        shell.publish(message);
        shell.capture_event();
        shell.invalidate_layout();
        shell.request_redraw();

        true
    }
}

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer> for Autocomplete<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<AutocompleteState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(AutocompleteState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.anchor), Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        if tree.children.is_empty() {
            tree.children.push(Tree::new(&self.anchor));
        } else {
            tree.children[0].diff(self.anchor.as_widget());
        }

        if tree.children.len() < 2 {
            tree.children.push(Tree::new(&self.content));
        } else {
            tree.children[1].diff(self.content.as_widget());
            tree.children.truncate(2);
        }
    }

    fn size(&self) -> Size<Length> {
        self.anchor.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.anchor.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<AutocompleteState>();
        self.sync_state(state);
        self.refresh_content_tree(&mut tree.children[1], state.highlighted);

        self.anchor
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
        let Tree {
            state, children, ..
        } = tree;
        let state = state.downcast_mut::<AutocompleteState>();
        self.sync_state(state);

        self.anchor
            .as_widget_mut()
            .operate(&mut children[0], layout, renderer, operation);
        self.sync_input_focus(state);
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
        let Tree {
            state, children, ..
        } = tree;
        let state = state.downcast_mut::<AutocompleteState>();
        self.sync_state(state);
        self.refresh_content_tree(&mut children[1], state.highlighted);

        if self.is_open(state) && state.input_focused && self.handle_keyboard(state, event, shell) {
            self.refresh_content_tree(&mut children[1], state.highlighted);
            return;
        }

        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        self.anchor.as_widget_mut().update(
            &mut children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            &mut local_shell,
            viewport,
        );

        shell_relay::propagate_to_parent(&mut local_shell, shell);
        drop(local_shell);

        for message in local_messages {
            self.handle_message(state, message, shell);
        }

        self.sync_input_focus(state);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.anchor.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
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
        self.anchor.as_widget().draw(
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
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        let Tree {
            state, children, ..
        } = tree;
        let state = state.downcast_mut::<AutocompleteState>();
        self.sync_state(state);
        self.refresh_content_tree(&mut children[1], state.highlighted);

        let (anchor_tree, content_tree) = children.split_at_mut(1);
        let anchor_state = &mut anchor_tree[0];
        let open = self.is_open(state);
        let on_dismiss = self.on_dismiss.clone();

        let popover_overlay = match (open, content_tree.get_mut(0)) {
            (true, Some(content_state)) => {
                Some(overlay::Element::new(Box::new(PopoverOverlay::new(
                    translated_bounds(layout.bounds(), translation),
                    &mut self.content,
                    content_state,
                    self.placement,
                    self.width,
                    self.collision,
                    self.gap,
                    Some(AutocompleteMessage::Dismiss),
                    move |message, shell: &mut Shell<'_, Message>| match message {
                        AutocompleteMessage::Parent(message) => shell.publish(message),
                        AutocompleteMessage::Dismiss => {
                            state.dismissed = true;
                            state.highlighted = None;
                            if let Some(message) = on_dismiss.clone() {
                                shell.publish(message);
                            }
                            shell.capture_event();
                            shell.invalidate_layout();
                            shell.request_redraw();
                        }
                    },
                ))))
            }
            _ => None,
        };

        let anchor_overlay = self
            .anchor
            .as_widget_mut()
            .overlay(anchor_state, layout, renderer, viewport, translation)
            .map(|overlay| overlay.map(&parent_message::<Message>));

        match (anchor_overlay, popover_overlay) {
            (Some(anchor), Some(popover)) => {
                Some(overlay::Group::with_children(vec![anchor, popover]).overlay())
            }
            (Some(anchor), None) => Some(anchor),
            (None, Some(popover)) => Some(popover),
            (None, None) => None,
        }
    }
}

impl<'a, Message> From<Autocomplete<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(autocomplete: Autocomplete<'a, Message>) -> Self {
        Element::new(autocomplete)
    }
}

fn key_action(event: &Event) -> Option<AutocompleteKeyAction> {
    match event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key::Named::ArrowUp),
            ..
        }) => Some(AutocompleteKeyAction::Navigate(
            AutocompleteNavigation::Previous,
        )),
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key::Named::ArrowDown),
            ..
        }) => Some(AutocompleteKeyAction::Navigate(
            AutocompleteNavigation::Next,
        )),
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key::Named::Enter),
            repeat: false,
            ..
        }) => Some(AutocompleteKeyAction::SelectHighlighted),
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key::Named::Escape),
            repeat: false,
            ..
        }) => Some(AutocompleteKeyAction::Dismiss),
        _ => None,
    }
}

fn initial_highlight(open: bool, item_count: usize) -> Option<usize> {
    (open && item_count > 0).then_some(0)
}

fn navigate_highlight(
    current: Option<usize>,
    item_count: usize,
    direction: AutocompleteNavigation,
) -> Option<usize> {
    if item_count == 0 {
        return None;
    }

    match (direction, current.filter(|index| *index < item_count)) {
        (AutocompleteNavigation::Next, Some(index)) => Some((index + 1) % item_count),
        (AutocompleteNavigation::Next, None) => Some(0),
        (AutocompleteNavigation::Previous, Some(0)) => Some(item_count - 1),
        (AutocompleteNavigation::Previous, Some(index)) => Some(index - 1),
        (AutocompleteNavigation::Previous, None) => Some(item_count - 1),
    }
}

fn parent_message<Message>(message: AutocompleteMessage<Message>) -> Message {
    match message {
        AutocompleteMessage::Parent(message) => message,
        AutocompleteMessage::Dismiss => {
            unreachable!("autocomplete dismiss is handled before parent message mapping")
        }
    }
}

#[cfg(test)]
mod autocomplete_tests {
    use iced::keyboard::{
        key::{Code, Named, Physical},
        Key, Location, Modifiers,
    };

    use crate::widgets::input;

    use super::*;

    #[derive(Debug, Clone)]
    enum TestMessage {
        Input,
    }

    fn autocomplete(value: &str) -> Autocomplete<'_, TestMessage> {
        Autocomplete::new(input::default("Search", value).on_input(|_| TestMessage::Input))
            .open(true)
            .item_count(2)
    }

    fn key_pressed(key: Key, repeat: bool) -> Event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: Physical::Code(Code::Enter),
            location: Location::Standard,
            modifiers: Modifiers::NONE,
            text: None,
            repeat,
        })
    }

    #[test]
    fn arrow_up_maps_to_previous_navigation() {
        assert_eq!(
            key_action(&key_pressed(Key::Named(Named::ArrowUp), false)),
            Some(AutocompleteKeyAction::Navigate(
                AutocompleteNavigation::Previous
            ))
        );
    }

    #[test]
    fn arrow_down_maps_to_next_navigation() {
        assert_eq!(
            key_action(&key_pressed(Key::Named(Named::ArrowDown), false)),
            Some(AutocompleteKeyAction::Navigate(
                AutocompleteNavigation::Next
            ))
        );
    }

    #[test]
    fn enter_maps_to_select_highlighted() {
        assert_eq!(
            key_action(&key_pressed(Key::Named(Named::Enter), false)),
            Some(AutocompleteKeyAction::SelectHighlighted)
        );
    }

    #[test]
    fn repeated_enter_does_not_select() {
        assert_eq!(
            key_action(&key_pressed(Key::Named(Named::Enter), true)),
            None
        );
    }

    #[test]
    fn escape_maps_to_dismiss() {
        assert_eq!(
            key_action(&key_pressed(Key::Named(Named::Escape), false)),
            Some(AutocompleteKeyAction::Dismiss)
        );
    }

    #[test]
    fn next_navigation_moves_highlight_forward() {
        assert_eq!(
            navigate_highlight(Some(0), 3, AutocompleteNavigation::Next),
            Some(1)
        );
    }

    #[test]
    fn previous_navigation_wraps_highlight() {
        assert_eq!(
            navigate_highlight(Some(0), 3, AutocompleteNavigation::Previous),
            Some(2)
        );
    }

    #[test]
    fn navigation_without_items_has_no_highlight() {
        assert_eq!(
            navigate_highlight(Some(0), 0, AutocompleteNavigation::Next),
            None
        );
    }

    #[test]
    fn input_value_change_reopens_dismissed_panel() {
        let autocomplete = autocomplete("rust");
        let mut state = AutocompleteState {
            highlighted: None,
            dismissed: true,
            input_focused: false,
            open: true,
            item_count: 2,
            input_value: "ru".into(),
        };

        autocomplete.sync_state(&mut state);

        assert!(!state.dismissed);
        assert_eq!(state.highlighted, Some(0));
    }

    #[test]
    fn empty_items_clear_highlight_and_dismissed_state() {
        let autocomplete = autocomplete("rust").item_count(0);
        let mut state = AutocompleteState {
            highlighted: Some(1),
            dismissed: true,
            input_focused: false,
            open: true,
            item_count: 2,
            input_value: "rust".into(),
        };

        autocomplete.sync_state(&mut state);

        assert!(!state.dismissed);
        assert_eq!(state.highlighted, None);
    }

    #[test]
    fn focus_loss_marks_panel_as_dismissed() {
        let autocomplete = autocomplete("rust");
        let mut state = AutocompleteState {
            highlighted: Some(0),
            dismissed: false,
            input_focused: true,
            open: true,
            item_count: 2,
            input_value: "rust".into(),
        };

        autocomplete.dismiss_without_shell(&mut state);

        assert!(state.dismissed);
        assert_eq!(state.highlighted, None);
    }
}
