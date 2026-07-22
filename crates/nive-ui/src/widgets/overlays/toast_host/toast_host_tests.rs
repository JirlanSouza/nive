use iced::{advanced::mouse, alignment, widget::text, Event, Size};

use crate::widgets::controls::button;

use super::host::{alignment_for, hash_toast_id, is_bottom, overlay_padding};
use super::*;

#[test]
fn default_position_is_bottom_end() {
    assert_eq!(ToastPosition::default(), ToastPosition::BottomEnd);
}

#[test]
fn position_alignment_matches_logical_corner() {
    assert_eq!(
        alignment_for(ToastPosition::TopStart),
        (alignment::Horizontal::Left, alignment::Vertical::Top)
    );
    assert_eq!(
        alignment_for(ToastPosition::BottomEnd),
        (alignment::Horizontal::Right, alignment::Vertical::Bottom)
    );
}

#[test]
fn overlay_padding_never_falls_below_narrow_clearance() {
    let padding = overlay_padding(ToastInsets::NONE);
    assert_eq!(padding.top, NARROW_CLEARANCE);
    assert_eq!(padding.left, NARROW_CLEARANCE);
}

#[test]
fn overlay_padding_widens_for_larger_safe_insets() {
    let padding = overlay_padding(ToastInsets {
        top: 40.0,
        ..ToastInsets::NONE
    });
    assert_eq!(padding.top, 40.0);
    assert_eq!(padding.bottom, NARROW_CLEARANCE);
}

#[test]
fn bottom_positions_are_recognized_for_inward_growth() {
    assert!(is_bottom(ToastPosition::BottomStart));
    assert!(is_bottom(ToastPosition::BottomEnd));
    assert!(!is_bottom(ToastPosition::TopStart));
    assert!(!is_bottom(ToastPosition::TopEnd));
}

/// `toasts()` renders through `ToastStack`, keyed by toast id, precisely
/// so a same-length swap (a dismiss immediately backfilled by promotion)
/// can't hand that slot's live widget state — an open Tooltip, mid-press
/// interaction — to whichever different toast now occupies it. That
/// guarantee only holds if the key is a stable function of toast
/// identity: the same id must always key the same, and distinct ids
/// must key apart.
#[test]
fn toast_id_hashes_are_stable_and_distinguish_identity() {
    assert_eq!(hash_toast_id(42_u64), hash_toast_id(42_u64));
    assert_ne!(hash_toast_id(1_u64), hash_toast_id(2_u64));
}

mod geometry {
    use super::*;
    use crate::test_support::WidgetHarness;

    #[derive(Clone, Copy)]
    struct FakeToast(u64);

    impl ToastPresentation for FakeToast {
        type Id = u64;

        fn id(&self) -> u64 {
            self.0
        }

        fn title(&self) -> &str {
            "Saved"
        }

        fn body(&self) -> Option<&str> {
            None
        }

        fn tone(&self) -> ToastTone {
            ToastTone::Info
        }
    }

    /// End-to-end companion to `push_keeps_newest_three_toasts` (which
    /// proves `ToastState` never *hands* the host more than three
    /// visible toasts): this proves the host's own `MAX_VISIBLE` cap
    /// holds even if a caller ignores that and hands it more anyway —
    /// each rendered card contributes exactly one focusable dismiss
    /// button, so counting those counts cards.
    #[test]
    fn host_never_renders_more_than_three_cards_even_if_handed_more() {
        let toasts: Vec<FakeToast> = (0..5).map(FakeToast).collect();
        let content: Element<'_, &'static str> = text("Content").into();
        let host: Element<'_, &'static str> = ToastHost::new(content)
            .toasts(
                toasts.iter(),
                |_id: u64| "dismiss",
                |_toast: &FakeToast| None,
            )
            .into();

        let mut harness = WidgetHarness::new(host, Size::new(400.0, 300.0));

        assert_eq!(harness.focused_count().total, MAX_VISIBLE);
    }
}

mod identity {
    use super::*;
    use crate::test_support::WidgetHarness;

    #[derive(Clone, Copy)]
    struct FakeToast(u64);

    impl ToastPresentation for FakeToast {
        type Id = u64;

        fn id(&self) -> u64 {
            self.0
        }

        fn title(&self) -> &str {
            "Saved"
        }

        fn body(&self) -> Option<&str> {
            None
        }

        fn tone(&self) -> ToastTone {
            ToastTone::Info
        }
    }

    fn host(ids: [u64; 3]) -> Element<'static, &'static str> {
        let toasts: &'static [FakeToast] =
            ids.into_iter().map(FakeToast).collect::<Vec<_>>().leak();
        let content: Element<'static, &'static str> = text("Content").into();
        ToastHost::new(content)
            .toasts(
                toasts.iter(),
                |_id: u64| "dismiss",
                |_toast: &FakeToast| None,
            )
            .into()
    }

    /// The bug this regression guards: `ToastState` always backfills a
    /// dismissal from the queue in the same call (`dismiss` -> `retain`
    /// -> `promote_queued`), so the visible count practically never
    /// changes across a single card's removal — the child count `diff`
    /// sees stays at three. A diff that only compares child count (like
    /// `iced::widget::keyed_column`'s) treats that as "nothing changed"
    /// and reuses the vacated slot's `Tree` positionally, handing a
    /// different toast whatever interaction state (here: keyboard focus)
    /// the previous occupant left behind. `ToastStack::diff` must catch
    /// this same-length identity swap and reset that slot.
    #[test]
    fn same_length_swap_does_not_carry_focus_to_the_new_occupant() {
        let mut harness = WidgetHarness::new(host([0, 1, 2]), Size::new(400.0, 300.0));

        harness.focus_next();
        harness.focus_next();
        assert_eq!(
            harness.focused_widgets(),
            1,
            "sanity: the middle card's dismiss button is focused"
        );

        // Same length (3), but id 1 (the focused slot) is replaced by a
        // different toast — exactly what a dismiss-then-promote produces.
        harness.replace(host([0, 99, 2]));

        assert_eq!(harness.focused_widgets(), 0);
    }

