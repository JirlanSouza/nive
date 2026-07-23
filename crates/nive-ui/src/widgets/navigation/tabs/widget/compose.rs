use iced::{
    advanced::{
        layout::{self, Layout},
        overlay,
        widget::{operation, Tree},
    },
    Rectangle, Vector,
};

use crate::widgets::navigation::tabs::geometry::measure_and_translate;
use crate::widgets::navigation::tabs::style as theme_tabs;
use crate::widgets::navigation::tabs::{TabBar, TabBarFocus, TabBarState};

impl<'a, Id, Message> TabBar<'a, Id, Message>
where
    Id: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    pub(super) fn layout_impl(
        &self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_ref::<TabBarState<Id>>();
        let mut content = self.content_element(state);
        let node = content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        // Walk the layout tree to find the strip container and apply the scroll
        // translation. The bar container has a single Row child whose children
        // are: [left_chevron_slot, strip_container, right_chevron_slot,
        // all_tabs_button_slot].
        let metrics = theme_tabs::metrics(self.size);
        let min_tab_width = metrics.min_tab_width;
        let (content_width, strip_width, translated_node, viewport_tab_bounds) =
            measure_and_translate(
                node,
                state.scroll_offset,
                min_tab_width,
                metrics.max_tab_width,
                metrics.tab_gap,
            );

        let state = tree.state.downcast_mut::<TabBarState<Id>>();

        state.overflow.offset = state.scroll_offset;
        state.overflow.update_extents(content_width, strip_width);
        state.content_width = state.overflow.content_extent;
        state.strip_width = state.overflow.viewport_extent;
        state.max_scroll = state.overflow.max_offset;
        state.has_overflow = state.overflow.has_overflow;

        // Auto-reveal the active tab when it changed outside the visible
        // viewport. Minimum displacement: scroll just enough to reveal it.
        let active_changed = self.active != state.last_active_id;
        if active_changed {
            state.last_active_id = self.active.clone();
            if let Some(active) = &state.last_active_id {
                let displayed = self.displayed_tabs();
                let display_index = displayed
                    .iter()
                    .position(|displayed| &displayed.item.id == active);
                if let Some(bounds) = display_index.and_then(|index| viewport_tab_bounds.get(index))
                {
                    if bounds.x < 0.0 {
                        state.overflow.offset += bounds.x;
                    } else if bounds.x + bounds.width > strip_width {
                        state.overflow.offset += bounds.x + bounds.width - strip_width;
                    }
                }
            }
        }
        state.overflow.clamp_offset();
        state.scroll_offset = state.overflow.offset;
        self.reconcile_focus(state);

        translated_node
    }

    pub(super) fn operate_impl(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<TabBarState<Id>>();
        state.focus.expose(operation, None, layout.bounds());
        operation.focusable(
            None,
            layout.bounds(),
            &mut TabBarFocus {
                focus: &mut state.focus,
                pressed_id: &mut state.pressed_id,
            },
        );
        let state = tree.state.downcast_ref::<TabBarState<Id>>();
        let mut content = self.content_element(state);
        content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    pub(super) fn overlay_impl<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        self.overlay_content = {
            let state = tree.state.downcast_ref::<TabBarState<Id>>();
            self.content_element(state)
        };
        // `content_element` is rebuilt from live interaction state (hover,
        // scroll, the "all tabs" overflow menu), which can change between the
        // last `diff`/`layout` pass and this call. Re-diff before recursing
        // so `tree.children[0]` matches the freshly built content instead of
        // a stale shape, which previously panicked deep inside the overflow
        // menu's buttons.
        tree.children[0].diff(self.overlay_content.as_widget());
        self.overlay_content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}
