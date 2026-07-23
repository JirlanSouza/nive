use iced::{
    advanced::{mouse, renderer, text::Renderer as _, widget::Tree, Layout, Renderer as _, Widget},
    alignment,
    border::Radius,
    widget, Background, Border, Color, Font, Pixels, Rectangle, Shadow,
};

use crate::theme::choice::{self, ChoiceMetrics, ResolvedChoiceState};
use crate::widgets::controls::single_choice::{SingleChoice, SingleChoiceKind, SingleChoiceState};

impl<Message> SingleChoice<'_, Message>
where
    Message: Clone,
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
        let metrics = self.metrics(*theme);
        let state = tree.state.downcast_ref::<SingleChoiceState>();
        let resolved = self.resolved_state(state, cursor, layout.bounds());
        let palette = choice::palette(*theme, resolved);
        let anchor = self.anchor_bounds(layout, metrics);
        let radius = match self.kind {
            SingleChoiceKind::Checkbox => metrics.checkbox_radius,
            SingleChoiceKind::Radio | SingleChoiceKind::Switch => anchor.height / 2.0,
        };

        let anchor_background = if self.kind == SingleChoiceKind::Radio
            && resolved.control.selected
            && resolved.control.enabled
        {
            theme
                .control(
                    crate::theme::ControlRole::Selectable,
                    crate::theme::ControlState::ENABLED,
                )
                .background
        } else {
            palette.background
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: anchor,
                border: Border {
                    color: palette.perimeter,
                    width: metrics.perimeter_width,
                    radius: Radius::new(radius),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(anchor_background),
        );

        self.draw_mark(
            renderer, theme, &metrics, anchor, &resolved, &palette, viewport,
        );

        let content = self.content();
        content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );

        if resolved.control.interaction.focused {
            let (bounds, focus_radius) = match self.kind {
                SingleChoiceKind::Checkbox => (
                    metrics.indicator_focus_bounds(anchor),
                    metrics.checkbox_focus_radius(),
                ),
                SingleChoiceKind::Radio => (
                    metrics.indicator_focus_bounds(anchor),
                    metrics.radio_focus_radius(),
                ),
                SingleChoiceKind::Switch => (
                    metrics.track_focus_bounds(anchor),
                    metrics.switch_focus_radius(),
                ),
            };
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        color: palette.focus,
                        width: metrics.focus_stroke_width,
                        radius: Radius::new(focus_radius),
                    },
                    ..renderer::Quad::default()
                },
                Color::TRANSPARENT,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_mark(
        &self,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        metrics: &ChoiceMetrics,
        anchor: Rectangle,
        resolved: &ResolvedChoiceState,
        palette: &choice::ChoicePalette,
        viewport: &Rectangle,
    ) {
        match self.kind {
            SingleChoiceKind::Checkbox if resolved.mixed => {
                let mark = Rectangle {
                    x: anchor.x + anchor.width * 0.25,
                    y: anchor.center_y() - 1.0,
                    width: anchor.width * 0.5,
                    height: 2.0,
                };
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: mark,
                        border: Border {
                            radius: Radius::new(1.0),
                            ..Border::default()
                        },
                        ..renderer::Quad::default()
                    },
                    palette.mark,
                );
            }
            SingleChoiceKind::Checkbox if resolved.control.selected => {
                renderer.fill_text(
                    iced::advanced::text::Text {
                        content: "✓".to_owned(),
                        bounds: anchor.size(),
                        size: Pixels(anchor.height * 0.72),
                        line_height: widget::text::LineHeight::default(),
                        font: Font::DEFAULT,
                        align_x: widget::text::Alignment::Center,
                        align_y: alignment::Vertical::Center,
                        shaping: widget::text::Shaping::Advanced,
                        wrapping: widget::text::Wrapping::None,
                    },
                    anchor.center(),
                    palette.mark,
                    *viewport,
                );
            }
            SingleChoiceKind::Radio if resolved.control.selected => {
                let dot_size = metrics.radio_dot_size();
                let dot = Rectangle {
                    x: anchor.center_x() - dot_size / 2.0,
                    y: anchor.center_y() - dot_size / 2.0,
                    width: dot_size,
                    height: dot_size,
                };
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: dot,
                        border: Border {
                            radius: Radius::new(dot_size / 2.0),
                            ..Border::default()
                        },
                        ..renderer::Quad::default()
                    },
                    if resolved.control.enabled {
                        theme.tone(crate::theme::ToneRole::Accent).color
                    } else {
                        palette.mark
                    },
                );
            }
            SingleChoiceKind::Switch => {
                let thumb_size = metrics.switch_thumb_size;
                let thumb = Rectangle {
                    x: if resolved.control.selected {
                        anchor.x + anchor.width - metrics.switch_thumb_inset - thumb_size
                    } else {
                        anchor.x + metrics.switch_thumb_inset
                    },
                    y: anchor.y + metrics.switch_thumb_inset,
                    width: thumb_size,
                    height: thumb_size,
                };
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: thumb,
                        border: Border {
                            radius: Radius::new(thumb_size / 2.0),
                            ..Border::default()
                        },
                        ..renderer::Quad::default()
                    },
                    palette.mark,
                );
            }
            _ => {}
        }
    }
}
