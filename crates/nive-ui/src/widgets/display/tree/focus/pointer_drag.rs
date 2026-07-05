use iced::{
    advanced::{mouse, Layout},
    keyboard, Event, Point,
};

use crate::interaction::{ClickModifiers, TransferOperation};

use super::{TreeFocus, TreeFocusState};

impl<'a, Id, Message> TreeFocus<'a, Id, Message>
where
    Id: Clone + Ord + 'a,
    Message: Clone + 'a,
{
    pub(super) fn pointer_context_message(
        &self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) -> Option<Message> {
        let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) = event else {
            return None;
        };

        let bounds = layout.bounds();
        let position = cursor.position_in(bounds)?;
        let tree_state = self.tree.state_or_default();

        self.tree
            .context_request_at_tree_event(&tree_state, position.y, position)
    }

    pub(super) fn pointer_drag_message(
        &self,
        state: &mut TreeFocusState,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) -> Option<Message> {
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.drag.origin = cursor.position_in(bounds);
                state.drag.active = false;
                None
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let origin = state.drag.origin?;
                let position = cursor.position_in(bounds)?;

                if !state.drag.active {
                    if distance(origin, position) < 4.0 {
                        return None;
                    }

                    state.drag.active = true;
                    let tree_state = self.tree.state_or_default();
                    let id = self.tree.row_id_at(&tree_state, origin.y)?;
                    let event = self.tree.drag_start_tree_event(
                        &tree_state,
                        &id,
                        click_modifiers(state.modifiers),
                    )?;

                    return self.tree.publish(event);
                }

                let tree_state = self.tree.state_or_default();
                let event = self.tree.drop_feedback_at_tree_event(
                    &tree_state,
                    position.y,
                    click_modifiers(state.modifiers),
                    requested_operation(state.modifiers),
                )?;

                self.tree.publish(event)
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let was_active = state.drag.active;
                state.drag.origin = None;
                state.drag.active = false;

                if !was_active {
                    return None;
                }

                let position = cursor.position_in(bounds)?;
                let tree_state = self.tree.state_or_default();
                let event = self.tree.drop_release_at_tree_event(
                    &tree_state,
                    position.y,
                    click_modifiers(state.modifiers),
                    requested_operation(state.modifiers),
                )?;

                self.tree.publish(event)
            }
            Event::Mouse(mouse::Event::CursorLeft) => {
                state.drag.origin = None;
                state.drag.active = false;
                None
            }
            _ => None,
        }
    }
}

pub(super) fn update_modifiers(state: &mut TreeFocusState, event: &Event) {
    match event {
        Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
            state.modifiers = *modifiers;
        }
        Event::Keyboard(keyboard::Event::KeyPressed { modifiers, .. })
        | Event::Keyboard(keyboard::Event::KeyReleased { modifiers, .. }) => {
            state.modifiers = *modifiers;
        }
        _ => {}
    }
}

fn click_modifiers(modifiers: keyboard::Modifiers) -> ClickModifiers {
    ClickModifiers::new(modifiers.command(), modifiers.shift())
}

fn requested_operation(modifiers: keyboard::Modifiers) -> Option<TransferOperation> {
    if modifiers.alt() {
        Some(TransferOperation::Link)
    } else if modifiers.command() {
        Some(TransferOperation::Copy)
    } else if modifiers.shift() {
        Some(TransferOperation::Move)
    } else {
        None
    }
}

fn distance(a: Point, b: Point) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;

    (dx * dx + dy * dy).sqrt()
}
