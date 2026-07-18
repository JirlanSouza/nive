//! Controlled document/view tab collection widget.
//!
//! `TabBar` is id-based and controlled: applications own tab order, active id,
//! dirty state, pin state, and close policy. The widget owns only ephemeral UI
//! state such as scroll offset, pointer gestures, the active drag session, and
//! the insertion-preview target.
//!
//! Tab identity is required at construction ([`TabItem::new(id, label)`]) and
//! flows through every interaction payload. Active tab state is app-owned:
//! feeding a new active id back to the widget reveals the active tab inside the
//! scroll viewport without emitting a domain message.
//!
//! Interaction intents are enabled by bar-level callbacks. Presence of a
//! callback enables the corresponding capability; absence disables it. Close
//! affordances render only when [`TabBar::on_close_request`] is present, even
//! if items are marked [`TabItem::closable`]. Reorder and tear-off only fire
//! when their respective callbacks are present.
//!
//! Pinned tabs render as a leading group while preserving relative order
//! within each partition. Reorder hit-testing runs [`crate::interaction::linear_insertion`]
//! per pinned or unpinned segment, so a pinned tab cannot enter the unpinned
//! area and vice versa.
//!
//! Overflow is scroll-based: the strip is wrapped in a horizontal clip and the
//! application of `scroll_offset` shifts the visible tab strip
//! left by that many pixels. Chevrons appear when measured content overflow is
//! detected and disappear again when no overflow remains. An all-tabs trigger
//! opens a dropdown overlay listing every `TabItem` (visible or not).
//!
//! Reorder and tear-off drives go through the shared gesture-driven
//! [`DragSession`]; TabBar is its first consumer. The dragged tab is tracked by
//! id, not by position index, so an app-applied reorder between `DragStarted`
//! and `DragReleased` cannot address the wrong tab. Tear-off payloads carry
//! the source-window-relative release position; spawning the new window
//! remains app/runtime work.
//!
//! ```

//! use nive_ui::widgets::{TabBar, TabItem};
//!
//! #[derive(Clone, Debug, PartialEq, Eq)]
//! enum TabId {
//!     Overview,
//!     Details,
//! }
//!
//! #[derive(Clone, Debug)]
//! enum Message {
//!     Select(TabId),
//! }
//!
//! let tabs = [TabId::Overview, TabId::Details];
//! let bar: nive_ui::Element<'_, Message> = TabBar::new(TabId::Overview)
//!     .tabs(tabs.iter().cloned().map(|id| {
//!         TabItem::new(id, "Tab")
//!     }))
//!     .on_select(Message::Select)
//!     .into();
//! ```
//!
//! [`DragSession`]: crate::interaction::dnd

use std::{borrow::Cow, time::Duration};

use iced::{
    advanced::{
        layout::{self, Layout, Node},
        mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Renderer as _, Shell, Widget,
    },
    keyboard,
    widget::{container, stack, text, Row, Space},
    window, Alignment, Event, Length, Padding, Point, Rectangle, Shadow, Size, Vector,
};

use crate::advanced::pressable::{draw_focus_ring_with_placement, FocusRingPlacement};
use crate::interaction::dnd::{DragSession, DragSessionFeedback, DragSessionOutcome};
use crate::interaction::{
    CollectionTransferPayload, ContextInvocation, ContextPosition, ContextRequest, ContextTarget,
    Drag, DropDecision, LinearInsertion, Orientation, PointerButton, PointerGestureKind,
    PointerGestureState, SelectionSnapshot, TransferData, TransferOperation, TransferOperations,
};
use crate::theme::{ControlSize, SurfaceRole};
use crate::Element;

use self::style as theme_tabs;
use super::overflow::{wheel_delta, Overflow, OverflowAxis, OverflowDirection};

mod style;
#[cfg(test)]
mod widget_tests;
use crate::widgets::controls::button::{self, ButtonFocusRing, GroupedItemKind, GroupedItemSpec};
use crate::widgets::navigation::dropdown_menu::{DropdownMenu, DropdownMenuItem};
use crate::widgets::overlays::popover::{
    PopoverCollision, PopoverOverlay, PopoverPlacement, PopoverWidth,
};
use crate::widgets::overlays::tooltip as tooltip_widget;
use crate::widgets::overlays::TooltipScope;
use crate::widgets::primitives::{icon as icon_widget, IconRole};

type SelectCallback<'a, Id, Message> = Box<dyn Fn(Id) -> Message + 'a>;
type CloseCallback<'a, Id, Message> = Box<dyn Fn(TabCloseRequest<Id>) -> Message + 'a>;
type ContextCallback<'a, Id, Message> = Box<dyn Fn(ContextRequest<Id>) -> Message + 'a>;
type ReorderCallback<'a, Id, Message> = Box<dyn Fn(TabDrop<Id>) -> Message + 'a>;
type TearOffCallback<'a, Id, Message> = Box<dyn Fn(TabTearOff<Id>) -> Message + 'a>;

const TEAR_OFF_HYSTERESIS: f32 = 24.0;
const CHEVRON_SCROLL_STEP_FACTOR: f32 = 0.8;
const INSERTION_MARKER_WIDTH: f32 = 2.0;
const HIDDEN_AFFORDANCE_WIDTH: f32 = 0.1;

/// Controlled tab strip keyed by stable tab IDs.
pub struct TabBar<'a, Id, Message> {
    active: Option<Id>,
    tabs: Vec<TabItem<'a, Id>>,
    size: ControlSize,
    role: SurfaceRole,
    active_role: SurfaceRole,
    width: Option<Length>,
    on_select: Option<SelectCallback<'a, Id, Message>>,
    on_close_request: Option<CloseCallback<'a, Id, Message>>,
    on_context: Option<ContextCallback<'a, Id, Message>>,
    on_reorder: Option<ReorderCallback<'a, Id, Message>>,
    on_tear_off: Option<TearOffCallback<'a, Id, Message>>,
    /// Cached menu element used by the all-tabs dropdown overlay.
    menu: Element<'a, MenuMessage<Id>>,
}

/// Data for one tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabItem<'a, Id> {
    id: Id,
    label: Cow<'a, str>,
    icon: Option<IconRole>,
    dirty: bool,
    pinned: bool,
    closable: bool,
    disabled: bool,
    tooltip: Option<Cow<'a, str>>,
}

/// Close request emitted by `TabBar`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TabCloseRequest<Id> {
    /// Tab being requested for close.
    pub id: Id,
    /// Input that caused the request.
    pub trigger: TabCloseTrigger,
}

/// Input source for a close request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TabCloseTrigger {
    /// Visible close affordance.
    CloseButton,
    /// Middle pointer click.
    MiddleClick,
}

/// Reorder drop request emitted by `TabBar`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TabDrop<Id> {
    /// Singleton dragged-tab payload.
    pub payload: CollectionTransferPayload<Id>,
    /// Legal insertion target.
    pub target: TabDropTarget<Id>,
    /// Effective transfer operation.
    pub operation: TransferOperation,
}

/// Tab reorder insertion target.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TabDropTarget<Id> {
    /// Insert before the target tab.
    Before(Id),
    /// Insert after the target tab.
    After(Id),
}

/// Tear-off request emitted by `TabBar`.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct TabTearOff<Id> {
    /// Singleton dragged-tab payload.
    pub payload: CollectionTransferPayload<Id>,
    /// Release position in source-window coordinates.
    pub position: Point,
}

