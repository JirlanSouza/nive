use iced::{
    advanced::{
        layout, mouse, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Color, Event, Length, Rectangle, Size,
};

use super::super::event::{ColorPickerControl, ColorPickerEvent};
use super::{
    control_state::ControlState,
    control_widget,
    drag::{handle_drag, interaction},
    keyboard::{adjust_unit, unit_slider_action},
    metrics::slider_size,
    render::{bounded_marker_center, draw_alpha_surface, draw_marker},
};

pub(super) struct AlphaSlider {
    color: Color,
    alpha: f32,
    disabled: bool,
}

impl AlphaSlider {
    pub(super) fn new(color: Color, alpha: f32, disabled: bool) -> Self {
        Self {
            color,
            alpha,
            disabled,
        }
    }
}

impl Widget<ColorPickerEvent, crate::theme::Theme, iced::Renderer> for AlphaSlider {
    fn tag(&self) -> tree::Tag {
        control_widget::tag()
    }

    fn state(&self) -> tree::State {
        control_widget::state()
    }

    fn size(&self) -> Size<Length> {
        slider_size()
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        control_widget::fixed_layout(limits, slider_size())
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, ColorPickerEvent>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<ControlState>();
        let bounds = layout.bounds();

        control_widget::focus_on_press(event, bounds, cursor, shell, ColorPickerControl::Alpha);

        handle_drag(
            event,
            bounds,
            cursor,
            state,
            self.disabled,
            shell,
            |point| {
                let ratio = (point.y / bounds.height).clamp(0.0, 1.0);

                ColorPickerEvent::AlphaChanged(ratio)
            },
        );

        control_widget::handle_keyboard(self.disabled, state, event, shell, |event| {
            let action = unit_slider_action(event)?;

            Some(ColorPickerEvent::AlphaChanged(adjust_unit(
                self.alpha, action,
            )))
        });
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        control_widget::operate_focus(
            tree,
            layout,
            self.disabled,
            operation,
            ColorPickerControl::Alpha,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        interaction(
            self.disabled,
            tree,
            layout,
            cursor,
            mouse::Interaction::Pointer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        _inherited_style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<ControlState>();

        let marker = bounded_marker_center(
            bounds,
            bounds.center_x(),
            bounds.y + self.alpha * bounds.height,
        );
        draw_alpha_surface(renderer, state, bounds, self.color);
        draw_marker(renderer, theme, marker, self.color, state.is_focused());
    }
}
