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
    keyboard::{adjust_hue, slider_action},
    metrics::slider_size,
    render::{bounded_marker_center, draw_hue_surface, draw_marker},
};

pub(super) struct HueSlider {
    color: Color,
    hue: f32,
    disabled: bool,
}

impl HueSlider {
    pub(super) fn new(color: Color, hue: f32, disabled: bool) -> Self {
        Self {
            color,
            hue,
            disabled,
        }
    }
}

impl Widget<ColorPickerEvent, crate::theme::Theme, iced::Renderer> for HueSlider {
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

        control_widget::focus_on_press(event, bounds, cursor, shell, ColorPickerControl::Hue);

        handle_drag(
            event,
            bounds,
            cursor,
            state,
            self.disabled,
            shell,
            |point| {
                let ratio = (point.y / bounds.height).clamp(0.0, 1.0);

                ColorPickerEvent::HueChanged((ratio * 359.999).clamp(0.0, 359.999))
            },
        );

        control_widget::handle_keyboard(self.disabled, state, event, shell, |event| {
            let (action, large) = slider_action(event)?;

            Some(ColorPickerEvent::HueChanged(adjust_hue(
                self.hue, action, large,
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
            ColorPickerControl::Hue,
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

        draw_hue_surface(renderer, state, bounds);
        let marker = bounded_marker_center(
            bounds,
            bounds.center_x(),
            bounds.y + self.hue / 360.0 * bounds.height,
        );
        draw_marker(renderer, theme, marker, self.color, state.is_focused());
    }
}