#[derive(Debug, Clone)]
struct TabBarState<Id: Clone + Eq + 'static> {
    overflow: Overflow,
    scroll_offset: f32,
    max_scroll: f32,
    content_width: f32,
    strip_width: f32,
    has_overflow: bool,
    last_active_id: Option<Id>,
    dragged_id: Option<Id>,
    insertion_target: Option<TabDropTarget<Id>>,
    tab_bounds: Vec<(Id, Rectangle, bool)>,
    close_bounds: Vec<(Id, Rectangle)>,
    hovered_id: Option<Id>,
    focused_id: Option<Id>,
    previous_focus_order: Vec<Id>,
    focused: bool,
    pressed_id: Option<Id>,
    invalid_target: bool,
    edge_scroll: Option<OverflowDirection>,
    last_redraw: Option<iced::time::Instant>,
    strip_bounds: Option<Rectangle>,
    left_chevron: Option<Rectangle>,
    right_chevron: Option<Rectangle>,
    all_tabs_button: Option<Rectangle>,
    menu_open: bool,
    gestures: PointerGestureState<TabRegion>,
    drag_session: DragSession<CollectionTransferPayload<Id>, (), TabDropTarget<Id>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TabRegion {
    Tab(usize),
    Close(usize),
    ChevronLeft,
    ChevronRight,
    AllTabsButton,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusMovement {
    Previous,
    Next,
    First,
    Last,
}

#[derive(Debug, Clone)]
struct DisplayedTab<'a, 'b, Id> {
    #[allow(dead_code)]
    original_index: usize,
    item: &'a TabItem<'b, Id>,
}

#[cfg(test)]
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct DisplayTab<Id> {
    original_index: usize,
    id: Id,
    pinned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AllTabsMenuEntry<'a, Id> {
    id: Id,
    label: Cow<'a, str>,
    icon: Option<IconRole>,
    active: bool,
    dirty: bool,
    pinned: bool,
    disabled: bool,
}

/// Internal messages produced by the all-tabs dropdown overlay.
#[derive(Debug, Clone)]
enum MenuMessage<Id> {
    Select(Id),
    Dismiss,
}

impl<'a, Id, Message> TabBar<'a, Id, Message>
where
    Id: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    /// Builds a controlled tab bar with the active tab id.
    pub fn new(active: impl Into<Option<Id>>) -> Self {
        Self {
            active: active.into(),
            tabs: Vec::new(),
            size: ControlSize::Sm,
            role: SurfaceRole::Chrome,
            active_role: SurfaceRole::Canvas,
            width: None,
            on_select: None,
            on_close_request: None,
            on_context: None,
            on_reorder: None,
            on_tear_off: None,
            menu: iced::widget::Space::new().into(),
        }
    }

    /// Replaces the app-owned active tab id.
    pub fn active(mut self, active: impl Into<Option<Id>>) -> Self {
        self.active = active.into();
        self.menu = self.build_menu();
        self
    }

    /// Replaces all tabs from an iterator.
    pub fn tabs(mut self, tabs: impl IntoIterator<Item = TabItem<'a, Id>>) -> Self {
        self.tabs = tabs.into_iter().collect();
        self.menu = self.build_menu();
        self
    }

    /// Adds one tab as a small-builder convenience.
    pub fn push(mut self, tab: TabItem<'a, Id>) -> Self {
        self.tabs.push(tab);
        self.menu = self.build_menu();
        self
    }

    /// Adds one tab as a small-builder convenience.
    pub fn tab(self, tab: TabItem<'a, Id>) -> Self {
        self.push(tab)
    }

    /// Maps tab selection into app messages.
    pub fn on_select(mut self, f: impl Fn(Id) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    /// Conditionally maps tab selection into app messages.
    pub fn on_select_maybe(mut self, f: Option<impl Fn(Id) -> Message + 'a>) -> Self {
        self.on_select = f.map(|f| Box::new(f) as SelectCallback<'a, Id, Message>);
        self
    }

    /// Maps close requests into app messages.
    ///
    /// Close affordances and middle-click close are disabled when this callback
    /// is absent, even if items are marked [`TabItem::closable`].
    pub fn on_close_request(mut self, f: impl Fn(TabCloseRequest<Id>) -> Message + 'a) -> Self {
        self.on_close_request = Some(Box::new(f));
        self
    }

    /// Conditionally maps close requests into app messages.
    pub fn on_close_request_maybe(
        mut self,
        f: Option<impl Fn(TabCloseRequest<Id>) -> Message + 'a>,
    ) -> Self {
        self.on_close_request = f.map(|f| Box::new(f) as CloseCallback<'a, Id, Message>);
        self
    }

    /// Maps context requests into app messages.
    pub fn on_context(mut self, f: impl Fn(ContextRequest<Id>) -> Message + 'a) -> Self {
        self.on_context = Some(Box::new(f));
        self
    }

    /// Conditionally maps context requests into app messages.
    pub fn on_context_maybe(
        mut self,
        f: Option<impl Fn(ContextRequest<Id>) -> Message + 'a>,
    ) -> Self {
        self.on_context = f.map(|f| Box::new(f) as ContextCallback<'a, Id, Message>);
        self
    }

    /// Maps legal reorder drops into app messages.
    pub fn on_reorder(mut self, f: impl Fn(TabDrop<Id>) -> Message + 'a) -> Self {
        self.on_reorder = Some(Box::new(f));
        self
    }

    /// Conditionally maps legal reorder drops into app messages.
    pub fn on_reorder_maybe(mut self, f: Option<impl Fn(TabDrop<Id>) -> Message + 'a>) -> Self {
        self.on_reorder = f.map(|f| Box::new(f) as ReorderCallback<'a, Id, Message>);
        self
    }

    /// Maps tear-off requests into app messages.
    ///
    /// The position is source-window relative. Spawning or positioning a new
    /// window remains app/runtime work.
    pub fn on_tear_off(mut self, f: impl Fn(TabTearOff<Id>) -> Message + 'a) -> Self {
        self.on_tear_off = Some(Box::new(f));
        self
    }

    /// Conditionally maps tear-off requests into app messages.
    pub fn on_tear_off_maybe(mut self, f: Option<impl Fn(TabTearOff<Id>) -> Message + 'a>) -> Self {
        self.on_tear_off = f.map(|f| Box::new(f) as TearOffCallback<'a, Id, Message>);
        self
    }

    /// Sets the control size.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn xs(self) -> Self {
        self.size(ControlSize::Xs)
    }

    pub fn sm(self) -> Self {
        self.size(ControlSize::Sm)
    }

    pub fn md(self) -> Self {
        self.size(ControlSize::Md)
    }

    pub fn lg(self) -> Self {
        self.size(ControlSize::Lg)
    }

    /// Sets the surface role.
    pub fn role(mut self, role: SurfaceRole) -> Self {
        self.role = role;
        self
    }

    /// Sets the adjacent-content surface painted inside the active tab.
    ///
    /// The default is [`SurfaceRole::Canvas`]. The paint remains contained by
    /// the tab and does not create another host surface or structural seam.
    pub fn active_role(mut self, role: SurfaceRole) -> Self {
        self.active_role = role;
        self
    }

    crate::impl_layout_builders!(width_opt, fill_width_opt, shrink_width_opt);

    fn displayed_tabs<'b>(&'b self) -> Vec<DisplayedTab<'b, 'a, Id>> {
        let mut pinned = Vec::new();
        let mut unpinned = Vec::new();

        for (original_index, item) in self.tabs.iter().enumerate() {
            let displayed = DisplayedTab {
                original_index,
                item,
            };

            if item.pinned {
                pinned.push(displayed);
            } else {
                unpinned.push(displayed);
            }
        }

        pinned.extend(unpinned);
        pinned
    }

    fn content_element(&self, state: &TabBarState<Id>) -> Element<'_, Message> {
        let metrics = theme_tabs::metrics(self.size);
        let visible = self.displayed_tabs();
        let close_enabled = self.on_close_request.is_some();

        // Always reserve fixed slots in the outer row for chevrons and the
        // all-tabs trigger so the layout tree keeps a stable index for the
        // scrollable strip. Hidden affordances become zero-width spacers.
        let left_chevron = self.chevron_button(
            metrics,
            IconRole::NiveDisclosureLeft,
            state.overflow.show_start_chevron(),
            state.has_overflow,
            "Scroll tabs toward start",
        );
        let right_chevron = self.chevron_button(
            metrics,
            IconRole::NiveDisclosureRight,
            state.overflow.show_end_chevron(),
            state.has_overflow,
            "Scroll tabs toward end",
        );
        let all_tabs_button = self.chevron_button(
            metrics,
            IconRole::ViewMore,
            state.has_overflow,
            state.has_overflow,
            "Show all tabs",
        );

        let mut tabs_row = Row::new()
            .spacing(metrics.tab_gap)
            .align_y(Alignment::Center)
            .width(Length::Shrink)
            .height(Length::Fixed(metrics.tab_height));
        for displayed in visible {
            tabs_row = tabs_row.push(self.tab_element(displayed, metrics, close_enabled, state));
        }

        // Wrap the tab strip in a horizontal clip so the scroll offset exposes
        // only the visible viewport. The translation is applied during
        // `Widget::layout` once the natural tab bounds are known.
        let strip = container(tabs_row)
            .width(Length::Fill)
            .height(Length::Fixed(metrics.tab_height))
            .clip(true);

        let bar_row = Row::new()
            .spacing(0.0)
            .align_y(Alignment::Center)
            .push(left_chevron)
            .push(strip)
            .push(right_chevron)
            .push(all_tabs_button);

        let mut bar = container(bar_row)
            .style(theme_tabs::bar_style(self.role))
            .padding(
                Padding::ZERO
                    .vertical(metrics.bar_padding_v)
                    .horizontal(metrics.bar_padding_h),
            )
            .height(Length::Fixed(metrics.height));

        if let Some(width) = self.width {
            bar = bar.width(width);
        }

        TooltipScope::new(bar).into()
    }

    fn chevron_button(
        &self,
        metrics: theme_tabs::TabBarMetrics,
        role: IconRole,
        actionable: bool,
        reserve: bool,
        tooltip: &'static str,
    ) -> Element<'_, Message> {
        button::Button::custom(
            icon_widget::role(role)
                .custom_size(metrics.icon_size)
                .color_maybe((!actionable).then_some(iced::Color::TRANSPARENT))
                .into(),
        )
        .disabled(!actionable)
        .tooltip(tooltip)
        .width(Length::Fixed(if reserve {
            metrics.close_side
        } else {
            HIDDEN_AFFORDANCE_WIDTH
        }))
        .into_grouped_item(GroupedItemSpec {
            size: metrics.size,
            radius: metrics.radius.into(),
            height: metrics.tab_height,
            padding_h: 0.0,
            selected: false,
            destructive: false,
            kind: GroupedItemKind::Embedded,
        })
    }

    fn tab_element<'b>(
        &'b self,
        displayed: DisplayedTab<'b, 'a, Id>,
        metrics: theme_tabs::TabBarMetrics,
        close_enabled: bool,
        state: &TabBarState<Id>,
    ) -> Element<'b, Message> {
        let tab = displayed.item;
        let selected = self.active.as_ref().is_some_and(|active| active == &tab.id);
        let has_close = tab.closable && close_enabled;
        let show_close = has_close
            && (selected
                || state.hovered_id.as_ref().is_some_and(|id| id == &tab.id)
                || state.focused_id.as_ref().is_some_and(|id| id == &tab.id));
        let mut content = self.main_content(tab, metrics);
        let status_side = if has_close {
            metrics.close_side
        } else {
            metrics.status_side
        };

        let close = container(
            icon_widget::role(IconRole::WindowClose)
                .custom_size(metrics.close_icon_size)
                .color_maybe((!show_close).then_some(iced::Color::TRANSPARENT)),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill);
        let dirty = container(Space::new())
            .style(theme_tabs::status_indicator_style(
                metrics.dirty_size,
                tab.dirty && !show_close,
            ))
            .width(Length::Fixed(metrics.dirty_size))
            .height(Length::Fixed(metrics.dirty_size))
            .center_x(Length::Fill)
            .center_y(Length::Fill);
        let status: Element<'_, Message> = container(stack![dirty, close])
            .width(Length::Fixed(status_side))
            .height(Length::Fixed(metrics.tab_height))
            .into();
        content = content.push(status);

        let content: Element<'_, Message> = container(content)
            .style(theme_tabs::tab_content_style(selected, tab.disabled))
            .padding(Padding::ZERO.horizontal(metrics.padding_h))
            .height(Length::Fixed(metrics.tab_height))
            .clip(true)
            .into();

        match tab.tooltip.as_deref() {
            Some(label) => tooltip_widget::Tooltip::new(content, label).into(),
            None => content,
        }
    }

    fn main_content<'b>(
        &'b self,
        tab: &'b TabItem<'a, Id>,
        metrics: theme_tabs::TabBarMetrics,
    ) -> Row<'b, Message, crate::theme::Theme, iced::Renderer> {
        let label = text(tab.label.as_ref())
            .size(metrics.font_size)
            .shaping(text::Shaping::Auto)
            .wrapping(text::Wrapping::None);
        let mut content = Row::new()
            .spacing(metrics.gap)
            .align_y(Alignment::Center)
            .height(Length::Fill)
            .width(Length::Shrink);

        if let Some(icon) = tab.icon {
            content = content.push(icon_widget::role(icon).custom_size(metrics.icon_size));
        }

        if tab.pinned {
            content =
                content.push(icon_widget::role(IconRole::TabPinned).custom_size(metrics.icon_size));
        }

        content = content.push(label);
        content
    }

    fn menu_entries(&self) -> Vec<AllTabsMenuEntry<'a, Id>> {
        self.displayed_tabs()
            .into_iter()
            .map(|displayed| {
                let tab = displayed.item;
                AllTabsMenuEntry {
                    id: tab.id.clone(),
                    label: tab.label.clone(),
                    icon: tab.icon,
                    active: self.active.as_ref().is_some_and(|active| active == &tab.id),
                    dirty: tab.dirty,
                    pinned: tab.pinned,
                    disabled: tab.disabled,
                }
            })
            .collect()
    }

    /// Returns the dropdown overlay content tree for the all-tabs menu.
    fn build_menu(&self) -> Element<'a, MenuMessage<Id>> {
        let mut menu = DropdownMenu::<'a, MenuMessage<Id>>::new();
        for entry in self.menu_entries() {
            let mut item = DropdownMenuItem::new(entry.label)
                .selected(entry.active)
                .disabled(entry.disabled)
                .on_press_maybe((!entry.disabled).then(|| MenuMessage::Select(entry.id)));
            if let Some(icon) = entry.icon.or(entry.pinned.then_some(IconRole::TabPinned)) {
                item = item.icon(icon);
            }
            if entry.dirty {
                item = item.trailing("●");
            }
            menu = menu.push(item);
        }
        menu.into()
    }

    fn context_request(&self, region: TabRegion, position: Point) -> Option<ContextRequest<Id>> {
        match region {
            TabRegion::Tab(display_index) => {
                let displayed = self.displayed_tabs();
                let displayed = displayed.get(display_index)?;
                let tab = displayed.item;
                Some(ContextRequest {
                    target: ContextTarget::Item(tab.id.clone()),
                    selection: SelectionSnapshot {
                        selected: vec![tab.id.clone()],
                        focused: Some(tab.id.clone()),
                        anchor: Some(tab.id.clone()),
                    },
                    position: ContextPosition::Pointer(position),
                    invocation: ContextInvocation::SecondaryClick,
                })
            }
            TabRegion::Empty => Some(ContextRequest {
                target: ContextTarget::Empty,
                selection: SelectionSnapshot::default(),
                position: ContextPosition::Pointer(position),
                invocation: ContextInvocation::SecondaryClick,
            }),
            _ => None,
        }
    }

    fn close_request(&self, region: TabRegion) -> Option<TabCloseRequest<Id>> {
        let TabRegion::Tab(display_index) = region else {
            return None;
        };
        let displayed = self.displayed_tabs();
        let displayed = displayed.get(display_index)?;
        let tab = displayed.item;

        (tab.closable && self.on_close_request.is_some()).then(|| TabCloseRequest {
            id: tab.id.clone(),
            trigger: TabCloseTrigger::MiddleClick,
        })
    }

    fn close_button_request(&self, close_index: usize) -> Option<TabCloseRequest<Id>> {
        let tab = self
            .displayed_tabs()
            .into_iter()
            .filter(|displayed| {
                displayed.item.closable
                    && !displayed.item.disabled
                    && self.on_close_request.is_some()
            })
            .nth(close_index)?
            .item;

        Some(TabCloseRequest {
            id: tab.id.clone(),
            trigger: TabCloseTrigger::CloseButton,
        })
    }

    fn enabled_focus_order(&self) -> Vec<Id> {
        self.displayed_tabs()
            .into_iter()
            .filter(|displayed| !displayed.item.disabled)
            .map(|displayed| displayed.item.id.clone())
            .collect()
    }

    fn reconcile_focus(&self, state: &mut TabBarState<Id>) {
        let enabled = self.enabled_focus_order();
        let focused_is_valid = state
            .focused_id
            .as_ref()
            .is_some_and(|focused| enabled.contains(focused));

        if !focused_is_valid {
            state.focused_id = self
                .active
                .as_ref()
                .filter(|active| enabled.contains(active))
                .cloned()
                .or_else(|| {
                    state.focused_id.as_ref().and_then(|removed| {
                        let old_index = state
                            .previous_focus_order
                            .iter()
                            .position(|id| id == removed)?;
                        enabled
                            .get(old_index.min(enabled.len().saturating_sub(1)))
                            .cloned()
                    })
                })
                .or_else(|| enabled.first().cloned());
        }

        state.previous_focus_order = enabled;
    }

    fn move_focus(&self, state: &mut TabBarState<Id>, movement: FocusMovement) {
        let enabled = self.enabled_focus_order();
        if enabled.is_empty() {
            state.focused_id = None;
            return;
        }
        let current = state
            .focused_id
            .as_ref()
            .and_then(|focused| enabled.iter().position(|id| id == focused))
            .unwrap_or(0);
        let target = match movement {
            FocusMovement::Previous => current.saturating_sub(1),
            FocusMovement::Next => (current + 1).min(enabled.len() - 1),
            FocusMovement::First => 0,
            FocusMovement::Last => enabled.len() - 1,
        };
        state.focused_id = Some(enabled[target].clone());
    }

    /// Probe a per-segment reorder decision for the dragged tab id at `pointer`.
    fn reorder_decision(
        &self,
        dragged_id: Id,
        pointer: Point,
        tab_bounds: &[(Id, Rectangle, bool)],
    ) -> DropDecision<TabDropTarget<Id>> {
        let pinned = self
            .tabs
            .iter()
            .find(|tab| tab.id == dragged_id)
            .map(|tab| tab.pinned)
            .unwrap_or(false);

        let segment: Vec<(Id, Rectangle)> = tab_bounds
            .iter()
            .filter(|(_, _, item_pinned)| *item_pinned == pinned)
            .map(|(id, bounds, _)| (id.clone(), *bounds))
            .collect();

        let pointer_main = Orientation::Horizontal.main_position(pointer);
        let other_segment_exists = tab_bounds
            .iter()
            .any(|(_, _, item_pinned)| *item_pinned != pinned);

        if other_segment_exists {
            if pinned {
                let segment_end = segment
                    .iter()
                    .map(|(_, bounds)| {
                        Orientation::Horizontal.main_position(bounds.position())
                            + Orientation::Horizontal.main_length(bounds.size())
                    })
                    .fold(f32::MIN, f32::max);
                if pointer_main > segment_end {
                    return DropDecision::Reject;
                }
            } else {
                let segment_start = segment
                    .iter()
                    .map(|(_, bounds)| Orientation::Horizontal.main_position(bounds.position()))
                    .fold(f32::MAX, f32::min);
                if pointer_main < segment_start {
                    return DropDecision::Reject;
                }
            }
        }

        let Some(insertion) =
            crate::interaction::linear_insertion(Orientation::Horizontal, pointer, segment.clone())
        else {
            return DropDecision::Reject;
        };

        let target = match insertion {
            LinearInsertion::Before(id) => TabDropTarget::Before(id),
            LinearInsertion::After(id) => TabDropTarget::After(id),
        };

        DropDecision::accept(target, TransferOperation::Move)
    }
}

