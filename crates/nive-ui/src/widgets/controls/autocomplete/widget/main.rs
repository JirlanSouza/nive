use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    keyboard::{self, key::Named},
    Event, Length, Rectangle, Size, Vector,
};

use super::state::take_selection_request;
use super::{
    AutocompleteCallbacks, AutocompleteHandles, AutocompleteLatch, AutocompleteState,
    AutocompleteWidget, Navigation, ResultsSnapshot,
};
use crate::widgets::controls::autocomplete::{
    AutocompleteHighlight, AutocompleteResults, AutocompleteSuggestion,
};
use crate::Element;

impl<'a, T, Message> AutocompleteWidget<'a, T, Message>
where
    T: Clone + Eq,
{
    pub(in crate::widgets::controls::autocomplete) fn new(
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

    pub(in crate::widgets::controls::autocomplete) fn suggestions(
        &self,
    ) -> Option<&[AutocompleteSuggestion<'a, T>]> {
        self.results.as_suggestions()
    }

    pub(in crate::widgets::controls::autocomplete) fn model_valid(&self) -> bool {
        self.results.has_unique_values()
    }

    pub(in crate::widgets::controls::autocomplete) fn initial_highlight(&self) -> Option<T> {
        if self.policy != AutocompleteHighlight::First || !self.model_valid() {
            return None;
        }
        self.suggestions()?
            .iter()
            .find(|suggestion| !suggestion.is_disabled())
            .map(|suggestion| suggestion.value().clone())
    }

    pub(in crate::widgets::controls::autocomplete) fn highlighted_index(
        &self,
        value: Option<&T>,
    ) -> Option<usize> {
        if !self.model_valid() {
            return None;
        }
        let value = value?;
        self.suggestions()?
            .iter()
            .position(|suggestion| !suggestion.is_disabled() && suggestion.value() == value)
    }

    pub(super) fn sync_state(&self, state: &mut AutocompleteState<T>, dismissal_requested: bool) {
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

    pub(in crate::widgets::controls::autocomplete) fn sync_tree(&self, tree: &mut Tree)
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

    pub(super) fn latch(
        state: &mut AutocompleteState<T>,
        query: &str,
        results: &AutocompleteResults<'_, T>,
    ) {
        state.latch = Some(AutocompleteLatch {
            query: query.to_owned(),
            results: ResultsSnapshot::from_results(results),
            focus_generation: state.focus_generation,
        });
    }

    pub(super) fn navigate(&self, state: &mut AutocompleteState<T>, direction: Navigation) -> bool {
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

    pub(super) fn navigation(event: &Event) -> Option<Navigation> {
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

    pub(in crate::widgets::controls::autocomplete) fn is_named_key(
        event: &Event,
        named: Named,
    ) -> bool {
        matches!(
            event,
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key),
                ..
            }) if *key == named
        )
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
