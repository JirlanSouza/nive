use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    alignment,
    widget::{column, container, mouse_area, row, stack, text},
    Alignment, Event, Length, Padding, Rectangle, Size, Vector,
};

pub use nive_core::{AnnouncementPoliteness, ToastPresentation, ToastTone};

use crate::theme::{
    self, surface as theme_surface, ShapeSize, SurfaceRole, TextRole, ToneRole, TypographyRole,
};
use crate::widgets::{controls::button, primitives::icon, IconRole};
use crate::{Element, Renderer, Theme};

/// Card maximum width; shrinks to fill minus clearance on narrow viewports.
const CARD_MAX_WIDTH: f32 = 360.0;
/// Clearance kept from each viewport edge on narrow layouts.
const NARROW_CLEARANCE: f32 = 16.0;
const CARD_PADDING: f32 = 12.0;
const CARD_GAP: f32 = 8.0;
const STACK_GAP: f32 = 8.0;
const ICON_SIZE: f32 = 16.0;
const TITLE_SIZE: f32 = 14.0;
const BODY_SIZE: f32 = 14.0;
const BODY_LINE_HEIGHT: f32 = 1.4;
const MAX_BODY_LINES: f32 = 3.0;
/// At most three toasts are ever rendered by a single host; the runtime is
/// responsible for keeping its visible set within this cap.
const MAX_VISIBLE: usize = 3;

/// Logical stacking corner. `Start`/`End` stay direction-ready ahead of a
/// full RTL resolver; rendering remains physical-LTR (`Start` = left,
/// `End` = right) until one exists.
///
/// The physical `TopLeft`/`TopRight`/`BottomLeft`/`BottomRight` variants are
/// removed with no facade:
///
/// ```compile_fail
/// use nive_ui::widgets::overlays::ToastPosition;
///
/// let _ = ToastPosition::TopLeft;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastPosition {
    TopStart,
    TopEnd,
    BottomStart,
    #[default]
    BottomEnd,
}

/// Safe-area clearance the host must keep the stack clear of (viewport edges,
/// window chrome such as a status bar), supplied by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ToastInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl ToastInsets {
    pub const NONE: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };
}

/// Overlays a toast stack on top of `content`.
///
/// ## Accessibility contract (preparatory)
///
/// No native AccessKit live-region emission exists in this Iced version, so
/// the points below are settled semantics for whoever wires that surface,
/// not yet-enforced runtime behavior:
///
/// - Announcement politeness follows [`ToastTone::announcement_politeness`].
/// - Only the newest toast is ever meant to announce — never the full stack
///   after a reorder. `toasts()` already receives items newest-first, so
///   the first item is that toast.
/// - The tone icon is decorative; the card owns the accessible name.
///
/// One point *is* enforced today: `ToastHost` never moves keyboard focus
/// into a toast on its own — a toast's dismiss/action buttons are reachable
/// by `Tab` like any other control, but appearing never steals focus.
pub struct ToastHost<'a, Message> {
    content: Element<'a, Message>,
    items: Vec<(u64, Element<'a, Message>)>,
    position: ToastPosition,
    insets: ToastInsets,
    on_pause: Option<Message>,
    on_resume: Option<Message>,
    on_focus_enter: Option<Message>,
    on_focus_exit: Option<Message>,
}