fn snapshot_tab_region<Id>(
    tab_bounds: &[(Id, Rectangle, bool)],
    close_bounds: &[(Id, Rectangle)],
    left_chevron: Option<Rectangle>,
    right_chevron: Option<Rectangle>,
    all_tabs_button: Option<Rectangle>,
    position: Point,
) -> TabRegion {
    if let Some(bounds) = left_chevron {
        if bounds.contains(position) {
            return TabRegion::ChevronLeft;
        }
    }
    if let Some(bounds) = right_chevron {
        if bounds.contains(position) {
            return TabRegion::ChevronRight;
        }
    }
    if let Some(bounds) = all_tabs_button {
        if bounds.contains(position) {
            return TabRegion::AllTabsButton;
        }
    }
    for (index, (_, bounds)) in close_bounds.iter().enumerate() {
        if bounds.contains(position) {
            return TabRegion::Close(index);
        }
    }
    for (index, (_, bounds, _)) in tab_bounds.iter().enumerate() {
        if bounds.contains(position) {
            return TabRegion::Tab(index);
        }
    }
    TabRegion::Empty
}

#[derive(Debug, Clone)]
struct HitGeometry<Id> {
    tab_bounds: Vec<(Id, Rectangle, bool)>,
    close_bounds: Vec<(Id, Rectangle)>,
    left_chevron: Option<Rectangle>,
    right_chevron: Option<Rectangle>,
    all_tabs_button: Option<Rectangle>,
    strip_bounds: Option<Rectangle>,
}

