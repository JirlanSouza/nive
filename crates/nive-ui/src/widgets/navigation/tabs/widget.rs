use std::time::Duration;

use iced::{
    advanced::{
        layout::{self, Layout},
        mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Renderer as _, Shell, Widget,
    },
    keyboard, window, Event, Length, Rectangle, Shadow, Size, Vector,
};

use super::geometry::{
    autoscroll_step, edge_scroll_direction, event_position, gesture_to_pointer, hit_geometry,
    insertion_marker_bounds, measure_and_translate, owns_wheel_event, singleton_payload,
    snapshot_tab_region,
};
use super::style as theme_tabs;
use super::{
    FocusMovement, TabBar, TabBarFocus, TabBarState, TabDrop, TabDropTarget, TabRegion, TabTearOff,
    CHEVRON_SCROLL_STEP_FACTOR, TEAR_OFF_HYSTERESIS,
};
use crate::advanced::pressable::{draw_focus_ring_with_placement, FocusRingPlacement};
use crate::interaction::dnd::{DragSessionFeedback, DragSessionOutcome};
use crate::interaction::{
    CollectionTransferPayload, Drag, DropDecision, PointerButton, PointerGestureKind, TransferData,
    TransferOperation, TransferOperations,
};
use crate::widgets::controls::button::ButtonFocusRing;
use crate::widgets::navigation::overflow::{wheel_delta, OverflowAxis, OverflowDirection};

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
        vec![Tree::new(self.content_element(&state))]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_ref::<TabBarState<Id>>();
        let content = self.content_element(state);

        if tree.children.is_empty() {
            tree.children.push(Tree::new(&content));
        } else {
            tree.children[0].diff(content.as_widget());
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
        state.focus.expose(operation, None, layout.bounds());
        operation.focusable(
            None,
            layout.bounds(),
            &mut TabBarFocus {
                focus: &mut state.focus,
                pressed_id: &mut state.pressed_id,
            },
        );
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

        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(iced::touch::Event::FingerPressed { .. })
        ) && event_position(event, cursor).is_some_and(|position| bounds.contains(position))
        {
            state.focus.focus_from_pointer();
        }

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

        if matches!(event, Event::Window(window::Event::Unfocused)) {
            state.focus.deactivate();
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

        if state.focus.is_active() {
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
                    state.focus.focus_from_keyboard();
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
                    state.focus.focus_from_keyboard();
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
            if !owns_wheel_event(state.has_overflow, bounds, cursor) {
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

        if state.focus.is_focus_visible() {
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
        self.overlay_content = {
            let state = tree.state.downcast_ref::<TabBarState<Id>>();
            self.content_element(state)
        };
        // `content_element` is rebuilt from live interaction state (hover,
        // scroll, the "all tabs" overflow menu), which can change between the
        // last `diff`/`layout` pass and this call. Re-diff before recursing
        // so `tree.children[0]` matches the freshly built content instead of
        // a stale shape, which previously panicked deep inside the overflow
        // menu's buttons.
        tree.children[0].diff(self.overlay_content.as_widget());
        self.overlay_content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}
