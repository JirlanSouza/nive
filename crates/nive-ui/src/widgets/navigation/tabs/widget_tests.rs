use iced::{
    advanced::{
        layout::{Layout, Limits, Node},
        mouse,
        widget::Tree,
        Shell,
    },
    keyboard::{self, key, Location},
    Event, Font, Pixels, Point, Rectangle, Size, Vector,
};

use super::*;
use crate::interaction::ContextTarget;

const ORIGIN: Vector = Vector::new(50.0, 30.0);

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Select(u8),
    Close(TabCloseRequest<u8>),
    Context(ContextRequest<u8>),
    Drop(TabDrop<u8>),
    Tear(TabTearOff<u8>),
}

struct UpdateResult {
    messages: Vec<Msg>,
    captured: bool,
    layout_invalid: bool,
}

struct Harness<'a> {
    element: Element<'a, Msg>,
    tree: Tree,
    node: Node,
    renderer: iced::Renderer,
    size: Size,
    cursor: mouse::Cursor,
}

impl<'a> Harness<'a> {
    fn new(element: Element<'a, Msg>, size: Size) -> Self {
        let tree = Tree::new(&element);
        let mut harness = Self {
            element,
            tree,
            node: Node::new(Size::ZERO),
            renderer: test_renderer(),
            size,
            cursor: mouse::Cursor::Unavailable,
        };
        harness.layout();
        harness.layout();
        harness.sync_geometry();
        harness
    }

    fn layout(&mut self) {
        self.element.as_widget_mut().diff(&mut self.tree);
        self.relayout();
    }

    /// Lays out against the existing tree, the way the runtime does after an
    /// internal `invalidate_layout` — no fresh `diff` to reconcile a changed
    /// content shape.
    fn relayout(&mut self) {
        self.node = self.element.as_widget_mut().layout(
            &mut self.tree,
            &self.renderer,
            &Limits::new(Size::ZERO, self.size),
        );
    }

    /// Sends an event without the auto-relayout `update` performs, so a test
    /// can drive the runtime's event-then-relayout-without-diff sequence.
    fn update_without_relayout(&mut self, event: Event) {
        if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
            self.cursor = mouse::Cursor::Available(position);
        }

