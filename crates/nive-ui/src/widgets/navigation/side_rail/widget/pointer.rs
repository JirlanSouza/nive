use iced::{
    advanced::{mouse, widget::Tree, Layout},
    Rectangle,
};

use crate::widgets::navigation::side_rail::widget::{SideRail, SideRailState};

impl<'a, Id, Message> SideRail<'a, Id, Message>
where
    Id: Clone + 'a,
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
        let state = tree.state.downcast_ref::<SideRailState>();
        let content = self.content_element(state);
        let interaction = content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        );

        if interaction != mouse::Interaction::None {
            return interaction;
        }

        let over_chevron = state
            .up_chevron
            .is_some_and(|bounds| cursor.is_over(bounds))
            || state
                .down_chevron
                .is_some_and(|bounds| cursor.is_over(bounds));

        if state.overflow.has_overflow && over_chevron {
            return mouse::Interaction::Pointer;
        }

        mouse::Interaction::None
    }
}
