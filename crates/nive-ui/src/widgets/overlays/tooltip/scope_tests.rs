use std::time::Duration;

use iced::{advanced::widget::operation, widget::row, Event, Size};

use super::*;
use crate::test_support::WidgetHarness;

#[test]
fn scoped_neighbor_uses_warm_delay_and_pointer_wins() {
    let start = iced::time::Instant::now();
    let mut harness = WidgetHarness::new(
        scope_at(start, true, false, false, true),
        Size::new(320.0, 120.0),
    );
    harness.update(redraw(start));
    harness.replace(scope_at(
        start + Duration::from_millis(500),
        true,
        false,
        false,
        true,
    ));
    harness.update(redraw(start + Duration::from_millis(500)));
    assert_eq!(visible_keys(&mut harness), vec![1]);

    harness.replace(scope_at(
        start + Duration::from_millis(550),
        false,
        true,
        true,
        false,
    ));
    harness.update(redraw(start + Duration::from_millis(550)));
    assert!(visible_keys(&mut harness).is_empty());

    harness.replace(scope_at(
        start + Duration::from_millis(650),
        false,
        true,
        true,
        false,
    ));
    harness.update(redraw(start + Duration::from_millis(650)));
    assert_eq!(visible_keys(&mut harness), vec![2]);
}

#[test]
fn scoped_warm_neighbor_invalidates_layout_when_the_winner_changes() {
    let start = iced::time::Instant::now();
    let mut harness = WidgetHarness::new(
        scope_at(start, true, false, false, false),
        Size::new(320.0, 120.0),
    );
    harness.update(redraw(start));
    harness.replace(scope_at(
        start + Duration::from_millis(500),
        true,
        false,
        false,
        false,
    ));
    let first = harness.update(redraw(start + Duration::from_millis(500)));
    assert!(first.layout_invalid);
    assert_eq!(visible_keys(&mut harness), vec![1]);

    harness.replace(scope_at(
        start + Duration::from_millis(550),
        false,
        true,
        false,
        false,
    ));
    let gap = harness.update(redraw(start + Duration::from_millis(550)));
    assert!(gap.layout_invalid);

    harness.replace(scope_at(
        start + Duration::from_millis(650),
        false,
        true,
        false,
        false,
    ));
    let neighbor = harness.update(redraw(start + Duration::from_millis(650)));
    assert!(neighbor.layout_invalid);
    assert_eq!(visible_keys(&mut harness), vec![2]);
}

#[test]
fn same_key_reentry_uses_cold_delay() {
    let start = iced::time::Instant::now();
    let mut harness = WidgetHarness::new(
        scope_at(start, true, false, false, false),
        Size::new(320.0, 120.0),
    );
    harness.update(redraw(start));
    harness.replace(scope_at(
        start + Duration::from_millis(500),
        true,
        false,
        false,
        false,
    ));
    harness.update(redraw(start + Duration::from_millis(500)));
    assert_eq!(visible_keys(&mut harness), vec![1]);

    harness.replace(scope_at(
        start + Duration::from_millis(550),
        false,
        false,
        false,
        false,
    ));
    harness.update(redraw(start + Duration::from_millis(550)));
    harness.replace(scope_at(
        start + Duration::from_millis(600),
        true,
        false,
        false,
        false,
    ));
    harness.update(redraw(start + Duration::from_millis(600)));
    harness.replace(scope_at(
        start + Duration::from_millis(1_099),
        true,
        false,
        false,
        false,
    ));
    harness.update(redraw(start + Duration::from_millis(1_099)));
    assert!(visible_keys(&mut harness).is_empty());

    harness.replace(scope_at(
        start + Duration::from_millis(1_100),
        true,
        false,
        false,
        false,
    ));
    harness.update(redraw(start + Duration::from_millis(1_100)));
    assert_eq!(visible_keys(&mut harness), vec![1]);
}