        let mut messages = Vec::new();
        let mut clipboard = iced::advanced::clipboard::Null;
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(4096.0, 4096.0));
        let mut shell = Shell::new(&mut messages);

        self.element.as_widget_mut().update(
            &mut self.tree,
            &event,
            Layout::with_offset(ORIGIN, &self.node),
            self.cursor,
            &self.renderer,
            &mut clipboard,
            &mut shell,
            &viewport,
        );
    }

    fn update(&mut self, event: Event) -> UpdateResult {
        let result = self.update_raw(event);
        self.sync_after(result)
    }

    fn update_raw(&mut self, event: Event) -> UpdateResult {
        if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
            self.cursor = mouse::Cursor::Available(position);
        }

        let mut messages = Vec::new();
        let mut clipboard = iced::advanced::clipboard::Null;
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(4096.0, 4096.0));
        let mut shell = Shell::new(&mut messages);

        self.element.as_widget_mut().update(
            &mut self.tree,
            &event,
            Layout::with_offset(ORIGIN, &self.node),
            self.cursor,
            &self.renderer,
            &mut clipboard,
            &mut shell,
            &viewport,
        );

        let captured = shell.is_event_captured();
        let layout_invalid = shell.is_layout_invalid();
        drop(shell);

        if layout_invalid {
            self.layout();
        }

        UpdateResult {
            messages,
            captured,
            layout_invalid,
        }
    }

    fn sync_after(&mut self, result: UpdateResult) -> UpdateResult {
        if result.layout_invalid {
            self.sync_geometry();
        }
        result
    }

    fn sync_geometry(&mut self) {
        let position = Point::new(ORIGIN.x + 1.0, ORIGIN.y + 1.0);
        let _ = self.update_raw(Event::Mouse(mouse::Event::CursorMoved { position }));
    }

    fn move_to(&mut self, position: Point) -> UpdateResult {
        self.update(Event::Mouse(mouse::Event::CursorMoved { position }))
    }

    fn click(&mut self, button: mouse::Button, position: Point) -> UpdateResult {
        let mut messages = Vec::new();
        let mut captured = false;

        for event in [
            Event::Mouse(mouse::Event::CursorMoved { position }),
            Event::Mouse(mouse::Event::ButtonPressed(button)),
            Event::Mouse(mouse::Event::CursorMoved { position }),
            Event::Mouse(mouse::Event::ButtonReleased(button)),
        ] {
            let result = self.update(event);
            messages.extend(result.messages);
            captured = result.captured;
        }

        UpdateResult {
            messages,
            captured,
            layout_invalid: false,
        }
    }

    fn drag(&mut self, from: Point, to: Point) -> UpdateResult {
        let mid = Point::new((from.x + to.x) / 2.0, (from.y + to.y) / 2.0);
        let mut messages = Vec::new();
        let mut captured = false;

        for event in [
            Event::Mouse(mouse::Event::CursorMoved { position: from }),
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Event::Mouse(mouse::Event::CursorMoved { position: mid }),
            Event::Mouse(mouse::Event::CursorMoved { position: to }),
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        ] {
            let result = self.update(event);
            messages.extend(result.messages);
            captured = result.captured;
        }

        UpdateResult {
            messages,
            captured,
            layout_invalid: false,
        }
    }

    fn drag_without_release(&mut self, from: Point, to: Point) {
        let _ = self.update(Event::Mouse(mouse::Event::CursorMoved { position: from }));
        let _ = self.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));
        let _ = self.update(Event::Mouse(mouse::Event::CursorMoved { position: to }));
    }

    fn wheel_pixels(&mut self, x: f32) -> UpdateResult {
        self.update(Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x, y: 0.0 },
        }))
    }

    fn click_overlay(&mut self, offset: Vector) -> UpdateResult {
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(4096.0, 4096.0));
        let mut overlay = self
            .element
            .as_widget_mut()
            .overlay(
                &mut self.tree,
                Layout::with_offset(ORIGIN, &self.node),
                &self.renderer,
                &viewport,
                Vector::ZERO,
            )
            .expect("overlay");
        let node = overlay
            .as_overlay_mut()
            .layout(&self.renderer, viewport.size());
        let layout = Layout::new(&node);
        let bounds = layout
            .children()
            .next()
            .map_or_else(|| layout.bounds(), |child| child.bounds());
        let position = Point::new(bounds.x + offset.x, bounds.y + offset.y);
        let mut messages = Vec::new();
        let mut captured = false;
        let mut layout_invalid = false;
        let mut clipboard = iced::advanced::clipboard::Null;

        for event in [
            Event::Mouse(mouse::Event::CursorMoved { position }),
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        ] {
            let mut shell = Shell::new(&mut messages);
            overlay.as_overlay_mut().update(
                &event,
                layout,
                mouse::Cursor::Available(position),
                &self.renderer,
                &mut clipboard,
                &mut shell,
            );
            captured |= shell.is_event_captured();
            layout_invalid |= shell.is_layout_invalid();
        }

        drop(overlay);

        if layout_invalid {
            self.layout();
            self.sync_geometry();
        }

        UpdateResult {
            messages,
            captured,
            layout_invalid,
        }
    }

    fn update_overlay(&mut self, event: Event, cursor: mouse::Cursor) -> UpdateResult {
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(4096.0, 4096.0));
        let mut overlay = self
            .element
            .as_widget_mut()
            .overlay(
                &mut self.tree,
                Layout::with_offset(ORIGIN, &self.node),
                &self.renderer,
                &viewport,
                Vector::ZERO,
            )
            .expect("overlay");
        let node = overlay
            .as_overlay_mut()
            .layout(&self.renderer, viewport.size());
        let mut messages = Vec::new();
        let mut clipboard = iced::advanced::clipboard::Null;
        let mut shell = Shell::new(&mut messages);

        overlay.as_overlay_mut().update(
            &event,
            Layout::new(&node),
            cursor,
            &self.renderer,
            &mut clipboard,
            &mut shell,
        );

        let captured = shell.is_event_captured();
        let layout_invalid = shell.is_layout_invalid();
        drop(shell);
        drop(overlay);

        if layout_invalid {
            self.layout();
            self.sync_geometry();
        }

        UpdateResult {
            messages,
            captured,
            layout_invalid,
        }
    }

    fn draw(&mut self) {
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(4096.0, 4096.0));

        self.element.as_widget().draw(
            &self.tree,
            &mut self.renderer,
            &crate::theme::Theme::Dark,
            &iced::advanced::renderer::Style::default(),
            Layout::with_offset(ORIGIN, &self.node),
            mouse::Cursor::Unavailable,
            &viewport,
        );
    }

    /// Mirrors `UserInterface::draw`: the overlay is laid out once, then
    /// rebuilt and painted against that cached layout, so the drawing instance
    /// never runs `layout` itself.
    fn draw_overlay(&mut self) {
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(4096.0, 4096.0));
        let node = {
            let mut overlay = self
                .element
                .as_widget_mut()
                .overlay(
                    &mut self.tree,
                    Layout::with_offset(ORIGIN, &self.node),
                    &self.renderer,
                    &viewport,
                    Vector::ZERO,
                )
                .expect("overlay");

            overlay
                .as_overlay_mut()
                .layout(&self.renderer, viewport.size())
        };
        let mut overlay = self
            .element
            .as_widget_mut()
            .overlay(
                &mut self.tree,
                Layout::with_offset(ORIGIN, &self.node),
                &self.renderer,
                &viewport,
                Vector::ZERO,
            )
            .expect("overlay");

        overlay.as_overlay_mut().draw(
            &mut self.renderer,
            &crate::theme::Theme::Dark,
            &iced::advanced::renderer::Style::default(),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
        );
    }

    fn cursor_left(&mut self) -> UpdateResult {
        self.update(Event::Mouse(mouse::Event::CursorLeft))
    }

    fn pixel_at(&mut self, point: Point) -> [u8; 4] {
        self.draw();

        let surface = Size::new(ORIGIN.x + self.size.width, ORIGIN.y + self.size.height);

        crate::test_support::pixel(
            &crate::test_support::rasterize(&mut self.renderer, surface),
            point,
        )
    }

    fn interaction_at(&mut self, position: Point) -> mouse::Interaction {
        self.move_to(position);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(4096.0, 4096.0));

        self.element.as_widget().mouse_interaction(
            &self.tree,
            Layout::with_offset(ORIGIN, &self.node),
            mouse::Cursor::Available(position),
            &viewport,
            &self.renderer,
        )
    }

    fn state(&self) -> &TabBarState<u8> {
        self.tree.state.downcast_ref::<TabBarState<u8>>()
    }

    fn tab_center(&self, id: u8) -> Point {
        let (_, bounds, _) = self
            .state()
            .tab_bounds
            .iter()
            .find(|(tab_id, _, _)| *tab_id == id)
            .expect("tab bounds");

        Point::new(
            bounds.x + bounds.width / 2.0,
            bounds.y + bounds.height / 2.0,
        )
    }

    fn tab_bounds(&self, id: u8) -> Rectangle {
        self.state()
            .tab_bounds
            .iter()
            .find(|(tab_id, _, _)| *tab_id == id)
            .map(|(_, bounds, _)| *bounds)
            .expect("tab bounds")
    }

    fn all_tabs_button_center(&self) -> Point {
        let bounds = self.state().all_tabs_button.expect("all-tabs button");

        Point::new(
            bounds.x + bounds.width / 2.0,
            bounds.y + bounds.height / 2.0,
        )
    }
}

