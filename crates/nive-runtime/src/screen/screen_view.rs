use crate::DialogRequest;

/// The render output of an [`Application`](crate::Application) window: its
/// content element plus an optional modal dialog.
///
/// Returned by [`Application::view`](crate::Application::view). The runtime
/// wraps the content in a `DialogHost` so a dialog, when present, is composed
/// automatically.
pub struct ScreenView<'a, Message, Theme = nive_ui::Theme, Renderer = nive_ui::Renderer> {
    /// The window's main content.
    pub content: iced::Element<'a, Message, Theme, Renderer>,
    /// An optional modal dialog to compose over the content.
    pub dialog: Option<DialogRequest<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> ScreenView<'a, Message, Theme, Renderer>
where
    Message: 'a,
{
    pub fn new(content: impl Into<iced::Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            dialog: None,
        }
    }

    pub fn dialog(mut self, dialog: DialogRequest<'a, Message, Theme, Renderer>) -> Self {
        self.dialog = Some(dialog);
        self
    }

    pub fn dialog_maybe(
        mut self,
        dialog: Option<DialogRequest<'a, Message, Theme, Renderer>>,
    ) -> Self {
        self.dialog = dialog;
        self
    }

    pub fn map<T: 'a>(
        self,
        map_message: impl Fn(Message) -> T + Copy + 'a,
    ) -> ScreenView<'a, T, Theme, Renderer>
    where
        Theme: 'a,
        Renderer: iced::advanced::Renderer + 'a,
    {
        ScreenView {
            content: self.content.map(map_message),
            dialog: self.dialog.map(|dialog| dialog.map(map_message)),
        }
    }
}

impl<'a, Message> ScreenView<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn has_dialog(&self) -> bool {
        self.dialog.is_some()
    }

    pub fn into_element(self) -> nive_ui::Element<'a, Message> {
        match self.dialog {
            Some(dialog) => {
                let (content, dismiss, initial_focus, id) = dialog.into_parts();
                let on_backdrop = dismiss.on_backdrop();
                let on_escape = dismiss.on_escape();

                let mut host = nive_ui::widgets::overlays::DialogHost::new(self.content).dialog(
                    content,
                    on_backdrop,
                    on_escape,
                    initial_focus,
                );
                if let Some(id) = id {
                    host = host.dialog_id(id);
                }
                host.into()
            }
            None => nive_ui::widgets::overlays::DialogHost::new(self.content).into(),
        }
    }
}

#[cfg(test)]
mod screen_view_tests {
    use super::*;
    use crate::DialogDismiss;
    use iced::widget::text;

    #[test]
    fn map_preserves_dialog_and_maps_dismiss_message() {
        let screen: ScreenView<'_, u8> = ScreenView::new(text("content")).dialog(
            DialogRequest::new(text("dialog")).dismiss(DialogDismiss::backdrop_or_escape(3_u8)),
        );

        let mapped = screen.map(|message| message + 1);

