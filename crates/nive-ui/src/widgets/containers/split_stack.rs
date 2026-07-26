mod sizing;
mod state;
mod widget;

use iced::{widget::Id, Length};

use crate::interaction::Orientation;
use crate::theme::ControlSize;
use crate::Element;

pub use sizing::SplitSizing;

/// Logical pixels past a minimum that propose a collapse.
const DEFAULT_COLLAPSE_THRESHOLD: f32 = 32.0;

/// A divider move proposed by a [`SplitStack`].
///
/// A drag only ever moves the two panes bordering the divider, so the two new
/// lengths describe the change completely.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplitResize {
    /// Index of the divider that moved.
    pub divider: usize,
    /// New length of pane `divider`, in logical pixels.
    pub leading: f32,
    /// New length of pane `divider + 1`, in logical pixels.
    pub trailing: f32,
}

/// A collapse proposed by dragging a divider past its neighbour's minimum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplitCollapse {
    /// Index of the divider that was dragged.
    pub divider: usize,
    /// Index of the pane that should collapse.
    pub pane: usize,
    /// Length that pane held when the drag started, for restoring it later.
    pub restore: f32,
}

/// One pane of a [`SplitStack`].
pub struct SplitStackPane<'a, Message> {
    content: Element<'a, Message>,
    sizing: SplitSizing,
    minimum: f32,
    collapsible: bool,
}

impl<'a, Message> SplitStackPane<'a, Message> {
    /// Builds a pane that holds `size` logical pixels through container resizes.
    pub fn fixed(content: impl Into<Element<'a, Message>>, size: f32) -> Self {
        Self {
            content: content.into(),
            sizing: SplitSizing::Fixed(size),
            minimum: 0.0,
            collapsible: false,
        }
    }

    /// Builds the pane that absorbs whatever the fixed panes leave over.
    pub fn fill(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            sizing: SplitSizing::Fill,
            minimum: 0.0,
            collapsible: false,
        }
    }

    /// Sets the smallest length this pane may render at, in logical pixels.
    ///
    /// A neighbouring divider stops once this pane reaches it.
    pub fn minimum(mut self, minimum: f32) -> Self {
        self.minimum = minimum;
        self
    }

    /// Lets dragging the neighbouring divider past this pane's minimum collapse it.
    ///
    /// The stack also needs [`SplitStack::on_collapse`]; without it no pane
    /// collapses however far the drag insists.
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }
}

/// An N-pane splitter that owns one axis, with app-owned pixel sizes.
///
/// `SplitStack` lays out its panes along [`Self::orientation`], separated by
/// focusable dividers. Every pane is either [`SplitSizing::Fixed`] at a logical
/// pixel length or [`SplitSizing::Fill`], and exactly one pane fills; a stack
/// declaring none or several normalizes deterministically rather than failing.
///
/// Dragging a divider changes the two panes bordering it and nothing else, at
/// any position in its travel — panes further away are not part of the
/// computation. A divider stops once an adjacent pane reaches its minimum
/// instead of pushing the pane beyond it. Growing the container grows the
/// filling pane alone.
///
/// Sizes are app-owned: [`Self::on_resize`] proposes new lengths and the stack
/// only resizes once the app feeds them back. Prefer [`super::SplitPane`] for a
/// two-pane split whose panes should scale with their container instead.
pub struct SplitStack<'a, Message> {
    contents: Vec<Element<'a, Message>>,
    sizing: Vec<SplitSizing>,
    minimums: Vec<f32>,
    collapsible: Vec<bool>,
    orientation: Orientation,
    on_resize: Option<Box<dyn Fn(SplitResize) -> Message + 'a>>,
    on_collapse: Option<Box<dyn Fn(SplitCollapse) -> Message + 'a>>,
    collapse_threshold: f32,
    locked: bool,
    id: Option<Id>,
    size: ControlSize,
    width: Length,
    height: Length,
}

impl<'a, Message> SplitStack<'a, Message>
where
    Message: 'a,
{
    /// Builds an empty stack along `orientation`.
    pub fn new(orientation: Orientation) -> Self {
        Self {
            contents: Vec::new(),
            sizing: Vec::new(),
            minimums: Vec::new(),
            collapsible: Vec::new(),
            orientation,
            on_resize: None,
            on_collapse: None,
            collapse_threshold: DEFAULT_COLLAPSE_THRESHOLD,
            locked: false,
            id: None,
            size: ControlSize::Sm,
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    /// Builds an empty stack laying panes out side by side.
    pub fn horizontal() -> Self {
        Self::new(Orientation::Horizontal)
    }

    /// Builds an empty stack laying panes out top to bottom.
    pub fn vertical() -> Self {
        Self::new(Orientation::Vertical)
    }

    /// Sets the axis panes are laid out along.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Appends one pane after the panes already added.
    pub fn pane(mut self, pane: SplitStackPane<'a, Message>) -> Self {
        self.contents.push(pane.content);
        self.sizing.push(pane.sizing);
        self.minimums.push(pane.minimum);
        self.collapsible.push(pane.collapsible);
        self
    }

    /// Appends several panes in order.
    pub fn panes(self, panes: impl IntoIterator<Item = SplitStackPane<'a, Message>>) -> Self {
        panes.into_iter().fold(self, Self::pane)
    }

    /// Emits app-owned length updates from drag and keyboard adjustment.
    ///
    /// Without this callback the dividers are display-only: they are not
    /// focusable, claim no resize gesture, and present no resize affordance.
    pub fn on_resize(mut self, message: impl Fn(SplitResize) -> Message + 'a) -> Self {
        self.on_resize = Some(Box::new(message));
        self
    }

    /// Conditionally emits app-owned length updates.
    pub fn on_resize_maybe(
        mut self,
        message: Option<impl Fn(SplitResize) -> Message + 'a>,
    ) -> Self {
        self.on_resize = message.map(|message| Box::new(message) as _);
        self
    }

    /// Emits collapse requests when a divider is dragged past a neighbour's minimum.
    ///
    /// Only panes marked [`SplitStackPane::collapsible`] are ever proposed, and
    /// each drag proposes at most one collapse.
    pub fn on_collapse(mut self, message: impl Fn(SplitCollapse) -> Message + 'a) -> Self {
        self.on_collapse = Some(Box::new(message));
        self
    }

    /// Sets how far past a minimum a drag must travel to propose a collapse.
    ///
    /// Measured in logical pixels; the default is `32`.
    pub fn collapse_threshold(mut self, threshold: f32) -> Self {
        self.collapse_threshold = if threshold.is_finite() {
            threshold.max(0.0)
        } else {
            DEFAULT_COLLAPSE_THRESHOLD
        };
        self
    }

    /// Prevents interactive resize when set.
    pub fn locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }

    /// Sets an id for app-driven focus operations.
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the control size deriving the divider grip and hit target.
    ///
    /// This does not change the one-pixel visual and layout seam. The default is
    /// [`ControlSize::Sm`].
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Uses the extra-small divider size.
    pub fn xs(self) -> Self {
        self.size(ControlSize::Xs)
    }

    /// Uses the small divider size.
    pub fn sm(self) -> Self {
        self.size(ControlSize::Sm)
    }

    /// Uses the medium divider size.
    pub fn md(self) -> Self {
        self.size(ControlSize::Md)
    }

    /// Uses the large divider size.
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

impl<'a, Message> From<SplitStack<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(split_stack: SplitStack<'a, Message>) -> Self {
        Element::new(split_stack)
    }
}
