use super::row_layout::{allocate_text, protected};
use super::*;
use crate::widgets::display::metadata::style as theme_metadata;
use iced::{mouse, widget::Space, Event, Size};

#[test]
fn allocation_preserves_principal_then_both_markers() {
    assert_eq!(
        allocate_text(80.0, 40.0, Some(60.0), 8.0, 10.0),
        TextAllocation {
            principal: 40.0,
            secondary: 32.0,
            gap: 8.0
        }
    );
    assert_eq!(
        allocate_text(35.0, 40.0, Some(60.0), 8.0, 10.0),
        TextAllocation {
            principal: 17.0,
            secondary: 10.0,
            gap: 8.0
        }
    );
    assert_eq!(
        allocate_text(15.0, 40.0, Some(60.0), 8.0, 10.0),
        TextAllocation {
            principal: 15.0,
            secondary: 0.0,
            gap: 0.0
        }
    );
}

#[test]
fn allocation_handles_complete_exact_and_principal_only_cases() {
    assert_eq!(
        allocate_text(108.0, 40.0, Some(60.0), 8.0, 10.0),
        TextAllocation {
            principal: 40.0,
            secondary: 60.0,
            gap: 8.0,
        }
    );
    assert_eq!(
        allocate_text(24.0, 40.0, None, 8.0, 10.0),
        TextAllocation {
            principal: 24.0,
            secondary: 0.0,
            gap: 0.0,
        }
    );
}

#[test]
fn only_shrink_and_fixed_slots_are_protected() {
    assert!(protected(Length::Shrink));
    assert!(protected(Length::Fixed(40.0)));
    assert!(!protected(Length::Fill));
    assert!(!protected(Length::FillPortion(3)));
}

#[test]
fn real_layout_protects_fixed_trailing_content_and_stays_finite() {
    let node = crate::test_support::layout(
        DataRow::<()>::new("Principal content that must remain visible")
            .value("Secondary metadata that yields first")
            .tone(ToneRole::Warning)
            .trailing(Space::new().width(Length::Fixed(40.0)))
            .fill_width()
            .into(),
        Size::new(140.0, 100.0),
    );
    let children = node.children();
    let trailing = children.last().expect("trailing slot");
    assert_eq!(node.size().width, 140.0);
    assert_eq!(trailing.size().width, 40.0);
    assert!(trailing.bounds().x + trailing.bounds().width <= node.size().width);
    assert!(children
        .iter()
        .all(|child| child.bounds().width.is_finite() && child.bounds().x.is_finite()));
}

#[test]
fn real_layout_keeps_indicator_reservation_and_minimum_height_stable() {
    let reserved = crate::test_support::layout(
        DataRow::<()>::new("Healthy")
            .reserve_indicator()
            .fill_width()
            .into(),
        Size::new(240.0, 100.0),
    );
    let toned = crate::test_support::layout(
        DataRow::<()>::new("Healthy").success().fill_width().into(),
        Size::new(240.0, 100.0),
    );
    assert_eq!(reserved.size(), toned.size());
    assert_eq!(reserved.children()[0].size(), toned.children()[0].size());
    assert!(reserved.size().height >= theme_metadata::metrics(ControlSize::Sm).minimum_height);
}

#[test]
fn arbitrary_leading_and_trailing_events_are_delegated() {
    let messages = crate::test_support::event_messages(
        DataRow::new("Static row")
            .leading(crate::test_support::event_probe(1_u8))
            .trailing(crate::test_support::event_probe(2_u8))
            .fill_width()
            .into(),
        Size::new(240.0, 80.0),
        Event::Mouse(mouse::Event::CursorEntered),
    );
    assert_eq!(messages, vec![1, 2]);
}
