mod builder;
mod widget;

#[cfg(test)]
mod tests;

use std::borrow::Cow;

use iced::{border::Radius, touch, Length, Rectangle};

use crate::advanced::focus::FocusState;
use crate::theme::ControlSize;
use crate::Element;
use crate::IconRef;

/// One typed option in a canonical [`SegmentedControl`].
///
/// Values must be unique. Canonical content is a nonempty one-line label with
/// an optional leading semantic icon.
pub struct SegmentedOption<'a, T> {
    value: T,
    label: Cow<'a, str>,
    icon: Option<IconRef>,
    disabled: bool,
}

/// Visual composition for a typed segmented choice.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SegmentedControlVariant {
    /// A neutral track with an inset selected thumb.
    #[default]
    Default,
    /// One perimeter with edge-to-edge joined item states.
    Linked,
}

/// A controlled typed selector for two through five fixed modes or filters.
///
/// The semantic name, unique option values, and selected value form one group
/// model. Invalid models remain finite and noninteractive without inventing
/// application state. The group contributes one focus entry; physical LTR
/// Left/Right navigation is bounded and Home/End reach enabled extremes.
/// Callback absence is display-only. Retained metadata and keyboard behavior do
/// not yet imply native accessibility-tree emission.
///
/// ```
/// use nive_ui::prelude::*;
///
/// let _ = SegmentedControl::<_, ()>::new(
///     "Mode",
///     1,
///     [SegmentedOption::new(1, "One"), SegmentedOption::new(2, "Two")],
/// );
/// ```
pub struct SegmentedControl<'a, T, Message> {
    semantic_name: Cow<'a, str>,
    selected: T,
    options: Vec<SegmentedOption<'a, T>>,
    size: ControlSize,
    width: Length,
    variant: SegmentedControlVariant,
    disabled: bool,
    id: Option<iced::widget::Id>,
    on_select: Option<Box<dyn Fn(T) -> Message + 'a>>,
    contents: Vec<Element<'a, Message>>,
}

impl<'a, T, Message> From<SegmentedControl<'a, T, Message>> for Element<'a, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    fn from(control: SegmentedControl<'a, T, Message>) -> Self {
        Element::new(control)
    }
}

#[derive(Debug, Default)]
struct SegmentedState {
    focus: FocusState,
    focused_index: Option<usize>,
    pressed_index: Option<usize>,
    touch: Option<(touch::Finger, usize)>,
    item_bounds: Vec<Rectangle>,
}

struct SegmentedFocus<'a> {
    focus: &'a mut FocusState,
    focused_index: &'a mut Option<usize>,
    pressed_index: &'a mut Option<usize>,
    touch: &'a mut Option<(touch::Finger, usize)>,
}

fn segment_radius(
    variant: SegmentedControlVariant,
    index: usize,
    item_count: usize,
    radius: f32,
) -> Radius {
    match variant {
        SegmentedControlVariant::Default => Radius::new(radius),
        SegmentedControlVariant::Linked if item_count == 1 => Radius::new(radius),
        SegmentedControlVariant::Linked if index == 0 => Radius::default().left(radius),
        SegmentedControlVariant::Linked if index + 1 == item_count => {
            Radius::default().right(radius)
        }
        SegmentedControlVariant::Linked => Radius::default(),
    }
}

fn inset_radius(radius: Radius, inset: f32) -> Radius {
    Radius {
        top_left: (radius.top_left - inset).max(0.0),
        top_right: (radius.top_right - inset).max(0.0),
        bottom_right: (radius.bottom_right - inset).max(0.0),
        bottom_left: (radius.bottom_left - inset).max(0.0),
    }
}
