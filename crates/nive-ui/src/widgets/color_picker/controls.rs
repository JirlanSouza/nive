mod alpha_slider;
mod control_state;
mod control_widget;
mod drag;
mod hue_slider;
mod keyboard;
mod metrics;
mod render;
mod saturation_value;

use iced::Color;

use crate::Element;

use super::{event::ColorPickerEvent, hsva_color::HsvaColor};

use self::{
    alpha_slider::AlphaSlider, hue_slider::HueSlider, saturation_value::SaturationValueArea,
};

pub(super) fn saturation_value_area<'a>(
    color: Color,
    hsva: HsvaColor,
    disabled: bool,
) -> Element<'a, ColorPickerEvent> {
    Element::new(SaturationValueArea::new(color, hsva, disabled))
}

pub(super) fn hue_slider<'a>(
    color: Color,
    hue: f32,
    disabled: bool,
) -> Element<'a, ColorPickerEvent> {
    Element::new(HueSlider::new(color, hue, disabled))
}

pub(super) fn alpha_slider<'a>(
    color: Color,
    alpha: f32,
    disabled: bool,
) -> Element<'a, ColorPickerEvent> {
    Element::new(AlphaSlider::new(color, alpha, disabled))
}
