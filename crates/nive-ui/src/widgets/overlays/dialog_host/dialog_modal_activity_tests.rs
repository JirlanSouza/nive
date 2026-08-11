use super::{DialogHost, DialogInitialFocus};
use crate::{accessibility::FocusRoot, test_support::WidgetHarness, widgets::button, Element};
use iced::{Event, Size};

/// The kernel-level coverage in `modal_host` reaches the same code through a
/// crate-private host. This is the public composition a consumer actually
/// writes — `FocusRoot(DialogHost(..).dialog(..))` — so the contract holds for
/// the API as shipped, not just for the shared kernel.
#[test]
fn an_open_dialog_lets_a_frame_signal_reach_the_focus_root() {
    let mut harness = dialog_harness(true);

    assert!(
        harness.overlay_bounds().is_some(),
        "the dialog must already be overlaying, or the dispatch below would \
         reach the root through an empty overlay phase and pass for the wrong \
         reason"
    );

    let result = harness.dispatch(frame());

    assert_eq!(
        result.messages,
        vec!["modal-open"],
        "an open dialog must still let the window observe its own modal \
         activity"
    );
    assert!(
        !result.captured,
        "reporting the frame signal as captured would drop the overlay for \
         the frame, blanking the dialog"
    );
}

/// Guards the carve-out against widening: everything that is genuinely input
/// stays swallowed by the open dialog.
#[test]
fn an_open_dialog_still_swallows_real_input_from_the_base() {
    let mut harness = dialog_harness(true);

    let result = harness.dispatch(Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: iced::keyboard::Key::Character("a".into()),
        modified_key: iced::keyboard::Key::Character("a".into()),
        physical_key: iced::keyboard::key::Physical::Code(iced::keyboard::key::Code::KeyA),
        location: iced::keyboard::Location::Standard,
        modifiers: iced::keyboard::Modifiers::NONE,
        text: None,
        repeat: false,
    }));

    assert!(result.messages.is_empty());
    assert!(result.captured);
}

fn frame() -> Event {
    Event::Window(iced::window::Event::RedrawRequested(
        iced::time::Instant::now(),
    ))
}

fn dialog_harness(open: bool) -> WidgetHarness<'static, &'static str> {
    WidgetHarness::new(rooted_dialog(open), Size::new(360.0, 240.0))
}

fn rooted_dialog(open: bool) -> Element<'static, &'static str> {
    let host = DialogHost::new(
        button::primary("Base")
            .id(iced::widget::Id::new("dialog-base"))
            .on_press("base"),
    );
    let host = if open {
        host.dialog(
            button::primary("Close")
                .id(iced::widget::Id::new("dialog-close"))
                .on_press("close"),
            Some("backdrop"),
            Some("escape"),
            DialogInitialFocus::default(),
        )
    } else {
        host
    };

    FocusRoot::new(host)
        .on_modal_change(|active| if active { "modal-open" } else { "modal-closed" })
        .into()
}
