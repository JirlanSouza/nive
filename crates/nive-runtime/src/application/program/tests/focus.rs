use super::*;

mod runtime_ui;

use runtime_ui::RuntimeUiState;

fn assert_single_focus_root<'a, Message: 'a>(element: nive_ui::Element<'a, Message>) {
    let tree = iced::advanced::widget::Tree::new(&element);
    let probe: nive_ui::Element<'a, Message> =
        nive_ui::accessibility::FocusRoot::new(iced::widget::text("")).into();
    let root_tag = iced::advanced::widget::Tree::new(&probe).tag;

    assert_eq!(tree.tag, root_tag);
    assert_eq!(tree.children.len(), 1);
    assert_ne!(tree.children[0].tag, root_tag);
}

struct RuntimeViewHarness<'a> {
    element: nive_ui::Element<'a, RuntimeMessage<TestApp>>,
    tree: iced::advanced::widget::Tree,
    node: iced::advanced::layout::Node,
    renderer: nive_ui::Renderer,
    cursor: iced::advanced::mouse::Cursor,
    viewport: iced::Rectangle,
}

#[derive(Debug, Clone, PartialEq)]
struct ManagedTarget {
    id: Option<iced::widget::Id>,
    current: bool,
    active: bool,
    visible: bool,
}

impl<'a> RuntimeViewHarness<'a> {
    fn new(mut element: nive_ui::Element<'a, RuntimeMessage<TestApp>>) -> Self {
        let viewport = iced::Rectangle::with_size(Size::new(480.0, 180.0));
        let mut tree = iced::advanced::widget::Tree::new(&element);
        let renderer = iced_renderer::fallback::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            iced::Font::default(),
            iced::Pixels(14.0),
        ));
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &iced::advanced::layout::Limits::new(Size::ZERO, viewport.size()),
        );

        Self {
            element,
            tree,
            node,
            renderer,
            cursor: iced::advanced::mouse::Cursor::Unavailable,
            viewport,
        }
    }

    fn operate(&mut self, operation: &mut dyn iced::advanced::widget::Operation) {
        self.element.as_widget_mut().operate(
            &mut self.tree,
            iced::advanced::Layout::new(&self.node),
            &self.renderer,
            operation,
        );
    }

    fn focus(&mut self, id: iced::widget::Id) {
        self.operate(&mut iced::advanced::widget::operation::focusable::focus(id));
    }

    fn navigate(&mut self, direction: nive_ui::accessibility::FocusDirection) {
        direction.operate(|operation| self.operate(operation));
    }

    fn update(&mut self, event: iced::Event) {
        let mut messages = Vec::new();
        let mut shell = iced::advanced::Shell::new(&mut messages);
        let mut clipboard = iced::advanced::clipboard::Null;
        self.element.as_widget_mut().update(
            &mut self.tree,
            &event,
            iced::advanced::Layout::new(&self.node),
            self.cursor,
            &self.renderer,
            &mut clipboard,
            &mut shell,
            &self.viewport,
        );
    }

    fn managed_targets(&mut self) -> Vec<ManagedTarget> {
        #[derive(Default)]
        struct Probe(Vec<ManagedTarget>);

        impl iced::advanced::widget::Operation for Probe {
            fn traverse(
                &mut self,
                operate: &mut dyn FnMut(&mut dyn iced::advanced::widget::Operation),
            ) {
                operate(self);
            }

            fn custom(
                &mut self,
                id: Option<&iced::widget::Id>,
                _bounds: iced::Rectangle,
                state: &mut dyn std::any::Any,
            ) {
                let Some(state) = state.downcast_ref::<nive_ui::advanced::focus::FocusState>()
                else {
                    return;
                };
                self.0.push(ManagedTarget {
                    id: id.cloned(),
                    current: iced::advanced::widget::operation::Focusable::is_focused(state),
                    active: state.is_active(),
                    visible: state.is_focus_visible(),
                });
            }
        }

        let mut probe = Probe::default();
        self.operate(&mut probe);
        probe.0
    }
}

#[test]
fn runtime_wraps_normal_secondary_and_host_composition_once() {
    let now = Instant::now();
    let mut program = program();
    let main_id = main_window_id(&program);
    let secondary_id = window::Id::unique();
    program
        .core
        .registry
        .set_opened(WindowHandle::new(TestWindow::Secondary, secondary_id));
    if let Some(app) = program.app.as_mut() {
        app.show_dialog = true;
    }
    let _toast = program.core.toasts.push(Toast::info("Saved"), now, None);

    assert_single_focus_root(program.view(main_id));
    assert_single_focus_root(program.view(secondary_id));
}

#[test]
fn runtime_wraps_bootstrap_composition_once() {
    let (program, _task) = plain_program::<BootstrapTestApp>()
        .unwrap_or_else(|error| panic!("test program failed: {error}"));
    let bootstrap_id = program
        .bootstrap
        .as_ref()
        .map(|bootstrap| bootstrap.window_id)
        .unwrap_or_else(window::Id::unique);

    assert_single_focus_root(program.view(bootstrap_id));
}

#[cfg(feature = "devtools")]
#[test]
fn runtime_wraps_devtools_composition_once() {
    let program = devtools_program(DevtoolsConfig::from_env_value(Some("open")));
    let devtools_id = program
        .devtools
        .as_ref()
        .and_then(|devtools| devtools.window_id)
        .unwrap_or_else(window::Id::unique);

    assert_single_focus_root(program.view(devtools_id));
}