impl<'a, Message> ToastHost<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            items: Vec::new(),
            position: ToastPosition::default(),
            insets: ToastInsets::NONE,
            on_pause: None,
            on_resume: None,
            on_focus_enter: None,
            on_focus_exit: None,
        }
    }

    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    /// Sets host-provided safe insets the stack must stay clear of.
    pub fn safe_insets(mut self, insets: ToastInsets) -> Self {
        self.insets = insets;
        self
    }

    pub fn on_hover(mut self, pause: Message, resume: Message) -> Self {
        self.on_pause = Some(pause);
        self.on_resume = Some(resume);
        self
    }

    /// Pauses expiry while keyboard focus sits inside the toast stack (a
    /// Tab-focused dismiss or action button), mirroring `on_hover` for
    /// pointer users so a toast never disappears out from under someone
    /// mid-interaction.
    pub fn on_focus_within(mut self, enter: Message, exit: Message) -> Self {
        self.on_focus_enter = Some(enter);
        self.on_focus_exit = Some(exit);
        self
    }

    /// Supplies the toasts to render.
    ///
    /// `on_action` extracts an optional `(label, Message)` secondary action
    /// from the concrete item type; the neutral [`ToastPresentation`]
    /// contract itself never carries an application `Message` (the action
    /// lives on the runtime-owned typed toast, not on the core contract).
    pub fn toasts<T>(
        mut self,
        toasts: impl IntoIterator<Item = &'a T>,
        on_dismiss: impl Fn(T::Id) -> Message + Copy + 'a,
        on_action: impl Fn(&'a T) -> Option<(&'a str, Message)> + Copy + 'a,
    ) -> Self
    where
        T: ToastPresentation + 'a,
        T::Id: Hash,
    {
        self.items = toasts
            .into_iter()
            .take(MAX_VISIBLE)
            .map(|toast| {
                (
                    hash_toast_id(toast.id()),
                    toast_view(toast, on_dismiss, on_action(toast)),
                )
            })
            .collect();
        self
    }

    pub fn into_element(mut self) -> Element<'a, Message> {
        if self.items.is_empty() {
            return self.content;
        }

        // Newest item is always first in `self.items` (nearest the origin).
        // For a bottom origin the newest must land closest to the bottom
        // edge, so the visual column is built oldest-first; for a top
        // origin the given (newest-first) order already reads top-down.
        if is_bottom(self.position) {
            self.items.reverse();
        }

        // Keyed by toast id so a dismissal immediately backfilled by
        // promotion — the common case, since it leaves the visible count
        // unchanged — can't hand the vacated slot's live widget state (an
        // open Tooltip, mid-press interaction) to whatever different toast
        // now occupies it; a positionally diffed `column` would. This uses a
        // local `ToastStack`, not `iced::widget::keyed_column`: that widget's
        // own `diff()` only consults keys when the child count changes,
        // silently falling back to positional diffing for a same-length
        // swap — exactly the case a dismiss-then-promote always produces.
        let toast_stack = ToastStack::new(self.items)
            .spacing(STACK_GAP)
            .width(Length::Fill)
            .max_width(CARD_MAX_WIDTH);
        let toast_stack: Element<'a, Message> = match (self.on_focus_enter, self.on_focus_exit) {
            (Some(enter), Some(exit)) => FocusWithinArea::new(toast_stack)
                .on_focus_within(enter, exit)
                .into(),
            _ => toast_stack.into(),
        };
        let toast_stack: Element<'a, Message> = match (self.on_pause, self.on_resume) {
            (Some(pause), Some(resume)) => mouse_area(toast_stack)
                .on_enter(pause)
                .on_exit(resume)
                .into(),
            _ => toast_stack,
        };
        let (horizontal, vertical) = alignment_for(self.position);
        let overlay = container(toast_stack)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(horizontal)
            .align_y(vertical)
            .padding(overlay_padding(self.insets));

        stack(vec![self.content, overlay.into()])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl<'a, Message> From<ToastHost<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(host: ToastHost<'a, Message>) -> Self {
        host.into_element()
    }
}

fn hash_toast_id<Id: Hash>(id: Id) -> u64 {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    hasher.finish()
}

fn overlay_padding(insets: ToastInsets) -> Padding {
    Padding {
        top: insets.top.max(NARROW_CLEARANCE),
        right: insets.right.max(NARROW_CLEARANCE),
        bottom: insets.bottom.max(NARROW_CLEARANCE),
        left: insets.left.max(NARROW_CLEARANCE),
    }
}

