use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Renderer as _, Shell, Widget,
    },
    keyboard::{self, key::Named},
    touch, window, Background, Border, Color, Event, Length, Point, Rectangle, Shadow, Size,
    Vector,
};
use std::time::Duration;

use crate::{
    advanced::focus::{FocusState, FocusVisibility},
    theme::{BorderRole, ControlRole, ControlState},
    Element,
};

use super::{MENU_LIST_INSET, MENU_ROW_HEIGHT, MENU_ROW_RADIUS, MENU_SEPARATOR_MARGIN};

const SEPARATOR_HEIGHT: f32 = 1.0 + MENU_SEPARATOR_MARGIN * 2.0;
const TYPEAHEAD_TIMEOUT: Duration = Duration::from_millis(700);

#[derive(Debug, Clone)]
pub(super) struct MenuSlot<Message> {
    pub(super) eligible: bool,
    pub(super) separator: bool,
    activation: Option<Message>,
    label: Option<String>,
}

impl<Message> MenuSlot<Message> {
    pub(super) fn row(
        eligible: bool,
        activation: Option<Message>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            eligible,
            separator: false,
            activation,
            label: Some(label.into()),
        }
    }

    pub(super) fn separator() -> Self {
        Self {
            eligible: false,
            separator: true,
            activation: None,
            label: None,
        }
    }

    fn height(&self) -> f32 {
        if self.separator {
            SEPARATOR_HEIGHT
        } else {
            MENU_ROW_HEIGHT
        }
    }
}

pub(super) struct MenuList<'a, Message> {
    content: Element<'a, Message>,
    slots: Vec<MenuSlot<Message>>,
}

impl<'a, Message> MenuList<'a, Message> {
    pub(super) fn new(
        content: impl Into<Element<'a, Message>>,
        slots: Vec<MenuSlot<Message>>,
    ) -> Self {
        Self {
            content: content.into(),
            slots,
        }
    }
}

#[derive(Debug)]
pub(super) struct MenuListState {
    focus: FocusState,
    pub(super) highlight: Option<usize>,
    typeahead: String,
    typeahead_deadline: Option<iced::time::Instant>,
    now: Option<iced::time::Instant>,
}

impl Default for MenuListState {
    fn default() -> Self {
        Self {
            focus: FocusState::new(FocusVisibility::Auto),
            highlight: None,
            typeahead: String::new(),
            typeahead_deadline: None,
            now: None,
        }
    }
}

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer> for MenuList<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<MenuListState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(MenuListState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
        let state = tree.state.downcast_mut::<MenuListState>();
        if state
            .highlight
            .is_some_and(|index| !self.slots.get(index).is_some_and(|slot| slot.eligible))
        {
            state.highlight = first_eligible(&self.slots);
        }
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
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
        let state = tree.state.downcast_mut::<MenuListState>();
        state.focus.register(operation, None, layout.bounds());
        if state.focus.is_active() && state.highlight.is_none() {
            state.highlight = first_eligible(&self.slots);
        }
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

        let state = tree.state.downcast_mut::<MenuListState>();
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            state.now = Some(*now);
            if state
                .typeahead_deadline
                .is_some_and(|deadline| *now > deadline)
            {
                state.typeahead.clear();
                state.typeahead_deadline = None;
            }
        }
        if let Some(position) = pointer_position(event, cursor) {
            state.highlight = slot_at(&self.slots, layout.bounds(), position)
                .filter(|index| self.slots[*index].eligible);
        }
        if is_primary_press(event)
            && pointer_position(event, cursor).is_some_and(|point| layout.bounds().contains(point))
        {
            state.focus.focus_from_pointer();
        }

        if state.focus.is_active() {
            if let Event::Keyboard(keyboard::Event::KeyPressed {
                text: Some(text), ..
            }) = event
            {
                if !text.is_empty() && text.chars().all(|character| !character.is_control()) {
                    let now = state.now.unwrap_or_else(iced::time::Instant::now);
                    if state
                        .typeahead_deadline
                        .is_none_or(|deadline| now > deadline)
                    {
                        state.typeahead.clear();
                    }
                    state.typeahead.push_str(text);
                    state.typeahead_deadline = Some(now + TYPEAHEAD_TIMEOUT);
                    if let Some(index) =
                        typeahead_match(&self.slots, state.highlight, state.typeahead.as_str())
                    {
                        state.highlight = Some(index);
                    }
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
            }
            if matches!(
                event,
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::Enter | Named::Space),
                    ..
                })
            ) {
                if let Some(message) = state
                    .highlight
                    .and_then(|index| self.slots.get(index))
                    .and_then(|slot| slot.activation.clone())
                {
                    shell.publish(message);
                    shell.capture_event();
                }
                return;
            }
            let moved = match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::ArrowDown),
                    ..
                }) => move_highlight(&self.slots, state.highlight, 1),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::ArrowUp),
                    ..
                }) => move_highlight(&self.slots, state.highlight, -1),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::Home),
                    ..
                }) => first_eligible(&self.slots),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::End),
                    ..
                }) => last_eligible(&self.slots),
                _ => return,
            };
            if moved != state.highlight {
                state.highlight = moved;
                shell.request_redraw();
            }
            shell.capture_event();
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
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<MenuListState>();
        if let Some(bounds) = state
            .highlight
            .and_then(|index| slot_bounds(&self.slots, layout.bounds(), index))
        {
            let control = theme.control(ControlRole::Standard, ControlState::HOVERED);
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border::default().rounded(MENU_ROW_RADIUS),
                    shadow: Shadow::default(),
                    snap: true,
                },
                control.background,
            );
        }
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
        if state.focus.is_focus_visible() {
            if let Some(bounds) = state
                .highlight
                .and_then(|index| slot_bounds(&self.slots, layout.bounds(), index))
            {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + 1.0,
                            y: bounds.y + 1.0,
                            width: (bounds.width - 2.0).max(0.0),
                            height: (bounds.height - 2.0).max(0.0),
                        },
                        border: Border {
                            color: theme.border(BorderRole::Focus).color,
                            width: 1.0,
                            radius: MENU_ROW_RADIUS.into(),
                        },
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    Background::Color(Color::TRANSPARENT),
                );
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
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message> From<MenuList<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(list: MenuList<'a, Message>) -> Self {
        Element::new(list)
    }
}