fn test_renderer() -> iced::Renderer {
    iced_renderer::fallback::Renderer::Secondary(iced_tiny_skia::Renderer::new(
        Font::default(),
        Pixels(14.0),
    ))
}

fn item(id: u8) -> TabItem<'static, u8> {
    TabItem::new(id, format!("Tab {id}"))
}

fn standard_bar() -> TabBar<'static, u8, Msg> {
    TabBar::new(1)
        .tabs([item(1).pinned(true), item(2).closable(true), item(3)])
        .fill_width()
        .on_select(Msg::Select)
        .on_close_request(Msg::Close)
        .on_context(Msg::Context)
        .on_reorder(Msg::Drop)
}

fn tear_off_bar() -> TabBar<'static, u8, Msg> {
    standard_bar().on_tear_off(Msg::Tear)
}

fn overflow_bar(active: u8) -> TabBar<'static, u8, Msg> {
    TabBar::new(active)
        .tabs((1..=10).map(|id| TabItem::new(id, format!("Very long tab {id}"))))
        .fill_width()
        .on_select(Msg::Select)
        .on_close_request(Msg::Close)
        .on_context(Msg::Context)
        .on_reorder(Msg::Drop)
}

#[test]
fn harness_layout_and_click_selects_tab() {
    let mut harness = Harness::new(standard_bar().into(), Size::new(480.0, 80.0));
    let result = harness.click(mouse::Button::Left, harness.tab_center(2));

    assert_eq!(result.messages, vec![Msg::Select(2)]);
}