#[test]
fn an_unshown_candidate_does_not_warm_its_neighbor() {
    let start = iced::time::Instant::now();
    let mut harness = WidgetHarness::new(
        scope_at(start, true, false, false, false),
        Size::new(320.0, 120.0),
    );
    harness.update(redraw(start));
    harness.replace(scope_at(
        start + Duration::from_millis(200),
        false,
        true,
        false,
        false,
    ));
    harness.update(redraw(start + Duration::from_millis(200)));
    harness.replace(scope_at(
        start + Duration::from_millis(699),
        false,
        true,
        false,
        false,
    ));
    harness.update(redraw(start + Duration::from_millis(699)));
    assert!(visible_keys(&mut harness).is_empty());

    harness.replace(scope_at(
        start + Duration::from_millis(700),
        false,
        true,
        false,
        false,
    ));
    harness.update(redraw(start + Duration::from_millis(700)));
    assert_eq!(visible_keys(&mut harness), vec![2]);
}

#[test]
fn nested_scopes_and_independent_trees_keep_separate_sessions() {
    let start = iced::time::Instant::now();
    let mut nested = WidgetHarness::new(nested_at(start), Size::new(320.0, 120.0));
    nested.update(redraw(start));
    nested.replace(nested_at(start + Duration::from_millis(500)));
    nested.update(redraw(start + Duration::from_millis(500)));
    assert_eq!(visible_keys(&mut nested), vec![1, 2]);

    let mut independent = WidgetHarness::new(
        scope_at(
            start + Duration::from_millis(500),
            true,
            false,
            false,
            false,
        ),
        Size::new(320.0, 120.0),
    );
    independent.update(redraw(start + Duration::from_millis(500)));
    independent.replace(scope_at(
        start + Duration::from_millis(600),
        true,
        false,
        false,
        false,
    ));
    independent.update(redraw(start + Duration::from_millis(600)));
    assert!(visible_keys(&mut independent).is_empty());
}

fn scope_at(
    now: iced::time::Instant,
    first_hovered: bool,
    second_hovered: bool,
    first_focused: bool,
    second_focused: bool,
) -> Element<'static, ()> {
    TooltipScope::new(row![
        Tooltip::new(iced::widget::Space::new().width(20).height(20), "First")
            .at(now)
            .intent(first_hovered, first_focused),
        Tooltip::new(iced::widget::Space::new().width(20).height(20), "Second")
            .at(now)
            .intent(second_hovered, second_focused),
    ])
    .at(now)
    .into()
}

fn nested_at(now: iced::time::Instant) -> Element<'static, ()> {
    TooltipScope::new(row![
        Tooltip::new(iced::widget::Space::new().width(20).height(20), "Outer")
            .at(now)
            .intent(true, false),
        TooltipScope::new(
            Tooltip::new(iced::widget::Space::new().width(20).height(20), "Inner")
                .at(now)
                .intent(true, false),
        )
        .at(now),
    ])
    .at(now)
    .into()
}

fn redraw(now: iced::time::Instant) -> Event {
    Event::Window(iced::window::Event::RedrawRequested(now))
}

fn visible_keys(harness: &mut WidgetHarness<'_, ()>) -> Vec<u64> {
    struct VisibleKeys {
        index: u64,
        visible: Vec<u64>,
    }

    impl operation::Operation for VisibleKeys {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
            operate(self);
        }

        fn custom(
            &mut self,
            _id: Option<&iced::advanced::widget::Id>,
            _bounds: iced::Rectangle,
            state: &mut dyn std::any::Any,
        ) {
            if let Some(state) = state.downcast_ref::<widget::TooltipState>() {
                self.index += 1;
                if state.visible {
                    self.visible.push(self.index);
                }
            }
        }
    }

    let mut keys = VisibleKeys {
        index: 0,
        visible: Vec::new(),
    };
    harness.operate(&mut keys);
    keys.visible
}
