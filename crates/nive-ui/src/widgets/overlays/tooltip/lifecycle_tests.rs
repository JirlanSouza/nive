use std::time::Duration;

use iced::{
    advanced::widget::operation,
    mouse,
    widget::{column, Id},
    Event, Point, Size,
};

use super::*;
use crate::{
    accessibility::FocusRoot,
    test_support::{named_probe, WidgetHarness},
    theme::Theme,
    widgets::{ColorInput, Input, PathInput},
};

#[test]
fn defaults_and_surface_metrics_are_exact() {
    let tooltip = Tooltip::<()>::new(iced::widget::Space::new(), "Help");
    let style = surface_style(&Theme::Dark);

    assert_eq!(tooltip.placement, TooltipPlacement::Bottom);
    assert_eq!(tooltip.delay, Duration::from_millis(500));
    assert_eq!(style.border.width, 1.0);
    assert_eq!(style.border.radius, Radius::new(4.0));
}

#[test]
fn isolated_tooltip_waits_five_hundred_milliseconds() {
    let start = iced::time::Instant::now();
    let mut harness =
        WidgetHarness::new(tooltip_at(start, 1, true, false), Size::new(320.0, 120.0));
    harness.update(redraw(start));
    assert_eq!(visible_keys(&mut harness), Vec::<u64>::new());

    harness.replace(tooltip_at(
        start + Duration::from_millis(499),
        1,
        true,
        false,
    ));
    harness.update(redraw(start + Duration::from_millis(499)));
    assert!(visible_keys(&mut harness).is_empty());

    harness.replace(tooltip_at(
        start + Duration::from_millis(500),
        1,
        true,
        false,
    ));
    harness.update(redraw(start + Duration::from_millis(500)));
    assert_eq!(visible_keys(&mut harness).len(), 1);
}

#[test]
fn isolated_tooltip_invalidates_layout_when_overlay_visibility_changes() {
    let start = iced::time::Instant::now();
    let tooltip: Element<'_, ()> = Tooltip::new(
        iced::widget::Space::new().width(20).height(20),
        "Pointer help",
    )
    .delay(Duration::ZERO)
    .at(start)
    .into();
    let mut harness = WidgetHarness::new(tooltip, Size::new(320.0, 120.0));

    harness.set_cursor(Point::new(10.0, 10.0));
    let shown = harness.update(redraw(start));
    assert!(shown.layout_invalid);
    assert!(harness.draw_overlay());

    harness.set_cursor(Point::new(100.0, 100.0));
    let hidden = harness.update(redraw(start + Duration::from_millis(1)));
    assert!(hidden.layout_invalid);
    assert!(!harness.has_overlay());
}

#[test]
fn focused_form_keeps_tooltip_layout_valid_across_color_to_path_hover() {
    let start = iced::time::Instant::now();
    let focused = Id::unique();
    let mut harness = WidgetHarness::new(
        form_tooltip_transfer_view(focused.clone()),
        Size::new(360.0, 180.0),
    );
    harness.focus(focused.clone());

    let color = harness.named_bounds("color").expect("ColorInput bounds");
    harness.set_cursor(color.center());
    harness.update(redraw(start));
    harness.replace(form_tooltip_transfer_view(focused.clone()));
    let color_shown = harness.update(redraw(start + Duration::from_millis(500)));
    assert!(color_shown.layout_invalid);
    assert!(harness.draw_overlay());

    harness.replace(form_tooltip_transfer_view(focused));
    let path = harness.named_bounds("path").expect("PathInput bounds");
    harness.set_cursor(Point::new(path.x + path.width - 5.0, path.center_y()));
    let color_hidden = harness.update(redraw(start + Duration::from_millis(501)));
    assert!(color_hidden.layout_invalid);

    let path_shown = harness.update(redraw(start + Duration::from_millis(1_001)));
    assert!(path_shown.layout_invalid);
    assert!(harness.draw_overlay());
}

#[test]
fn color_input_tooltip_closes_when_pointer_moves_to_empty_space() {
    let start = iced::time::Instant::now();
    let focused = Id::unique();
    let mut harness = WidgetHarness::new(
        form_tooltip_transfer_view(focused.clone()),
        Size::new(360.0, 180.0),
    );
    harness.focus(focused);

    let color = harness.named_bounds("color").expect("ColorInput bounds");
    harness.set_cursor(color.center());
    harness.update(redraw(start));
    harness.update(redraw(start + Duration::from_millis(500)));
    assert!(harness.has_overlay());

    let empty = Point::new(340.0, 170.0);
    harness.set_cursor(empty);
    let left = harness.update(Event::Mouse(mouse::Event::CursorMoved { position: empty }));

    assert!(left.layout_invalid);
    assert_eq!(left.redraw_request, iced::window::RedrawRequest::NextFrame);
    assert!(!harness.has_overlay());
}

