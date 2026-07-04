use iced::{keyboard, mouse, Point};

/// Normalized pointer button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    /// Primary pointer button.
    Primary,
    /// Secondary pointer button.
    Secondary,
    /// Middle pointer button.
    Middle,
    /// Additional pointer button.
    Auxiliary(u16),
}

impl From<mouse::Button> for PointerButton {
    fn from(button: mouse::Button) -> Self {
        match button {
            mouse::Button::Left => Self::Primary,
            mouse::Button::Right => Self::Secondary,
            mouse::Button::Middle => Self::Middle,
            mouse::Button::Back => Self::Auxiliary(4),
            mouse::Button::Forward => Self::Auxiliary(5),
            mouse::Button::Other(button) => Self::Auxiliary(button),
        }
    }
}

/// Normalized pointer gesture kind.
///
/// This enum is non-exhaustive; app matches should include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PointerGestureKind {
    /// A pointer button was pressed.
    Pressed,
    /// A pointer button was released.
    Released,
    /// A click completed without crossing the drag threshold.
    Clicked {
        /// Consecutive click count.
        count: u8,
    },
    /// A drag crossed the configured threshold.
    DragStarted,
    /// The pointer moved during an active drag.
    DragMoved,
    /// A drag released over the widget.
    DragReleased,
    /// A drag was cancelled.
    DragCancelled,
}

/// Widget-agnostic pointer gesture carrying a widget-defined region.
#[derive(Debug, Clone, PartialEq)]
pub struct PointerGesture<Region> {
    /// Gesture kind.
    pub kind: PointerGestureKind,
    /// Button associated with the gesture.
    pub button: PointerButton,
    /// Widget-defined region where the gesture applies.
    pub region: Region,
    /// Pointer position in widget coordinates.
    pub position: Point,
    /// Current keyboard modifiers.
    pub modifiers: keyboard::Modifiers,
}

mod gesture_state;

#[allow(unused_imports)]
pub(crate) use gesture_state::*;
