mod builder;
mod draw;
mod update;

#[cfg(test)]
mod tests;

use std::borrow::Cow;

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    touch, widget, Event, Length, Rectangle, Size, Vector,
};

use crate::advanced::focus::FocusState;
use crate::theme::{choice::ChoicePersistentState, ControlSize, FieldValidation};
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SingleChoiceKind {
    Checkbox,
    Radio,
    Switch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SingleChoiceLayout {
    Leading,
    Setting,
}

pub(super) struct SingleChoice<'a, Message> {
    kind: SingleChoiceKind,
    layout: SingleChoiceLayout,
    label: Cow<'a, str>,
    description: Option<Cow<'a, str>>,
    persistent: ChoicePersistentState,
    validation: FieldValidation,
    size: ControlSize,
    width: Length,
    disabled: bool,
    id: Option<widget::Id>,
    on_activate: Option<Message>,
    register_focus: bool,
    focused_override: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PressSource {
    Pointer,
    Touch(touch::Finger),
    Space,
}

#[derive(Debug, Default)]
struct SingleChoiceState {
    focus: FocusState,
    press: Option<PressSource>,
}

impl<Message> Widget<Message, crate::theme::Theme, iced::Renderer> for SingleChoice<'_, Message>
where
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SingleChoiceState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(SingleChoiceState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(self.content())]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content().as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn size_hint(&self) -> Size<Length> {
        self.size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let mut content = self.content();
        content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &limits.width(self.width))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<SingleChoiceState>();

        if self.register_focus && self.on_activate.is_some() && !self.disabled {
            state
                .focus
                .register(operation, self.id.as_ref(), layout.bounds());
        } else {
            state.focus.clear();
        }

        let mut content = self.content();
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
        self.update_impl(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if self.on_activate.is_some() && !self.disabled && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
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
        self.draw_impl(
            tree,
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'a>(
        &'a mut self,
        _tree: &'a mut Tree,
        _layout: Layout<'a>,
        _renderer: &iced::Renderer,
        _viewport: &Rectangle,
        _translation: Vector,
    ) -> Option<overlay::Element<'a, Message, crate::theme::Theme, iced::Renderer>> {
        None
    }
}

impl<'a, Message> From<SingleChoice<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(choice: SingleChoice<'a, Message>) -> Self {
        Element::new(choice)
    }
}
