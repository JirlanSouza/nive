//! Private shared modal-hosting kernel consumed by [`super::DialogHost`] and
//! `CommandPalette`.
//!
//! The kernel owns scrim resolution and the base -> scrim -> content ->
//! nested-overlay paint order, inert/event-captured base content, initial
//! focus, trapped traversal, nested-overlay-first event priority,
//! single-modal-session-per-window, inactive anchor-only invoker return, and
//! a viewport-alignment input that resolves either a centered frame (Dialog)
//! or a top-centered frame (CommandPalette) from one overlay geometry. It is
//! crate-private: there is no public general-purpose modal container.
//!
//! Every open session is also reported to the enclosing
//! [`FocusRoot`](crate::accessibility::FocusRoot) on each collection pass, so
//! window-level modal activity is observable without a host opting in — see
//! [`FocusRoot::on_modal_change`](crate::accessibility::FocusRoot::on_modal_change).

mod initial_focus;
mod overlay;
mod widget;

pub(crate) use initial_focus::InitialFocusFn;
pub(crate) use widget::{ModalAlignment, ModalHost};

#[cfg(test)]
mod modal_host_tests {
    use super::{ModalAlignment, ModalHost};
    use crate::{test_support::WidgetHarness, widgets::button, Element};
    use iced::{mouse, Event, Point, Size};

    fn no_op_initial_focus<Message>() -> super::InitialFocusFn<'static, Message> {
        Box::new(|_, _, _, _| {})
    }

    /// Builds `FocusRoot(ModalHost(base))`, optionally with an open session,
    /// mirroring how the runtime roots every window's content.
    fn rooted(open: bool) -> Element<'static, &'static str> {
        let mut host = ModalHost::new(iced::widget::text("base"));
        if open {
            host = host.modal(
                iced::widget::text("content"),
                None,
                None,
                no_op_initial_focus(),
                ModalAlignment::Centered,
            );
        }
        crate::accessibility::FocusRoot::new(Element::from(host))
            .on_modal_change(|active| if active { "modal-open" } else { "modal-closed" })
            .into()
    }

    fn pump(harness: &mut WidgetHarness<'_, &'static str>) -> Vec<&'static str> {
        harness
            .update(Event::Mouse(mouse::Event::CursorMoved {
                position: Point::new(1.0, 1.0),
            }))
            .messages
    }

    #[test]
    fn an_open_session_publishes_modal_activity_to_the_focus_root() {
        let mut harness = WidgetHarness::new(rooted(true), Size::new(300.0, 200.0));

        assert_eq!(
            pump(&mut harness),
            vec!["modal-open"],
            "the kernel reports its open session without the host wiring anything"
        );
    }

    #[test]
    fn modal_activity_is_published_only_on_change() {
        let mut harness = WidgetHarness::new(rooted(true), Size::new(300.0, 200.0));
        let _opened = pump(&mut harness);

        assert!(
            pump(&mut harness).is_empty(),
            "a steady open session must not republish every event"
        );
    }

    #[test]
    fn dropping_an_open_session_republishes_inactive() {
        let mut harness = WidgetHarness::new(rooted(true), Size::new(300.0, 200.0));
        let _opened = pump(&mut harness);

        // The host simply stops rendering the modal, so the kernel never gets
        // to report a close. The per-pass recount must still settle to
        // inactive rather than pinning modal activity on forever.
        harness.replace(rooted(false));

        assert_eq!(pump(&mut harness), vec!["modal-closed"]);
    }

    #[test]
    fn a_closed_host_never_reports_modal_activity() {
        let mut harness = WidgetHarness::new(rooted(false), Size::new(300.0, 200.0));

        assert!(pump(&mut harness).is_empty());
    }

    #[test]
    fn top_centered_alignment_places_content_near_the_top() {
        let base = iced::widget::text("base");
        let host = ModalHost::new(base).modal(
            button::primary("Content")
                .id(iced::widget::Id::new("content"))
                .on_press("pressed"),
            Some("backdrop"),
            Some("escape"),
            no_op_initial_focus(),
            ModalAlignment::TopCentered(40.0),
        );
        let mut harness = WidgetHarness::new(Element::from(host), Size::new(400.0, 400.0));

        let bounds = harness.overlay_bounds().expect("open modal overlay");

        assert!(
            bounds.y < 100.0,
            "expected near-top placement, got {bounds:?}"
        );
    }

    #[test]
    fn centered_alignment_places_content_in_the_middle() {
        let base = iced::widget::text("base");
        let host = ModalHost::new(base).modal(
            button::primary("Content")
                .id(iced::widget::Id::new("content"))
                .on_press("pressed"),
            Some("backdrop"),
            Some("escape"),
            no_op_initial_focus(),
            ModalAlignment::Centered,
        );
        let mut harness = WidgetHarness::new(Element::from(host), Size::new(400.0, 400.0));

        let bounds = harness.overlay_bounds().expect("open modal overlay");

        assert!(
            bounds.y > 100.0,
            "expected centered placement, got {bounds:?}"
        );
    }

    #[test]
    fn base_content_is_inert_while_a_modal_is_open() {
        let base = button::primary("Base")
            .id(iced::widget::Id::new("base"))
            .on_press("base-pressed");
        let host: Element<'static, &'static str> = ModalHost::new(base)
            .modal(
                iced::widget::text("content"),
                None,
                None,
                no_op_initial_focus(),
                ModalAlignment::Centered,
            )
            .into();
        let mut harness = WidgetHarness::new(host, Size::new(300.0, 200.0));
        let base_bounds = harness.bounds();
        harness.set_cursor(Point::new(base_bounds.x + 5.0, base_bounds.y + 5.0));

        let result = harness.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));

        assert!(result.messages.is_empty());
    }
}
