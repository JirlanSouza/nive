mod draw;
mod update;

use std::{cell::Cell, rc::Rc};

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Event, Length, Rectangle, Size, Vector,
};

use super::{MenuLevelContext, MenuList, MenuListState, MenuSlot};
use crate::widgets::navigation::menu::MENU_MAX_WIDTH;
use crate::{advanced::focus::FocusState, Element};

use super::helpers::{
    ensure_overlay_nodes, first_eligible, max_trailing_width, natural_width,
    reconcile_open_submenu, slot_bounds, sync_logical_focus,
};
use super::HighlightOrigin;

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
        if !self.level_active(state) {
            state.forget_highlight_session();
        }
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
            .or_else(|| {
                // Only recover when a highlight existed and its row left the
                // model. A highlight the pointer cleared has no label, and a
                // rebuild must not invent one for it.
                state
                    .highlighted_label
                    .is_some()
                    .then(|| first_eligible(&self.slots))
                    .flatten()
            });
        state.set_highlight(&self.slots, reconciled, HighlightOrigin::Reconciliation);
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
        let entered = self.level_active(state) && !state.highlight_established();
        if entered {
            state.set_highlight(
                &self.slots,
                first_eligible(&self.slots),
                HighlightOrigin::Entry,
            );
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
        self.update_impl(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
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
        self.draw_impl(tree, renderer, theme, style, layout, cursor, viewport);
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