#[test]
fn disabled_tab_ignores_present_select_callback() {
    let bar = TabBar::new(1)
        .tabs([item(1), item(2).disabled(true)])
        .fill_width()
        .on_select(Msg::Select);
    let mut harness = Harness::new(bar.into(), Size::new(480.0, 80.0));
    let result = harness.click(mouse::Button::Left, harness.tab_center(2));

    assert!(result.messages.is_empty());
}

#[test]
fn overflow_menu_selection_uses_on_select() {
    let mut harness = Harness::new(overflow_bar(5).into(), Size::new(320.0, 80.0));
    let open = harness.click(mouse::Button::Left, harness.all_tabs_button_center());

    assert!(open.captured);
    assert!(harness.state().menu_open.get());

    let result = harness.click_overlay(Vector::new(20.0, 16.0));

    assert_eq!(result.messages, vec![Msg::Select(1)]);
    assert!(!harness.state().menu_open.get());
}

#[test]
fn the_active_tab_composes_its_own_indicator() {
    let metrics = super::style::metrics(crate::theme::ControlSize::Sm);
    let bar = TabBar::new(2u8)
        .tabs([item(1), item(2), item(3)])
        .fill_width()
        .on_select(Msg::Select);
    let mut harness = Harness::new(bar.into(), Size::new(640.0, 80.0));
    let root = Layout::with_offset(ORIGIN, &harness.node);
    let bar_row = root.children().next().expect("bar row");
    let strip = bar_row.children().nth(1).expect("strip");
    let tabs: Vec<Rectangle> = strip
        .children()
        .next()
        .expect("tabs row")
        .children()
        .map(|tab| {
            // Every tab keeps the same three-layer shape regardless of state.
            assert_eq!(tab.children().count(), 3);
            tab.bounds()
        })
        .collect();

    // Only the active tab paints its indicator; the layer is transparent on the
    // others. Probe the top-edge band where the indicator sits.
    let band = |tab: Rectangle| {
        Point::new(
            tab.x + tab.width / 2.0,
            tab.y + metrics.indicator_width / 2.0,
        )
    };
    let active = harness.pixel_at(band(tabs[1]));
    let inactive = harness.pixel_at(band(tabs[0]));

    assert_ne!(active, inactive);
    assert_eq!(inactive, harness.pixel_at(band(tabs[2])));
}

#[test]
fn long_labels_fit_the_width_cap_instead_of_being_sliced_by_it() {
    let metrics = super::style::metrics(crate::theme::ControlSize::Sm);
    let long = "Regional capacity forecast with a very long trailing title";
    let bar = TabBar::new(1u8)
        .tabs([TabItem::new(1u8, long), TabItem::new(2u8, long)])
        .fill_width()
        .on_select(Msg::Select);
    let harness = Harness::new(bar.into(), Size::new(1024.0, 80.0));

    for id in [1u8, 2] {
        let width = harness.tab_bounds(id).width;

        assert!(width <= metrics.max_tab_width, "{id}: {width}");
        assert!(width >= metrics.min_tab_width, "{id}: {width}");
    }
}

