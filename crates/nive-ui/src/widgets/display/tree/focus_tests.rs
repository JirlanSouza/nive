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