#[test]
fn ignored_tab_traversal_uses_root_anchor_and_keyboard_visibility() {
    let program = program();
    let main_id = main_window_id(&program);
    let mut harness = RuntimeViewHarness::new(program.view(main_id));
    let first = iced::widget::Id::new("runtime-focus-first");
    let input = iced::widget::Id::new("runtime-focus-input");

    harness.focus(first.clone());
    assert!(harness.managed_targets().iter().any(|target| {
        target.id.as_ref() == Some(&first) && target.current && target.active && target.visible
    }));

    harness.cursor = iced::advanced::mouse::Cursor::Available(Point::new(440.0, 160.0));
    harness.update(iced::Event::Mouse(
        iced::advanced::mouse::Event::ButtonPressed(iced::advanced::mouse::Button::Left),
    ));
    assert!(harness.managed_targets().iter().any(|target| {
        target.id.as_ref() == Some(&first) && target.current && !target.active && !target.visible
    }));

    harness.update(iced::Event::Keyboard(keyboard::Event::ModifiersChanged(
        keyboard::Modifiers::default(),
    )));
    harness.navigate(nive_ui::accessibility::FocusDirection::Next);

    let targets = harness.managed_targets();
    assert_eq!(targets.iter().filter(|target| target.active).count(), 1);
    assert!(targets.iter().any(|target| {
        target.id.as_ref() == Some(&input) && target.current && target.active && target.visible
    }));
}

#[test]
fn native_input_pointer_blur_keeps_the_runtime_traversal_anchor() {
    let program = program();
    let main_id = main_window_id(&program);
    let mut harness = RuntimeViewHarness::new(program.view(main_id));
    let input = iced::widget::Id::new("runtime-focus-input");
    let next = iced::widget::Id::new("runtime-focus-second");

    harness.cursor = iced::advanced::mouse::Cursor::Available(Point::new(20.0, 42.0));
    harness.update(iced::Event::Mouse(
        iced::advanced::mouse::Event::ButtonPressed(iced::advanced::mouse::Button::Left),
    ));
    assert!(harness
        .managed_targets()
        .iter()
        .any(|target| { target.id.as_ref() == Some(&input) && target.current && target.active }));

    harness.cursor = iced::advanced::mouse::Cursor::Available(Point::new(440.0, 160.0));
    harness.update(iced::Event::Mouse(
        iced::advanced::mouse::Event::ButtonPressed(iced::advanced::mouse::Button::Left),
    ));
    assert!(harness.managed_targets().iter().any(|target| {
        target.id.as_ref() == Some(&input) && target.current && !target.active && !target.visible
    }));

    harness.update(iced::Event::Keyboard(keyboard::Event::ModifiersChanged(
        keyboard::Modifiers::default(),
    )));
    harness.navigate(nive_ui::accessibility::FocusDirection::Next);

    assert!(harness.managed_targets().iter().any(|target| {
        target.id.as_ref() == Some(&next) && target.current && target.active && target.visible
    }));
}

/// End-to-end acceptance for the modal/toast contract, through the real
/// `view()` composition and the runtime's own event order rather than a
/// hand-fed `ModalActive` message: an open dialog must make the window report
/// modal activity on the follow-up frame, and a closed one must report it
/// gone, so toast expiry pauses for exactly as long as the dialog is up.
#[test]
fn an_open_dialog_pauses_toast_expiry_through_the_real_frame_path() {
    let now = Instant::now();
    let mut program = program();
    let main_id = main_window_id(&program);
    let _toast = program.core.toasts.push(Toast::info("Saved"), now, None);
    if let Some(app) = program.app.as_mut() {
        app.show_dialog = true;
    }

    let mut runtime = RuntimeUiState::new(Size::new(480.0, 180.0));
    let opened = runtime.dispatch(program.view(main_id), redraw_requested());

    assert!(
        opened
            .messages
            .iter()
            .any(|message| matches!(message, NiveMessage::Core(CoreMessage::ModalActive(true)))),
        "an open dialog must report modal activity on a plain frame, or the \
         runtime never learns a modal is up"
    );
    assert_eq!(
        opened.status,
        iced::event::Status::Ignored,
        "the frame must stay ignored so the runtime retains the overlay for drawing"
    );
    for message in opened.messages {
        let _task = program.update(message);
    }

    let _task = program.update_core(CoreMessage::ToastTick(now + Duration::from_secs(5)));
    assert!(
        program.core.toasts.has_visible(),
        "toast expiry must pause while the dialog is open"
    );

    if let Some(app) = program.app.as_mut() {
        app.show_dialog = false;
    }

    // The runtime cache carries the same window state into the rebuilt view,
    // which is the only way the closing edge is observable at all.
    let closed = runtime.dispatch(program.view(main_id), redraw_requested());

    assert!(
        closed
            .messages
            .iter()
            .any(|message| matches!(message, NiveMessage::Core(CoreMessage::ModalActive(false)))),
        "closing the dialog must report modal activity gone"
    );
    for message in closed.messages {
        let _task = program.update(message);
    }

    let _task = program.update_core(CoreMessage::ToastTick(now + Duration::from_secs(9)));
    assert!(
        !program.core.toasts.has_visible(),
        "toast expiry must resume once the dialog closes"
    );
}

fn redraw_requested() -> iced::Event {
    iced::Event::Window(window::Event::RedrawRequested(Instant::now()))
}