#[test]
fn overflow_menu_overlay_draws_against_a_matching_state_tree() {
    // Labels long enough for the menu entries to ellipsize: truncation is what
    // makes `MeasuredText` swap its child for a tooltip-wrapped one.
    let bar = TabBar::new(5u8)
        .tabs((1..=10).map(|id| {
            TabItem::new(
                id,
                format!("Regional capacity forecast with a very long trailing title {id}"),
            )
        }))
        .fill_width()
        .on_select(Msg::Select);
    let mut harness = Harness::new(bar.into(), Size::new(320.0, 80.0));
    harness.click(mouse::Button::Left, harness.all_tabs_button_center());

    assert!(harness.state().menu_open.get());

    // Mirrors a real frame: the bar paints, then the open menu's overlay does.
    for _ in 0..2 {
        harness.draw();
        harness.draw_overlay();
    }
}

#[test]
fn overflow_menu_escape_closes_without_domain_selection() {
    let mut harness = Harness::new(overflow_bar(5).into(), Size::new(320.0, 80.0));
    harness.click(mouse::Button::Left, harness.all_tabs_button_center());
    assert!(harness.state().menu_open.get());

    let result = harness.update_overlay(escape_pressed(), mouse::Cursor::Unavailable);

    assert!(result.captured);
    assert!(result.messages.is_empty());
    assert!(!harness.state().menu_open.get());
}

#[test]
fn overflow_menu_outside_press_closes_without_domain_selection() {
    let mut harness = Harness::new(overflow_bar(5).into(), Size::new(320.0, 80.0));
    harness.click(mouse::Button::Left, harness.all_tabs_button_center());
    assert!(harness.state().menu_open.get());

    let outside = Point::new(800.0, 600.0);
    let result = harness.update_overlay(
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        mouse::Cursor::Available(outside),
    );

    assert!(result.captured);
    assert!(result.messages.is_empty());
    assert!(!harness.state().menu_open.get());
}

fn escape_pressed() -> Event {
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: keyboard::Key::Named(key::Named::Escape),
        modified_key: keyboard::Key::Named(key::Named::Escape),
        physical_key: keyboard::key::Physical::Code(key::Code::Escape),
        location: Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
        text: None,
        repeat: false,
    })
}

#[test]
fn middle_click_targets_rendered_tab() {
    let mut harness = Harness::new(standard_bar().into(), Size::new(480.0, 80.0));
    let result = harness.click(mouse::Button::Middle, harness.tab_center(2));

    assert_eq!(
        result.messages,
        vec![Msg::Close(TabCloseRequest {
            id: 2,
            trigger: TabCloseTrigger::MiddleClick,
        })]
    );
}

#[test]
fn context_click_targets_rendered_tab_once() {
    let mut harness = Harness::new(standard_bar().into(), Size::new(480.0, 80.0));
    let result = harness.click(mouse::Button::Right, harness.tab_center(3));

    assert_eq!(result.messages.len(), 1);
    let Msg::Context(request) = &result.messages[0] else {
        panic!("expected context request");
    };
    assert_eq!(request.target, ContextTarget::Item(3));
}

#[test]
fn secondary_click_outside_widget_emits_nothing_and_does_not_capture() {
    let mut harness = Harness::new(standard_bar().into(), Size::new(480.0, 80.0));
    let outside = Point::new(ORIGIN.x - 20.0, ORIGIN.y - 20.0);
    let result = harness.click(mouse::Button::Right, outside);

    assert!(result.messages.is_empty());
    assert!(!result.captured);
}

#[test]
fn layout_never_panics_with_empty_tabs() {
    let mut harness = Harness::new(
        TabBar::new(None).fill_width().on_select(Msg::Select).into(),
        Size::new(320.0, 80.0),
    );

    let _ = harness.move_to(Point::new(ORIGIN.x + 10.0, ORIGIN.y + 10.0));
}

