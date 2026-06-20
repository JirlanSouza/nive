use iced::{Border, Color};

use crate::theme::BorderSpec;

pub(super) fn border_with_radius(
    spec: BorderSpec,
    radius: impl Into<iced::border::Radius>,
) -> Border {
    Border {
        color: spec.color,
        width: spec.width,
        radius: radius.into(),
    }
}

pub(super) fn transparent_border_with_radius(radius: impl Into<iced::border::Radius>) -> Border {
    border_with_radius(BorderSpec::none(), radius)
}

pub(super) fn alpha_when_disabled(color: Color, disabled: bool) -> Color {
    if disabled {
        color.scale_alpha(0.5)
    } else {
        color
    }
}