    #[test]
    fn repeated_relayout_without_a_diff_in_between_does_not_panic() {
        let mut harness = WidgetHarness::new(host([0, 1, 2]), Size::new(400.0, 300.0));

        harness.relayout(Size::new(200.0, 300.0));
        harness.draw();
        harness.relayout(Size::new(150.0, 300.0));
        harness.draw();
        harness.relayout(Size::new(400.0, 300.0));
        harness.draw();
    }

    fn host_with_pause(ids: [u64; 3]) -> Element<'static, &'static str> {
        let toasts: &'static [FakeToast] =
            ids.into_iter().map(FakeToast).collect::<Vec<_>>().leak();
        let content: Element<'static, &'static str> = text("Content").into();
        ToastHost::new(content)
            .on_hover("pause", "resume")
            .on_focus_within("focus-enter", "focus-exit")
            .toasts(
                toasts.iter(),
                |_id: u64| "dismiss",
                |_toast: &FakeToast| None,
            )
            .into()
    }

    #[test]
    fn resize_while_a_card_is_focused_does_not_panic() {
        let mut harness = WidgetHarness::new(host_with_pause([0, 1, 2]), Size::new(400.0, 300.0));

        harness.focus_next();
        harness.focus_next();
        assert_eq!(harness.focused_widgets(), 1);

        harness.relayout(Size::new(200.0, 300.0));
        harness.draw();
        harness.relayout(Size::new(150.0, 300.0));
        harness.draw();
    }
}

mod no_auto_focus {
    use super::*;
    use crate::test_support::WidgetHarness;

    #[derive(Clone, Copy)]
    struct FakeToast;

    impl ToastPresentation for FakeToast {
        type Id = u64;

        fn id(&self) -> u64 {
            1
        }

        fn title(&self) -> &str {
            "Saved"
        }

        fn body(&self) -> Option<&str> {
            None
        }

        fn tone(&self) -> ToastTone {
            ToastTone::Info
        }
    }

    /// Part of the 4.2 accessibility contract that *is* enforced today
    /// (the rest is preparatory, see `ToastHost`'s rustdoc): a toast
    /// becoming visible must never move keyboard focus into it.
    #[test]
    fn appearing_toast_never_auto_focuses_any_control() {
        let content: Element<'_, &'static str> = text("Content").into();
        let host: Element<'_, &'static str> = ToastHost::new(content)
            .toasts(
                std::iter::once(&FakeToast),
                |_id: u64| "dismiss",
                |_toast: &FakeToast| None,
            )
            .into();

        let mut harness = WidgetHarness::new(host, Size::new(400.0, 300.0));

        // Sanity: the toast's own dismiss button is a real focusable
        // control, so this isn't vacuously true for lack of one.
        assert!(harness.focused_count().total > 0);
        assert_eq!(harness.focused_widgets(), 0);
    }
}

mod focus_within {
    use super::*;
    use crate::test_support::WidgetHarness;
    use iced::widget::Id;

    fn focusable_area(id: Id) -> Element<'static, &'static str> {
        FocusWithinArea::new(button::ghost("Action").id(id).on_press("pressed"))
            .on_focus_within("focus-entered", "focus-left")
            .into()
    }

    /// Focus is applied immediately by the `operate()`-based
    /// `focusable::focus` operation; `FocusWithinArea` only observes it
    /// on the next `update()` pass, so tests dispatch a no-op event
    /// after focusing to give it that chance.
    fn pump(harness: &mut WidgetHarness<'_, &'static str>) -> Vec<&'static str> {
        harness
            .update(Event::Mouse(mouse::Event::CursorMoved {
                position: iced::Point::new(1.0, 1.0),
            }))
            .messages
    }

    #[test]
    fn focusing_the_content_publishes_enter() {
        let id = Id::new("toast-action");
        let mut harness = WidgetHarness::new(focusable_area(id.clone()), Size::new(200.0, 80.0));
        harness.focus(id);

        assert_eq!(pump(&mut harness), vec!["focus-entered"]);
    }

    #[test]
    fn steady_focus_does_not_republish() {
        let id = Id::new("toast-action");
        let mut harness = WidgetHarness::new(focusable_area(id.clone()), Size::new(200.0, 80.0));
        harness.focus(id);
        let _entered = pump(&mut harness);

        assert!(pump(&mut harness).is_empty());
    }

    #[test]
    fn losing_focus_publishes_exit() {
        let id = Id::new("toast-action");
        let other = Id::new("elsewhere");
        let mut harness = WidgetHarness::new(focusable_area(id.clone()), Size::new(200.0, 80.0));
        harness.focus(id);
        let _entered = pump(&mut harness);

        harness.focus(other);

        assert_eq!(pump(&mut harness), vec!["focus-left"]);
    }

    #[test]
    fn never_focused_never_publishes() {
        let id = Id::new("toast-action");
        let mut harness = WidgetHarness::new(focusable_area(id), Size::new(200.0, 80.0));

        assert!(pump(&mut harness).is_empty());
    }
}
