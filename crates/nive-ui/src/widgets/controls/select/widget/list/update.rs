use iced::{
    advanced::{mouse, widget::Tree, Clipboard, Layout, Shell},
    keyboard::{self, key::Named},
    window, Event, Rectangle,
};

use crate::widgets::controls::select::widget::helpers::{
    first_enabled, is_enabled, is_primary_press, is_release, last_enabled, move_highlight,
    option_at, pointer_position, primary_position, release_position, set_highlight,
    typeahead_match,
};
use crate::widgets::controls::select::widget::{
    SelectEvent, SelectList, SelectListState, TYPEAHEAD_TIMEOUT,
};

impl<'a, T> SelectList<'a, T>
where
    T: Clone + Eq + 'a,
{
    #[allow(clippy::too_many_arguments)]
    pub(super) fn update_impl(
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
}
