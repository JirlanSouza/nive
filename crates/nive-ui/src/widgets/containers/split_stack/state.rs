use iced::{advanced::widget::tree, Point};

use crate::advanced::focus::FocusState;
use crate::interaction::PointerGestureState;

#[derive(Debug)]
pub(super) struct SplitStackState {
    pub gestures: PointerGestureState<SplitStackRegion>,
    pub drag: Option<DragSession>,
    pub focus: FocusState,
    pub focused_divider: usize,
    pub resolved: Vec<f32>,
    pub available: f32,
}

impl Default for SplitStackState {
    fn default() -> Self {
        Self {
            gestures: PointerGestureState::new(),
            drag: None,
            focus: FocusState::default(),
            focused_divider: 0,
            resolved: Vec::new(),
            available: 0.0,
        }
    }
}

impl SplitStackState {
    pub(super) fn new_state() -> tree::State {
        tree::State::new(Self::default())
    }
}

/// Lengths captured when a drag started, so moves never accumulate drift.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct DragSession {
    pub divider: usize,
    pub origin_leading: f32,
    pub origin_trailing: f32,
    pub origin_position: Point,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SplitStackRegion {
    Divider(usize),
}