fn toast_view<'a, Message, T>(
    toast: &'a T,
    on_dismiss: impl Fn(T::Id) -> Message + Copy + 'a,
    action: Option<(&'a str, Message)>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    T: ToastPresentation + 'a,
{
    let tone_role = toast_tone_role(toast.tone());
    let icon_color = theme::active().tone(tone_role).color;
    let tone_icon = icon::role(icon_role_for(toast.tone()))
        .custom_size(ICON_SIZE)
        .color(icon_color);

    let title = text(toast.title())
        .size(TITLE_SIZE)
        .font(theme::typography(TypographyRole::BodyStrong).font)
        .style(theme::text::style(TextRole::Primary))
        .shaping(text::Shaping::Auto);

    let mut text_column = column![title].spacing(CARD_GAP / 2.0);

    if let Some(body) = toast.body() {
        let body_max_height = BODY_SIZE * BODY_LINE_HEIGHT * MAX_BODY_LINES;
        text_column = text_column.push(
            container(
                text(body)
                    .size(BODY_SIZE)
                    .line_height(text::LineHeight::Relative(BODY_LINE_HEIGHT))
                    .style(theme::text::style(TextRole::Secondary))
                    .shaping(text::Shaping::Auto),
            )
            .max_height(body_max_height)
            .clip(true),
        );
    }

    let close = button::icon(IconRole::WindowClose, "Dismiss notification")
        .sm()
        .tooltip("Dismiss")
        .on_press(on_dismiss(toast.id()));

    let mut controls = row![].spacing(CARD_GAP);
    if let Some((label, message)) = action {
        controls = controls.push(button::ghost(label).sm().on_press(message));
    }
    controls = controls.push(close);

    let content = row![tone_icon, text_column.width(Length::Fill), controls]
        .spacing(CARD_GAP)
        .align_y(Alignment::Start)
        .width(Length::Fill);

    container(content)
        .style(theme_surface::style_with_radius(
            SurfaceRole::Elevated,
            theme::active().shape(ShapeSize::Lg).radius(),
        ))
        .padding(Padding::new(CARD_PADDING))
        .width(Length::Fill)
        .max_width(CARD_MAX_WIDTH)
        .into()
}

fn toast_tone_role(tone: ToastTone) -> ToneRole {
    match tone {
        ToastTone::Info => ToneRole::Info,
        ToastTone::Success => ToneRole::Success,
        ToastTone::Warning => ToneRole::Warning,
        ToastTone::Danger => ToneRole::Danger,
    }
}

fn icon_role_for(tone: ToastTone) -> IconRole {
    match tone {
        ToastTone::Info => IconRole::DialogInformation,
        ToastTone::Success => IconRole::DialogSuccess,
        ToastTone::Warning => IconRole::DialogWarning,
        ToastTone::Danger => IconRole::DialogError,
    }
}

fn is_bottom(position: ToastPosition) -> bool {
    matches!(
        position,
        ToastPosition::BottomStart | ToastPosition::BottomEnd
    )
}

fn alignment_for(position: ToastPosition) -> (alignment::Horizontal, alignment::Vertical) {
    match position {
        ToastPosition::TopStart => (alignment::Horizontal::Left, alignment::Vertical::Top),
        ToastPosition::TopEnd => (alignment::Horizontal::Right, alignment::Vertical::Top),
        ToastPosition::BottomStart => (alignment::Horizontal::Left, alignment::Vertical::Bottom),
        ToastPosition::BottomEnd => (alignment::Horizontal::Right, alignment::Vertical::Bottom),
    }
}

/// Vertical stack of toast cards keyed by toast id.
///
/// A near-clone of `iced::widget::keyed_column`'s internals (same flex
/// layout, same per-child delegation), except its `diff()` actually compares
/// every index's key, not just the first and last. Upstream's version uses
/// that boundary check purely to decide *whether the child count changed* —
/// when a dismiss is immediately backfilled by promotion (the common case,
/// since the visible count stays the same), the count never changes, so its
/// diff silently falls through to a plain positional zip and a different
/// toast inherits whatever live widget state (an open Tooltip, mid-press
/// interaction) the previous occupant of that slot left behind.
struct ToastStack<'a, Message> {
    keys: Vec<u64>,
    children: Vec<Element<'a, Message>>,
    spacing: f32,
    width: Length,
    max_width: f32,
}