#[test]
fn fast_drag_uses_origin_region() {
    let mut harness = Harness::new(standard_bar().into(), Size::new(480.0, 80.0));
    let from = harness.tab_center(1);
    let to = Point::new(from.x + 180.0, from.y);

    harness.drag_without_release(from, to);

    assert_eq!(harness.state().dragged_id, Some(1));
}

#[test]
fn chevron_right_scrolls_strip() {
    let mut harness = Harness::new(overflow_bar(1).into(), Size::new(320.0, 80.0));
    assert!(harness.state().has_overflow);
    let strip_width = harness.state().strip_width;
    let first_before = harness.tab_bounds(1);
    let right = harness.state().right_chevron.expect("right chevron");
    let center = Point::new(right.x + right.width / 2.0, right.y + right.height / 2.0);

    let result = harness.click(mouse::Button::Left, center);

    let expected = (CHEVRON_SCROLL_STEP_FACTOR * strip_width).min(harness.state().max_scroll);
    assert!(result.captured);
    assert!((harness.state().scroll_offset - expected).abs() < 0.01);
    // The step is wider than a tab, so the first one scrolls out of the strip
    // and stops offering an interaction area at all.
    assert!(first_before.width < expected);
    assert!(!harness.state().tab_bounds.iter().any(|(id, _, _)| *id == 1));
}

#[test]
fn an_enabled_tab_shows_the_pointer_cursor_a_disabled_one_does_not() {
    let bar = TabBar::new(1u8)
        .tabs([item(1), item(2).disabled(true)])
        .fill_width()
        .on_select(Msg::Select);
    let mut harness = Harness::new(bar.into(), Size::new(480.0, 80.0));
    let enabled = harness.tab_center(1);
    let disabled = harness.tab_center(2);

    assert_eq!(harness.interaction_at(enabled), mouse::Interaction::Pointer);
    assert_eq!(harness.interaction_at(disabled), mouse::Interaction::None);
}

#[test]
fn a_clipped_tab_offers_the_pointer_over_its_visible_part() {
    let mut harness = Harness::new(overflow_bar(1).into(), Size::new(320.0, 80.0));
    let strip = harness.state().strip_bounds.expect("strip");
    let (_, cut, _) = harness
        .state()
        .tab_bounds
        .iter()
        .rev()
        .find(|(_, bounds, _)| bounds.x + bounds.width >= strip.x + strip.width - 1.0)
        .cloned()
        .expect("clipped tab");

    assert_eq!(
        harness.interaction_at(Point::new(cut.x + 4.0, cut.y + cut.height / 2.0)),
        mouse::Interaction::Pointer
    );
}

#[test]
fn dragging_an_unselected_tab_relayouts_without_a_panic() {
    // active = 1, so tab 3 is unselected: without the drag veil it is a plain
    // container, and starting a drag would grow it into a stack. That shape
    // change comes from internal state, which the runtime relayouts without a
    // diff to reconcile.
    let mut harness = Harness::new(standard_bar().into(), Size::new(480.0, 80.0));
    let from = harness.tab_center(3);
    let to = Point::new(from.x - 180.0, from.y);

    harness.update_without_relayout(Event::Mouse(mouse::Event::CursorMoved { position: from }));
    harness.update_without_relayout(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    harness.update_without_relayout(Event::Mouse(mouse::Event::CursorMoved { position: to }));
    harness.relayout();
    harness.draw();

    assert_eq!(harness.state().dragged_id, Some(3));
}

#[test]
fn reaching_a_scroll_end_relayouts_without_a_panic() {
    let mut harness = Harness::new(overflow_bar(1).into(), Size::new(320.0, 80.0));
    assert!(harness.state().overflow.show_end_chevron());
    assert!(!harness.state().overflow.show_start_chevron());

    // Scroll fully right: the right chevron reaches its end and the left one
    // becomes actionable, flipping both chevrons' `actionable` from internal
    // state. The runtime then relayouts against the existing tree, with no diff
    // to absorb a content shape that changed with `actionable`.
    harness.update_without_relayout(Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Pixels {
            x: -10_000.0,
            y: 0.0,
        },
    }));
    harness.relayout();
    harness.draw();

    assert!(harness.state().overflow.show_start_chevron());
    assert!(!harness.state().overflow.show_end_chevron());
}

