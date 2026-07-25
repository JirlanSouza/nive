use iced::{
    advanced::{mouse, renderer, widget::Tree, Layout, Renderer as _},
    Rectangle, Shadow,
};

use crate::widgets::navigation::tabs::geometry::insertion_marker_bounds;
use crate::widgets::navigation::tabs::style as theme_tabs;
use crate::widgets::navigation::tabs::{TabBar, TabBarState};

impl<'a, Id, Message> TabBar<'a, Id, Message>
where
    Id: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    pub(super) fn mouse_interaction_impl(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<TabBarState<Id>>();
        let content = self.content_element(state);
        let interaction = content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        );

        if state.dragged_id.is_some() {
            return state.drag_session.mouse_interaction();
        }

        if interaction != mouse::Interaction::None {
            return interaction;
        }

        mouse::Interaction::None
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_impl(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<TabBarState<Id>>();
        let bounds = layout.bounds();
        let metrics = theme_tabs::metrics(self.size);

        self.draw_bar_chrome(renderer, theme, bounds, metrics);

        // Each tab paints its own fill, focus ring, active indicator and drag
        // veil. What is left here belongs to the collection, not to any tab:
        // the marker for where a dragged tab would land.
        let content = self.content_element(state);
        content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );

        if state.dragged_id.is_some() {
            // Lives in the scrolled strip, so it needs the same recorte the
            // strip viewport applies to its content.
            let strip = state.strip_bounds.unwrap_or(bounds);

            renderer.with_layer(strip, |renderer| {
                self.draw_insertion_marker(state, renderer, theme, metrics);
            });
        }
    }

    fn draw_bar_chrome(
        &self,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        bounds: Rectangle,
        metrics: theme_tabs::TabBarMetrics,
    ) {
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: iced::Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            theme_tabs::strip_background(theme, self.role),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: bounds.y + bounds.height - metrics.seam_width,
                    width: bounds.width,
                    height: metrics.seam_width,
                },
                border: iced::Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            theme_tabs::strip_divider(theme, self.role),
        );
    }

    fn draw_insertion_marker(
        &self,
        state: &TabBarState<Id>,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        metrics: theme_tabs::TabBarMetrics,
    ) {
        let Some(target) = &state.insertion_target else {
            return;
        };
        let Some(marker) = insertion_marker_bounds(target, &state.tab_bounds, metrics.tab_gap)
        else {
            return;
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: marker,
                border: iced::Border::default().rounded(1.0),
                shadow: Shadow::default(),
                snap: true,
            },
            theme_tabs::insertion_marker_color(theme),
        );
    }
}
