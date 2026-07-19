use std::{cell::Cell, rc::Rc};

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    keyboard::{self, key::Named},
    Event, Length, Rectangle, Size, Vector,
};

use super::{AutocompleteHighlight, AutocompleteResults};
use crate::{
    widgets::{
        navigation::menu::{MENU_LIST_INSET, MENU_ROW_HEIGHT},
        overlays::anchored_overlay::scroll::EnsureVisibleHandle,
    },
    Element,
};

#[derive(Clone)]
pub(super) struct AutocompleteHandles {
    input_focused: Rc<Cell<bool>>,
    highlighted_index: Rc<Cell<Option<usize>>>,
    ensure_pending: Rc<Cell<bool>>,
    local_closed: Rc<Cell<bool>>,
}

pub(super) struct AutocompleteCallbacks<'a, T, Message> {
    on_select: Option<Rc<dyn Fn(T) -> Message + 'a>>,
    on_dismiss: Option<Message>,
}

impl<'a, T, Message> AutocompleteCallbacks<'a, T, Message> {
    pub(super) fn new(
        on_select: Option<Rc<dyn Fn(T) -> Message + 'a>>,
        on_dismiss: Option<Message>,
    ) -> Self {
        Self {
            on_select,
            on_dismiss,
        }
    }
}

impl AutocompleteHandles {
    pub(super) fn new() -> Self {
        Self {
            input_focused: Rc::new(Cell::new(false)),
            highlighted_index: Rc::new(Cell::new(None)),
            ensure_pending: Rc::new(Cell::new(true)),
            local_closed: Rc::new(Cell::new(false)),
        }
    }

    pub(super) fn input_focused(&self) -> Rc<Cell<bool>> {
        Rc::clone(&self.input_focused)
    }

    pub(super) fn highlighted_index(&self) -> Rc<Cell<Option<usize>>> {
        Rc::clone(&self.highlighted_index)
    }

    pub(super) fn ensure_pending(&self) -> Rc<Cell<bool>> {
        Rc::clone(&self.ensure_pending)
    }

