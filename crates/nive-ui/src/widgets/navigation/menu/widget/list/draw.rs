use iced::{
    advanced::{mouse, renderer, widget::Tree, Layout, Renderer as _},
    Background, Border, Color, Rectangle, Shadow,
};

use crate::theme::{
    choice::{self, ChoiceStateInput},
    BorderRole, ControlRole, FieldValidation,
};
use crate::widgets::navigation::menu::widget::helpers::slot_bounds;
use crate::widgets::navigation::menu::widget::{MenuList, MenuListState};
use crate::widgets::navigation::menu::MENU_ROW_RADIUS;

impl<'a, Message> MenuList<'a, Message>
where
    Message: Clone + 'a,
{
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_impl(
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
}
