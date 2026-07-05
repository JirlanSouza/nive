use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Renderer as _, Shell, Widget,
    },
    border::Radius,
    keyboard::{self, key},
    widget::Id,
    Color, Event, Length, Rectangle, Shadow, Size, Vector,
};

use crate::widgets::controls::button::{self as theme_button, ButtonFocusRing};
use crate::Element;

pub struct Pressable<'a, Message> {
    content: Element<'a, Message>,
    on_press: Message,
    id: Option<Id>,
    radius: Radius,
    ring: ButtonFocusRing,
}

#[derive(Debug, Default)]
struct PressableState {
    focused: bool,
}

impl<'a, Message> Pressable<'a, Message> {
    pub fn new(
        content: impl Into<Element<'a, Message>>,
        on_press: Message,
        id: Option<Id>,
        radius: Radius,
        ring: ButtonFocusRing,
    ) -> Self {
        Self {
            content: content.into(),
            on_press,
            id,
            radius,
            ring,
        }
    }
}

impl<'a, Message> Pressable<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn maybe(
        content: impl Into<Element<'a, Message>>,
        on_press: Option<Message>,
        radius: Radius,
        ring: ButtonFocusRing,
    ) -> Element<'a, Message> {
        let content = content.into();

        match on_press {
            Some(message) => Self::new(content, message, None, radius, ring).into(),
            None => content,
        }
    }
}

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer> for Pressable<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<PressableState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(PressableState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
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
        let state = tree.state.downcast_mut::<PressableState>();

        operation.focusable(self.id.as_ref(), layout.bounds(), state);
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

        if shell.is_event_captured() {
            return;
        }

        let state = tree.state.downcast_ref::<PressableState>();

        if state.focused && is_keyboard_activation(event) {
            shell.publish(self.on_press.clone());
            shell.capture_event();
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
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );

        let state = tree.state.downcast_ref::<PressableState>();

        if state.focused {
            draw_focus_ring(renderer, theme, layout.bounds(), self.radius, self.ring);
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
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl operation::Focusable for PressableState {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }
}

impl<'a, Message> From<Pressable<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(button: Pressable<'a, Message>) -> Self {
        Element::new(button)
    }
}

pub fn draw_focus_ring(
    renderer: &mut iced::Renderer,
    theme: &crate::theme::Theme,
    bounds: Rectangle,
    radius: Radius,
    ring: ButtonFocusRing,
) {
    let border = theme_button::focus_ring(theme, ring, outset_radius(radius, border_width(theme)));

    if border.width <= 0.0 || border.color.a <= 0.0 {
        return;
    }

    renderer.fill_quad(
        renderer::Quad {
            bounds: outset_bounds(bounds, border.width),
            border,
            shadow: Shadow::default(),
            snap: true,
        },
        Color::TRANSPARENT,
    );
}

fn border_width(theme: &crate::theme::Theme) -> f32 {
    theme_button::focus_ring(theme, ButtonFocusRing::Default, Radius::default()).width
}

fn outset_bounds(bounds: Rectangle, width: f32) -> Rectangle {
    Rectangle {
        x: bounds.x - width,
        y: bounds.y - width,
        width: bounds.width + width * 2.0,
        height: bounds.height + width * 2.0,
    }
}

fn outset_radius(radius: Radius, width: f32) -> Radius {
    Radius {
        top_left: radius.top_left + width,
        top_right: radius.top_right + width,
        bottom_right: radius.bottom_right + width,
        bottom_left: radius.bottom_left + width,
    }
}

pub fn is_keyboard_activation(event: &Event) -> bool {
    matches!(
        event,
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key::Named::Enter | key::Named::Space),
            repeat: false,
            ..
        })
    )
}

#[cfg(test)]
mod pressable_tests {
    use super::*;
    use iced::keyboard::{
        key::{Code, Named, Physical},
        Key, Location, Modifiers,
    };

    fn key_pressed(key: Key) -> Event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: Physical::Code(Code::Enter),
            location: Location::Standard,
            modifiers: Modifiers::NONE,
            text: None,
            repeat: false,
        })
    }

    #[test]
    fn enter_activates_focused_pressable() {
        assert!(is_keyboard_activation(&key_pressed(Key::Named(
            Named::Enter
        ))));
    }

    #[test]
    fn space_activates_focused_pressable() {
        assert!(is_keyboard_activation(&key_pressed(Key::Named(
            Named::Space
        ))));
    }

    #[test]
    fn repeated_key_press_does_not_activate_pressable() {
        let event = Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Named(Named::Enter),
            modified_key: Key::Named(Named::Enter),
            physical_key: Physical::Code(Code::Enter),
            location: Location::Standard,
            modifiers: Modifiers::NONE,
            text: None,
            repeat: true,
        });

        assert!(!is_keyboard_activation(&event));
    }

    #[test]
    fn focus_ring_bounds_follow_theme_border_width() {
        let bounds = Rectangle::new(iced::Point::new(10.0, 20.0), Size::new(30.0, 40.0));

        assert_eq!(
            outset_bounds(bounds, 2.0),
            Rectangle::new(iced::Point::new(8.0, 18.0), Size::new(34.0, 44.0))
        );
    }

    #[test]
    fn focus_ring_radius_follows_theme_border_width() {
        assert_eq!(outset_radius(Radius::new(4.0), 2.0), Radius::new(6.0));
    }
}