fn first_eligible<Message>(slots: &[MenuSlot<Message>]) -> Option<usize> {
    slots.iter().position(|slot| slot.eligible)
}

fn last_eligible<Message>(slots: &[MenuSlot<Message>]) -> Option<usize> {
    slots.iter().rposition(|slot| slot.eligible)
}

fn move_highlight<Message>(
    slots: &[MenuSlot<Message>],
    current: Option<usize>,
    direction: isize,
) -> Option<usize> {
    let start = current.or_else(|| {
        if direction > 0 {
            first_eligible(slots)
        } else {
            last_eligible(slots)
        }
    })? as isize;
    let mut index = start + direction;
    while index >= 0 && (index as usize) < slots.len() {
        if slots[index as usize].eligible {
            return Some(index as usize);
        }
        index += direction;
    }
    current.or(Some(start as usize))
}

fn typeahead_match<Message>(
    slots: &[MenuSlot<Message>],
    current: Option<usize>,
    prefix: &str,
) -> Option<usize> {
    if slots.is_empty() {
        return None;
    }
    let prefix = prefix.to_lowercase();
    let start = current.unwrap_or(slots.len() - 1);
    (1..=slots.len())
        .map(|offset| (start + offset) % slots.len())
        .find(|index| {
            let slot = &slots[*index];
            slot.eligible
                && slot
                    .label
                    .as_deref()
                    .is_some_and(|label| label.to_lowercase().starts_with(&prefix))
        })
}

fn slot_at<Message>(slots: &[MenuSlot<Message>], bounds: Rectangle, point: Point) -> Option<usize> {
    slots.iter().enumerate().find_map(|(index, _)| {
        slot_bounds(slots, bounds, index)
            .is_some_and(|bounds| bounds.contains(point))
            .then_some(index)
    })
}

fn slot_bounds<Message>(
    slots: &[MenuSlot<Message>],
    bounds: Rectangle,
    target: usize,
) -> Option<Rectangle> {
    let slot = slots.get(target)?;
    let y = bounds.y
        + MENU_LIST_INSET
        + slots[..target]
            .iter()
            .map(|slot| slot.height())
            .sum::<f32>();
    Some(Rectangle::new(
        Point::new(bounds.x + MENU_LIST_INSET, y),
        Size::new(
            (bounds.width - MENU_LIST_INSET * 2.0).max(0.0),
            slot.height(),
        ),
    ))
}

fn is_primary_press(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
    )
}

fn pointer_position(event: &Event, cursor: mouse::Cursor) -> Option<Point> {
    match event {
        Event::Touch(touch::Event::FingerPressed { position, .. })
        | Event::Touch(touch::Event::FingerMoved { position, .. })
        | Event::Touch(touch::Event::FingerLifted { position, .. }) => Some(*position),
        _ => cursor.position(),
    }
}
