use iced::{advanced::clipboard, mouse, widget::Space};

use super::{
    event_probe, named_probe, AnchoredGeometryFixture, EnsureVisibleFixture, FakeClock,
    FormStateFixture, PopupStateFixture, WidgetHarness,
};

#[test]
fn named_probe_reports_bounds_without_child_indices() {
    let content = named_probe("form-control", Space::new().width(80).height(32));
    let mut harness = WidgetHarness::<()>::new(content, iced::Size::new(320.0, 200.0));

    assert_eq!(
        harness.named_bounds("form-control"),
        Some(iced::Rectangle::new(
            iced::Point::ORIGIN,
            iced::Size::new(80.0, 32.0),
        ))
    );
    assert_eq!(harness.named_bounds("missing"), None);
}

#[test]
fn harness_preserves_event_and_clipboard_state() {
    let mut harness = WidgetHarness::new(event_probe("updated"), iced::Size::new(100.0, 80.0));
    harness.set_cursor(iced::Point::new(4.0, 4.0));
    harness.set_clipboard(clipboard::Kind::Standard, "copied");

    let result = harness.update(iced::Event::Mouse(mouse::Event::CursorEntered));

    assert_eq!(result.messages, vec!["updated"]);
    assert!(!result.captured);
    assert!(!result.layout_invalid);
    assert!(!result.input_method_enabled);
    assert_eq!(harness.clipboard(clipboard::Kind::Standard), Some("copied"));
    assert_eq!(harness.bounds().size(), iced::Size::new(24.0, 20.0));

    harness.clear_cursor();
}

#[test]
fn form_state_fixture_covers_required_interaction_modes() {
    assert!(FormStateFixture::INTERACTIVE
        .iter()
        .any(|state| state.hovered));
    assert!(FormStateFixture::INTERACTIVE
        .iter()
        .any(|state| state.focused));
    assert!(FormStateFixture::INTERACTIVE
        .iter()
        .any(|state| state.pressed));
    assert!(FormStateFixture::read_only().read_only);
    assert!(FormStateFixture::disabled().disabled);
}

#[test]
fn overlay_fixtures_are_deterministic_and_finite() {
    let mut clock = FakeClock::at(100);
    clock.advance(500);
    assert_eq!(clock.now_ms(), 600);

    let geometry = AnchoredGeometryFixture::new(
        iced::Rectangle::new(iced::Point::new(20.0, 20.0), iced::Size::new(80.0, 24.0)),
        iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(320.0, 200.0)),
        iced::Size::new(180.0, 120.0),
    );
    assert!(geometry.anchor.width.is_finite());
    assert!(geometry.viewport.height.is_finite());
    assert!(geometry.intrinsic_content.width.is_finite());

    let ensure_visible = EnsureVisibleFixture {
        viewport: geometry.viewport,
        target: geometry.anchor,
        current_offset: 0.0,
    };
    assert_eq!(ensure_visible.current_offset, 0.0);
    assert_eq!(ensure_visible.target, geometry.anchor);

    let state = PopupStateFixture::enabled();
    assert!(state.capable);
    assert!(!state.disabled);
    assert!(!state.open);
    assert!(!state.selected);
    assert!(!state.highlighted);
    assert!(!state.focused);
    assert!(!state.pressed);
}