    pub(super) fn local_closed(&self) -> Rc<Cell<bool>> {
        Rc::clone(&self.local_closed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResultsSnapshot<T> {
    Suggestions(Vec<SuggestionSnapshot<T>>),
    Loading,
    Empty(String),
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SuggestionSnapshot<T> {
    value: T,
    label: String,
    leading: Option<crate::IconRole>,
    trailing: Option<String>,
    disabled: bool,
}

impl<T> ResultsSnapshot<T>
where
    T: Clone + Eq,
{
    fn from_results(results: &AutocompleteResults<'_, T>) -> Self {
        match results {
            AutocompleteResults::Suggestions(suggestions) => Self::Suggestions(
                suggestions
                    .iter()
                    .map(|suggestion| SuggestionSnapshot {
                        value: suggestion.value().clone(),
                        label: suggestion.label().to_owned(),
                        leading: suggestion.leading_icon(),
                        trailing: suggestion.trailing_text().map(str::to_owned),
                        disabled: suggestion.is_disabled(),
                    })
                    .collect(),
            ),
            AutocompleteResults::Loading => Self::Loading,
            AutocompleteResults::Empty(message) => Self::Empty(message.to_string()),
            AutocompleteResults::Error(message) => Self::Error(message.to_string()),
        }
    }
}

#[derive(Debug)]
struct AutocompleteState<T> {
    highlighted: Option<T>,
    query: String,
    results: Option<ResultsSnapshot<T>>,
    was_open: bool,
    initialized: bool,
    focus_generation: u64,
    input_was_focused: bool,
    latch: Option<AutocompleteLatch<T>>,
    dismissal_message_pending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AutocompleteLatch<T> {
    query: String,
    results: ResultsSnapshot<T>,
    focus_generation: u64,
}

impl<T> Default for AutocompleteState<T> {
    fn default() -> Self {
        Self {
            highlighted: None,
            query: String::new(),
            results: None,
            was_open: false,
            initialized: false,
            focus_generation: 0,
            input_was_focused: false,
            latch: None,
            dismissal_message_pending: false,
        }
    }
}

pub(super) struct AutocompleteWidget<'a, T, Message>
where
    T: Clone + Eq,
{
    content: Element<'a, Message>,
    query: String,
    results: AutocompleteResults<'a, T>,
    open: bool,
    policy: AutocompleteHighlight,
    handles: AutocompleteHandles,
    callbacks: AutocompleteCallbacks<'a, T, Message>,
}

impl<'a, T, Message> AutocompleteWidget<'a, T, Message>
where
    T: Clone + Eq,
{
    pub(super) fn new(
        content: Element<'a, Message>,
        query: String,
        results: AutocompleteResults<'a, T>,
        open: bool,
        policy: AutocompleteHighlight,
        handles: AutocompleteHandles,
        callbacks: AutocompleteCallbacks<'a, T, Message>,
    ) -> Self {
        Self {
            content,
            query,
            results,
            open,
            policy,
            handles,
            callbacks,
        }
    }

    fn suggestions(&self) -> Option<&[super::AutocompleteSuggestion<'a, T>]> {
        self.results.as_suggestions()
    }

    fn model_valid(&self) -> bool {
        self.results.has_unique_values()
    }

    fn initial_highlight(&self) -> Option<T> {
        if self.policy != AutocompleteHighlight::First || !self.model_valid() {
            return None;
        }
        self.suggestions()?
            .iter()
            .find(|suggestion| !suggestion.is_disabled())
            .map(|suggestion| suggestion.value().clone())
    }

    fn highlighted_index(&self, value: Option<&T>) -> Option<usize> {
        if !self.model_valid() {
            return None;
        }
        let value = value?;
        self.suggestions()?
            .iter()
            .position(|suggestion| !suggestion.is_disabled() && suggestion.value() == value)
    }

    fn sync_state(&self, state: &mut AutocompleteState<T>, dismissal_requested: bool) {
        let snapshot = ResultsSnapshot::from_results(&self.results);
        let query_changed = state.initialized && state.query != self.query;
        let results_changed = state
            .results
            .as_ref()
            .is_some_and(|previous| previous != &snapshot);
        let opened = self.open && (!state.was_open || !state.initialized);
        let input_focused = self.handles.input_focused.get();
        let focus_entered = input_focused && !state.input_was_focused;

        if focus_entered {
            state.focus_generation = state.focus_generation.wrapping_add(1);
            state.input_was_focused = true;
        }
        if !self.open || opened || query_changed || results_changed || focus_entered {
            state.latch = None;
        }
        if dismissal_requested && self.open {
            state.latch = Some(AutocompleteLatch {
                query: self.query.clone(),
                results: snapshot.clone(),
                focus_generation: state.focus_generation,
            });
        }
        if self.callbacks.on_dismiss.is_none() {
            state.dismissal_message_pending = false;
        }
        let effectively_open = self.open && state.latch.is_none();

        if !effectively_open || self.suggestions().is_none() || !self.model_valid() {
            state.highlighted = None;
        } else if query_changed || opened {
            state.highlighted = self.initial_highlight();
        } else if results_changed {
            let retained = state
                .highlighted
                .as_ref()
                .filter(|value| self.highlighted_index(Some(value)).is_some())
                .cloned();
            state.highlighted = retained.or_else(|| self.initial_highlight());
        } else if self.highlighted_index(state.highlighted.as_ref()).is_none() {
            state.highlighted = self.initial_highlight();
        }

        state.query.clone_from(&self.query);
        state.results = Some(snapshot);
        state.was_open = self.open;
        state.initialized = true;
        self.handles.local_closed.set(state.latch.is_some());
        let index = self.highlighted_index(state.highlighted.as_ref());
        if self.handles.highlighted_index.replace(index) != index && index.is_some() {
            self.handles.ensure_pending.set(true);
        }
    }

    fn sync_tree(&self, tree: &mut Tree)
    where
        T: 'static,
    {
        let selection_requested = take_selection_request(&mut tree.children[0]);
        let dismissal_requested =
            crate::widgets::overlays::popover::take_dismissal_request(&mut tree.children[0]);
        self.sync_state(
            tree.state.downcast_mut::<AutocompleteState<T>>(),
            selection_requested || dismissal_requested,
        );
    }

    fn latch(state: &mut AutocompleteState<T>, query: &str, results: &AutocompleteResults<'_, T>) {
        state.latch = Some(AutocompleteLatch {
            query: query.to_owned(),
            results: ResultsSnapshot::from_results(results),
            focus_generation: state.focus_generation,
        });
    }

    fn navigate(&self, state: &mut AutocompleteState<T>, direction: Navigation) -> bool {
        if !self.open || !self.model_valid() {
            return false;
        }
        let Some(suggestions) = self.suggestions() else {
            return false;
        };
        let current = self.highlighted_index(state.highlighted.as_ref());
        let next = match direction {
            Navigation::Next => suggestions
                .iter()
                .enumerate()
                .find(|(index, suggestion)| {
                    current.is_none_or(|current| *index > current) && !suggestion.is_disabled()
                })
                .map(|(index, _)| index)
                .or(current),
            Navigation::Previous => suggestions
                .iter()
                .enumerate()
                .rev()
                .find(|(index, suggestion)| {
                    current.is_none_or(|current| *index < current) && !suggestion.is_disabled()
                })
                .map(|(index, _)| index)
                .or(current),
        };
        state.highlighted = next.map(|index| suggestions[index].value().clone());
        if self.handles.highlighted_index.replace(next) != next && next.is_some() {
            self.handles.ensure_pending.set(true);
        }
        true
    }

    fn navigation(event: &Event) -> Option<Navigation> {
        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::ArrowDown),
                ..
            }) => Some(Navigation::Next),
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::ArrowUp),
                ..
            }) => Some(Navigation::Previous),
            _ => None,
        }
    }

    fn is_named_key(event: &Event, named: Named) -> bool {
        matches!(
            event,
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key),
                ..
            }) if *key == named
        )
    }
}

