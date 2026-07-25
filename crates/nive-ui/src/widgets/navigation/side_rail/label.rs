use iced::{
    advanced::text,
    alignment, mouse,
    widget::{canvas, text::LineHeight},
    Pixels, Point, Radians, Rectangle, Vector,
};

use crate::theme::{ControlRole, ControlState, TextRole, TypographyRole};

use super::RailSide;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RailLabelCanvas {
    pub(super) text: String,
    pub(super) side: RailSide,
    pub(super) font_size: f32,
    pub(super) line_height: f32,
    pub(super) selected: bool,
    pub(super) disabled: bool,
}

impl<Message> canvas::Program<Message, crate::theme::Theme, iced::Renderer> for RailLabelCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        theme: &crate::theme::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let color = self.label_color(theme);
        let angle = rotation_radians(self.side);
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);

        frame.with_save(|frame| {
            frame.translate(Vector::new(center.x, center.y));
            frame.rotate(Radians(angle));
            frame.fill_text(canvas::Text {
                content: self.text.clone(),
                position: Point::ORIGIN,
                max_width: bounds.height,
                color,
                size: Pixels(self.font_size),
                line_height: LineHeight::Absolute(Pixels(self.line_height)),
                font: theme.typography(TypographyRole::Label).font,
                align_x: text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                shaping: text::Shaping::Auto,
            });
        });

        vec![frame.into_geometry()]
    }
}

impl RailLabelCanvas {
    /// Reflects active/selected/disabled state via the shared combined-state
    /// resolver rather than always resolving `TextRole::Secondary`.
    pub(super) fn label_color(&self, theme: &crate::theme::Theme) -> iced::Color {
        if !self.selected && !self.disabled {
            return theme.text(TextRole::Secondary).color;
        }

        let mut state = ControlState::new();
        if self.selected {
            state = state.selected();
        }
        if self.disabled {
            state = state.disabled();
        }

        theme.control(ControlRole::Selectable, state).foreground
    }
}

pub(super) fn rotation_radians(side: RailSide) -> f32 {
    match side {
        RailSide::Left => -std::f32::consts::FRAC_PI_2,
        RailSide::Right => std::f32::consts::FRAC_PI_2,
    }
}