impl<Id> Default for HitGeometry<Id> {
    fn default() -> Self {
        Self {
            tab_bounds: Vec::new(),
            close_bounds: Vec::new(),
            left_chevron: None,
            right_chevron: None,
            all_tabs_button: None,
            strip_bounds: None,
        }
    }
}

fn hit_geometry<Id: Clone + Eq>(
    layout: Layout<'_>,
    displayed: &[DisplayedTab<'_, '_, Id>],
    close_enabled: bool,
    close_side: f32,
) -> HitGeometry<Id> {
    let Some(bar_row) = layout.children().next() else {
        return HitGeometry::default();
    };

    let mut bar_children = bar_row.children();
    let left_chevron = bar_children.next();
    let strip = bar_children.next();
    let right_chevron = bar_children.next();
    let all_tabs_button = bar_children.next();

    let tab_bounds: Vec<(Id, Rectangle, bool)> = strip
        .and_then(|strip| strip.children().next())
        .map(|tabs_row| {
            tabs_row
                .children()
                .enumerate()
                .filter_map(|(index, tab_layout)| {
                    let displayed = displayed.get(index)?;
                    Some((
                        displayed.item.id.clone(),
                        tab_layout.bounds(),
                        displayed.item.pinned,
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    let close_bounds = tab_bounds
        .iter()
        .filter_map(|(id, bounds, _)| {
            let item = displayed
                .iter()
                .find(|displayed| displayed.item.id == *id)?
                .item;
            (close_enabled && item.closable && !item.disabled).then_some((
                id.clone(),
                Rectangle {
                    x: bounds.x + bounds.width - close_side,
                    y: bounds.y,
                    width: close_side,
                    height: bounds.height,
                },
            ))
        })
        .collect();

    HitGeometry {
        tab_bounds,
        close_bounds,
        left_chevron: visible_slot_bounds(left_chevron),
        right_chevron: visible_slot_bounds(right_chevron),
        all_tabs_button: visible_slot_bounds(all_tabs_button),
        strip_bounds: strip.map(|layout| layout.bounds()),
    }
}

fn visible_slot_bounds(layout: Option<Layout<'_>>) -> Option<Rectangle> {
    layout
        .map(|layout| layout.bounds())
        .filter(|bounds| bounds.width > 0.5)
}

fn edge_scroll_direction(
    pointer: Point,
    strip: Rectangle,
    zone: f32,
    offset: f32,
    max_offset: f32,
) -> Option<OverflowDirection> {
    if !strip.contains(pointer) {
        return None;
    }
    if pointer.x <= strip.x + zone && offset > 0.0 {
        Some(OverflowDirection::Backward)
    } else if pointer.x >= strip.x + strip.width - zone && offset < max_offset {
        Some(OverflowDirection::Forward)
    } else {
        None
    }
}

fn autoscroll_step(direction: OverflowDirection, elapsed: Duration) -> f32 {
    let distance = 360.0 * elapsed.min(Duration::from_millis(50)).as_secs_f32();
    match direction {
        OverflowDirection::Backward => -distance,
        OverflowDirection::Forward => distance,
    }
}

impl<'a, Id, Message> Default for TabBar<'a, Id, Message>
where
    Id: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    fn default() -> Self {
        Self::new(None)
    }
}

impl<Id: Clone + Eq + 'static> Default for TabBarState<Id> {
    fn default() -> Self {
        Self {
            overflow: Overflow::default(),
            scroll_offset: 0.0,
            max_scroll: 0.0,
            content_width: 0.0,
            strip_width: 0.0,
            has_overflow: false,
            last_active_id: None,
            dragged_id: None,
            insertion_target: None,
            tab_bounds: Vec::new(),
            close_bounds: Vec::new(),
            hovered_id: None,
            focused_id: None,
            previous_focus_order: Vec::new(),
            focused: false,
            pressed_id: None,
            invalid_target: false,
            edge_scroll: None,
            last_redraw: None,
            strip_bounds: None,
            left_chevron: None,
            right_chevron: None,
            all_tabs_button: None,
            menu_open: false,
            gestures: PointerGestureState::default(),
            drag_session: DragSession::default(),
        }
    }
}

impl<Id: Clone + Eq + 'static> operation::Focusable for TabBarState<Id> {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
        self.pressed_id = None;
    }
}

impl<'a, Id> TabItem<'a, Id> {
    /// Creates tab data with required identity and label.
    pub fn new(id: Id, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id,
            label: label.into(),
            icon: None,
            dirty: false,
            pinned: false,
            closable: false,
            disabled: false,
            tooltip: None,
        }
    }

    /// Returns the tab id.
    pub fn id(&self) -> &Id {
        &self.id
    }

    /// Returns the tab label.
    pub fn label(&self) -> &str {
        self.label.as_ref()
    }

    pub fn icon(mut self, icon: IconRole) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    pub fn pinned(mut self, pinned: bool) -> Self {
        self.pinned = pinned;
        self
    }

    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
}

impl<'a, Id, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for TabBar<'a, Id, Message>
where
    Id: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TabBarState<Id>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TabBarState::<Id>::default())
    }

    fn children(&self) -> Vec<Tree> {
        let state = TabBarState::<Id>::default();
        vec![
            Tree::new(self.content_element(&state)),
            Tree::new(&self.menu),
        ]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_ref::<TabBarState<Id>>();
        let content = self.content_element(state);

        if tree.children.is_empty() {
            tree.children.push(Tree::new(&content));
            tree.children.push(Tree::new(&self.menu));
        } else {
            tree.children[0].diff(content.as_widget());
            if tree.children.len() > 1 {
                tree.children[1].diff(self.menu.as_widget());
            } else {
                tree.children.push(Tree::new(&self.menu));
            }
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width.unwrap_or(Length::Shrink), Length::Shrink)
    }

    fn size_hint(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_ref::<TabBarState<Id>>();
        let mut content = self.content_element(state);
        let node = content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        // Walk the layout tree to find the strip container and apply the scroll
        // translation. The bar container has a single Row child whose children
        // are: [left_chevron_slot, strip_container, right_chevron_slot,
        // all_tabs_button_slot].
        let metrics = theme_tabs::metrics(self.size);
        let min_tab_width = metrics.min_tab_width;
        let (content_width, strip_width, translated_node, viewport_tab_bounds) =
            measure_and_translate(
                node,
                state.scroll_offset,
                min_tab_width,
                metrics.max_tab_width,
                metrics.tab_gap,
            );

        let state = tree.state.downcast_mut::<TabBarState<Id>>();

        state.overflow.offset = state.scroll_offset;
        state.overflow.update_extents(content_width, strip_width);
        state.content_width = state.overflow.content_extent;
        state.strip_width = state.overflow.viewport_extent;
        state.max_scroll = state.overflow.max_offset;
        state.has_overflow = state.overflow.has_overflow;

        // Auto-reveal the active tab when it changed outside the visible
        // viewport. Minimum displacement: scroll just enough to reveal it.
        let active_changed = self.active != state.last_active_id;
        if active_changed {
            state.last_active_id = self.active.clone();
            if let Some(active) = &state.last_active_id {
                let displayed = self.displayed_tabs();
                let display_index = displayed
                    .iter()
                    .position(|displayed| &displayed.item.id == active);
                if let Some(bounds) = display_index.and_then(|index| viewport_tab_bounds.get(index))
                {
                    if bounds.x < 0.0 {
                        state.overflow.offset += bounds.x;
                    } else if bounds.x + bounds.width > strip_width {
                        state.overflow.offset += bounds.x + bounds.width - strip_width;
                    }
                }
            }
        }
        state.overflow.clamp_offset();
        state.scroll_offset = state.overflow.offset;
        self.reconcile_focus(state);

        translated_node
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<TabBarState<Id>>();
        operation.focusable(None, layout.bounds(), state);
        let state = tree.state.downcast_ref::<TabBarState<Id>>();
        let mut content = self.content_element(state);
        content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        {
            let state = tree.state.downcast_ref::<TabBarState<Id>>();
            let mut content = self.content_element(state);
            content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        if shell.is_event_captured() && !matches!(event, Event::Mouse(_) | Event::Touch(_)) {
            return;
        }

        let displayed = self.displayed_tabs();
        let metrics = theme_tabs::metrics(self.size);
        let hit_geometry = hit_geometry(
            layout,
            &displayed,
            self.on_close_request.is_some(),
            metrics.close_side,
        );
        let state = tree.state.downcast_mut::<TabBarState<Id>>();
        state.tab_bounds = hit_geometry.tab_bounds;
        state.close_bounds = hit_geometry.close_bounds;
        state.left_chevron = hit_geometry.left_chevron;
        state.right_chevron = hit_geometry.right_chevron;
        state.all_tabs_button = hit_geometry.all_tabs_button;
        state.strip_bounds = hit_geometry.strip_bounds;
        state.hovered_id = cursor.position().and_then(|position| {
            state
                .tab_bounds
                .iter()
                .find(|(_, bounds, _)| bounds.contains(position))
                .map(|(id, _, _)| id.clone())
        });
        self.reconcile_focus(state);

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            if state.dragged_id.is_some() {
                if let Some(direction) = state.edge_scroll {
                    let elapsed = state
                        .last_redraw
                        .map_or(Duration::ZERO, |last| now.saturating_duration_since(last));
                    state.last_redraw = Some(*now);
                    state.overflow.offset = state.scroll_offset;
                    let step = autoscroll_step(direction, elapsed);
                    state.overflow.offset =
                        (state.overflow.offset + step).clamp(0.0, state.max_scroll);
                    state.scroll_offset = state.overflow.offset;
                    shell.invalidate_layout();
                    shell.request_redraw();
                    return;
                }
            }
        }

        if matches!(event, Event::Window(window::Event::Unfocused)) && state.dragged_id.is_some() {
            state.dragged_id = None;
            state.insertion_target = None;
            state.invalid_target = false;
            state.edge_scroll = None;
            state.last_redraw = None;
            state.drag_session.cancel();
            shell.request_redraw();
            return;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.pressed_id = state.hovered_id.clone();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Mouse(mouse::Event::CursorLeft) => state.pressed_id = None,
            _ => {}
        }

        if state.focused {
            if let Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(named),
                repeat: false,
                ..
            }) = event
            {
                let movement = match named {
                    keyboard::key::Named::ArrowLeft => Some(FocusMovement::Previous),
                    keyboard::key::Named::ArrowRight => Some(FocusMovement::Next),
                    keyboard::key::Named::Home => Some(FocusMovement::First),
                    keyboard::key::Named::End => Some(FocusMovement::Last),
                    _ => None,
                };

                if let Some(movement) = movement {
                    self.move_focus(state, movement);
                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();
                    return;
                }

                if matches!(
                    named,
                    keyboard::key::Named::Enter | keyboard::key::Named::Space
                ) {
                    if let (Some(on_select), Some(focused)) = (&self.on_select, &state.focused_id) {
                        shell.publish(on_select(focused.clone()));
                        shell.capture_event();
                        shell.request_redraw();
                    }
                    return;
                }
            }
        }

        if let Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) = event {
            if !state.has_overflow {
                return;
            }
            let delta_x = wheel_delta(OverflowAxis::Horizontal, *delta);
            state.overflow.offset = state.scroll_offset;
            state.overflow.scroll_by(delta_x);
            state.scroll_offset = state.overflow.offset;
            if delta_x != 0.0 {
                shell.invalidate_layout();
                shell.request_redraw();
                shell.capture_event();
            }
            return;
        }

        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Escape),
            ..
        }) = event
        {
            if state.dragged_id.is_some() {
                state.dragged_id = None;
                state.insertion_target = None;
                state.pressed_id = None;
                state.invalid_target = false;
                state.edge_scroll = None;
                state.last_redraw = None;
                state.drag_session.cancel();
                shell.request_redraw();
                shell.capture_event();
                return;
            }
        }

        // Esc closes the open all-tabs menu.
        if state.menu_open {
            if let Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) = event
            {
                state.menu_open = false;
                shell.request_redraw();
                shell.capture_event();
                return;
            }
        }

        let bounds_for_gestures = bounds;
        // Snapshot the data needed by `tab_region_at` so the closure does not
        // borrow `state` (which `handle_event` mutably borrows through the
        // gesture state).
        let tab_bounds = state.tab_bounds.clone();
        let close_bounds = state.close_bounds.clone();
        let left_chevron = state.left_chevron;
        let right_chevron = state.right_chevron;
        let all_tabs_button = state.all_tabs_button;
        let gestures = state
            .gestures
            .handle_event(event, std::time::Instant::now(), |point| {
                if bounds_for_gestures.contains(point) {
                    Some(snapshot_tab_region(
                        &tab_bounds,
                        &close_bounds,
                        left_chevron,
                        right_chevron,
                        all_tabs_button,
                        point,
                    ))
                } else {
                    None
                }
            });

        // We cannot borrow state mutably while gestures still borrow it; clone
        // the gestures out so we can mutate state underneath.
        for gesture in gestures {
            let region = gesture.region;
            match (gesture.button, gesture.kind, region) {
                (PointerButton::Secondary, PointerGestureKind::Clicked { .. }, region) => {
                    if let Some(on_context) = &self.on_context {
                        if let Some(request) = self.context_request(region, gesture.position) {
                            shell.publish(on_context(request));
                            shell.capture_event();
                        }
                    }
                }
                (PointerButton::Middle, PointerGestureKind::Clicked { .. }, region) => {
                    if let Some(on_close) = &self.on_close_request {
                        if let Some(request) = self.close_request(region) {
                            shell.publish(on_close(request));
                            shell.capture_event();
                        }
                    }
                }
                (
                    PointerButton::Primary,
                    PointerGestureKind::Clicked { .. },
                    TabRegion::Close(close_index),
                ) => {
                    if let (Some(on_close), Some(request)) = (
                        &self.on_close_request,
                        self.close_button_request(close_index),
                    ) {
                        shell.publish(on_close(request));
                        shell.capture_event();
                    }
                }
                (
                    PointerButton::Primary,
                    PointerGestureKind::Clicked { .. },
                    TabRegion::Tab(display_index),
                ) => {
                    if let Some(tab) = self
                        .displayed_tabs()
                        .get(display_index)
                        .map(|item| item.item)
                    {
                        if !tab.disabled {
                            state.focused_id = Some(tab.id.clone());
                            if let Some(on_select) = &self.on_select {
                                shell.publish(on_select(tab.id.clone()));
                                shell.capture_event();
                            }
                        }
                    }
                }
                (
                    PointerButton::Primary,
                    PointerGestureKind::Clicked { .. },
                    TabRegion::ChevronLeft,
                ) if state.has_overflow => {
                    state.overflow.offset = state.scroll_offset;
                    state
                        .overflow
                        .page_step(OverflowDirection::Backward, CHEVRON_SCROLL_STEP_FACTOR);
                    state.scroll_offset = state.overflow.offset;
                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();
                }
                (
                    PointerButton::Primary,
                    PointerGestureKind::Clicked { .. },
                    TabRegion::ChevronRight,
                ) if state.has_overflow => {
                    state.overflow.offset = state.scroll_offset;
                    state
                        .overflow
                        .page_step(OverflowDirection::Forward, CHEVRON_SCROLL_STEP_FACTOR);
                    state.scroll_offset = state.overflow.offset;
                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();
                }
                (
                    PointerButton::Primary,
                    PointerGestureKind::Clicked { .. },
                    TabRegion::AllTabsButton,
                ) => {
                    state.menu_open = true;
                    shell.request_redraw();
                    shell.capture_event();
                }
                (
                    PointerButton::Primary,
                    PointerGestureKind::DragStarted,
                    TabRegion::Tab(display_index),
                ) if self.on_reorder.is_some() => {
                    let displayed_tabs = self.displayed_tabs();
                    let Some(displayed) = displayed_tabs.get(display_index) else {
                        continue;
                    };
                    if displayed.item.disabled {
                        continue;
                    }
                    let dragged_id = displayed.item.id.clone();
                    state.dragged_id = Some(dragged_id.clone());

                    let outcome = state.drag_session.handle_gesture(
                        &gesture_to_pointer(&gesture, TabRegion::Tab(display_index)),
                        || {
                            Some(Drag::<CollectionTransferPayload<Id>, ()> {
                                payload: TransferData::local(singleton_payload(dragged_id.clone())),
                                origin: (),
                                operations: TransferOperations::MOVE,
                                preferred: TransferOperation::Move,
                            })
                        },
                        |_context| DropDecision::<TabDropTarget<Id>>::Reject,
                    );

                    if let DragSessionOutcome::Feedback(_) = outcome {
                        // Drag has started; first move event will probe targets.
                    }
                }
                (PointerButton::Primary, PointerGestureKind::DragMoved, _) => {
                    let Some(dragged_id) = state.dragged_id.clone() else {
                        continue;
                    };
                    let tab_bounds = state.tab_bounds.clone();
                    let mut probed_target: Option<TabDropTarget<Id>> = None;
                    let outcome = state.drag_session.handle_gesture(
                        &gesture_to_pointer(&gesture, TabRegion::Empty),
                        || None,
                        |context| {
                            let decision =
                                if context.preferred_operation() == Some(TransferOperation::Move) {
                                    self.reorder_decision(
                                        dragged_id.clone(),
                                        context.position,
                                        &tab_bounds,
                                    )
                                } else {
                                    DropDecision::<TabDropTarget<Id>>::Reject
                                };
                            probed_target = match &decision {
                                DropDecision::Accept { target, .. } => Some(target.clone()),
                                _ => None,
                            };
                            decision
                        },
                    );

                    if let DragSessionOutcome::Feedback(feedback) = outcome {
                        let accepted = matches!(feedback, DragSessionFeedback::Accepted(_));
                        state.insertion_target = accepted.then_some(probed_target).flatten();
                        state.invalid_target = !accepted;
                        let direction = state.strip_bounds.and_then(|strip| {
                            edge_scroll_direction(
                                gesture.position,
                                strip,
                                theme_tabs::metrics(self.size).height,
                                state.scroll_offset,
                                state.max_scroll,
                            )
                        });
                        if direction != state.edge_scroll {
                            state.last_redraw = None;
                        }
                        state.edge_scroll = direction;
                        if direction.is_some() {
                            shell.request_redraw();
                        }
                        shell.request_redraw();
                    }
                }
                (PointerButton::Primary, PointerGestureKind::DragReleased, _) => {
                    let Some(dragged_id) = state.dragged_id.clone() else {
                        state.drag_session.cancel();
                        continue;
                    };
                    let strip_outer = Rectangle {
                        x: bounds.x - TEAR_OFF_HYSTERESIS,
                        y: bounds.y - TEAR_OFF_HYSTERESIS,
                        width: bounds.width + TEAR_OFF_HYSTERESIS * 2.0,
                        height: bounds.height + TEAR_OFF_HYSTERESIS * 2.0,
                    };

                    if self.on_tear_off.is_some() && !strip_outer.contains(gesture.position) {
                        let payload = singleton_payload(dragged_id.clone());
                        if let Some(on_tear_off) = &self.on_tear_off {
                            shell.publish(on_tear_off(TabTearOff {
                                payload,
                                position: gesture.position,
                            }));
                            shell.capture_event();
                        }
                        state.dragged_id = None;
                        state.insertion_target = None;
                        state.invalid_target = false;
                        state.edge_scroll = None;
                        state.last_redraw = None;
                        state.drag_session.cancel();
                        continue;
                    }

                    if !strip_outer.contains(gesture.position) {
                        state.dragged_id = None;
                        state.insertion_target = None;
                        state.invalid_target = false;
                        state.edge_scroll = None;
                        state.last_redraw = None;
                        state.drag_session.cancel();
                        continue;
                    }

                    // If dragged id is no longer present in tabs, silently end.
                    if !self.tabs.iter().any(|tab| tab.id == dragged_id) {
                        state.dragged_id = None;
                        state.insertion_target = None;
                        state.invalid_target = false;
                        state.edge_scroll = None;
                        state.last_redraw = None;
                        state.drag_session.cancel();
                        continue;
                    }

                    let tab_bounds = state.tab_bounds.clone();
                    let outcome = state.drag_session.handle_gesture(
                        &gesture_to_pointer(&gesture, TabRegion::Empty),
                        || None,
                        |context| {
                            self.reorder_decision(dragged_id.clone(), context.position, &tab_bounds)
                        },
                    );

                    if let DragSessionOutcome::Commit(Some(commit)) = outcome {
                        if let Some(on_reorder) = &self.on_reorder {
                            let payload = match &commit.payload {
                                TransferData::Local(payload) => payload.clone(),
                                _ => singleton_payload(dragged_id.clone()),
                            };
                            shell.publish(on_reorder(TabDrop {
                                payload,
                                target: commit.target,
                                operation: commit.operation,
                            }));
                            shell.capture_event();
                        }
                    }
                    state.dragged_id = None;
                    state.insertion_target = None;
                    state.invalid_target = false;
                    state.edge_scroll = None;
                    state.last_redraw = None;
                }
                (PointerButton::Primary, PointerGestureKind::DragCancelled, _) => {
                    state.dragged_id = None;
                    state.insertion_target = None;
                    state.invalid_target = false;
                    state.edge_scroll = None;
                    state.last_redraw = None;
                    state.drag_session.cancel();
                }
                _ => {}
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<TabBarState<Id>>();
        let content = self.content_element(state);
        let interaction = content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        );

        if state.dragged_id.is_some() {
            return state.drag_session.mouse_interaction();
        }

        if interaction != mouse::Interaction::None {
            return interaction;
        }

        mouse::Interaction::None
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<TabBarState<Id>>();
        let bounds = layout.bounds();
        let metrics = theme_tabs::metrics(self.size);
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: iced::Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            theme_tabs::strip_background(theme, self.role),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: bounds.y + bounds.height - metrics.seam_width,
                    width: bounds.width,
                    height: metrics.seam_width,
                },
                border: iced::Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            theme_tabs::strip_divider(theme, self.role),
        );
        for (id, tab_bounds, _) in &state.tab_bounds {
            let Some(tab) = self.tabs.iter().find(|tab| &tab.id == id) else {
                continue;
            };
            let selected = self.active.as_ref().is_some_and(|active| active == id);
            let hovered = state
                .hovered_id
                .as_ref()
                .is_some_and(|hovered| hovered == id);
            let pressed = state
                .pressed_id
                .as_ref()
                .is_some_and(|pressed| pressed == id);
            let background = theme_tabs::tab_background(
                theme,
                self.active_role,
                selected,
                hovered,
                pressed,
                tab.disabled,
            );
            if background.a > 0.0 {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: *tab_bounds,
                        border: iced::Border::default(),
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    background,
                );
            }
        }
        let content = self.content_element(state);
        content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );

        if let Some(dragged) = &state.dragged_id {
            if let Some((_, bounds, _)) = state.tab_bounds.iter().find(|(id, _, _)| id == dragged) {
                let mut subdued = theme_tabs::strip_background(theme, self.role);
                subdued.a = 0.45;
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: *bounds,
                        border: iced::Border::default(),
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    subdued,
                );
            }
        }

        if state.focused {
            if let Some(focused) = &state.focused_id {
                if let Some((_, bounds, _)) =
                    state.tab_bounds.iter().find(|(id, _, _)| id == focused)
                {
                    draw_focus_ring_with_placement(
                        renderer,
                        theme,
                        *bounds,
                        metrics.radius.into(),
                        ButtonFocusRing::Default,
                        FocusRingPlacement::Inset,
                    );
                }
            }
        }

        if let Some(active) = &self.active {
            if let Some((_, bounds, _)) = state.tab_bounds.iter().find(|(id, _, _)| id == active) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x,
                            y: bounds.y,
                            width: bounds.width,
                            height: metrics.indicator_width,
                        },
                        border: iced::Border::default(),
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    theme_tabs::active_indicator(theme),
                );
            }
        }

        if state.dragged_id.is_some() {
            if let Some(target) = &state.insertion_target {
                let metrics = theme_tabs::metrics(self.size);
                if let Some(marker) =
                    insertion_marker_bounds(target, &state.tab_bounds, metrics.tab_gap)
                {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: marker,
                            border: iced::Border::default().rounded(1.0),
                            shadow: Shadow::default(),
                            snap: true,
                        },
                        theme_tabs::insertion_marker_color(theme),
                    );
                }
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        let _ = (renderer, viewport);
        let anchor_bounds = translation_applied_bounds(layout.bounds(), translation);
        let trigger_for_anchor = tree.state.downcast_ref::<TabBarState<Id>>().all_tabs_button;
        let state = tree.state.downcast_mut::<TabBarState<Id>>();

        if !state.menu_open {
            return None;
        }

        let trigger_bounds = trigger_for_anchor
            .map(|bounds| Rectangle {
                x: bounds.x + translation.x,
                y: bounds.y + translation.y,
                width: bounds.width,
                height: bounds.height,
            })
            .unwrap_or(anchor_bounds);

        // The menu element is refreshed in `update` before `overlay` runs, so it
        // reflects the latest tab set.
        let on_select_ref = self.on_select.as_ref();
        let menu_state = &mut tree.children[1];
        let menu: &'b mut Element<'a, MenuMessage<Id>> = &mut self.menu;
        let on_dismiss_message = MenuMessage::<Id>::Dismiss;

        let on_message =
            move |message: MenuMessage<Id>, parent_shell: &mut Shell<'_, Message>| match message {
                MenuMessage::Select(id) => {
                    if let Some(on_select) = on_select_ref {
                        parent_shell.publish(on_select(id));
                    }
                    state.menu_open = false;
                    parent_shell.capture_event();
                    parent_shell.request_redraw();
                }
                MenuMessage::Dismiss => {
                    state.menu_open = false;
                    parent_shell.capture_event();
                    parent_shell.request_redraw();
                }
            };

        let overlay = PopoverOverlay::new(
            trigger_bounds,
            menu,
            menu_state,
            PopoverPlacement::BottomEnd,
            PopoverWidth::MatchAnchor,
            PopoverCollision::default(),
            0.0,
            Some(on_dismiss_message),
            on_message,
        )
        .on_key_press(|event| match event {
            keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            } => Some(MenuMessage::<Id>::Dismiss),
            _ => None,
        });

        Some(overlay::Element::new(Box::new(overlay)))
    }
}