#[derive(Debug, Clone, Copy)]
enum Navigation {
    Previous,
    Next,
}

pub(super) struct HighlightVisibility<'a, T, Message> {
    content: Element<'a, Message>,
    highlighted_index: Rc<Cell<Option<usize>>>,
    ensure_pending: Rc<Cell<bool>>,
    ensure_visible: EnsureVisibleHandle,
    suggestions: Vec<(T, bool)>,
    local_closed: Rc<Cell<bool>>,
    on_select: Option<Rc<dyn Fn(T) -> Message + 'a>>,
}

#[derive(Debug, Default)]
struct HighlightVisibilityState {
    selection_requested: bool,
}

fn take_selection_request(tree: &mut Tree) -> bool {
    if tree.tag == tree::Tag::of::<HighlightVisibilityState>() {
        return std::mem::take(
            &mut tree
                .state
                .downcast_mut::<HighlightVisibilityState>()
                .selection_requested,
        );
    }
    tree.children.iter_mut().any(take_selection_request)
}

impl<'a, T, Message> HighlightVisibility<'a, T, Message>
where
    T: Clone,
{
    pub(super) fn new(
        content: Element<'a, Message>,
        highlighted_index: Rc<Cell<Option<usize>>>,
        ensure_pending: Rc<Cell<bool>>,
        ensure_visible: EnsureVisibleHandle,
        suggestions: Vec<(T, bool)>,
        local_closed: Rc<Cell<bool>>,
        on_select: Option<Rc<dyn Fn(T) -> Message + 'a>>,
    ) -> Self {
        Self {
            content,
            highlighted_index,
            ensure_pending,
            ensure_visible,
            suggestions,
            local_closed,
            on_select,
        }
    }

    fn request_highlight_visible(&self, layout: Layout<'_>) {
        if !self.ensure_pending.replace(false) {
            return;
        }
        let Some(index) = self
            .highlighted_index
            .get()
            .filter(|index| *index < self.suggestions.len())
        else {
            return;
        };
        let bounds = layout.bounds();
        self.ensure_visible.request(Rectangle {
            x: bounds.x + MENU_LIST_INSET,
            y: bounds.y + MENU_LIST_INSET + index as f32 * MENU_ROW_HEIGHT,
            width: (bounds.width - MENU_LIST_INSET * 2.0).max(0.0),
            height: MENU_ROW_HEIGHT,
        });
    }

    fn pressed_suggestion(
        &self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) -> Option<T> {
        if self.local_closed.get() || self.on_select.is_none() {
            return None;
        }
        let point = match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => cursor.position(),
            Event::Touch(iced::touch::Event::FingerPressed { position, .. }) => Some(*position),
            _ => None,
        }?;
        let bounds = layout.bounds();
        if point.x < bounds.x + MENU_LIST_INSET
            || point.x > bounds.x + bounds.width - MENU_LIST_INSET
            || point.y < bounds.y + MENU_LIST_INSET
        {
            return None;
        }
        let index = ((point.y - bounds.y - MENU_LIST_INSET) / MENU_ROW_HEIGHT).floor() as usize;
        let (value, disabled) = self.suggestions.get(index)?;
        let row_bounds = Rectangle {
            x: bounds.x + MENU_LIST_INSET,
            y: bounds.y + MENU_LIST_INSET + index as f32 * MENU_ROW_HEIGHT,
            width: (bounds.width - MENU_LIST_INSET * 2.0).max(0.0),
            height: MENU_ROW_HEIGHT,
        };
        (!disabled && row_bounds.contains(point)).then(|| value.clone())
    }
}