#[test]
fn a_chevron_at_its_end_keeps_its_reserved_slot_while_overflow_lasts() {
    let metrics = super::style::metrics(crate::theme::ControlSize::Sm);
    let harness = Harness::new(overflow_bar(1).into(), Size::new(320.0, 80.0));
    // Offset 0: the left chevron cannot scroll while the right one can.
    assert!(!harness.state().overflow.show_start_chevron());
    assert!(harness.state().overflow.show_end_chevron());

    // The inactive end still holds its full slot, so reaching a boundary never
    // reflows the strip. The glyph merely dims — a renderer-dependent tint that
    // the software test backend cannot assert, so it is verified visually.
    let left = harness.state().left_chevron.expect("left slot reserved");
    assert_eq!(left.width, metrics.close_side);
}

#[test]
fn chevron_slots_collapse_without_overflow() {
    let harness = Harness::new(standard_bar().into(), Size::new(640.0, 80.0));
    assert!(!harness.state().has_overflow);

    // No overflow: both slots collapse below the hit threshold and paint nothing.
    assert!(harness.state().left_chevron.is_none());
    assert!(harness.state().right_chevron.is_none());
}

#[test]
fn active_scroll_chevrons_show_the_pointer_cursor() {
    let mut harness = Harness::new(overflow_bar(1).into(), Size::new(320.0, 80.0));
    assert!(harness.state().has_overflow);
    let right = harness.state().right_chevron.expect("right chevron");
    let center = Point::new(right.x + right.width / 2.0, right.y + right.height / 2.0);

    assert_eq!(harness.interaction_at(center), mouse::Interaction::Pointer);
}

#[test]
fn hidden_scroll_chevrons_keep_the_default_cursor() {
    let mut harness = Harness::new(standard_bar().into(), Size::new(640.0, 80.0));
    assert!(!harness.state().has_overflow);

    // Where the right chevron collapses to when there is nothing to scroll.
    let bar = Layout::with_offset(ORIGIN, &harness.node).bounds();
    let edge = Point::new(bar.x + bar.width - 1.0, bar.y + bar.height / 2.0);

    assert_eq!(harness.interaction_at(edge), mouse::Interaction::None);
}

#[test]
fn hovering_a_half_visible_tab_paints_nothing_past_the_strip() {
    let mut harness = Harness::new(overflow_bar(1).into(), Size::new(320.0, 80.0));
    let strip = harness.state().strip_bounds.expect("strip");
    // The tab cut off by the strip's right edge: its full box reaches past it.
    let (_, cut, _) = harness
        .state()
        .tab_bounds
        .iter()
        .rev()
        .find(|(_, bounds, _)| bounds.x + bounds.width >= strip.x + strip.width - 1.0)
        .cloned()
        .expect("clipped tab");
    let outside = Point::new(strip.x + strip.width + 2.0, cut.y + cut.height / 2.0);
    let before = harness.pixel_at(outside);

    harness.move_to(Point::new(cut.x + 4.0, cut.y + cut.height / 2.0));

    assert!(harness.state().hovered_id.is_some());
    assert_eq!(harness.pixel_at(outside), before);
}

#[test]
fn a_tab_scrolled_past_the_strip_stops_answering_the_pointer() {
    let mut harness = Harness::new(overflow_bar(1).into(), Size::new(320.0, 80.0));
    let strip = harness.state().strip_bounds.expect("strip");
    let first = harness.tab_bounds(1);

    // Left of the strip is another region of the shell, not the tab bar.
    assert!(first.x >= strip.x);

    harness.wheel_pixels(-strip.width);
    let visible = harness.state().tab_bounds.clone();

    assert!(visible.iter().all(|(_, bounds, _)| {
        bounds.x >= strip.x - 0.01 && bounds.x + bounds.width <= strip.x + strip.width + 0.01
    }));

    let outside = Point::new(strip.x - 20.0, strip.y + strip.height / 2.0);
    harness.move_to(outside);

    assert!(harness.state().hovered_id.is_none());
}

