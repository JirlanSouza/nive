use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Id, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    widget::{container, scrollable},
    Background, Border, Event, Length, Rectangle, Shadow, Size, Vector,
};

use crate::theme::{
    self, surface as theme_surface, BorderRole, PaddingRole, ShapeSize, SurfaceRole,
};
use crate::{Element, Renderer, Theme};

use super::DialogSize;

/// Canonical modal Dialog anatomy: a fixed header, a body that owns the only
/// vertical scroll region, and an optional fixed footer, painted as one
/// borderless [`SurfaceRole::Dialog`] frame.
///
/// `Dialog` exposes no generic `Length`, raw width/height, raw padding, raw
/// radius, or `ControlSize` builder. Semantic width comes only from
/// [`DialogSize`]; the hosting [`crate::widgets::overlays::DialogHost`]
/// clamps the resolved frame to the safe viewport.
pub struct Dialog<'a, Message> {
    header: Option<Element<'a, Message>>,
    body: Element<'a, Message>,
    footer: Option<Element<'a, Message>>,
    size: DialogSize,
}

impl<'a, Message> Dialog<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(body: impl Into<Element<'a, Message>>) -> Self {
        Self {
            header: None,
            body: wrap_body(body.into()),
            footer: None,
            size: DialogSize::default(),
        }
    }

    pub fn header(mut self, header: impl Into<Element<'a, Message>>) -> Self {
        self.header = Some(wrap_header(header.into()));
        self
    }

    pub fn footer(mut self, footer: impl Into<Element<'a, Message>>) -> Self {
        self.footer = Some(wrap_footer(footer.into()));
        self
    }

    pub fn size(mut self, size: DialogSize) -> Self {
        self.size = size;
        self
    }

    fn slots(&self) -> Vec<&Element<'a, Message>> {
        let mut slots = Vec::with_capacity(3);
        if let Some(header) = &self.header {
            slots.push(header);
        }
        slots.push(&self.body);
        if let Some(footer) = &self.footer {
            slots.push(footer);
        }
        slots
    }
}

