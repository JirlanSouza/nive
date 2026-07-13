#[cfg(not(test))]
use nive_ui::Element;

#[cfg(not(test))]
pub(crate) fn probe<'a, Message>(
    _name: &'static str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    content.into()
}

#[cfg(test)]
pub(crate) use testing::{clear, probe, snapshot};

#[cfg(test)]
mod testing {
    use std::{cell::RefCell, collections::BTreeMap};

    use iced::{
        advanced::{
            layout::{self, Layout},
            mouse, overlay, renderer,
            widget::{operation, tree, Tree},
            Clipboard, Shell, Widget,
        },
        Event, Rectangle, Size, Vector,
    };

    use nive_ui::{Element, Theme};

    thread_local! {
        static BOUNDS: RefCell<BTreeMap<&'static str, Rectangle>> = const { RefCell::new(BTreeMap::new()) };
    }

    pub(crate) fn clear() {
        BOUNDS.with(|bounds| bounds.borrow_mut().clear());
    }

    pub(crate) fn snapshot() -> BTreeMap<&'static str, Rectangle> {
        BOUNDS.with(|bounds| bounds.borrow().clone())
    }

    pub(crate) fn probe<'a, Message>(
        name: &'static str,
        content: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message>
    where
        Message: 'a,
    {
        Element::new(LayoutProbe {
            name,
            content: content.into(),
        })
    }

    struct LayoutProbe<'a, Message> {
        name: &'static str,
        content: Element<'a, Message>,
    }

    impl<'a, Message> Widget<Message, Theme, iced::Renderer> for LayoutProbe<'a, Message>
    where
        Message: 'a,
    {
        fn tag(&self) -> tree::Tag {
            self.content.as_widget().tag()
        }

        fn state(&self) -> tree::State {
            self.content.as_widget().state()
        }

        fn children(&self) -> Vec<Tree> {
            self.content.as_widget().children()
        }

        fn diff(&self, tree: &mut Tree) {
            self.content.as_widget().diff(tree);
        }

        fn size(&self) -> Size<iced::Length> {
            self.content.as_widget().size()
        }

        fn size_hint(&self) -> Size<iced::Length> {
            self.content.as_widget().size_hint()
        }

        fn layout(
            &mut self,
            tree: &mut Tree,
            renderer: &iced::Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            self.content.as_widget_mut().layout(tree, renderer, limits)
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
                .operate(tree, layout, renderer, operation);
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
            self.content.as_widget_mut().update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }

        fn mouse_interaction(
            &self,
            tree: &Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &iced::Renderer,
        ) -> mouse::Interaction {
            BOUNDS.with(|bounds| {
                bounds.borrow_mut().insert(self.name, layout.bounds());
            });

            self.content
                .as_widget()
                .mouse_interaction(tree, layout, cursor, viewport, renderer)
        }

        fn draw(
            &self,
            tree: &Tree,
            renderer: &mut iced::Renderer,
            theme: &Theme,
            inherited_style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            self.content.as_widget().draw(
                tree,
                renderer,
                theme,
                inherited_style,
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
        ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
            self.content
                .as_widget_mut()
                .overlay(tree, layout, renderer, viewport, translation)
        }
    }
}
