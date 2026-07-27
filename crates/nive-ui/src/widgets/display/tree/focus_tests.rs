use iced::{
    keyboard::{
        self,
        key::{Code, Named, Physical},
        Key, Location, Modifiers,
    },
    mouse,
    widget::Id,
    Event, Length, Point, Size,
};

use super::super::{row_height, TreeEvent, TreeNode, TreeState, TreeStateChange};
use super::Tree;
use crate::theme::ControlSize;
use crate::{accessibility::FocusRoot, test_support::WidgetHarness, Element};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Tree(TreeEvent<&'static str>),
}

fn arrow_down() -> Event {
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: Key::Named(Named::ArrowDown),
        modified_key: Key::Named(Named::ArrowDown),
        physical_key: Physical::Code(Code::ArrowDown),
        location: Location::Standard,
        modifiers: Modifiers::NONE,
        text: None,
        repeat: false,
    })
}

#[test]
fn pointer_focused_tree_keeps_one_composite_target_for_arrow_navigation() {
    let id = Id::new("tree");
    let mut state = TreeState::default();
    state.select_only("b");
    let tree: Element<'_, Message> = Tree::new([
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ])
    .id(id.clone())
    .state(&state)
    .height(Length::Fixed(48.0))
    .on_event(Message::Tree)
    .into();
    let mut harness = WidgetHarness::new(FocusRoot::new(tree).into(), Size::new(240.0, 80.0));

    assert_eq!(harness.managed_focus().entries.len(), 1);
    harness.set_cursor(Point::new(120.0, row_height(ControlSize::Sm) * 1.5));
    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    let result = harness.update(arrow_down());

    assert!(result.captured);
    assert!(matches!(
        result.messages.as_slice(),
        [Message::Tree(TreeEvent {
            state_change: Some(TreeStateChange::SetSelection(selection)),
            ..
        })] if selection.focused == Some("c")
    ));
}

#[test]
fn a_row_offers_the_pointer_cursor_whenever_clicking_it_does_something() {
    // A Tree handles the click itself and only hands the row's button an
    // `on_press` when that click would change state — so with nothing selected
    // the button reports as inert while the row is still fully clickable. The
    // cursor must follow the click, not the button.
    for selection in [None, Some("root")] {
        let mut state = TreeState::default();
        if let Some(id) = selection {
            state.select_only(id);
        }
        let element: Element<'_, Message> = Tree::new([TreeNode::leaf("root", "Root")])
            .state(&state)
            .on_event(Message::Tree)
            .size(ControlSize::Sm)
            .into();
        let mut harness = WidgetHarness::new(element, Size::new(240.0, 200.0));
        let at = Point::new(80.0, row_height(ControlSize::Sm) / 2.0);
        harness.set_cursor(at);
        harness.update(Event::Mouse(mouse::Event::CursorMoved { position: at }));

        let cursor = harness.mouse_interaction();
        harness.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));
        let released = harness.update(Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        )));

        assert!(
            !released.messages.is_empty(),
            "clicking a row must do something (selection={selection:?})"
        );
        assert_eq!(
            cursor,
            mouse::Interaction::Pointer,
            "a clickable row must show the pointer cursor (selection={selection:?})"
        );
    }
}

#[test]
fn hover_paints_the_whole_row_not_just_the_button() {
    use crate::test_support::pixel;

    // Indentation and the expander are siblings of the row's button, so a fill
    // painted by the button alone would stop short of the row's leading edge.
    let state: TreeState<&'static str> = TreeState::default();
    let build = |state: &'static TreeState<&'static str>| -> Element<'static, Message> {
        Tree::new([TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")])
            .state(state)
            .on_event(Message::Tree)
            .size(ControlSize::Sm)
            .into()
    };
    let state: &'static TreeState<&'static str> = Box::leak(Box::new(state));
    let width = 240.0;
    let height = row_height(ControlSize::Sm);
    let mut cold = WidgetHarness::new(build(state), Size::new(width, 120.0));
    let mut hot = WidgetHarness::new(build(state), Size::new(width, 120.0));
    let at = Point::new(width / 2.0, height / 2.0);
    hot.set_cursor(at);
    hot.update(Event::Mouse(mouse::Event::CursorMoved { position: at }));

    let (cold, hot) = (cold.pixmap(), hot.pixmap());
    let mut columns_touched = 0;
    for x in 0..width as u32 {
        let touched = (0..height as u32).any(|y| {
            let p = Point::new(x as f32, y as f32);
            pixel(&cold, p) != pixel(&hot, p)
        });
        if touched {
            columns_touched += 1;
        }
    }

    assert_eq!(
        columns_touched, width as u32,
        "hover must reach every column of the row, including the indentation and \
         expander that sit outside the button"
    );
}
