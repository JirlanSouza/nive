mod helpers;
mod state;
mod widget;

use iced::{widget::Id, Length};

use crate::interaction::Orientation;
use crate::theme::{ControlSize, SurfaceRole};
use crate::Element;

use self::state::SnapConfig;

/// Minimum leading and trailing pane sizes for a [`SplitPane`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplitPaneConstraints {
    pub leading_min: f32,
    pub trailing_min: f32,
}

impl SplitPaneConstraints {
    pub const fn new(leading_min: f32, trailing_min: f32) -> Self {
        Self {
            leading_min,
            trailing_min,
        }
    }

    pub fn clamp_ratio(self, ratio: f32, available: f32) -> f32 {
        helpers::clamp_ratio(ratio, self, available)
    }
}

/// A two-pane splitter with app-owned ratio state.
///
/// `SplitPane` lays out exactly two children separated by a focusable grip. The
/// builder `ratio` is the single source of truth. Pointer drag, touch drag,
/// keyboard arrow adjustment, snap, and double-click reset emit candidate
/// ratios through [`Self::on_change`]. The pane only resizes after the app feeds
/// the emitted value back into [`Self::ratio`]. It defaults to
/// [`ControlSize::Sm`]; the selected control size derives the visible grip and
/// resize target while the visual and layout divider remain one logical pixel.
pub struct SplitPane<'a, Message> {
    leading: Element<'a, Message>,
    trailing: Element<'a, Message>,
    orientation: Orientation,
    ratio: f32,
    constraints: SplitPaneConstraints,
    on_change: Option<Box<dyn Fn(f32) -> Message + 'a>>,
    locked: bool,
    snap: Option<SnapConfig>,
    id: Option<Id>,
    handle_role: SurfaceRole,
    size: ControlSize,
    width: Length,
    height: Length,
}

impl<'a, Message> SplitPane<'a, Message>
where
    Message: 'a,
{
    pub fn new(
        leading: impl Into<Element<'a, Message>>,
        trailing: impl Into<Element<'a, Message>>,
    ) -> Self {
        Self {
            leading: leading.into(),
            trailing: trailing.into(),
            orientation: Orientation::Horizontal,
            ratio: 0.5,
            constraints: SplitPaneConstraints::new(0.0, 0.0),
            on_change: None,
            locked: false,
            snap: None,
            id: None,
            handle_role: SurfaceRole::Canvas,
            size: ControlSize::Sm,
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    /// Sets the splitter orientation.
    ///
    /// [`Orientation::Horizontal`] lays panes out side by side. Use
    /// [`Orientation::Vertical`] for stacked top/bottom panes.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Lays panes out side by side.
    pub fn horizontal(self) -> Self {
        self.orientation(Orientation::Horizontal)
    }

    /// Lays panes out top to bottom.
    pub fn vertical(self) -> Self {
        self.orientation(Orientation::Vertical)
    }

    /// Sets the app-owned leading-pane ratio.
    pub fn ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio;
        self
    }

    /// Sets the minimum leading and trailing pane sizes in pixels.
    pub fn min_sizes(mut self, leading_min: f32, trailing_min: f32) -> Self {
        self.constraints = SplitPaneConstraints::new(leading_min, trailing_min);
        self
    }

    /// Sets the ratio constraints.
    pub fn constraints(mut self, constraints: SplitPaneConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    /// Emits app-owned ratio updates from drag, touch, keyboard, and reset.
    ///
    /// Without this callback, interactive input is normalized and focusable, but
    /// no ratio message is emitted and the pane does not resize.
    pub fn on_change(mut self, message: impl Fn(f32) -> Message + 'a) -> Self {
        self.on_change = Some(Box::new(message));
        self
    }

    /// Conditionally emits app-owned ratio updates.
    pub fn on_change_maybe(mut self, message: Option<impl Fn(f32) -> Message + 'a>) -> Self {
        self.on_change = message.map(|message| Box::new(message) as _);
        self
    }

    /// Prevents interactive resize when set.
    pub fn locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }

    /// Enables snapping to the default points `0.25`, `0.5`, and `0.75`.
    pub fn snap(mut self, threshold: f32) -> Self {
        self.snap = Some(SnapConfig::new(threshold, vec![0.25, 0.5, 0.75]));
        self
    }

    /// Enables snapping to custom ratio points.
    pub fn snap_with_points(mut self, threshold: f32, points: Vec<f32>) -> Self {
        self.snap = Some(SnapConfig::new(threshold, points));
        self
    }

    /// Sets an id for app-driven focus operations.
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn handle_role(mut self, role: SurfaceRole) -> Self {
        self.handle_role = role;
        self
    }

    /// Sets the control size used to derive the splitter grip and hit target.
    ///
    /// This does not expose raw pixel metrics or change the one-pixel visual
    /// and layout divider. The default is [`ControlSize::Sm`].
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Uses the extra-small splitter size.
    pub fn xs(self) -> Self {
        self.size(ControlSize::Xs)
    }

    /// Uses the small splitter size.
    pub fn sm(self) -> Self {
        self.size(ControlSize::Sm)
    }

    /// Uses the medium splitter size.
    pub fn md(self) -> Self {
        self.size(ControlSize::Md)
    }

    /// Uses the large splitter size.
    pub fn lg(self) -> Self {
        self.size(ControlSize::Lg)
    }

    crate::impl_layout_builders!(
        width_direct,
        height_direct,
        fill_width_direct,
        fill_height_direct,
        fill_direct
    );
}

impl<'a, Message> From<SplitPane<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(split_pane: SplitPane<'a, Message>) -> Self {
        Element::new(split_pane)
    }
}
