use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Color, Event, Length, Rectangle, Size, Vector,
};

use crate::{
    advanced::pressable::{draw_focus_ring, is_keyboard_activation},
    theme::ShapeSize,
    widgets::{
        controls::button::ButtonFocusRing,
        overlays::anchored_overlay::{translated_bounds, ColorInputCompatOverlay},
    },
};

use super::{
    event::{popover_key_pressed, trigger_pressed, ColorInputEvent},
    state::ColorInputState,
    view::{color_input_popover, ColorInputPopover},
};

pub(super) struct ColorInputWidget<'a, Message> {
    value: Color,
    disabled: bool,
    tooltip: &'a str,
    popover: ColorInputPopover<'a, Message>,
    on_change: Option<Box<dyn Fn(Color) -> Message + 'a>>,
}

impl<'a, Message> ColorInputWidget<'a, Message>
where
    Message: Clone + 'a,
{
    pub(super) fn new(
        value: Color,
        disabled: bool,
        tooltip: &'a str,
        on_change: Option<Box<dyn Fn(Color) -> Message + 'a>>,
    ) -> Self {
        let enabled = !disabled && on_change.is_some();
        let state = ColorInputState::default();

        Self {
            value,
            disabled,
            tooltip,
            popover: color_input_popover(value, disabled, tooltip, enabled, &state),
            on_change,
        }
    }

    fn enabled(&self) -> bool {
        !self.disabled && self.on_change.is_some()
    }

    fn build_popover(&self, state: &ColorInputState) -> ColorInputPopover<'a, Message> {
        color_input_popover(
            state.value(self.value),
            self.disabled,
            self.tooltip,
            self.enabled(),
            state,
        )
    }

    fn refresh_popover(&mut self, state: &ColorInputState) {
        self.popover = self.build_popover(state);
    }
}

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for ColorInputWidget<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ColorInputState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(ColorInputState::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.popover.children()
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<ColorInputState>();
        if !self.enabled() {
            state.clear_focus();
        }
        let popover = self.build_popover(state);

        popover.diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.popover.size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.popover.size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let Tree {
            state, children, ..
        } = tree;
        self.refresh_popover(state.downcast_ref::<ColorInputState>());

        self.popover
            .layout_anchor(&mut children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        if self.enabled() {
            let state = tree.state.downcast_mut::<ColorInputState>();

            state.register(operation, None, layout.bounds());
        }

        self.popover
            .operate_anchor(&mut tree.children[0], layout, renderer, operation);
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
        let Tree {
            state, children, ..
        } = tree;
        self.refresh_popover(state.downcast_ref::<ColorInputState>());

        self.popover.update_anchor(
            &mut children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if shell.is_event_captured() {
            return;
        }

        let active = state.downcast_ref::<ColorInputState>().is_active();
        let pointer_pressed = trigger_pressed(self.enabled(), event, layout.bounds(), cursor);

        if self.enabled() && ((active && is_keyboard_activation(event)) || pointer_pressed) {
            let state = state.downcast_mut::<ColorInputState>();
            if pointer_pressed {
                state.focus_from_pointer();
            }
            state.toggle_with(self.value);
            shell.capture_event();
            shell.invalidate_layout();
            shell.request_redraw();
        }

        self.refresh_popover(state.downcast_ref::<ColorInputState>());
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if self.enabled() && cursor.is_over(layout.bounds()) {
            return mouse::Interaction::Pointer;
        }

        self.popover
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
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
        self.popover.draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );

        let state = tree.state.downcast_ref::<ColorInputState>();

        if self.enabled() && state.is_focus_visible() {
            draw_focus_ring(
                renderer,
                theme,
                layout.bounds(),
                theme.shape(ShapeSize::Md).radius(),
                ButtonFocusRing::Default,
            );
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
        let Tree {
            state, children, ..
        } = tree;
        self.refresh_popover(state.downcast_ref::<ColorInputState>());

        let (anchor_tree, content_tree) = children.split_at_mut(1);
        let anchor_state = &mut anchor_tree[0];
        let content_state = &mut content_tree[0];
        let ColorInputWidget {
            popover, on_change, ..
        } = self;

        let ColorInputPopover {
            anchor,
            content,
            open,
            placement,
            width,
            collision,
            gap,
        } = popover;

        let anchor_overlay =
            anchor
                .as_widget_mut()
                .overlay(anchor_state, layout, renderer, viewport, translation);

        if !*open {
            return anchor_overlay;
        }

        let state = state.downcast_mut::<ColorInputState>();
        let on_change = on_change.as_deref();

        let popover = ColorInputCompatOverlay::new(
            translated_bounds(layout.bounds(), translation),
            content,
            content_state,
            *placement,
            *width,
            *collision,
            *gap,
            Some(ColorInputEvent::Dismiss),
            move |event: ColorInputEvent, shell: &mut Shell<'_, Message>| {
                event.apply(state).relay(on_change, shell);
            },
        )
        .trap_focus(true)
        .on_key_press(popover_key_pressed);

        let popover_overlay = overlay::Element::new(Box::new(popover));

        match anchor_overlay {
            Some(anchor) => {
                Some(overlay::Group::with_children(vec![anchor, popover_overlay]).overlay())
            }
            None => Some(popover_overlay),
        }
    }
}

#[cfg(test)]
#[path = "widget_tests.rs"]
mod color_input_focus_tests;