#[test]
fn wheel_scroll_clamps_and_captures() {
    let mut harness = Harness::new(overflow_bar(1).into(), Size::new(320.0, 80.0));

    let result = harness.wheel_pixels(-1000.0);
    assert!(result.captured);
    assert_eq!(harness.state().scroll_offset, harness.state().max_scroll);

    let result = harness.wheel_pixels(1000.0);
    assert!(result.captured);
    assert_eq!(harness.state().scroll_offset, 0.0);
}

#[test]
fn drag_to_trailing_empty_space_commits_after_last() {
    let mut harness = Harness::new(standard_bar().into(), Size::new(720.0, 80.0));
    let from = harness.tab_center(2);
    let last = harness.tab_bounds(3);
    let to = Point::new(last.x + last.width + 80.0, from.y);
    let result = harness.drag(from, to);

    assert_eq!(result.messages.len(), 1);
    let Msg::Drop(drop) = &result.messages[0] else {
        panic!("expected drop");
    };
    assert_eq!(drop.target, TabDropTarget::After(3));
}

#[test]
fn release_far_outside_without_tear_off_emits_nothing() {
    let mut harness = Harness::new(standard_bar().into(), Size::new(480.0, 80.0));
    let from = harness.tab_center(1);
    let to = Point::new(from.x, from.y + 200.0);
    let result = harness.drag(from, to);

    assert!(result.messages.is_empty());
    assert!(harness.state().dragged_id.is_none());
    assert!(harness.state().insertion_target.is_none());
}

#[test]
fn release_far_outside_with_tear_off_emits_tear_off() {
    let mut harness = Harness::new(tear_off_bar().into(), Size::new(480.0, 80.0));
    let from = harness.tab_center(1);
    let to = Point::new(from.x, from.y + 200.0);
    let result = harness.drag(from, to);

    assert_eq!(result.messages.len(), 1);
    let Msg::Tear(tear) = &result.messages[0] else {
        panic!("expected tear-off");
    };
    assert_eq!(tear.payload.ids, vec![1]);
    assert_eq!(tear.position, to);
}

#[test]
fn leaving_the_window_mid_drag_keeps_the_drag_alive() {
    let mut harness = Harness::new(standard_bar().into(), Size::new(480.0, 80.0));
    let from = harness.tab_center(1);
    harness.drag_without_release(from, Point::new(from.x + 80.0, from.y));

    let result = harness.cursor_left();

    assert!(result.messages.is_empty());
    // The button is still down, so the tab is still being dragged and the
    // gesture can still resolve into a reorder or a tear-off on release.
    assert!(harness.state().dragged_id.is_some());
}

#[test]
fn losing_window_focus_mid_drag_cancels() {
    let mut harness = Harness::new(standard_bar().into(), Size::new(480.0, 80.0));
    let from = harness.tab_center(1);
    harness.drag_without_release(from, Point::new(from.x + 80.0, from.y));

    let result = harness.update(Event::Window(iced::window::Event::Unfocused));

    assert!(result.messages.is_empty());
    assert!(harness.state().dragged_id.is_none());
    assert!(harness.state().insertion_target.is_none());
}

#[test]
fn drag_over_tab_sets_insertion_target() {
    let mut harness = Harness::new(standard_bar().into(), Size::new(480.0, 80.0));
    let from = harness.tab_center(2);
    let target = harness.tab_bounds(3);
    let left_half = Point::new(
        target.x + target.width * 0.25,
        target.y + target.height / 2.0,
    );
    harness.drag_without_release(from, left_half);
    assert_eq!(
        harness.state().insertion_target,
        Some(TabDropTarget::Before(3))
    );

    let right_half = Point::new(
        target.x + target.width * 0.75,
        target.y + target.height / 2.0,
    );
    let _ = harness.move_to(right_half);
    assert_eq!(
        harness.state().insertion_target,
        Some(TabDropTarget::After(3))
    );
}
