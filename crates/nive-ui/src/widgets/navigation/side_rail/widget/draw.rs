use iced::{
    advanced::{mouse, renderer, widget::Tree, Layout, Renderer as _},
    Rectangle, Shadow,
};

use crate::widgets::navigation::side_rail::style::{rail_background, seam_color};
use crate::widgets::navigation::side_rail::widget::{seam_bounds, SideRail, SideRailState};

impl<'a, Id, Message> SideRail<'a, Id, Message>
where
    Id: Clone + 'a,
    Message: Clone + 'a,
{
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
        let state = tree.state.downcast_ref::<SideRailState>();
        let bounds = layout.bounds();

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: iced::Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            rail_background(theme),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: seam_bounds(bounds, self.side),
                border: iced::Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            seam_color(theme),
        );

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
    }
}