#[test]
fn activating_the_anchor_dismisses_the_tooltip_until_the_pointer_returns() {
    let start = iced::time::Instant::now();
    let visible_at = start + Duration::from_millis(500);
    let mut harness =
        WidgetHarness::new(tooltip_at(start, 1, true, false), Size::new(320.0, 120.0));
    harness.update(redraw(start));
    harness.replace(tooltip_at(visible_at, 1, true, false));
    harness.update(redraw(visible_at));
    assert_eq!(visible_keys(&mut harness).len(), 1);

    // Pressing the anchor opens whatever the anchor owns, in the same place the
    // tooltip occupies, so the tooltip has to yield.
    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    assert!(
        visible_keys(&mut harness).is_empty(),
        "pressing the anchor must dismiss its tooltip"
    );

    // Still pointing at the anchor long afterwards must not bring it back, or it
    // would surface on top of what the press opened.
    let much_later = visible_at + Duration::from_secs(5);
    harness.replace(tooltip_at(much_later, 1, true, false));
    harness.update(redraw(much_later));
    assert!(
        visible_keys(&mut harness).is_empty(),
        "a resting pointer must not revive a dismissed tooltip"
    );

    // Leaving and deliberately approaching again explains the anchor once more.
    harness.replace(tooltip_at(much_later, 1, false, false));
    harness.update(redraw(much_later));
    let returned = much_later + Duration::from_millis(500);
    harness.replace(tooltip_at(much_later, 1, true, false));
    harness.update(redraw(much_later));
    harness.replace(tooltip_at(returned, 1, true, false));
    harness.update(redraw(returned));

    assert_eq!(visible_keys(&mut harness).len(), 1);
}

#[test]
fn the_focus_a_press_grants_does_not_strand_the_dismissed_tooltip() {
    let start = iced::time::Instant::now();
    let visible_at = start + Duration::from_millis(500);
    let mut harness =
        WidgetHarness::new(tooltip_at(start, 1, true, false), Size::new(320.0, 120.0));
    harness.update(redraw(start));
    harness.replace(tooltip_at(visible_at, 1, true, false));
    harness.update(redraw(visible_at));
    assert_eq!(visible_keys(&mut harness).len(), 1);

    // Activating an anchor also focuses it, so from here on the anchor is
    // focused as well as hovered.
    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    let held = visible_at + Duration::from_secs(5);
    harness.replace(tooltip_at(held, 1, true, true));
    harness.update(redraw(held));
    assert!(
        visible_keys(&mut harness).is_empty(),
        "a pointer resting on the anchor it just activated must not revive the tooltip"
    );

    // The pointer leaves while the press's focus remains. Focus alone must not
    // reveal what the reader just dismissed.
    let left = held + Duration::from_secs(1);
    harness.replace(tooltip_at(left, 1, false, true));
    harness.update(redraw(left));
    let waited = left + Duration::from_secs(1);
    harness.replace(tooltip_at(waited, 1, false, true));
    harness.update(redraw(waited));
    assert!(
        visible_keys(&mut harness).is_empty(),
        "residual focus from the press must not reveal the tooltip on its own"
    );

    // Approaching again is a fresh pointer intent and must be explained, even
    // though the anchor never lost the focus the press gave it.
    harness.replace(tooltip_at(waited, 1, true, true));
    harness.update(redraw(waited));
    let returned = waited + Duration::from_millis(500);
    harness.replace(tooltip_at(returned, 1, true, true));
    harness.update(redraw(returned));

    assert_eq!(
        visible_keys(&mut harness).len(),
        1,
        "the anchor must explain itself again once the pointer returns"
    );
}

fn tooltip_at(
    now: iced::time::Instant,
    key: u64,
    hovered: bool,
    focused: bool,
) -> Element<'static, ()> {
    Tooltip::new(
        iced::widget::Space::new().width(20).height(20),
        key.to_string(),
    )
    .at(now)
    .intent(hovered, focused)
    .into()
}

fn form_tooltip_transfer_view(focused: Id) -> Element<'static, ()> {
    FocusRoot::new(
        column![
            Input::new("Focused input", "")
                .id(focused)
                .on_change(|_| ()),
            named_probe(
                "color",
                ColorInput::new(iced::Color::BLACK).on_change(|_| ()),
            ),
            named_probe(
                "path",
                PathInput::new("Project path", "/workspace")
                    .on_change(|_| ())
                    .on_browse(()),
            ),
        ]
        .spacing(12),
    )
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