#[derive(Default)]
struct ToastStackState {
    keys: Vec<u64>,
}

impl<'a, Message> ToastStack<'a, Message> {
    fn new(items: Vec<(u64, Element<'a, Message>)>) -> Self {
        let (keys, children) = items.into_iter().unzip();
        Self {
            keys,
            children,
            spacing: 0.0,
            width: Length::Shrink,
            max_width: f32::INFINITY,
        }
    }

    fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = max_width;
        self
    }
}

impl<'a, Message> From<ToastStack<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(stack: ToastStack<'a, Message>) -> Self {
        Element::new(stack)
    }
}

impl<Message> Widget<Message, Theme, Renderer> for ToastStack<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ToastStackState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(ToastStackState {
            keys: self.keys.clone(),
        })
    }

    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<ToastStackState>();

        if tree.children.len() != self.children.len() {
            tree.children = self.children.iter().map(Tree::new).collect();
        } else {
            for (index, (child, child_tree)) in self
                .children
                .iter()
                .zip(tree.children.iter_mut())
                .enumerate()
            {
                if state.keys.get(index) == Some(&self.keys[index]) {
                    child.as_widget().diff(child_tree);
                } else {
                    *child_tree = Tree::new(child.as_widget());
                }
            }
        }

        state.keys.clone_from(&self.keys);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.max_width(self.max_width).width(self.width);

        layout::flex::resolve(
            layout::flex::Axis::Vertical,
            renderer,
            &limits,
            self.width,
            Length::Shrink,
            Padding::ZERO,
            self.spacing,
            Alignment::Start,
            &mut self.children,
            &mut tree.children,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget_mut()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for ((child, tree), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((child, state), layout) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            child
                .as_widget()
                .draw(state, renderer, theme, style, layout, cursor, viewport);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(
            &mut self.children,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

/// Transparent wrapper that publishes `on_enter`/`on_exit` when the standard
/// Iced keyboard-focus state of any widget in `content` (tracked per-widget
/// via `operation::Focusable`, the same mechanism `Tab` navigation and
/// `nive-ui`'s `Pressable`-based buttons already use) starts or stops being
/// true — the keyboard-focus counterpart to `mouse_area`'s hover events.
///
/// Every method delegates straight through to `content` except `update`,
/// which re-checks focus state after delegating and diffs it against the
/// previous pass so the message fires only on a genuine transition.
struct FocusWithinArea<'a, Message> {
    content: Element<'a, Message>,
    on_enter: Option<Message>,
    on_exit: Option<Message>,
}

#[derive(Debug, Default)]
struct FocusWithinState {
    focused: bool,
}

impl<'a, Message> FocusWithinArea<'a, Message> {
    fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            on_enter: None,
            on_exit: None,
        }
    }

    fn on_focus_within(mut self, enter: Message, exit: Message) -> Self {
        self.on_enter = Some(enter);
        self.on_exit = Some(exit);
        self
    }
}

impl<'a, Message> From<FocusWithinArea<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(area: FocusWithinArea<'a, Message>) -> Self {
        Element::new(area)
    }
}

