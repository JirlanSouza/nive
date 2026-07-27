use iced::{
    advanced::{mouse, widget::Tree, Clipboard, Layout, Shell},
    keyboard::{self, key::Named},
    touch, window, Event, Rectangle,
};

use crate::widgets::navigation::menu::widget::helpers::{
    close_submenu, first_eligible, is_primary_press, last_eligible, move_highlight, open_submenu,
    pointer_highlight_position, primary_press_position, reconcile_closed_submenu, release_position,
    slot_at, sync_logical_focus, typeahead_match, update_submenu_pointer_intent,
};
use crate::widgets::navigation::menu::widget::{
    HighlightOrigin, MenuList, MenuListState, TYPEAHEAD_TIMEOUT,
};

impl<'a, Message> MenuList<'a, Message>
where
    Message: Clone + 'a,
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
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        {
            let state = tree.state.downcast_mut::<MenuListState>();
            sync_logical_focus(&self.slots, state, self.focus_visible(state));
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

        let state = tree.state.downcast_mut::<MenuListState>();
        reconcile_closed_submenu(&self.slots, state);
        self.handle_redraw(state, event, shell);
        self.handle_pointer(state, event, layout, cursor, shell);
        self.handle_keyboard(state, event, layout, shell);
    }

    fn handle_redraw(
        &self,
        state: &mut MenuListState,
        event: &Event,
        shell: &mut Shell<'_, Message>,
    ) {
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            state.now = Some(*now);
            if state
                .typeahead_deadline
                .is_some_and(|deadline| *now > deadline)
            {
                state.typeahead.clear();
                state.typeahead_deadline = None;
            }
            if state
                .submenu_intent
                .is_some_and(|(_, deadline)| *now >= deadline)
            {
                if let Some((index, _)) = state.submenu_intent.take() {
                    open_submenu(&self.slots, state, index);
                    shell.invalidate_layout();
                    shell.request_redraw();
                }
            }
            if state
                .transfer_deadline
                .is_some_and(|deadline| *now >= deadline)
            {
                let child_contains_pointer = state
                    .open_submenu
                    .and_then(|index| self.slots.get(index))
                    .and_then(|slot| slot.branch.as_ref())
                    .is_some_and(|branch| {
                        branch.pointer_inside.get()
                            || state
                                .last_pointer
                                .is_some_and(|point| branch.child_bounds.get().contains(point))
                    });
                if child_contains_pointer {
                    state.transfer_deadline = None;
                } else {
                    close_submenu(&self.slots, state);
                    shell.invalidate_layout();
                    shell.request_redraw();
                }
            }
        }
    }

    fn handle_pointer(
        &self,
        state: &mut MenuListState,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) {
        if let Some(position) = pointer_highlight_position(event, cursor) {
            state.last_pointer = position;
            if let Some(pointer_inside) = &self.level_pointer_inside {
                pointer_inside.set(position.is_some_and(|point| layout.bounds().contains(point)));
            }
            let highlight = position
                .and_then(|position| slot_at(&self.slots, layout.bounds(), position))
                .filter(|index| self.slots[*index].eligible);
            state.set_highlight(&self.slots, highlight, HighlightOrigin::Decision);
            update_submenu_pointer_intent(&self.slots, state, highlight, shell);
            sync_logical_focus(&self.slots, state, self.focus_visible(state));
        }
        if is_primary_press(event)
            && primary_press_position(event, cursor)
                .is_some_and(|point| layout.bounds().contains(point))
        {
            if self.root {
                let focus = state.focus.as_mut().expect("root Menu focus state");
                focus.focus_from_pointer();
                self.shared_focus_visible.set(focus.is_focus_visible());
            }
            state.pressed = state.highlight;
            if matches!(event, Event::Touch(touch::Event::FingerPressed { .. }))
                && state.pressed.is_some()
            {
                shell.capture_event();
            }
            shell.request_redraw();
        }
        let released = release_position(event, cursor)
            .and_then(|position| slot_at(&self.slots, layout.bounds(), position))
            .filter(|index| Some(*index) == state.pressed);
        if let Some(index) = released {
            if self.slots[index].branch.is_some() {
                open_submenu(&self.slots, state, index);
                shell.capture_event();
                shell.invalidate_layout();
            } else if matches!(event, Event::Touch(touch::Event::FingerLifted { .. })) {
                if let Some(message) = self.slots[index].activation.clone() {
                    shell.publish(message);
                    shell.capture_event();
                }
            }
        }
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerLifted { .. })
                | Event::Touch(touch::Event::FingerLost { .. })
        ) {
            state.pressed = None;
            shell.request_redraw();
        }
    }

    fn handle_keyboard(
        &self,
        state: &mut MenuListState,
        event: &Event,
        layout: Layout<'_>,
        shell: &mut Shell<'_, Message>,
    ) {
        if self.level_active(state) {
            if matches!(
                event,
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::ArrowLeft),
                    ..
                })
            ) && !self.root
            {
                if let Some(open) = &self.level_open {
                    open.set(false);
                }
                shell.capture_event();
                shell.invalidate_layout();
                shell.request_redraw();
                return;
            }
            if matches!(
                event,
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::ArrowRight),
                    ..
                })
            ) {
                if let Some(index) = state
                    .highlight
                    .filter(|index| self.slots[*index].branch.is_some())
                {
                    open_submenu(&self.slots, state, index);
                    shell.capture_event();
                    shell.invalidate_layout();
                    shell.request_redraw();
                }
                return;
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
                    if let Some(index) =
                        typeahead_match(&self.slots, state.highlight, state.typeahead.as_str())
                    {
                        state.set_highlight(&self.slots, Some(index), HighlightOrigin::Decision);
                        self.request_highlight_visible(state, layout);
                        sync_logical_focus(&self.slots, state, self.focus_visible(state));
                    }
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
            }
            if matches!(
                event,
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::Enter | Named::Space),
                    ..
                })
            ) {
                if let Some(index) = state.highlight {
                    if self.slots[index].branch.is_some() {
                        open_submenu(&self.slots, state, index);
                        shell.capture_event();
                        shell.invalidate_layout();
                        shell.request_redraw();
                    } else if let Some(message) = self.slots[index].activation.clone() {
                        shell.publish(message);
                        shell.capture_event();
                    }
                }
                return;
            }
            let Some(nav) = navigation_intent(event) else {
                return;
            };
            // Entry parks a cursor without painting it, so the reader cannot see
            // where a step would start from. The first arrow therefore reveals
            // it in place; without this, opening a menu by clicking and pressing
            // Down would begin one row past where the menu is pointing.
            let parked = !state.highlight_is_visible(self.focus_visible(state));
            let moved = match nav {
                Nav::Step(_) if parked && state.highlight.is_some() => state.highlight,
                Nav::Step(direction) => move_highlight(&self.slots, state.highlight, direction),
                Nav::First => first_eligible(&self.slots),
                Nav::Last => last_eligible(&self.slots),
            };
            // `parked` alone is a reason to write: revealing keeps the same index
            // but promotes the cursor to a chosen row, which is what paints it.
            if moved != state.highlight || parked {
                state.set_highlight(&self.slots, moved, HighlightOrigin::Decision);
                self.request_highlight_visible(state, layout);
                sync_logical_focus(&self.slots, state, self.focus_visible(state));
                shell.request_redraw();
            }
            shell.capture_event();
        }
    }
}

/// Keyboard navigation a menu level understands.
///
/// Resolved before the highlight decision so the reveal rule below is written
/// once, rather than repeated per arrow key.
enum Nav {
    /// Move by one eligible row in this direction.
    Step(isize),
    First,
    Last,
}

fn navigation_intent(event: &Event) -> Option<Nav> {
    let Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event else {
        return None;
    };

    match key {
        keyboard::Key::Named(Named::ArrowDown) => Some(Nav::Step(1)),
        keyboard::Key::Named(Named::ArrowUp) => Some(Nav::Step(-1)),
        keyboard::Key::Named(Named::Home) => Some(Nav::First),
        keyboard::Key::Named(Named::End) => Some(Nav::Last),
        _ => None,
    }
}
