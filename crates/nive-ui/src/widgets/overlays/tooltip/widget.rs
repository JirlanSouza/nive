use std::time::Duration;

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Operation as _, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    keyboard::{self, key::Named},
    Event, Length, Point, Rectangle, Size, Vector,
};

use super::TooltipPlacement;
use crate::{
    widgets::overlays::{
        anchored_overlay::{resolve_geometry, GeometryInput},
        popover::{PopoverCollision, PopoverPlacement, PopoverWidth},
    },
    Element,
};

#[derive(Debug)]
pub(super) struct TooltipState {
    pub(super) owner_key: Option<u64>,
    pub(super) hovered: bool,
    pub(super) focused: bool,
    pub(super) visible: bool,
    pub(super) block_private_traversal: bool,
    entered_at: Option<iced::time::Instant>,
    pub(super) escape_suppressed: bool,
}

impl Default for TooltipState {
    fn default() -> Self {
        Self {
            owner_key: None,
            hovered: false,
            focused: false,
            visible: false,
            block_private_traversal: false,
            entered_at: None,
            escape_suppressed: false,
        }
    }
}

pub(super) struct TooltipWidget<'a, Message> {
    anchor: Element<'a, Message>,
    label: Element<'a, Message>,
    placement: TooltipPlacement,
    delay: Duration,
    now_override: Option<iced::time::Instant>,
    intent_override: Option<(bool, bool)>,
}

impl<'a, Message> TooltipWidget<'a, Message> {
    pub(super) fn new(
        anchor: Element<'a, Message>,
        label: Element<'a, Message>,
        placement: TooltipPlacement,
        delay: Duration,
        now_override: Option<iced::time::Instant>,
        intent_override: Option<(bool, bool)>,
    ) -> Self {
        Self {
            anchor,
            label,
            placement,
            delay,
            now_override,
            intent_override,
        }
    }
}

impl<Message> Widget<Message, crate::theme::Theme, iced::Renderer> for TooltipWidget<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TooltipState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TooltipState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.anchor), Tree::new(&self.label)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.anchor, &self.label]);
    }

    fn size(&self) -> Size<Length> {
        self.anchor.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.anchor.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.anchor
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<TooltipState>();
        operation.custom(None, layout.bounds(), state);
        let blocked = std::mem::take(&mut state.block_private_traversal);
        if !blocked {
            operation.traverse(&mut |operation| {
                self.anchor.as_widget_mut().operate(
                    &mut tree.children[0],
                    layout,
                    renderer,
                    operation,
                );
            });
        }
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
        self.anchor.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let (hovered, focused) = self.intent_override.unwrap_or_else(|| {
            (
                cursor.is_over(layout.bounds()),
                anchor_is_focused(&mut self.anchor, &mut tree.children[0], layout, renderer),
            )
        });
        let now = self.now_override.unwrap_or_else(|| event_now(event));
        let state = tree.state.downcast_mut::<TooltipState>();
        state.hovered = hovered;
        state.focused = focused;

        if is_escape(event) && state.visible {
            state.visible = false;
            state.escape_suppressed = true;
            shell.request_redraw();
        }

        let has_intent = (hovered || focused) && !state.escape_suppressed;
        if !hovered && !focused {
            state.escape_suppressed = false;
        }

        if !has_intent {
            state.visible = false;
            state.entered_at = None;
        } else {
            let entered_at = *state.entered_at.get_or_insert(now);
            let deadline = entered_at + self.delay;
            if now >= deadline {
                state.visible = true;
            } else {
                state.visible = false;
                shell.request_redraw_at(deadline);
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
        self.anchor.as_widget().mouse_interaction(
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
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.anchor.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, crate::theme::Theme, iced::Renderer>> {
        let (anchor_tree, label_tree) = tree.children.split_at_mut(1);
        let anchor_overlay = self.anchor.as_widget_mut().overlay(
            &mut anchor_tree[0],
            layout,
            renderer,
            viewport,
            translation,
        );
        let tooltip_overlay = tree.state.downcast_ref::<TooltipState>().visible.then(|| {
            overlay::Element::new(Box::new(TooltipOverlay {
                anchor_bounds: translated_bounds(layout.bounds(), translation),
                label: &mut self.label,
                label_state: &mut label_tree[0],
                placement: self.placement,
            }))
        });

        match (anchor_overlay, tooltip_overlay) {
            (Some(anchor), Some(tooltip)) => {
                Some(overlay::Group::with_children(vec![anchor, tooltip]).overlay())
            }
            (Some(anchor), None) => Some(anchor),
            (None, Some(tooltip)) => Some(tooltip),
            (None, None) => None,
        }
    }
}

struct TooltipOverlay<'a, 'b, Message> {
    anchor_bounds: Rectangle,
    label: &'b mut Element<'a, Message>,
    label_state: &'b mut Tree,
    placement: TooltipPlacement,
}

impl<Message> overlay::Overlay<Message, crate::theme::Theme, iced::Renderer>
    for TooltipOverlay<'_, '_, Message>
{
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(
            Size::ZERO,
            Size::new(280.0_f32.min(bounds.width), bounds.height),
        );
        let intrinsic = self
            .label
            .as_widget_mut()
            .layout(self.label_state, renderer, &limits);
        let geometry = resolve_geometry(GeometryInput {
            anchor: self.anchor_bounds,
            viewport: Rectangle::with_size(bounds),
            intrinsic_content: intrinsic.size(),
            placement: placement(self.placement),
            collision: PopoverCollision::FlipAndShift,
            width: PopoverWidth::Content,
            gap: 4.0,
        });
        let limits = layout::Limits::new(Size::ZERO, geometry.frame.size())
            .width(geometry.frame.width)
            .max_height(geometry.frame.height);
        self.label
            .as_widget_mut()
            .layout(self.label_state, renderer, &limits)
            .move_to(geometry.frame.position())
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let viewport = layout.bounds();
        self.label.as_widget_mut().update(
            self.label_state,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &viewport,
        );
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        self.label
            .as_widget_mut()
            .operate(self.label_state, layout, renderer, operation);
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let viewport = layout.bounds();
        self.label.as_widget().draw(
            self.label_state,
            renderer,
            theme,
            style,
            layout,
            cursor,
            &viewport,
        );
    }
}

fn anchor_is_focused<Message>(
    anchor: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &iced::Renderer,
) -> bool {
    let mut count = operation::focusable::count();
    anchor.as_widget_mut().operate(
        tree,
        layout,
        renderer,
        &mut operation::black_box(&mut count),
    );
    matches!(
        count.finish(),
        operation::Outcome::Some(count) if count.focused.is_some()
    )
}

fn event_now(event: &Event) -> iced::time::Instant {
    match event {
        Event::Window(iced::window::Event::RedrawRequested(now)) => *now,
        _ => iced::time::Instant::now(),
    }
}

fn is_escape(event: &Event) -> bool {
    matches!(
        event,
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(Named::Escape),
            ..
        })
    )
}

fn placement(placement: TooltipPlacement) -> PopoverPlacement {
    match placement {
        TooltipPlacement::Top => PopoverPlacement::TopCenter,
        TooltipPlacement::Right => PopoverPlacement::RightCenter,
        TooltipPlacement::Bottom => PopoverPlacement::BottomCenter,
        TooltipPlacement::Left => PopoverPlacement::LeftCenter,
    }
}

fn translated_bounds(bounds: Rectangle, translation: Vector) -> Rectangle {
    Rectangle::new(
        Point::new(bounds.x + translation.x, bounds.y + translation.y),
        bounds.size(),
    )
}