impl<'a, T, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for HighlightVisibility<'a, T, Message>
where
    T: Clone + 'a,
    Message: 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<HighlightVisibilityState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(HighlightVisibilityState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
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
        shell: &mut Shell<'_, Message>,
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
        self.request_highlight_visible(layout);
        if let (Some(value), Some(on_select)) = (
            self.pressed_suggestion(event, layout, cursor),
            &self.on_select,
        ) {
            tree.state
                .downcast_mut::<HighlightVisibilityState>()
                .selection_requested = true;
            self.local_closed.set(true);
            shell.publish(on_select(value));
            shell.capture_event();
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
        let child = self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        );
        if self.on_select.is_some()
            && !self.local_closed.get()
            && cursor.position().is_some_and(|point| {
                let bounds = layout.bounds();
                let index = ((point.y - bounds.y - MENU_LIST_INSET) / MENU_ROW_HEIGHT).floor();
                index >= 0.0
                    && self
                        .suggestions
                        .get(index as usize)
                        .is_some_and(|(_, disabled)| !disabled)
                    && point.x >= bounds.x + MENU_LIST_INSET
                    && point.x <= bounds.x + bounds.width - MENU_LIST_INSET
            })
        {
            mouse::Interaction::Pointer
        } else {
            child
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
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, T, Message> From<HighlightVisibility<'a, T, Message>> for Element<'a, Message>
where
    T: Clone + 'a,
    Message: 'a,
{
    fn from(visibility: HighlightVisibility<'a, T, Message>) -> Self {
        Element::new(visibility)
    }
}

impl<'a, T, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for AutocompleteWidget<'a, T, Message>
where
    T: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<AutocompleteState<T>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(AutocompleteState::<T>::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
        self.sync_tree(tree);
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
        self.sync_tree(tree);
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
        let input_focused = self.handles.input_focused.get();
        let state = tree.state.downcast_mut::<AutocompleteState<T>>();
        if input_focused && !state.input_was_focused {
            state.focus_generation = state.focus_generation.wrapping_add(1);
            state.input_was_focused = true;
            state.latch = None;
            self.handles.local_closed.set(false);
        } else if !input_focused && state.input_was_focused {
            state.input_was_focused = false;
            if self.open && state.latch.is_none() && self.callbacks.on_dismiss.is_some() {
                Self::latch(state, &self.query, &self.results);
                state.dismissal_message_pending = true;
                self.handles.local_closed.set(true);
            }
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
        self.sync_tree(tree);
        let state = tree.state.downcast_mut::<AutocompleteState<T>>();
        if std::mem::take(&mut state.dismissal_message_pending) {
            if let Some(on_dismiss) = &self.callbacks.on_dismiss {
                shell.publish(on_dismiss.clone());
                shell.request_redraw();
            }
        }
        let was_input_focused = self.handles.input_focused.get();
        if !self.handles.local_closed.get()
            && was_input_focused
            && Self::navigation(event).is_some_and(|direction| self.navigate(state, direction))
        {
            shell.capture_event();
            shell.request_redraw();
            return;
        }
        if self.open && !self.handles.local_closed.get() && was_input_focused {
            if Self::is_named_key(event, Named::Enter) {
                if let (Some(on_select), Some(value)) =
                    (&self.callbacks.on_select, state.highlighted.clone())
                {
                    Self::latch(state, &self.query, &self.results);
                    self.handles.local_closed.set(true);
                    shell.publish(on_select(value));
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
            } else if Self::is_named_key(event, Named::Tab) {
                if let Some(on_dismiss) = &self.callbacks.on_dismiss {
                    Self::latch(state, &self.query, &self.results);
                    self.handles.local_closed.set(true);
                    shell.publish(on_dismiss.clone());
                    shell.request_redraw();
                }
            }
        }
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
        let input_focused = self.handles.input_focused.get();
        if was_input_focused && !input_focused {
            state.input_was_focused = false;
            if self.open && state.latch.is_none() {
                if let Some(on_dismiss) = &self.callbacks.on_dismiss {
                    Self::latch(state, &self.query, &self.results);
                    self.handles.local_closed.set(true);
                    shell.publish(on_dismiss.clone());
                    shell.request_redraw();
                }
            }
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
        self.content.as_widget().mouse_interaction(
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
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        if self.handles.local_closed.get() {
            return None;
        }
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, T, Message> From<AutocompleteWidget<'a, T, Message>> for Element<'a, Message>
where
    T: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    fn from(widget: AutocompleteWidget<'a, T, Message>) -> Self {
        Element::new(widget)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::controls::{AutocompleteResults, AutocompleteSuggestion};

    fn results(order: &[u8]) -> AutocompleteResults<'static, u8> {
        AutocompleteResults::suggestions(
            order
                .iter()
                .map(|value| AutocompleteSuggestion::new(*value, format!("Value {value}")))
                .collect::<Vec<_>>(),
        )
    }

    fn widget(results: AutocompleteResults<'static, u8>) -> AutocompleteWidget<'static, u8, ()> {
        widget_with_policy(results, AutocompleteHighlight::None)
    }

    fn widget_with_policy(
        results: AutocompleteResults<'static, u8>,
        policy: AutocompleteHighlight,
    ) -> AutocompleteWidget<'static, u8, ()> {
        AutocompleteWidget::new(
            iced::widget::Space::new().into(),
            "v".into(),
            results,
            true,
            policy,
            AutocompleteHandles::new(),
            AutocompleteCallbacks::new(None, None),
        )
    }

    #[test]
    fn default_policy_starts_without_a_logical_highlight() {
        let autocomplete = widget(results(&[1, 2, 3]));
        let mut state = AutocompleteState::default();

        autocomplete.sync_state(&mut state, false);

        assert_eq!(state.highlighted, None);
        assert_eq!(autocomplete.handles.highlighted_index.get(), None);
    }

    #[test]
    fn first_policy_skips_disabled_and_falls_back_after_highlight_removal() {
        let initial = AutocompleteResults::suggestions(vec![
            AutocompleteSuggestion::new(1_u8, "One").disabled(true),
            AutocompleteSuggestion::new(2_u8, "Two"),
            AutocompleteSuggestion::new(3_u8, "Three"),
        ]);
        let autocomplete = widget_with_policy(initial, AutocompleteHighlight::First);
        let mut state = AutocompleteState::default();
        autocomplete.sync_state(&mut state, false);
        assert_eq!(state.highlighted, Some(2));

        assert!(autocomplete.navigate(&mut state, Navigation::Next));
        assert_eq!(state.highlighted, Some(3));

        let changed = widget_with_policy(results(&[1, 2]), AutocompleteHighlight::First);
        changed.sync_state(&mut state, false);

        assert_eq!(state.highlighted, Some(1));
        assert_eq!(changed.handles.highlighted_index.get(), Some(0));
    }

    #[test]
    fn result_reordering_preserves_highlight_by_typed_value() {
        let mut state = AutocompleteState::default();
        let first = widget(results(&[1, 2, 3]));
        first.sync_state(&mut state, false);
        assert!(first.navigate(&mut state, Navigation::Next));
        assert!(first.navigate(&mut state, Navigation::Next));
        assert_eq!(state.highlighted, Some(2));

        let reordered = widget(results(&[3, 2, 1]));
        reordered.sync_state(&mut state, false);

        assert_eq!(state.highlighted, Some(2));
        assert_eq!(reordered.handles.highlighted_index.get(), Some(1));
    }

    #[test]
    fn navigation_is_bounded_and_skips_disabled_values() {
        let results = AutocompleteResults::suggestions(vec![
            AutocompleteSuggestion::new(1_u8, "One").disabled(true),
            AutocompleteSuggestion::new(2_u8, "Two"),
            AutocompleteSuggestion::new(3_u8, "Three"),
        ]);
        let autocomplete = widget(results);
        let mut state = AutocompleteState::default();
        autocomplete.sync_state(&mut state, false);

        autocomplete.navigate(&mut state, Navigation::Next);
        assert_eq!(state.highlighted, Some(2));
        autocomplete.navigate(&mut state, Navigation::Previous);
        assert_eq!(state.highlighted, Some(2));
        autocomplete.navigate(&mut state, Navigation::Next);
        autocomplete.navigate(&mut state, Navigation::Next);
        assert_eq!(state.highlighted, Some(3));

        let mut reverse_state = AutocompleteState::default();
        autocomplete.sync_state(&mut reverse_state, false);
        autocomplete.navigate(&mut reverse_state, Navigation::Previous);
        assert_eq!(reverse_state.highlighted, Some(3));
        autocomplete.navigate(&mut reverse_state, Navigation::Previous);
        assert_eq!(reverse_state.highlighted, Some(2));
        autocomplete.navigate(&mut reverse_state, Navigation::Previous);
        assert_eq!(reverse_state.highlighted, Some(2));
    }

    #[test]
    fn dismissal_latch_survives_equal_rebuilds_and_resets_on_session_changes() {
        let mut state = AutocompleteState::default();
        let initial = widget(results(&[1, 2, 3]));
        initial.sync_state(&mut state, false);
        initial.sync_state(&mut state, true);
        assert!(state.latch.is_some());
        assert!(initial.handles.local_closed.get());

        let equal = widget(results(&[1, 2, 3]));
        equal.sync_state(&mut state, false);
        assert!(state.latch.is_some());
        assert!(equal.handles.local_closed.get());

        let changed_query: AutocompleteWidget<'static, u8, ()> = AutocompleteWidget::new(
            iced::widget::Space::new().into(),
            "changed".into(),
            results(&[1, 2, 3]),
            true,
            AutocompleteHighlight::None,
            AutocompleteHandles::new(),
            AutocompleteCallbacks::new(None, None),
        );
        changed_query.sync_state(&mut state, false);
        assert!(state.latch.is_none());
        assert!(!changed_query.handles.local_closed.get());

        changed_query.sync_state(&mut state, true);
        let refocused = widget(results(&[1, 2, 3]));
        refocused.handles.input_focused.set(true);
        refocused.sync_state(&mut state, false);
        assert!(state.latch.is_none());
        assert_eq!(state.focus_generation, 1);

        refocused.sync_state(&mut state, true);
        let changed_results = widget(results(&[4, 5]));
        changed_results.handles.input_focused.set(true);
        changed_results.sync_state(&mut state, false);
        assert!(state.latch.is_none());

        changed_results.sync_state(&mut state, true);
        let closed: AutocompleteWidget<'static, u8, ()> = AutocompleteWidget::new(
            iced::widget::Space::new().into(),
            "v".into(),
            results(&[1, 2, 3]),
            false,
            AutocompleteHighlight::None,
            AutocompleteHandles::new(),
            AutocompleteCallbacks::new(None, None),
        );
        closed.sync_state(&mut state, false);
        assert!(state.latch.is_none());
    }
}
