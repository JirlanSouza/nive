mod builder;
mod row_layout;

#[cfg(test)]
mod tests;

use std::borrow::Cow;

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, Tree},
        Clipboard, Layout, Renderer as _, Shell, Widget,
    },
    Event, Length, Rectangle, Size, Vector,
};

use crate::{
    theme::{ControlSize, ToneRole},
    Element,
};

/// Static, surface-neutral metadata row with protected peer slots.
///
/// Whole-row interaction belongs to a selectable host. Prefer a one-item
/// `ActionGroup` containing `ContentAction` for a trailing peer action. The
/// row uses density-resolved ControlSize minimum height, measured end ellipsis,
/// and exact-value disclosure only when constrained. A tone is valid only when
/// the visible principal/secondary cluster completely states its meaning.
pub struct DataRow<'a, Message> {
    label: Cow<'a, str>,
    value: Option<Cow<'a, str>>,
    tone: Option<ToneRole>,
    reserve_indicator: bool,
    leading: Option<Element<'a, Message>>,
    trailing: Option<Element<'a, Message>>,
    size: ControlSize,
    width: Option<Length>,
}

impl<'a, Message> From<DataRow<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(row: DataRow<'a, Message>) -> Self {
        row.into_element()
    }
}

struct DataRowLayout<'a, Message> {
    children: [Element<'a, Message>; 5],
    principal_text: Cow<'a, str>,
    secondary_text: Cow<'a, str>,
    has_indicator_slot: bool,
    leading_present: bool,
    secondary_present: bool,
    trailing_present: bool,
    indicator_width: f32,
    gap: f32,
    minimum_height: f32,
    width: Length,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TextAllocation {
    principal: f32,
    secondary: f32,
    gap: f32,
}

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for DataRowLayout<'a, Message>
where
    Message: Clone + 'a,
{
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(
            &self
                .children
                .iter()
                .map(Element::as_widget)
                .collect::<Vec<_>>(),
        );
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.layout_impl(tree, renderer, limits)
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        for ((child, tree), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child
                .as_widget_mut()
                .operate(tree, layout, renderer, operation);
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
        for ((child, tree), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
            if shell.is_event_captured() {
                break;
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
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
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
        let clip = layout.bounds().intersection(viewport).unwrap_or_default();
        renderer.with_layer(clip, |renderer| {
            for ((child, tree), layout) in self
                .children
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
            {
                child
                    .as_widget()
                    .draw(tree, renderer, theme, style, layout, cursor, &clip);
            }
        });
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        let children = self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|((child, tree), layout)| {
                child
                    .as_widget_mut()
                    .overlay(tree, layout, renderer, viewport, translation)
            })
            .collect::<Vec<_>>();
        (!children.is_empty()).then(|| overlay::Group::with_children(children).overlay())
    }
}

impl<'a, Message> From<DataRowLayout<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(row: DataRowLayout<'a, Message>) -> Self {
        Element::new(row)
    }
}