fn wrap_body<'a, Message: 'a>(body: Element<'a, Message>) -> Element<'a, Message> {
    let padding = theme::padding(PaddingRole::Dialog);

    scrollable(container(body).width(Length::Fill).padding(padding))
        .width(Length::Fill)
        .height(Length::Shrink)
        .into()
}

fn wrap_header<'a, Message: 'a>(header: Element<'a, Message>) -> Element<'a, Message> {
    container(header)
        .width(Length::Fill)
        .padding(slot_padding_v(PaddingRole::Panel))
        .into()
}

fn wrap_footer<'a, Message: 'a>(footer: Element<'a, Message>) -> Element<'a, Message> {
    container(footer)
        .width(Length::Fill)
        .padding(slot_padding_v(PaddingRole::Panel))
        .into()
}

/// Horizontal insets resolve [`PaddingRole::Dialog`]; header/footer vertical
/// insets resolve the panel-padding role. The two roles are combined here so
/// header and footer content stays within the same semantic column as body
/// content while owning their own, typically smaller, vertical rhythm.
fn slot_padding_v(vertical_role: PaddingRole) -> iced::Padding {
    let horizontal = theme::padding(PaddingRole::Dialog);
    let vertical = theme::padding(vertical_role);

    iced::Padding {
        top: vertical.top,
        bottom: vertical.bottom,
        left: horizontal.left,
        right: horizontal.right,
    }
}

const SEAM_WIDTH: f32 = 1.0;
const SEAM_VISIBILITY_EPSILON: f32 = 0.5;

/// Survives across `view()` rebuilds via `Tree::diff` (unlike `Dialog`
/// itself, rebuilt every frame), so `draw` can read the body's last known
/// scroll offset to decide seam visibility.
#[derive(Debug, Clone, Copy, Default)]
struct DialogSeamState {
    offset_y: f32,
}

/// Reads the body scrollable's current translation without downcasting its
/// private internal state.
#[derive(Default)]
struct ScrollOffsetProbe {
    offset_y: f32,
}

impl operation::Operation for ScrollOffsetProbe {
    fn traverse(&mut self, _operate: &mut dyn FnMut(&mut dyn operation::Operation)) {}

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn operation::Scrollable,
    ) {
        self.offset_y = translation.y;
    }
}

impl<'a, Message> Widget<Message, Theme, Renderer> for Dialog<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<DialogSeamState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(DialogSeamState::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.slots().iter().map(|slot| Tree::new(*slot)).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let widgets: Vec<_> = self.slots().iter().map(|slot| slot.as_widget()).collect();
        tree.diff_children(&widgets);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn size_hint(&self) -> Size<Length> {
        self.size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let max = limits.max();
        let width = self.size.target_width().min(max.width).max(0.0);
        let unbounded = layout::Limits::new(Size::ZERO, Size::new(width, f32::INFINITY));

        let mut nodes = Vec::with_capacity(3);
        let mut child_index = 0;
        let mut y = 0.0;

        let header_height = if let Some(header) = &mut self.header {
            let node = header.as_widget_mut().layout(
                &mut tree.children[child_index],
                renderer,
                &unbounded,
            );
            let height = node.size().height;
            nodes.push(node.move_to(iced::Point::new(0.0, y)));
            child_index += 1;
            y += height + SEAM_WIDTH;
            height + SEAM_WIDTH
        } else {
            0.0
        };

        // Footer is measured once here (throwaway tree, natural height only)
        // to reserve its budget before the body is laid out, then laid out
        // for real against its persistent tree slot below.
        let footer_reserved = self
            .footer
            .as_mut()
            .map(|footer| {
                let mut throwaway = Tree::new(&*footer);
                footer
                    .as_widget_mut()
                    .layout(&mut throwaway, renderer, &unbounded)
                    .size()
                    .height
                    + SEAM_WIDTH
            })
            .unwrap_or(0.0);

        let body_max_height = (max.height - header_height - footer_reserved).max(0.0);
        let body_limits = layout::Limits::new(Size::ZERO, Size::new(width, body_max_height));
        let body_node = self.body.as_widget_mut().layout(
            &mut tree.children[child_index],
            renderer,
            &body_limits,
        );
        let body_height = body_node.size().height;
        nodes.push(body_node.move_to(iced::Point::new(0.0, y)));
        child_index += 1;
        y += body_height;

        if let Some(footer) = &mut self.footer {
            y += SEAM_WIDTH;
            let node = footer.as_widget_mut().layout(
                &mut tree.children[child_index],
                renderer,
                &unbounded,
            );
            let height = node.size().height;
            nodes.push(node.move_to(iced::Point::new(0.0, y)));
            y += height;
        }

        layout::Node::with_children(Size::new(width, y), nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        // Visited body-first (then footer, then header), not in draw order:
        // focus-related operations (Tab trapping, initial-focus resolution)
        // should prefer body content and treat the header close affordance
        // as a last resort, independent of where it is painted.
        let order = self.focus_visit_order();
        let mut slots = self.slots_mut();
        let layout_children: Vec<_> = layout.children().collect();

        for index in order {
            slots[index].as_widget_mut().operate(
                &mut tree.children[index],
                layout_children[index],
                renderer,
                operation,
            );
        }
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
        let body_index = usize::from(self.header.is_some());

        for ((slot, state), child_layout) in self
            .slots_mut()
            .into_iter()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
        {
            slot.as_widget_mut().update(
                state,
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );

            if shell.is_event_captured() {
                break;
            }
        }

        if let Some(body_layout) = layout.children().nth(body_index) {
            let mut probe = ScrollOffsetProbe::default();
            self.body.as_widget_mut().operate(
                &mut tree.children[body_index],
                body_layout,
                renderer,
                &mut probe,
            );
            *tree.state.downcast_mut::<DialogSeamState>() = DialogSeamState {
                offset_y: probe.offset_y,
            };
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
        self.slots()
            .into_iter()
            .zip(tree.children.iter())
            .zip(layout.children())
            .map(|((slot, state), child_layout)| {
                slot.as_widget()
                    .mouse_interaction(state, child_layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let radius = theme.shape(ShapeSize::Lg).radius();
        let style = theme_surface::style_with_radius(SurfaceRole::Dialog, radius)(theme);
        container::draw_background(renderer, &style, bounds);

        let mut children = layout.children();
        let scroll_offset_y = tree.state.downcast_ref::<DialogSeamState>().offset_y;

        if let Some(header) = &self.header {
            if let Some(header_layout) = children.next() {
                header.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    inherited_style,
                    header_layout,
                    cursor,
                    viewport,
                );
                if header_seam_visible(scroll_offset_y) {
                    draw_seam(renderer, theme, header_layout.bounds(), bounds, true);
                }
            }
        }

        let body_index = if self.header.is_some() { 1 } else { 0 };
        if let Some(body_layout) = children.next() {
            self.body.as_widget().draw(
                &tree.children[body_index],
                renderer,
                theme,
                inherited_style,
                body_layout,
                cursor,
                viewport,
            );

            if let Some(footer) = &self.footer {
                if let Some(footer_layout) = children.next() {
                    let content_height = body_layout
                        .children()
                        .next()
                        .map_or(0.0, |content_layout| content_layout.bounds().height);
                    if footer_seam_visible(
                        scroll_offset_y,
                        content_height,
                        body_layout.bounds().height,
                    ) {
                        draw_seam(renderer, theme, body_layout.bounds(), bounds, true);
                    }
                    footer.as_widget().draw(
                        &tree.children[body_index + 1],
                        renderer,
                        theme,
                        inherited_style,
                        footer_layout,
                        cursor,
                        viewport,
                    );
                }
            }
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
        let children: Vec<_> = self
            .slots_mut()
            .into_iter()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
            .filter_map(|((slot, state), child_layout)| {
                slot.as_widget_mut()
                    .overlay(state, child_layout, renderer, viewport, translation)
            })
            .collect();

        (!children.is_empty()).then(|| overlay::Group::with_children(children).overlay())
    }
}

impl<'a, Message> Dialog<'a, Message> {
    fn slots_mut(&mut self) -> Vec<&mut Element<'a, Message>> {
        let mut slots = Vec::with_capacity(3);
        if let Some(header) = &mut self.header {
            slots.push(header);
        }
        slots.push(&mut self.body);
        if let Some(footer) = &mut self.footer {
            slots.push(footer);
        }
        slots
    }

    /// Slot indices (matching `slots()`/`slots_mut()`/`tree.children`/
    /// `layout.children()` order) visited body-first for focus-related
    /// operations: body, then footer, then header.
    fn focus_visit_order(&self) -> Vec<usize> {
        let body_index = usize::from(self.header.is_some());
        let mut order = vec![body_index];
        if self.footer.is_some() {
            order.push(body_index + 1);
        }
        if self.header.is_some() {
            order.push(0);
        }
        order
    }
}

fn header_seam_visible(scroll_offset_y: f32) -> bool {
    scroll_offset_y > SEAM_VISIBILITY_EPSILON
}

fn footer_seam_visible(scroll_offset_y: f32, content_height: f32, viewport_height: f32) -> bool {
    content_height - viewport_height - scroll_offset_y > SEAM_VISIBILITY_EPSILON
}

fn draw_seam(
    renderer: &mut Renderer,
    theme: &Theme,
    edge: Rectangle,
    frame: Rectangle,
    below: bool,
) {
    use iced::advanced::Renderer as _;

    let y = if below { edge.y + edge.height } else { edge.y };

    renderer.fill_quad(
        renderer::Quad {
            bounds: Rectangle {
                x: frame.x,
                y,
                width: frame.width,
                height: SEAM_WIDTH,
            },
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(theme.border(BorderRole::Subtle).color),
    );
}

impl<'a, Message> From<Dialog<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(dialog: Dialog<'a, Message>) -> Self {
        Element::new(dialog)
    }
}

#[cfg(test)]
mod dialog_widget_tests {
    use super::*;

    fn layout_dialog<Message: Clone + 'static>(
        dialog: Dialog<'static, Message>,
        max: Size,
    ) -> layout::Node {
        crate::test_support::layout(dialog.into(), max)
    }

    /// `layout` measures the footer against a throwaway `Tree::new(&*footer)` to
    /// reserve its height. That fresh tree is built from the footer's *current*
    /// elements, which a previous narrow pass may have left truncated — so a
    /// child tree can arrive describing a `Tooltip` while its fresh state says
    /// nothing is truncated, and the unbounded measuring pass then lays a plain
    /// `Text` against it.
    ///
    /// Reaching it needs the real resize path: `relayout` re-runs `layout` on
    /// the retained tree with no `diff` in between.
    #[test]
    fn remeasuring_a_footer_after_a_narrow_pass_does_not_mismatch_its_child_trees() {
        use crate::test_support::WidgetHarness;
        use crate::widgets::overlays::dialog::footer::{
            DialogAction, DialogActionFooter, DialogTerminalAction,
        };

        let dialog = || -> Element<'static, ()> {
            Dialog::new(iced::widget::text("Body"))
                .footer(DialogActionFooter::with_one(
                    DialogAction::cancel("Discard every unsaved change", ()),
                    DialogTerminalAction::primary("Save and close this document now", ()),
                ))
                .into()
        };

        let mut harness = WidgetHarness::new(dialog(), Size::new(900.0, 900.0));
        harness.draw();

        // Narrow first, so the action labels truncate and the footer's elements
        // are left holding tooltip-wrapped content.
        for width in [200.0, 900.0, 160.0, 900.0] {
            harness.relayout(Size::new(width, 900.0));
            harness.draw();
        }
    }

    #[test]
    fn header_seam_is_hidden_at_the_top_and_visible_once_scrolled() {
        assert!(!header_seam_visible(0.0));
        assert!(!header_seam_visible(SEAM_VISIBILITY_EPSILON));
        assert!(header_seam_visible(12.0));
    }

    #[test]
    fn footer_seam_is_visible_only_while_content_remains_below() {
        // No overflow at all: never visible, regardless of offset.
        assert!(!footer_seam_visible(0.0, 200.0, 200.0));

        // Overflowing content, unscrolled: visible.
        assert!(footer_seam_visible(0.0, 600.0, 200.0));

        // Scrolled to the exact bottom: hidden again.
        assert!(!footer_seam_visible(400.0, 600.0, 200.0));

        // Scrolled partway through overflowing content: still visible.
        assert!(footer_seam_visible(150.0, 600.0, 200.0));
    }

    #[test]
    fn resolves_semantic_widths_in_an_unconstrained_viewport() {
        for (size, expected) in [
            (DialogSize::Sm, 420.0),
            (DialogSize::Md, 560.0),
            (DialogSize::Lg, 720.0),
        ] {
            let dialog = Dialog::<()>::new(iced::widget::text("Body")).size(size);
            let node = layout_dialog(dialog, Size::new(2000.0, 2000.0));

            assert_eq!(node.size().width, expected);
        }
    }

    #[test]
    fn default_size_is_sm() {
        let dialog = Dialog::<()>::new(iced::widget::text("Body"));
        let node = layout_dialog(dialog, Size::new(2000.0, 2000.0));

        assert_eq!(node.size().width, 420.0);
    }

    #[test]
    fn width_clamps_to_the_limits_max_width() {
        let dialog = Dialog::<()>::new(iced::widget::text("Body")).size(DialogSize::Lg);
        let node = layout_dialog(dialog, Size::new(300.0, 2000.0));

        assert_eq!(node.size().width, 300.0);
    }

    #[test]
    fn body_and_slot_insets_grow_with_density() {
        use crate::theme::{testing::ThemeTestGuard, ThemeBuilder, ThemeMode};

        fn dialog_height(density: crate::theme::ThemeDensity) -> f32 {
            let theme = ThemeBuilder::new("Density test", ThemeMode::Dark)
                .density(density)
                .build();
            let _guard = ThemeTestGuard::activate(theme);

            let dialog = Dialog::<()>::new(iced::widget::text("Body"))
                .header(iced::widget::text("Title"))
                .footer(iced::widget::text("Footer"));
            layout_dialog(dialog, Size::new(420.0, 2000.0))
                .size()
                .height
        }

        let compact = dialog_height(crate::theme::ThemeDensity::Compact);
        let comfortable = dialog_height(crate::theme::ThemeDensity::Comfortable);

        assert!(
            compact < comfortable,
            "compact height {compact} should be shorter than comfortable height {comfortable}"
        );
    }

    #[test]
    fn total_height_never_exceeds_the_limits_max_height() {
        let long_body =
            iced::widget::column((0..200).map(|i| iced::widget::text(format!("Line {i}")).into()));
        let dialog = Dialog::<()>::new(long_body)
            .header(iced::widget::text("Title"))
            .footer(iced::widget::text("Footer"));
        let node = layout_dialog(dialog, Size::new(420.0, 400.0));

        assert!(node.size().height <= 400.0 + f32::EPSILON);
    }

    #[test]
    fn short_content_does_not_stretch_to_the_available_cap() {
        let dialog =
            Dialog::<()>::new(iced::widget::text("One line")).header(iced::widget::text("Title"));
        let node = layout_dialog(dialog, Size::new(420.0, 4000.0));

        assert!(node.size().height < 400.0);
    }

    #[test]
    fn header_and_footer_are_positioned_before_and_after_the_body() {
        let dialog = Dialog::<()>::new(iced::widget::text("Body"))
            .header(iced::widget::text("Title"))
            .footer(iced::widget::text("Footer"));
        let node = layout_dialog(dialog, Size::new(420.0, 2000.0));
        let children: Vec<_> = node.children().to_vec();

        assert_eq!(children.len(), 3);
        assert_eq!(children[0].bounds().y, 0.0);
        assert!(children[1].bounds().y >= children[0].bounds().y + children[0].bounds().height);
        assert!(children[2].bounds().y >= children[1].bounds().y + children[1].bounds().height);
    }

    #[test]
    fn body_consumed_enter_prevents_the_footer_default_from_also_firing() {
        use super::super::{DialogActionFooter, DialogTerminalAction};
        use iced::keyboard;

        let key = keyboard::Key::Named(keyboard::key::Named::Enter);
        let event = Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Enter),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::NONE,
            text: None,
            repeat: false,
        });

        let dialog = Dialog::new(AlwaysCapture).footer(DialogActionFooter::new(
            DialogTerminalAction::primary("Save", "save"),
        ));

        let messages: Vec<&'static str> =
            crate::test_support::event_messages(dialog.into(), Size::new(420.0, 400.0), event);

        assert!(messages.is_empty());
    }

    /// Minimal test-only widget that captures every event, standing in for a
    /// text editor or nested overlay that consumes Enter itself.
    struct AlwaysCapture;

    impl<Message> Widget<Message, Theme, Renderer> for AlwaysCapture {
        fn size(&self) -> Size<Length> {
            Size::new(Length::Fixed(10.0), Length::Fixed(10.0))
        }

        fn layout(
            &mut self,
            _tree: &mut Tree,
            _renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            layout::Node::new(limits.resolve(
                Length::Fixed(10.0),
                Length::Fixed(10.0),
                Size::new(10.0, 10.0),
            ))
        }

        fn update(
            &mut self,
            _tree: &mut Tree,
            _event: &Event,
            _layout: Layout<'_>,
            _cursor: mouse::Cursor,
            _renderer: &Renderer,
            _clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
            _viewport: &Rectangle,
        ) {
            shell.capture_event();
        }

        fn draw(
            &self,
            _tree: &Tree,
            _renderer: &mut Renderer,
            _theme: &Theme,
            _inherited_style: &renderer::Style,
            _layout: Layout<'_>,
            _cursor: mouse::Cursor,
            _viewport: &Rectangle,
        ) {
        }
    }

    impl<'a, Message: 'a> From<AlwaysCapture> for Element<'a, Message> {
        fn from(widget: AlwaysCapture) -> Self {
            Element::new(widget)
        }
    }
}
