use iced::{
    advanced::{mouse, renderer, widget::Tree, Layout, Renderer as _},
    border::Radius,
    Background, Border, Color, Rectangle, Shadow,
};

use crate::theme::{
    choice::{self, ChoicePersistentState, ChoiceStateInput},
    BorderRole, ControlRole, FieldValidation, TextRole, Theme,
};
use crate::widgets::controls::segmented_control::typed::{
    inset_radius, segment_radius, SegmentedControl, SegmentedControlVariant, SegmentedState,
};

impl<'a, T, Message> SegmentedControl<'a, T, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    pub(super) fn mouse_interaction_impl(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if !self.interactive() {
            return mouse::Interaction::None;
        }
        let state = tree.state.downcast_ref::<SegmentedState>();
        if cursor
            .position()
            .and_then(|point| self.item_at(state, layout, point))
            .is_some_and(|index| !self.options[index].disabled)
        {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_impl(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let metrics = self.metrics(*theme);
        let state = tree.state.downcast_ref::<SegmentedState>();
        let bounds = layout.bounds();
        let track = theme.control(ControlRole::Standard, crate::theme::ControlState::ENABLED);

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: theme.border(BorderRole::Default).color,
                    width: metrics.perimeter_width,
                    radius: Radius::new(metrics.form.radius),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(track.background),
        );

        let origin = bounds.position();
        for (index, item) in state.item_bounds.iter().enumerate() {
            let item = Rectangle {
                x: item.x + origin.x,
                y: item.y + origin.y,
                ..*item
            };
            let selected = self.options[index].value == self.selected;
            let hovered = cursor.is_over(item);
            let pressed = state.pressed_index == Some(index)
                || state.touch.is_some_and(|(_, pressed)| pressed == index);
            let focused =
                state.focus.is_focus_visible() && self.reconciled_focus(state) == Some(index);
            let resolved = choice::resolve_state(ChoiceStateInput {
                persistent: if selected {
                    ChoicePersistentState::Selected
                } else {
                    ChoicePersistentState::Unselected
                },
                validation: FieldValidation::Valid,
                callback_present: self.on_select.is_some() && self.model_valid(),
                disabled: self.disabled || self.options[index].disabled,
                hovered,
                pressed,
                focused,
            });
            let palette = choice::segment_palette(*theme, resolved);
            let fill = match self.variant {
                SegmentedControlVariant::Default if !selected && !hovered && !pressed => {
                    Color::TRANSPARENT
                }
                _ => palette.background,
            };
            let radius = segment_radius(
                self.variant,
                index,
                self.options.len(),
                metrics.form.radius.max(0.0),
            );
            renderer.fill_quad(
                renderer::Quad {
                    bounds: item,
                    border: Border {
                        color: if self.variant == SegmentedControlVariant::Linked {
                            palette.perimeter
                        } else {
                            Color::TRANSPARENT
                        },
                        width: if self.variant == SegmentedControlVariant::Linked {
                            metrics.perimeter_width
                        } else {
                            0.0
                        },
                        radius,
                    },
                    shadow: Shadow::default(),
                    snap: true,
                },
                fill,
            );

            if focused {
                let focus_bounds = metrics.segment_focus_bounds(item);
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: focus_bounds,
                        border: Border {
                            color: palette.focus,
                            width: metrics.focus_stroke_width,
                            radius: inset_radius(radius, metrics.form.focus_inset),
                        },
                        ..renderer::Quad::default()
                    },
                    Color::TRANSPARENT,
                );
            }

            let child_layout = layout.children().nth(index);
            if let Some(child_layout) = child_layout {
                let child_style = renderer::Style {
                    text_color: if self.disabled || self.options[index].disabled {
                        theme.text(TextRole::Disabled).color
                    } else {
                        palette.foreground
                    },
                };
                self.contents[index].as_widget().draw(
                    &tree.children[index],
                    renderer,
                    theme,
                    &child_style,
                    child_layout,
                    cursor,
                    viewport,
                );
            }
        }

        let _ = inherited_style;
    }
}
