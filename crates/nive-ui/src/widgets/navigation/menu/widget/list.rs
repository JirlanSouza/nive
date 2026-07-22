use std::{cell::Cell, rc::Rc};

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Renderer as _, Shell, Widget,
    },
    keyboard::{self, key::Named},
    touch, window, Background, Border, Color, Event, Length, Rectangle, Shadow, Size, Vector,
};

use super::{MenuLevelContext, MenuList, MenuListState, MenuSlot, TYPEAHEAD_TIMEOUT};
use crate::widgets::navigation::menu::{MENU_MAX_WIDTH, MENU_ROW_RADIUS};
use crate::{
    advanced::focus::FocusState,
    theme::{
        choice::{self, ChoiceStateInput},
        BorderRole, ControlRole, FieldValidation,
    },
    Element,
};

use super::helpers::{
    close_submenu, ensure_overlay_nodes, first_eligible, is_primary_press, last_eligible,
    max_trailing_width, move_highlight, natural_width, open_submenu, pointer_highlight_position,
    primary_press_position, reconcile_closed_submenu, reconcile_open_submenu, release_position,
    set_highlight, slot_at, slot_bounds, sync_logical_focus, typeahead_match,
    update_submenu_pointer_intent,
};

impl<'a, Message> MenuList<'a, Message> {
    pub(in crate::widgets::navigation::menu) fn new(
        content: impl Into<Element<'a, Message>>,
        slots: Vec<MenuSlot<Message>>,
        reserve_choice: bool,
        reserve_icon: bool,
        trailing_width: Rc<Cell<f32>>,
        context: MenuLevelContext,
    ) -> Self {
        Self {
            content: content.into(),
            slots,
            reserve_choice,
            reserve_icon,
            trailing_width,
            root: context.root,
            shared_focus_visible: context.shared_focus_visible,
            level_open: context.level_open,
            level_pointer_inside: context.level_pointer_inside,
            ensure_visible: context.ensure_visible,
        }
    }

    pub(in crate::widgets::navigation::menu) fn level_active(&self, state: &MenuListState) -> bool {
        if self.root {
            state.focus.as_ref().is_some_and(FocusState::is_active)
        } else {
            self.level_open.as_ref().is_some_and(|open| open.get())
        }
    }

    pub(in crate::widgets::navigation::menu) fn focus_visible(
        &self,
        state: &MenuListState,
    ) -> bool {
        if self.root {
            state
                .focus
                .as_ref()
                .is_some_and(FocusState::is_focus_visible)
        } else {
            self.shared_focus_visible.get()
        }
    }

    pub(in crate::widgets::navigation::menu) fn request_highlight_visible(
        &self,
        state: &MenuListState,
        layout: Layout<'_>,
    ) {
        if let Some(target) = state
            .highlight
            .and_then(|index| slot_bounds(&self.slots, layout.bounds(), index))
        {
            self.ensure_visible.request(target);
        }
    }
}

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer> for MenuList<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<MenuListState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(MenuListState::new(self.root))
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
        let state = tree.state.downcast_mut::<MenuListState>();
        let reconciled = state
            .highlighted_label
            .as_deref()
            .and_then(|label| {
                self.slots.iter().position(|slot| {
                    slot.eligible
                        && slot
                            .label
                            .as_deref()
                            .is_some_and(|current| current == label)
                })
            })
            .or_else(|| {
                state
                    .highlight
                    .filter(|index| self.slots.get(*index).is_some_and(|slot| slot.eligible))
            })
            .or_else(|| first_eligible(&self.slots));
        set_highlight(&self.slots, state, reconciled);
        reconcile_open_submenu(&self.slots, state);
        ensure_overlay_nodes(&self.slots, state);
        sync_logical_focus(&self.slots, state, self.focus_visible(state));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let maximum = limits.max().width.clamp(0.0, MENU_MAX_WIDTH);
        self.trailing_width
            .set(max_trailing_width(renderer, &self.slots));
        let width = natural_width(
            renderer,
            &self.slots,
            self.reserve_choice,
            self.reserve_icon,
        )
        .min(maximum);
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &limits.width(width))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<MenuListState>();
        if self.root {
            let focus = state.focus.as_mut().expect("root Menu focus state");
            if self.slots.iter().any(|slot| slot.eligible) {
                focus.register(operation, None, layout.bounds());
            } else {
                focus.clear();
            }
            self.shared_focus_visible.set(focus.is_focus_visible());
        }
        let entered = self.level_active(state) && state.highlight.is_none();
        if entered {
            set_highlight(&self.slots, state, first_eligible(&self.slots));
            self.request_highlight_visible(state, layout);
        }
        ensure_overlay_nodes(&self.slots, state);
        for ((index, slot), node) in self
            .slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.branch.is_some())
            .zip(state.overlay_nodes.iter_mut())
        {
            operation.custom(
                None,
                slot_bounds(&self.slots, layout.bounds(), index).unwrap_or_default(),
                node,
            );
            if let Some(branch) = &slot.branch {
                *branch.identity.borrow_mut() = node.identity().clone();
            }
        }
        sync_logical_focus(&self.slots, state, self.focus_visible(state));
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
        if let Some(position) = pointer_highlight_position(event, cursor) {
            state.last_pointer = position;
            if let Some(pointer_inside) = &self.level_pointer_inside {
                pointer_inside.set(position.is_some_and(|point| layout.bounds().contains(point)));
            }
            let highlight = position
                .and_then(|position| slot_at(&self.slots, layout.bounds(), position))
                .filter(|index| self.slots[*index].eligible);
            set_highlight(&self.slots, state, highlight);
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
                        set_highlight(&self.slots, state, Some(index));
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
            let moved = match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::ArrowDown),
                    ..
                }) => move_highlight(&self.slots, state.highlight, 1),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::ArrowUp),
                    ..
                }) => move_highlight(&self.slots, state.highlight, -1),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::Home),
                    ..
                }) => first_eligible(&self.slots),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::End),
                    ..
                }) => last_eligible(&self.slots),
                _ => return,
            };
            if moved != state.highlight {
                set_highlight(&self.slots, state, moved);
                self.request_highlight_visible(state, layout);
                sync_logical_focus(&self.slots, state, self.focus_visible(state));
                shell.request_redraw();
            }
            shell.capture_event();
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
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<MenuListState>();
        let focus_visible = self.focus_visible(state);
        for (index, slot) in self
            .slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| !slot.separator)
        {
            let Some(bounds) = slot_bounds(&self.slots, layout.bounds(), index) else {
                continue;
            };
            let resolved = choice::resolve_state(ChoiceStateInput {
                persistent: slot.persistent,
                validation: FieldValidation::Valid,
                callback_present: slot.eligible,
                disabled: slot.disabled,
                hovered: state.highlight == Some(index),
                pressed: state.pressed == Some(index),
                focused: focus_visible && state.highlight == Some(index),
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
        }
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
        if focus_visible {
            if let Some(bounds) = state
                .highlight
                .and_then(|index| slot_bounds(&self.slots, layout.bounds(), index))
            {
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
