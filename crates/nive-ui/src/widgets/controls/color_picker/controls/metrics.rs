use iced::{Length, Size};

pub(super) const SATURATION_VALUE_WIDTH: f32 = 176.0;
pub(super) const SATURATION_VALUE_HEIGHT: f32 = 136.0;
pub(super) const SLIDER_WIDTH: f32 = 16.0;
pub(super) const CONTROL_RADIUS: f32 = 8.0;
pub(super) const MARKER_SIZE: f32 = 16.0;

pub(super) fn saturation_value_size() -> Size<Length> {
    Size::new(
        Length::Fixed(SATURATION_VALUE_WIDTH),
        Length::Fixed(SATURATION_VALUE_HEIGHT),
    )
}

pub(super) fn slider_size() -> Size<Length> {
    Size::new(
        Length::Fixed(SLIDER_WIDTH),
        Length::Fixed(SATURATION_VALUE_HEIGHT),
    )
}
