use iced::{
    advanced::{
        layout, mouse, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Color, Event, Length, Rectangle, Size,
};

use super::super::{
    event::{ColorPickerControl, ColorPickerEvent},
    hsva_color::HsvaColor,
};
use super::{
    control_state::ControlState,
    control_widget,
    drag::{handle_drag, interaction},
    keyboard::{adjust_unit, saturation_value_action, SaturationValueAction},
    metrics::saturation_value_size,
    render::{bounded_marker_center, draw_marker, draw_saturation_value_surface},
};

pub(super) struct SaturationValueArea {
    color: Color,
    hsva: HsvaColor,
    disabled: bool,
}

impl SaturationValueArea {
    pub(super) fn new(color: Color, hsva: HsvaColor, disabled: bool) -> Self {
        Self {
            color,
            hsva,
            disabled,
        }
    }
}

impl Widget<ColorPickerEvent, crate::theme::Theme, iced::Renderer> for SaturationValueArea {
    fn tag(&self) -> tree::Tag {
        control_widget::tag()
    }

    fn state(&self) -> tree::State {
        control_widget::state()
    }

    fn size(&self) -> Size<Length> {
        saturation_value_size()
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        control_widget::fixed_layout(limits, saturation_value_size())
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

        control_widget::focus_on_press(
            event,
            bounds,
            cursor,
            shell,
            ColorPickerControl::SaturationValue,
        );

        handle_drag(
            event,
            bounds,
            cursor,
            state,
            self.disabled,
            shell,
            |point| {
                let saturation = (point.x / bounds.width).clamp(0.0, 1.0);
                let value = (1.0 - point.y / bounds.height).clamp(0.0, 1.0);

                ColorPickerEvent::SaturationValueChanged { saturation, value }
            },
        );

        control_widget::handle_keyboard(self.disabled, state, event, shell, |event| {
            let (action, large) = saturation_value_action(event)?;
            let mut saturation = self.hsva.saturation();
            let mut value = self.hsva.value();

            match action {
                SaturationValueAction::Saturation(action) => {
                    saturation = adjust_unit(saturation, action, large);
                }
                SaturationValueAction::Value(action) => {
                    value = adjust_unit(value, action, large);
                }
            }

            Some(ColorPickerEvent::SaturationValueChanged { saturation, value })
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
            ColorPickerControl::SaturationValue,
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
            mouse::Interaction::Crosshair,
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

        draw_saturation_value_surface(renderer, state, bounds, self.hsva.hue());
        let marker = bounded_marker_center(
            bounds,
            bounds.x + self.hsva.saturation() * bounds.width,
            bounds.y + (1.0 - self.hsva.value()) * bounds.height,
        );
        draw_marker(renderer, theme, marker, self.color, state.is_focused());
    }
}