        match mapped.dialog {
            Some(dialog) => {
                assert_eq!(dialog.dismiss_policy().on_backdrop(), Some(4));
                assert_eq!(dialog.dismiss_policy().on_escape(), Some(4));
            }
            None => panic!("dialog must exist"),
        }
    }

    // --- 6.10: ScreenView -> DialogHost composition, proved through the
    // public `into_element()` path rather than re-asserting DialogHost's own
    // internals (already covered exhaustively at the `nive-ui` level).
    //
    // "Primary/secondary windows" and "exactly one automatic DialogHost" are
    // proved at the full runtime-composition level instead of duplicated
    // here: see `application::program::tests::focus::
    // runtime_wraps_normal_secondary_and_host_composition_once` (asserts a
    // single `FocusRoot`/`DialogHost` wrap for both a main and a secondary
    // window with a dialog shown) and `..._wraps_bootstrap_composition_once`.
    // "Bootstrap" hosting a dialog is covered by `nive-ui`'s
    // `BootstrapView::details_dialog()` tests plus 6.7's note that
    // `program.rs`'s construction compiles unchanged against this contract.

    fn layout<'a, Message>(
        element: &mut nive_ui::Element<'a, Message>,
        viewport: iced::Size,
    ) -> (iced::advanced::widget::Tree, iced::advanced::layout::Node) {
        let renderer = iced_renderer::fallback::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            iced::Font::default(),
            iced::Pixels(14.0),
        ));
        let mut tree = iced::advanced::widget::Tree::new(&*element);
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &iced::advanced::layout::Limits::new(iced::Size::ZERO, viewport),
        );
        (tree, node)
    }

    fn renderer() -> nive_ui::Renderer {
        iced_renderer::fallback::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            iced::Font::default(),
            iced::Pixels(14.0),
        ))
    }

    #[test]
    fn into_element_without_a_dialog_has_no_overlay() {
        let mut element: nive_ui::Element<'_, u8> = ScreenView::new(text("content")).into_element();
        let viewport = iced::Size::new(400.0, 300.0);
        let (mut tree, node) = layout(&mut element, viewport);
        let renderer = renderer();

        let overlay = element.as_widget_mut().overlay(
            &mut tree,
            iced::advanced::Layout::new(&node),
            &renderer,
            &iced::Rectangle::with_size(viewport),
            iced::Vector::ZERO,
        );

        assert!(overlay.is_none(), "no dialog must mean no overlay layer");
    }

    #[test]
    fn into_element_with_a_dialog_produces_an_overlay() {
        let mut element: nive_ui::Element<'_, u8> = ScreenView::new(text("content"))
            .dialog(DialogRequest::new(text("dialog")))
            .into_element();
        let viewport = iced::Size::new(400.0, 300.0);
        let (mut tree, node) = layout(&mut element, viewport);
        let renderer = renderer();

        let overlay = element.as_widget_mut().overlay(
            &mut tree,
            iced::advanced::Layout::new(&node),
            &renderer,
            &iced::Rectangle::with_size(viewport),
            iced::Vector::ZERO,
        );

        assert!(
            overlay.is_some(),
            "a configured dialog must produce a composed overlay layer"
        );
    }

    fn dialog_screen<'a>(dismiss: DialogDismiss<u8>) -> ScreenView<'a, u8> {
        ScreenView::new(text("content")).dialog(DialogRequest::new(text("dialog")).dismiss(dismiss))
    }

    #[test]
    fn into_element_wires_separate_backdrop_and_escape_messages() {
        let viewport = iced::Size::new(400.0, 300.0);
        let renderer = renderer();

        // Backdrop route: a left click outside the (small, centered) dialog
        // bounds must publish exactly the backdrop message, never the
        // escape message.
        let mut backdrop_element: nive_ui::Element<'_, u8> =
            dialog_screen(DialogDismiss::none().with_backdrop(11).with_escape(22)).into_element();
        let (mut tree, node) = layout(&mut backdrop_element, viewport);
        let mut messages = Vec::new();
        let mut shell = iced::advanced::Shell::new(&mut messages);
        let mut clipboard = iced::advanced::clipboard::Null;
        let mut overlay = backdrop_element
            .as_widget_mut()
            .overlay(
                &mut tree,
                iced::advanced::Layout::new(&node),
                &renderer,
                &iced::Rectangle::with_size(viewport),
                iced::Vector::ZERO,
            )
            .expect("dialog overlay");
        let overlay_node = overlay.as_overlay_mut().layout(&renderer, viewport);
        overlay.as_overlay_mut().update(
            &iced::Event::Mouse(iced::advanced::mouse::Event::ButtonPressed(
                iced::advanced::mouse::Button::Left,
            )),
            iced::advanced::Layout::new(&overlay_node),
            iced::advanced::mouse::Cursor::Available(iced::Point::new(5.0, 5.0)),
            &renderer,
            &mut clipboard,
            &mut shell,
        );
        drop(overlay);
        assert_eq!(messages, vec![11]);

        // Escape route: a fresh dialog session, Escape key published, must
        // publish exactly the (distinct) escape message.
        let mut escape_element: nive_ui::Element<'_, u8> =
            dialog_screen(DialogDismiss::none().with_backdrop(11).with_escape(22)).into_element();
        let (mut tree, node) = layout(&mut escape_element, viewport);
        let mut messages = Vec::new();
        let mut shell = iced::advanced::Shell::new(&mut messages);
        let mut clipboard = iced::advanced::clipboard::Null;
        let mut overlay = escape_element
            .as_widget_mut()
            .overlay(
                &mut tree,
                iced::advanced::Layout::new(&node),
                &renderer,
                &iced::Rectangle::with_size(viewport),
                iced::Vector::ZERO,
            )
            .expect("dialog overlay");
        let overlay_node = overlay.as_overlay_mut().layout(&renderer, viewport);
        let key = iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape);
        overlay.as_overlay_mut().update(
            &iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: key.clone(),
                modified_key: key,
                physical_key: iced::keyboard::key::Physical::Code(
                    iced::keyboard::key::Code::Escape,
                ),
                location: iced::keyboard::Location::Standard,
                modifiers: iced::keyboard::Modifiers::NONE,
                text: None,
                repeat: false,
            }),
            iced::advanced::Layout::new(&overlay_node),
            iced::advanced::mouse::Cursor::Unavailable,
            &renderer,
            &mut clipboard,
            &mut shell,
        );
        drop(overlay);
        assert_eq!(messages, vec![22]);
    }

    #[test]
    fn a_later_dialog_call_replaces_rather_than_stacks() {
        let screen: ScreenView<'_, u8> = ScreenView::new(text("content"))
            .dialog(DialogRequest::new(text("first")).dismiss(DialogDismiss::backdrop(1)))
            .dialog(DialogRequest::new(text("second")).dismiss(DialogDismiss::backdrop(2)));

        let dialog = screen.dialog.expect("exactly one dialog request");
        assert_eq!(dialog.dismiss_policy().on_backdrop(), Some(2));
    }
}
