use iced::{
    advanced::{
        layout::Node,
        mouse,
        widget::{operation, Id, Tree},
    },
    Point, Rectangle, Size,
};

use crate::Element;

mod fixtures;
mod harness;
mod probes;

#[cfg(test)]
mod tests;

pub(crate) use harness::{event_messages, layout, pixel, rasterize, renderer};
pub(crate) use probes::{event_probe, named_probe};

pub(crate) struct WidgetHarness<'a, Message> {
    element: Element<'a, Message>,
    tree: Tree,
    node: Node,
    renderer: iced::Renderer,
    maximum: Size,
    cursor: mouse::Cursor,
    clipboard: MemoryClipboard,
}

mod focus;
pub(crate) use focus::*;

pub(crate) struct UpdateResult<Message> {
    pub(crate) messages: Vec<Message>,
    pub(crate) captured: bool,
    pub(crate) layout_invalid: bool,
    pub(crate) redraw_request: iced::window::RedrawRequest,
    pub(crate) input_method_enabled: bool,
}

#[derive(Debug, Default)]
pub(crate) struct MemoryClipboard {
    standard: Option<String>,
    primary: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FormStateFixture {
    pub(crate) hovered: bool,
    pub(crate) focused: bool,
    pub(crate) pressed: bool,
    pub(crate) read_only: bool,
    pub(crate) disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct FakeClock {
    now_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AnchoredGeometryFixture {
    pub(crate) anchor: Rectangle,
    pub(crate) viewport: Rectangle,
    pub(crate) intrinsic_content: Size,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct EnsureVisibleFixture {
    pub(crate) viewport: Rectangle,
    pub(crate) target: Rectangle,
    pub(crate) current_offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PopupStateFixture {
    pub(crate) capable: bool,
    pub(crate) disabled: bool,
    pub(crate) open: bool,
    pub(crate) selected: bool,
    pub(crate) highlighted: bool,
    pub(crate) focused: bool,
    pub(crate) pressed: bool,
}

struct NamedProbe<'a, Message> {
    id: Id,
    content: Element<'a, Message>,
}

struct BoundsProbe {
    id: Id,
    bounds: Option<Rectangle>,
}

struct EventProbe<Message> {
    message: Message,
}
