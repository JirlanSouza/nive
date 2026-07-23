use iced::{
    advanced::{mouse, renderer, widget::Tree, Layout, Renderer as _},
    Rectangle, Shadow,
};

use crate::advanced::pressable::{draw_focus_ring_with_placement, FocusRingPlacement};
use crate::widgets::controls::button::ButtonFocusRing;
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
        for (id, tab_bounds, _) in &state.tab_bounds {
            let Some(tab) = self.tabs.iter().find(|tab| &tab.id == id) else {
                continue;
            };
            let selected = self.active.as_ref().is_some_and(|active| active == id);
            let hovered = state
                .hovered_id
                .as_ref()
                .is_some_and(|hovered| hovered == id);
            let pressed = state
                .pressed_id
                .as_ref()
                .is_some_and(|pressed| pressed == id);
            let background = theme_tabs::tab_background(
                theme,
                self.active_role,
                selected,
                hovered,
                pressed,
                tab.disabled,
            );
            if background.a > 0.0 {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: *tab_bounds,
                        border: iced::Border::default(),
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    background,
                );
            }
        }
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

        if let Some(dragged) = &state.dragged_id {
            if let Some((_, bounds, _)) = state.tab_bounds.iter().find(|(id, _, _)| id == dragged) {
                let mut subdued = theme_tabs::strip_background(theme, self.role);
                subdued.a = 0.45;
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: *bounds,
                        border: iced::Border::default(),
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    subdued,
                );
            }
        }

        if state.focus.is_focus_visible() {
            if let Some(focused) = &state.focused_id {
                if let Some((_, bounds, _)) =
                    state.tab_bounds.iter().find(|(id, _, _)| id == focused)
                {
                    draw_focus_ring_with_placement(
                        renderer,
                        theme,
                        *bounds,
                        metrics.radius.into(),
                        ButtonFocusRing::Default,
                        FocusRingPlacement::Inset,
                    );
                }
            }
        }

        if let Some(active) = &self.active {
            if let Some((_, bounds, _)) = state.tab_bounds.iter().find(|(id, _, _)| id == active) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x,
                            y: bounds.y,
                            width: bounds.width,
                            height: metrics.indicator_width,
                        },
                        border: iced::Border::default(),
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    theme_tabs::active_indicator(theme),
                );
            }
        }

        if state.dragged_id.is_some() {
            if let Some(target) = &state.insertion_target {
                let metrics = theme_tabs::metrics(self.size);
                if let Some(marker) =
                    insertion_marker_bounds(target, &state.tab_bounds, metrics.tab_gap)
                {
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
        }
    }
}
