use iced::{
    advanced::widget::{operation, tree},
    Point,
};

use crate::interaction::PointerGestureState;

#[derive(Debug)]
pub(super) struct SplitPaneState {
    pub gestures: PointerGestureState<SplitPaneRegion>,
    pub drag: Option<DragSession>,
    pub focused: bool,
    pub available_length: f32,
}

impl Default for SplitPaneState {
    fn default() -> Self {
        Self {
            gestures: PointerGestureState::new(),
            drag: None,
            focused: false,
            available_length: 0.0,
        }
    }
}

impl SplitPaneState {
    pub(super) fn new_state() -> tree::State {
        tree::State::new(Self::default())
    }
}

impl operation::Focusable for SplitPaneState {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct DragSession {
    pub origin_ratio: f32,
    pub origin_position: Point,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SplitPaneRegion {
    Grip,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SnapConfig {
    pub threshold: f32,
    pub points: Vec<f32>,
}

impl SnapConfig {
    pub(super) fn new(threshold: f32, points: Vec<f32>) -> Self {
        Self {
            threshold: threshold.max(0.0),
            points,
        }
    }
}