impl<'a, Id, Message> From<TabBar<'a, Id, Message>> for Element<'a, Message>
where
    Id: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    fn from(tab_bar: TabBar<'a, Id, Message>) -> Self {
        Element::new(tab_bar)
    }
}

fn singleton_payload<Id: Clone>(id: Id) -> CollectionTransferPayload<Id> {
    CollectionTransferPayload::flat([id])
}

#[cfg(test)]
fn legal_reorder_target<Id>(
    dragged: &TabItem<'_, Id>,
    target: &TabDropTarget<Id>,
    tabs: &[TabItem<'_, Id>],
) -> bool
where
    Id: Eq,
{
    let target_pinned = match target {
        TabDropTarget::Before(id) | TabDropTarget::After(id) => tabs
            .iter()
            .find(|tab| &tab.id == id)
            .is_some_and(|tab| tab.pinned),
    };

    dragged.pinned == target_pinned
}

fn gesture_to_pointer<Region>(
    gesture: &crate::interaction::PointerGesture<Region>,
    region: Region,
) -> crate::interaction::PointerGesture<Region> {
    crate::interaction::PointerGesture {
        kind: gesture.kind,
        button: gesture.button,
        region,
        position: gesture.position,
        modifiers: gesture.modifiers,
    }
}

fn translation_applied_bounds(bounds: Rectangle, translation: Vector) -> Rectangle {
    Rectangle {
        x: bounds.x + translation.x,
        y: bounds.y + translation.y,
        width: bounds.width,
        height: bounds.height,
    }
}

/// Walks the freshly-laid-out content node to:
///  - measure total tab-strip content width and strip width,
///  - apply the scroll translation to the tab children,
///  - capture translated viewport-space tab bounds for active reveal.
fn measure_and_translate(
    node: Node,
    scroll_offset: f32,
    min_tab_width: f32,
    max_tab_width: f32,
    tab_gap: f32,
) -> (f32, f32, Node, Vec<Rectangle>) {
    let root_size = node.size();

    let Some(bar_row) = node.children().first() else {
        // Single-row constraint unsatisfied; return node unchanged.
        return (0.0, 0.0, node, Vec::new());
    };
    let Some(strip_container) = bar_row.children().get(1) else {
        return (0.0, 0.0, node, Vec::new());
    };

    let strip_width = strip_container.bounds().width;
    let Some(tabs_row) = strip_container.children().first() else {
        return (0.0, strip_width, node, Vec::new());
    };
    let row_x = tabs_row.bounds().x;
    let translate = Vector::new(-scroll_offset, 0.0);

    let mut viewport_tab_bounds = Vec::with_capacity(tabs_row.children().len());
    let mut translated_tabs: Vec<Node> = Vec::with_capacity(tabs_row.children().len());
    let mut next_x = row_x;
    for tab in tabs_row.children() {
        let tab_bounds = tab.bounds();
        let width = tab_bounds.width.clamp(min_tab_width, max_tab_width);
        let mut tab =
            Node::with_children(Size::new(width, tab_bounds.height), tab.children().to_vec())
                .move_to(Point::new(next_x, tab_bounds.y));
        tab.translate_mut(translate);
        viewport_tab_bounds.push(tab.bounds());
        translated_tabs.push(tab);
        next_x += width + tab_gap;
    }
    let content_width = if translated_tabs.is_empty() {
        0.0
    } else {
        next_x - tab_gap - row_x
    };

    let translated_tabs_row_size = Size::new(content_width, tabs_row.size().height);
    let translated_tabs_row_bounds = tabs_row.bounds();
    let translated_tabs_row = Node::with_children(translated_tabs_row_size, translated_tabs)
        .move_to(translated_tabs_row_bounds.position() + translate);

    let strip_container_size = strip_container.size();
    let strip_container_position = strip_container.bounds().position();
    let translated_strip_container =
        Node::with_children(strip_container_size, vec![translated_tabs_row])
            .move_to(strip_container_position);

    let new_bar_row_children: Vec<Node> = bar_row
        .children()
        .iter()
        .enumerate()
        .map(|(i, c)| {
            if i == 1 {
                translated_strip_container.clone()
            } else {
                c.clone()
            }
        })
        .collect();

    let bar_row_size = bar_row.size();
    let bar_row_position = bar_row.bounds().position();
    let new_bar_row =
        Node::with_children(bar_row_size, new_bar_row_children).move_to(bar_row_position);

    let _ = root_size;
    let new_root = Node::with_children(node.size(), vec![new_bar_row]);

    (content_width, strip_width, new_root, viewport_tab_bounds)
}

fn insertion_marker_bounds<Id: Eq>(
    target: &TabDropTarget<Id>,
    tab_bounds: &[(Id, Rectangle, bool)],
    gap: f32,
) -> Option<Rectangle> {
    let (target_id, before) = match target {
        TabDropTarget::Before(id) => (id, true),
        TabDropTarget::After(id) => (id, false),
    };
    let (_, bounds, _) = tab_bounds.iter().find(|(id, _, _)| id == target_id)?;
    let x = if before {
        bounds.x - (gap + INSERTION_MARKER_WIDTH) / 2.0
    } else {
        bounds.x + bounds.width + (gap - INSERTION_MARKER_WIDTH) / 2.0
    };

    Some(Rectangle {
        x,
        y: bounds.y,
        width: INSERTION_MARKER_WIDTH,
        height: bounds.height,
    })
}

#[cfg(test)]
#[allow(dead_code)]
fn translate_bounds(bounds: Rectangle, translation: Vector) -> Rectangle {
    Rectangle {
        x: bounds.x + translation.x,
        y: bounds.y + translation.y,
        width: bounds.width,
        height: bounds.height,
    }
}

#[cfg(test)]
mod tabs_tests {
    use super::*;

    #[allow(dead_code)]
    #[derive(Clone, Debug, PartialEq)]
    enum Message {
        Select(u8),
        Close(TabCloseRequest<u8>),
        Context(ContextRequest<u8>),
        Drop(TabDrop<u8>),
        Tear(TabTearOff<u8>),
    }

    fn item(id: u8) -> TabItem<'static, u8> {
        TabItem::new(id, "tab")
    }

    fn state() -> TabBarState<u8> {
        TabBarState::default()
    }

    #[test]
    fn item_requires_id_and_label_without_display_bound() {
        #[derive(Clone, Debug, PartialEq, Eq)]
        struct Id(u8);

        let item = TabItem::new(Id(1), "Overview");

        assert_eq!(item.id(), &Id(1));
        assert_eq!(item.label(), "Overview");
    }

    #[test]
    fn pinned_tabs_render_as_leading_partition() {
        let bar: TabBar<'_, u8, Message> =
            TabBar::new(1).tabs([item(1), item(2).pinned(true), item(3), item(4).pinned(true)]);

        let display: Vec<u8> = bar
            .displayed_tabs()
            .into_iter()
            .map(|d| d.item.id)
            .collect();

        assert_eq!(display, vec![2, 4, 1, 3]);
    }

    #[test]
    fn middle_click_close_requires_closable_item_and_callback() {
        let bar: TabBar<'_, u8, Message> = TabBar::new(1)
            .tabs([item(1).closable(true), item(2)])
            .on_close_request(Message::Close);

        assert_eq!(
            bar.close_request(TabRegion::Tab(0)),
            Some(TabCloseRequest {
                id: 1,
                trigger: TabCloseTrigger::MiddleClick
            })
        );
        assert_eq!(bar.close_request(TabRegion::Tab(1)), None);

        let disabled: TabBar<'_, u8, Message> = TabBar::new(1).tabs([item(1).closable(true)]);

        assert_eq!(disabled.close_request(TabRegion::Tab(0)), None);
    }

    #[test]
    fn context_request_uses_singleton_or_empty_snapshot() {
        let bar: TabBar<'_, u8, Message> = TabBar::new(1).tabs([item(1), item(2)]);

        let tab = bar
            .context_request(TabRegion::Tab(1), Point::new(10.0, 20.0))
            .expect("context");
        assert_eq!(tab.target, ContextTarget::Item(2));
        assert_eq!(tab.selection.selected, vec![2]);
        assert_eq!(tab.selection.focused, Some(2));

        let empty = bar
            .context_request(TabRegion::Empty, Point::new(30.0, 20.0))
            .expect("context");
        assert_eq!(empty.target, ContextTarget::Empty);
        assert!(empty.selection.selected.is_empty());
    }

    #[test]
    fn tab_pinned_icon_role_is_distinct_from_chevrons_and_menu() {
        assert!(IconRole::TabPinned != IconRole::NiveDisclosureLeft);
        assert!(IconRole::TabPinned != IconRole::NiveDisclosureRight);
        assert!(IconRole::TabPinned != IconRole::ViewMore);
        assert!(IconRole::TabPinned != IconRole::OpenMenu);
    }

    #[test]
    fn default_state_has_no_overflow() {
        let st = state();

        assert!(!st.has_overflow);
        assert_eq!(st.scroll_offset, 0.0);
        assert_eq!(st.max_scroll, 0.0);
        assert!(st.dragged_id.is_none());
        assert!(st.insertion_target.is_none());
        assert!(!st.menu_open);
    }

    #[test]
    fn singleton_payload_is_singleton() {
        let payload = singleton_payload(7_u8);

        assert_eq!(payload.ids, vec![7]);
        assert_eq!(payload.root_ids, vec![7]);
    }

    #[test]
    fn legal_reorder_target_rejects_cross_zone() {
        let tabs = [
            TabItem::new(1_u8, "A").pinned(true),
            TabItem::new(2_u8, "B"),
        ];
        let dragged_pinned = &tabs[0];
        let dragged_unpinned = &tabs[1];

        assert!(legal_reorder_target(
            dragged_pinned,
            &TabDropTarget::Before(1),
            &tabs
        ));
        assert!(!legal_reorder_target(
            dragged_unpinned,
            &TabDropTarget::Before(1),
            &tabs
        ));
        assert!(!legal_reorder_target(
            dragged_pinned,
            &TabDropTarget::After(2),
            &tabs
        ));
        assert!(legal_reorder_target(
            dragged_unpinned,
            &TabDropTarget::After(2),
            &tabs
        ));
    }

    #[test]
    fn tear_off_payload_shape_is_singleton() {
        let tear = TabTearOff {
            payload: singleton_payload(7),
            position: Point::new(100.0, 24.0),
        };

        assert_eq!(tear.payload.ids, vec![7]);
        assert_eq!(tear.payload.root_ids, vec![7]);
        assert_eq!(tear.position, Point::new(100.0, 24.0));
    }

    #[test]
    fn reorder_per_segment_rejects_cross_zone_pinned_drop() {
        let bar: TabBar<'_, u8, Message> = TabBar::new(1)
            .tabs([item(1).pinned(true), item(2)])
            .on_reorder(Message::Drop);

        let _state = TabBarState::<u8>::default();

        // Place pinned at x=0, unpinned at x=200. Pointer over the unpinned
        // segment but dragged tab is pinned.
        let tab_bounds = vec![
            (
                1_u8,
                Rectangle::new(Point::new(0.0, 0.0), Size::new(100.0, 40.0)),
                true,
            ),
            (
                2_u8,
                Rectangle::new(Point::new(200.0, 0.0), Size::new(100.0, 40.0)),
                false,
            ),
        ];

        // Drag the pinned tab and ask for a decision at the unpinned drop
        // position; must reject.
        let decision_pinned_over_unpinned =
            bar.reorder_decision(1, Point::new(220.0, 0.0), &tab_bounds);
        assert!(!decision_pinned_over_unpinned.is_accept());

        // Drag the unpinned tab and ask for a decision at the pinned drop
        // position; must reject.
        let decision_unpinned_over_pinned =
            bar.reorder_decision(2, Point::new(20.0, 0.0), &tab_bounds);
        assert!(!decision_unpinned_over_pinned.is_accept());

        // Same segment valid drops must accept.
        let decision_pinned_in_pinned = bar.reorder_decision(1, Point::new(20.0, 0.0), &tab_bounds);
        assert!(decision_pinned_in_pinned.is_accept());
        let decision_unpinned_in_unpinned =
            bar.reorder_decision(2, Point::new(220.0, 0.0), &tab_bounds);
        assert!(decision_unpinned_in_unpinned.is_accept());
    }

    #[test]
    fn drag_to_trailing_empty_space_accepts_after_last_same_segment() {
        let bar: TabBar<'_, u8, Message> = TabBar::new(1)
            .tabs([item(1), item(2), item(3)])
            .on_reorder(Message::Drop);
        let tab_bounds = vec![
            (
                1_u8,
                Rectangle::new(Point::new(0.0, 0.0), Size::new(50.0, 30.0)),
                false,
            ),
            (
                2_u8,
                Rectangle::new(Point::new(60.0, 0.0), Size::new(50.0, 30.0)),
                false,
            ),
            (
                3_u8,
                Rectangle::new(Point::new(120.0, 0.0), Size::new(50.0, 30.0)),
                false,
            ),
        ];

        assert_eq!(
            bar.reorder_decision(1, Point::new(220.0, 0.0), &tab_bounds),
            DropDecision::accept(TabDropTarget::After(3), TransferOperation::Move)
        );
    }

    #[test]
    fn insertion_marker_bounds_before_and_after_geometry() {
        let tab_bounds = vec![(
            7_u8,
            Rectangle::new(Point::new(20.0, 10.0), Size::new(80.0, 30.0)),
            false,
        )];

        assert_eq!(
            insertion_marker_bounds(&TabDropTarget::Before(7), &tab_bounds, 4.0),
            Some(Rectangle::new(
                Point::new(17.0, 10.0),
                Size::new(INSERTION_MARKER_WIDTH, 30.0)
            ))
        );
        assert_eq!(
            insertion_marker_bounds(&TabDropTarget::After(7), &tab_bounds, 4.0),
            Some(Rectangle::new(
                Point::new(101.0, 10.0),
                Size::new(INSERTION_MARKER_WIDTH, 30.0)
            ))
        );
    }

    #[test]
    fn menu_entries_reflect_active_set_after_tabs() {
        let entries = TabBar::<'_, u8, Message>::new(None)
            .tabs([item(1), item(2)])
            .active(2)
            .menu_entries();

        assert_eq!(
            entries,
            vec![
                AllTabsMenuEntry {
                    id: 1,
                    label: Cow::Borrowed("tab"),
                    icon: None,
                    active: false,
                    dirty: false,
                    pinned: false,
                    disabled: false,
                },
                AllTabsMenuEntry {
                    id: 2,
                    label: Cow::Borrowed("tab"),
                    icon: None,
                    active: true,
                    dirty: false,
                    pinned: false,
                    disabled: false,
                }
            ]
        );
    }

    #[test]
    fn menu_entries_keep_disabled_items_visible_and_inert() {
        let entries = TabBar::<'_, u8, Message>::new(1)
            .tabs([item(1), item(2).disabled(true)])
            .menu_entries();

        assert_eq!(entries.len(), 2);
        assert!(entries[0].active);
        assert!(entries[1].disabled);
    }

    #[test]
    fn composite_focus_starts_active_and_skips_disabled_items() {
        let bar: TabBar<'_, u8, Message> =
            TabBar::new(2).tabs([item(1), item(2), item(3).disabled(true), item(4)]);
        let mut state = state();

        bar.reconcile_focus(&mut state);
        assert_eq!(state.focused_id, Some(2));

        bar.move_focus(&mut state, FocusMovement::Next);
        assert_eq!(state.focused_id, Some(4));
        bar.move_focus(&mut state, FocusMovement::Last);
        assert_eq!(state.focused_id, Some(4));
        bar.move_focus(&mut state, FocusMovement::First);
        assert_eq!(state.focused_id, Some(1));
    }

    #[test]
    fn composite_focus_survives_reorder_by_id_and_recovers_after_removal() {
        let mut state = state();
        state.focused_id = Some(2);
        state.previous_focus_order = vec![1, 2, 3];

        let reordered: TabBar<'_, u8, Message> = TabBar::new(1).tabs([item(3), item(2), item(1)]);
        reordered.reconcile_focus(&mut state);
        assert_eq!(state.focused_id, Some(2));

        let removed: TabBar<'_, u8, Message> = TabBar::new(3).tabs([item(3), item(1)]);
        removed.reconcile_focus(&mut state);
        assert_eq!(state.focused_id, Some(3));
    }

    #[test]
    fn menu_entries_are_pinned_first_and_preserve_metadata() {
        let entries = TabBar::<'_, u8, Message>::new(2)
            .tabs([
                item(1).dirty(true),
                item(2).pinned(true).icon(IconRole::Folder),
                item(3).disabled(true),
            ])
            .menu_entries();

        assert_eq!(
            entries.iter().map(|entry| entry.id).collect::<Vec<_>>(),
            vec![2, 1, 3]
        );
        assert!(entries[0].active);
        assert!(entries[0].pinned);
        assert_eq!(entries[0].icon, Some(IconRole::Folder));
        assert!(entries[1].dirty);
        assert!(entries[2].disabled);
    }

    #[test]
    fn snapshot_tab_region_uses_cached_bounds() {
        let tab_bounds = vec![
            (
                1_u8,
                Rectangle::new(Point::new(10.0, 0.0), Size::new(50.0, 30.0)),
                false,
            ),
            (
                2_u8,
                Rectangle::new(Point::new(70.0, 0.0), Size::new(50.0, 30.0)),
                false,
            ),
        ];

        assert_eq!(
            snapshot_tab_region(&tab_bounds, &[], None, None, None, Point::new(20.0, 10.0)),
            TabRegion::Tab(0)
        );
        assert_eq!(
            snapshot_tab_region(&tab_bounds, &[], None, None, None, Point::new(80.0, 10.0)),
            TabRegion::Tab(1)
        );
        assert_eq!(
            snapshot_tab_region(&tab_bounds, &[], None, None, None, Point::new(200.0, 10.0)),
            TabRegion::Empty
        );
    }

    #[test]
    fn chevron_scroll_offsets_clamped() {
        let mut state = TabBarState::<u8>::default();
        state.overflow.update_extents(180.0, 100.0);
        state.overflow.offset = 5.0;

        state
            .overflow
            .page_step(OverflowDirection::Backward, CHEVRON_SCROLL_STEP_FACTOR);
        state.scroll_offset = state.overflow.offset;
        assert_eq!(state.scroll_offset, 0.0);
        assert!(!state.overflow.show_start_chevron());
        assert!(state.overflow.show_end_chevron());

        state
            .overflow
            .page_step(OverflowDirection::Forward, CHEVRON_SCROLL_STEP_FACTOR);
        state.scroll_offset = state.overflow.offset;
        assert_eq!(state.scroll_offset, 80.0);
        assert!(state.overflow.show_start_chevron());
        assert!(!state.overflow.show_end_chevron());
    }

    #[test]
    fn edge_autoscroll_direction_respects_zone_and_endpoints() {
        let strip = Rectangle::new(Point::new(10.0, 20.0), Size::new(200.0, 28.0));

        assert_eq!(
            edge_scroll_direction(Point::new(15.0, 30.0), strip, 28.0, 40.0, 100.0),
            Some(OverflowDirection::Backward)
        );
        assert_eq!(
            edge_scroll_direction(Point::new(205.0, 30.0), strip, 28.0, 40.0, 100.0),
            Some(OverflowDirection::Forward)
        );
        assert_eq!(
            edge_scroll_direction(Point::new(15.0, 30.0), strip, 28.0, 0.0, 100.0),
            None
        );
    }

    #[test]
    fn edge_autoscroll_step_clamps_suspended_frames_to_fifty_ms() {
        assert_eq!(
            autoscroll_step(OverflowDirection::Forward, Duration::from_millis(500)),
            18.0
        );
        assert_eq!(
            autoscroll_step(OverflowDirection::Backward, Duration::from_millis(50)),
            -18.0
        );
    }
}