impl<Message> Widget<Message, Theme, Renderer> for FocusWithinArea<'_, Message>
where
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<FocusWithinState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(FocusWithinState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let focused = any_focused(
            self.content.as_widget_mut(),
            &mut tree.children[0],
            layout,
            renderer,
        );
        let state = tree.state.downcast_mut::<FocusWithinState>();
        if focused != state.focused {
            state.focused = focused;
            let message = if focused {
                self.on_enter.clone()
            } else {
                self.on_exit.clone()
            };
            if let Some(message) = message {
                shell.publish(message);
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

/// Scans `widget`'s subtree for any `operation::Focusable` reporting itself
/// focused — the same per-widget state Iced's own Tab navigation toggles, so
/// this reads it rather than tracking focus independently.
fn any_focused<Message>(
    widget: &mut dyn Widget<Message, Theme, Renderer>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
) -> bool {
    struct AnyFocused(bool);

    impl operation::Operation for AnyFocused {
        fn focusable(
            &mut self,
            _id: Option<&iced::advanced::widget::Id>,
            _bounds: Rectangle,
            state: &mut dyn operation::Focusable,
        ) {
            self.0 |= state.is_focused();
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
            operate(self);
        }
    }

    let mut probe = AnyFocused(false);
    widget.operate(tree, layout, renderer, &mut probe);
    probe.0
}

#[cfg(test)]
mod toast_host_tests {
    use super::*;

    #[test]
    fn default_position_is_bottom_end() {
        assert_eq!(ToastPosition::default(), ToastPosition::BottomEnd);
    }

    #[test]
    fn position_alignment_matches_logical_corner() {
        assert_eq!(
            alignment_for(ToastPosition::TopStart),
            (alignment::Horizontal::Left, alignment::Vertical::Top)
        );
        assert_eq!(
            alignment_for(ToastPosition::BottomEnd),
            (alignment::Horizontal::Right, alignment::Vertical::Bottom)
        );
    }

    #[test]
    fn overlay_padding_never_falls_below_narrow_clearance() {
        let padding = overlay_padding(ToastInsets::NONE);
        assert_eq!(padding.top, NARROW_CLEARANCE);
        assert_eq!(padding.left, NARROW_CLEARANCE);
    }

    #[test]
    fn overlay_padding_widens_for_larger_safe_insets() {
        let padding = overlay_padding(ToastInsets {
            top: 40.0,
            ..ToastInsets::NONE
        });
        assert_eq!(padding.top, 40.0);
        assert_eq!(padding.bottom, NARROW_CLEARANCE);
    }

    #[test]
    fn bottom_positions_are_recognized_for_inward_growth() {
        assert!(is_bottom(ToastPosition::BottomStart));
        assert!(is_bottom(ToastPosition::BottomEnd));
        assert!(!is_bottom(ToastPosition::TopStart));
        assert!(!is_bottom(ToastPosition::TopEnd));
    }

    /// `toasts()` renders through `ToastStack`, keyed by toast id, precisely
    /// so a same-length swap (a dismiss immediately backfilled by promotion)
    /// can't hand that slot's live widget state — an open Tooltip, mid-press
    /// interaction — to whichever different toast now occupies it. That
    /// guarantee only holds if the key is a stable function of toast
    /// identity: the same id must always key the same, and distinct ids
    /// must key apart.
    #[test]
    fn toast_id_hashes_are_stable_and_distinguish_identity() {
        assert_eq!(hash_toast_id(42_u64), hash_toast_id(42_u64));
        assert_ne!(hash_toast_id(1_u64), hash_toast_id(2_u64));
    }

    mod geometry {
        use super::*;
        use crate::test_support::WidgetHarness;

        #[derive(Clone, Copy)]
        struct FakeToast(u64);

        impl ToastPresentation for FakeToast {
            type Id = u64;

            fn id(&self) -> u64 {
                self.0
            }

            fn title(&self) -> &str {
                "Saved"
            }

            fn body(&self) -> Option<&str> {
                None
            }

            fn tone(&self) -> ToastTone {
                ToastTone::Info
            }
        }

        /// End-to-end companion to `push_keeps_newest_three_toasts` (which
        /// proves `ToastState` never *hands* the host more than three
        /// visible toasts): this proves the host's own `MAX_VISIBLE` cap
        /// holds even if a caller ignores that and hands it more anyway —
        /// each rendered card contributes exactly one focusable dismiss
        /// button, so counting those counts cards.
        #[test]
        fn host_never_renders_more_than_three_cards_even_if_handed_more() {
            let toasts: Vec<FakeToast> = (0..5).map(FakeToast).collect();
            let content: Element<'_, &'static str> = text("Content").into();
            let host: Element<'_, &'static str> = ToastHost::new(content)
                .toasts(
                    toasts.iter(),
                    |_id: u64| "dismiss",
                    |_toast: &FakeToast| None,
                )
                .into();

            let mut harness = WidgetHarness::new(host, Size::new(400.0, 300.0));

            assert_eq!(harness.focused_count().total, MAX_VISIBLE);
        }
    }

    mod identity {
        use super::*;
        use crate::test_support::WidgetHarness;

        #[derive(Clone, Copy)]
        struct FakeToast(u64);

        impl ToastPresentation for FakeToast {
            type Id = u64;

            fn id(&self) -> u64 {
                self.0
            }

            fn title(&self) -> &str {
                "Saved"
            }

            fn body(&self) -> Option<&str> {
                None
            }

            fn tone(&self) -> ToastTone {
                ToastTone::Info
            }
        }

        fn host(ids: [u64; 3]) -> Element<'static, &'static str> {
            let toasts: &'static [FakeToast] =
                ids.into_iter().map(FakeToast).collect::<Vec<_>>().leak();
            let content: Element<'static, &'static str> = text("Content").into();
            ToastHost::new(content)
                .toasts(
                    toasts.iter(),
                    |_id: u64| "dismiss",
                    |_toast: &FakeToast| None,
                )
                .into()
        }

        /// The bug this regression guards: `ToastState` always backfills a
        /// dismissal from the queue in the same call (`dismiss` -> `retain`
        /// -> `promote_queued`), so the visible count practically never
        /// changes across a single card's removal — the child count `diff`
        /// sees stays at three. A diff that only compares child count (like
        /// `iced::widget::keyed_column`'s) treats that as "nothing changed"
        /// and reuses the vacated slot's `Tree` positionally, handing a
        /// different toast whatever interaction state (here: keyboard focus)
        /// the previous occupant left behind. `ToastStack::diff` must catch
        /// this same-length identity swap and reset that slot.
        #[test]
        fn same_length_swap_does_not_carry_focus_to_the_new_occupant() {
            let mut harness = WidgetHarness::new(host([0, 1, 2]), Size::new(400.0, 300.0));

            harness.focus_next();
            harness.focus_next();
            assert_eq!(
                harness.focused_widgets(),
                1,
                "sanity: the middle card's dismiss button is focused"
            );

            // Same length (3), but id 1 (the focused slot) is replaced by a
            // different toast — exactly what a dismiss-then-promote produces.
            harness.replace(host([0, 99, 2]));

            assert_eq!(harness.focused_widgets(), 0);
        }

        #[test]
        fn repeated_relayout_without_a_diff_in_between_does_not_panic() {
            let mut harness = WidgetHarness::new(host([0, 1, 2]), Size::new(400.0, 300.0));

            harness.relayout(Size::new(200.0, 300.0));
            harness.draw();
            harness.relayout(Size::new(150.0, 300.0));
            harness.draw();
            harness.relayout(Size::new(400.0, 300.0));
            harness.draw();
        }

        fn host_with_pause(ids: [u64; 3]) -> Element<'static, &'static str> {
            let toasts: &'static [FakeToast] =
                ids.into_iter().map(FakeToast).collect::<Vec<_>>().leak();
            let content: Element<'static, &'static str> = text("Content").into();
            ToastHost::new(content)
                .on_hover("pause", "resume")
                .on_focus_within("focus-enter", "focus-exit")
                .toasts(
                    toasts.iter(),
                    |_id: u64| "dismiss",
                    |_toast: &FakeToast| None,
                )
                .into()
        }

        #[test]
        fn resize_while_a_card_is_focused_does_not_panic() {
            let mut harness =
                WidgetHarness::new(host_with_pause([0, 1, 2]), Size::new(400.0, 300.0));

            harness.focus_next();
            harness.focus_next();
            assert_eq!(harness.focused_widgets(), 1);

            harness.relayout(Size::new(200.0, 300.0));
            harness.draw();
            harness.relayout(Size::new(150.0, 300.0));
            harness.draw();
        }
    }

    mod no_auto_focus {
        use super::*;
        use crate::test_support::WidgetHarness;

        #[derive(Clone, Copy)]
        struct FakeToast;

        impl ToastPresentation for FakeToast {
            type Id = u64;

            fn id(&self) -> u64 {
                1
            }

            fn title(&self) -> &str {
                "Saved"
            }

            fn body(&self) -> Option<&str> {
                None
            }

            fn tone(&self) -> ToastTone {
                ToastTone::Info
            }
        }

        /// Part of the 4.2 accessibility contract that *is* enforced today
        /// (the rest is preparatory, see `ToastHost`'s rustdoc): a toast
        /// becoming visible must never move keyboard focus into it.
        #[test]
        fn appearing_toast_never_auto_focuses_any_control() {
            let content: Element<'_, &'static str> = text("Content").into();
            let host: Element<'_, &'static str> = ToastHost::new(content)
                .toasts(
                    std::iter::once(&FakeToast),
                    |_id: u64| "dismiss",
                    |_toast: &FakeToast| None,
                )
                .into();

            let mut harness = WidgetHarness::new(host, Size::new(400.0, 300.0));

            // Sanity: the toast's own dismiss button is a real focusable
            // control, so this isn't vacuously true for lack of one.
            assert!(harness.focused_count().total > 0);
            assert_eq!(harness.focused_widgets(), 0);
        }
    }

    mod focus_within {
        use super::*;
        use crate::test_support::WidgetHarness;
        use iced::widget::Id;

        fn focusable_area(id: Id) -> Element<'static, &'static str> {
            FocusWithinArea::new(button::ghost("Action").id(id).on_press("pressed"))
                .on_focus_within("focus-entered", "focus-left")
                .into()
        }

        /// Focus is applied immediately by the `operate()`-based
        /// `focusable::focus` operation; `FocusWithinArea` only observes it
        /// on the next `update()` pass, so tests dispatch a no-op event
        /// after focusing to give it that chance.
        fn pump(harness: &mut WidgetHarness<'_, &'static str>) -> Vec<&'static str> {
            harness
                .update(Event::Mouse(mouse::Event::CursorMoved {
                    position: iced::Point::new(1.0, 1.0),
                }))
                .messages
        }

        #[test]
        fn focusing_the_content_publishes_enter() {
            let id = Id::new("toast-action");
            let mut harness =
                WidgetHarness::new(focusable_area(id.clone()), Size::new(200.0, 80.0));
            harness.focus(id);

            assert_eq!(pump(&mut harness), vec!["focus-entered"]);
        }

        #[test]
        fn steady_focus_does_not_republish() {
            let id = Id::new("toast-action");
            let mut harness =
                WidgetHarness::new(focusable_area(id.clone()), Size::new(200.0, 80.0));
            harness.focus(id);
            let _entered = pump(&mut harness);

            assert!(pump(&mut harness).is_empty());
        }

        #[test]
        fn losing_focus_publishes_exit() {
            let id = Id::new("toast-action");
            let other = Id::new("elsewhere");
            let mut harness =
                WidgetHarness::new(focusable_area(id.clone()), Size::new(200.0, 80.0));
            harness.focus(id);
            let _entered = pump(&mut harness);

            harness.focus(other);

            assert_eq!(pump(&mut harness), vec!["focus-left"]);
        }

        #[test]
        fn never_focused_never_publishes() {
            let id = Id::new("toast-action");
            let mut harness = WidgetHarness::new(focusable_area(id), Size::new(200.0, 80.0));

            assert!(pump(&mut harness).is_empty());
        }
    }
}
