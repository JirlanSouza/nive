use iced::{
    advanced::{
        layout, mouse, renderer,
        widget::{operation, Tree},
        Clipboard, Layout, Shell,
    },
    Color, Event, Length, Rectangle, Size,
};

use crate::{
    widgets::{ColorPicker, ColorSwatch, PopoverCollision, PopoverPlacement, PopoverWidth},
    Element,
};

use super::{event::ColorInputEvent, state::ColorInputState};

const PICKER_GAP: f32 = 12.0;

pub(super) struct ColorInputPopover<'a, Message> {
    pub(super) anchor: Element<'a, Message>,
    pub(super) content: Element<'a, ColorInputEvent>,
    pub(super) open: bool,
    pub(super) placement: PopoverPlacement,
    pub(super) width: PopoverWidth,
    pub(super) collision: PopoverCollision,
    pub(super) gap: f32,
}

impl<'a, Message> ColorInputPopover<'a, Message>
where
    Message: Clone + 'a,
{
    pub(super) fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.anchor), Tree::new(&self.content)]
    }

    pub(super) fn diff(&self, tree: &mut Tree) {
        if tree.children.len() > 2 {
            tree.children.truncate(2);
        }

        if tree.children.is_empty() {
            tree.children.push(Tree::new(&self.anchor));
        } else {
            tree.children[0].diff(self.anchor.as_widget());
        }

        if tree.children.len() < 2 {
            tree.children.push(Tree::new(&self.content));
        } else {
            tree.children[1].diff(self.content.as_widget());
        }
    }

    pub(super) fn size(&self) -> Size<Length> {
        self.anchor.as_widget().size()
    }

    pub(super) fn size_hint(&self) -> Size<Length> {
        self.anchor.as_widget().size_hint()
    }

    pub(super) fn layout_anchor(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.anchor.as_widget_mut().layout(tree, renderer, limits)
    }

    pub(super) fn operate_anchor(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        self.anchor
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
    }

    pub(super) fn update_anchor(
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
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }

    pub(super) fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.anchor
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    pub(super) fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.anchor.as_widget().draw(
            tree,
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }
}

pub(super) fn color_input_popover<'a, Message>(
    value: Color,
    disabled: bool,
    tooltip: &'a str,
    enabled: bool,
    state: &ColorInputState,
) -> ColorInputPopover<'a, Message>
where
    Message: Clone + 'a,
{
    ColorInputPopover {
        anchor: trigger(value, disabled, tooltip, enabled, state),
        content: panel(value, disabled, enabled),
        open: state.is_open() && enabled,
        placement: PopoverPlacement::BottomCenter,
        width: PopoverWidth::Content,
        collision: PopoverCollision::FlipAndShift,
        gap: PICKER_GAP,
    }
}

fn trigger<'a, Message>(
    value: Color,
    disabled: bool,
    tooltip: &'a str,
    enabled: bool,
    state: &ColorInputState,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    ColorSwatch::new(value)
        .size(20.0)
        .selected(state.is_open() && enabled)
        .disabled(disabled)
        .tooltip(tooltip)
        .into()
}

fn panel<'a>(value: Color, disabled: bool, enabled: bool) -> Element<'a, ColorInputEvent> {
    ColorPicker::<ColorInputEvent>::new(value)
        .disabled(disabled)
        .on_change_maybe(enabled.then_some(ColorInputEvent::Change))
        .into()
}
