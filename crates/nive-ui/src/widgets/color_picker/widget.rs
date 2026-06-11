use iced::{
    advanced::{
        layout, mouse, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Color, Event, Length, Rectangle, Size,
};

use crate::{widgets::shell_relay, Element};

use super::{
    event::{ColorPickerControl, ColorPickerEvent},
    state::ColorPickerState,
    view::{color_picker_content, color_picker_size},
};

pub(super) struct ColorPickerWidget<'a, Message> {
    pub(super) value: Color,
    pub(super) disabled: bool,
    pub(super) on_change: Option<Box<dyn Fn(Color) -> Message + 'a>>,
}

impl<'a, Message> ColorPickerWidget<'a, Message>
where
    Message: 'a,
{
    fn content(&self, state: &ColorPickerState) -> Element<'a, ColorPickerEvent> {
        color_picker_content(state.snapshot(), self.read_only())
    }

    fn sync_external(&self, tree: &mut Tree) {
        tree.state
            .downcast_mut::<ColorPickerState>()
            .sync_external(self.value);
    }

    fn read_only(&self) -> bool {
        self.disabled || self.on_change.is_none()
    }

    fn focus_control(
        &self,
        content: &mut Element<'a, ColorPickerEvent>,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        control: ColorPickerControl,
    ) {
        let mut operation = operation::focusable::focus::<()>(control.id());

        content
            .as_widget_mut()
            .operate(tree, layout, renderer, &mut operation);
    }
}

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for ColorPickerWidget<'a, Message>
where
    Message: 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ColorPickerState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(ColorPickerState::new(self.value))
    }

    fn children(&self) -> Vec<Tree> {
        let state = ColorPickerState::new(self.value);
        vec![Tree::new(&self.content(&state))]
    }

    fn diff(&self, tree: &mut Tree) {
        self.sync_external(tree);

        let state = tree.state.downcast_ref::<ColorPickerState>();
        let content = self.content(state);

        tree.diff_children(&[content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        color_picker_size()
    }

    fn size_hint(&self) -> Size<Length> {
        color_picker_size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.sync_external(tree);

        let state = tree.state.downcast_ref::<ColorPickerState>();
        let mut content = self.content(state);

        content
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
        self.sync_external(tree);

        let state = tree.state.downcast_ref::<ColorPickerState>();
        let mut content = self.content(state);

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
        self.sync_external(tree);

        let state = tree.state.downcast_ref::<ColorPickerState>();
        let mut content = self.content(state);

        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            &mut local_shell,
            viewport,
        );

        shell_relay::propagate_to_parent(&mut local_shell, shell);
        drop(local_shell);

        let state = tree.state.downcast_mut::<ColorPickerState>();

        for event in local_messages {
            let transition = event.apply(state);

            if let Some(control) = transition.focus() {
                self.focus_control(
                    &mut content,
                    &mut tree.children[0],
                    layout,
                    renderer,
                    control,
                );
            }

            if let Some(color) = transition.changed() {
                if let Some(on_change) = &self.on_change {
                    shell.publish(on_change(color));
                }
            }

            if transition.redraw() {
                shell.invalidate_layout();
                shell.request_redraw();
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
        let state = tree.state.downcast_ref::<ColorPickerState>();
        let content = self.content(state);

        content
            .as_widget()
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
        let state = tree.state.downcast_ref::<ColorPickerState>();
        let content = self.content(state);

        content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }
}
