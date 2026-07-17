use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Renderer as _, Shell, Widget,
    },
    Background, Event, Length, Rectangle, Size as IcedSize, Vector,
};

use super::style::{self as theme_input_group, InputGroupStatus, InputGroupVariant};
use crate::theme::FieldValidation;
use crate::Element;

pub(super) struct InputGroupFrame<'a, Message> {
    pub(super) content: Element<'a, Message>,
    pub(super) variant: InputGroupVariant,
    pub(super) input_validation: FieldValidation,
    pub(super) radius: f32,
    pub(super) disabled: bool,
}

#[derive(Debug, Default)]
struct InputGroupFrameState {
    focused: bool,
    hovered: bool,
}

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for InputGroupFrame<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<InputGroupFrameState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(InputGroupFrameState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }

    fn size(&self) -> IcedSize<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> IcedSize<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
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
        renderer: &iced::Renderer,
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
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let hovered = !self.disabled && cursor.is_over(layout.bounds());
        let state = tree.state.downcast_mut::<InputGroupFrameState>();

        if state.hovered != hovered {
            state.hovered = hovered;
            shell.request_redraw();
        }

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

        let focused = super::super::input::adapter::content_has_visual_focus(
            &mut self.content,
            &mut tree.children[0],
            layout,
            renderer,
        );
        let state = tree.state.downcast_mut::<InputGroupFrameState>();

        if state.focused != focused {
            state.focused = focused;
            shell.request_redraw();
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
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<InputGroupFrameState>();
        let status = frame_status(self.disabled, state);
        let style =
            theme_input_group::style(self.variant, self.input_validation, self.radius, status)(
                theme,
            );

        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border: style.border,
                shadow: style.shadow,
                snap: style.snap,
            },
            style
                .background
                .unwrap_or(Background::Color(iced::Color::TRANSPARENT)),
        );

        let inherited_style = renderer::Style {
            text_color: style.text_color.unwrap_or(inherited_style.text_color),
        };

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &inherited_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

fn frame_status(disabled: bool, state: &InputGroupFrameState) -> InputGroupStatus {
    if disabled {
        InputGroupStatus::Disabled
    } else if state.focused {
        InputGroupStatus::Focused
    } else if state.hovered {
        InputGroupStatus::Hovered
    } else {
        InputGroupStatus::Active
    }
}

#[cfg(test)]
mod input_group_frame_tests {
    use super::*;

    #[test]
    fn hovered_frame_uses_hovered_status() {
        let state = InputGroupFrameState {
            hovered: true,
            ..InputGroupFrameState::default()
        };

        assert_eq!(frame_status(false, &state), InputGroupStatus::Hovered);
    }

    #[test]
    fn focused_frame_keeps_focus_status_over_hover() {
        let state = InputGroupFrameState {
            focused: true,
            hovered: true,
        };

        assert_eq!(frame_status(false, &state), InputGroupStatus::Focused);
    }

}
