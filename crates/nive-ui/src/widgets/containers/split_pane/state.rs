use iced::{advanced::widget::tree, Point};

use crate::advanced::focus::FocusState;
use crate::interaction::PointerGestureState;

#[derive(Debug)]
pub(super) struct SplitPaneState {
    pub gestures: PointerGestureState<SplitPaneRegion>,
    pub drag: Option<DragSession>,
    pub focus: FocusState,
    pub available_length: f32,
}

impl Default for SplitPaneState {
    fn default() -> Self {
        Self {
            gestures: PointerGestureState::new(),
            drag: None,
            focus: FocusState::default(),
            available_length: 0.0,
        }
    }
}

impl SplitPaneState {
    pub(super) fn new_state() -> tree::State {
        tree::State::new(Self::default())
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
