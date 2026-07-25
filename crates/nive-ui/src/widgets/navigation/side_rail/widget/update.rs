use iced::{
    advanced::{mouse, widget::Tree, Clipboard, Layout, Shell},
    Event, Rectangle,
};

use crate::widgets::navigation::overflow::{wheel_delta, OverflowAxis, OverflowDirection};
use crate::widgets::navigation::side_rail::layout::hit_geometry;
use crate::widgets::navigation::side_rail::widget::{
    SideRail, SideRailState, CHEVRON_SCROLL_STEP_FACTOR,
};

impl<'a, Id, Message> SideRail<'a, Id, Message>
where
    Id: Clone + 'a,
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
            let state = tree.state.downcast_ref::<SideRailState>();
            let mut content = self.content_element(state);
            content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        let hit_geometry = hit_geometry(layout);
        let state = tree.state.downcast_mut::<SideRailState>();
        state.up_chevron = hit_geometry.up_chevron;
        state.down_chevron = hit_geometry.down_chevron;

        if !cursor.is_over(layout.bounds()) {
            return;
        }

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) if state.overflow.has_overflow => {
                let delta_y = wheel_delta(OverflowAxis::Vertical, *delta);
                state.overflow.offset = state.scroll_offset;
                state.overflow.scroll_by(delta_y);
                state.scroll_offset = state.overflow.offset;
                if delta_y != 0.0 {
                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if state.overflow.has_overflow =>
            {
                let direction = if state
                    .up_chevron
                    .is_some_and(|bounds| cursor.is_over(bounds))
                {
                    Some(OverflowDirection::Backward)
                } else if state
                    .down_chevron
                    .is_some_and(|bounds| cursor.is_over(bounds))
                {
                    Some(OverflowDirection::Forward)
                } else {
                    None
                };

                if let Some(direction) = direction {
                    state.overflow.offset = state.scroll_offset;
                    state
                        .overflow
                        .page_step(direction, CHEVRON_SCROLL_STEP_FACTOR);
                    state.scroll_offset = state.overflow.offset;
                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            _ => {}
        }
    }
}
