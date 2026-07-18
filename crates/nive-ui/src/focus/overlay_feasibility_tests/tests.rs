use super::*;

#[derive(Default)]
struct DelegatedOperation {
    containers: usize,
}

impl operation::Operation for DelegatedOperation {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
        operate(self);
    }

    fn container(&mut self, _id: Option<&Id>, _bounds: Rectangle) {
        self.containers += 1;
    }
}

#[test]
fn root_adapter_binds_base_and_nested_overlays_without_changing_overlay_behavior() {
    let calls = Arc::new(Mutex::new(Calls::default()));
    let renderer = crate::test_support::renderer();
    let mut delegated = DelegatedOperation::default();
    let mut base = ManagedState { level: 0 };
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(200.0, 120.0));

    {
        let mut binding = BindingOperation {
            inner: &mut delegated,
            calls: Arc::clone(&calls),
        };
        binding.custom(None, bounds, &mut base);
        binding.container(None, bounds);
    }

    let parent = ProbeOverlay {
        level: 1,
        size: Size::new(120.0, 80.0),
        index: 7.0,
        calls: Arc::clone(&calls),
        state: ManagedState { level: 1 },
        nested: Some(ProbeNested {
            level: 2,
            size: Size::new(60.0, 40.0),
            index: 9.0,
            calls: Arc::clone(&calls),
            state: ManagedState { level: 2 },
        }),
    };
    let mut overlay =
        RootOverlay::wrap(overlay::Element::new(Box::new(parent)), Arc::clone(&calls));
    let parent_node = overlay
        .as_overlay_mut()
        .layout(&renderer, Size::new(200.0, 120.0));
    let parent_layout = Layout::new(&parent_node);

    assert_eq!(parent_node.size(), Size::new(120.0, 80.0));
    assert_eq!(overlay.as_overlay().index(), 7.0);
    assert_eq!(
        overlay.as_overlay().mouse_interaction(
            parent_layout,
            mouse::Cursor::Available(Point::new(10.0, 10.0)),
            &renderer,
        ),
        mouse::Interaction::Pointer
    );
    assert_eq!(
        overlay.as_overlay().mouse_interaction(
            parent_layout,
            mouse::Cursor::Available(Point::new(150.0, 100.0)),
            &renderer,
        ),
        mouse::Interaction::None
    );

    overlay
        .as_overlay_mut()
        .operate(parent_layout, &renderer, &mut delegated);
    let mut messages = Vec::new();
    let mut shell = Shell::new(&mut messages);
    let mut clipboard = clipboard::Null;
    overlay.as_overlay_mut().update(
        &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        parent_layout,
        mouse::Cursor::Available(Point::new(10.0, 10.0)),
        &renderer,
        &mut clipboard,
        &mut shell,
    );
    drop(shell);
    overlay.as_overlay().draw(
        &mut crate::test_support::renderer(),
        &Theme::default(),
        &renderer::Style::default(),
        parent_layout,
        mouse::Cursor::Unavailable,
    );

    let mut nested = overlay
        .as_overlay_mut()
        .overlay(parent_layout, &renderer)
        .expect("nested overlay");
    let nested_node = nested
        .as_overlay_mut()
        .layout(&renderer, Size::new(200.0, 120.0));
    let nested_layout = Layout::new(&nested_node);

    assert_eq!(nested_node.size(), Size::new(60.0, 40.0));
    assert_eq!(nested.as_overlay().index(), 9.0);
    assert_eq!(
        nested.as_overlay().mouse_interaction(
            nested_layout,
            mouse::Cursor::Available(Point::new(10.0, 10.0)),
            &renderer,
        ),
        mouse::Interaction::Pointer
    );
    nested
        .as_overlay_mut()
        .operate(nested_layout, &renderer, &mut delegated);
    let mut nested_messages = Vec::new();
    let mut nested_shell = Shell::new(&mut nested_messages);
    nested.as_overlay_mut().update(
        &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        nested_layout,
        mouse::Cursor::Available(Point::new(10.0, 10.0)),
        &renderer,
        &mut clipboard,
        &mut nested_shell,
    );
    drop(nested_shell);
    nested.as_overlay().draw(
        &mut crate::test_support::renderer(),
        &Theme::default(),
        &renderer::Style::default(),
        nested_layout,
        mouse::Cursor::Unavailable,
    );

    assert_eq!(messages, vec![1]);
    assert_eq!(nested_messages, vec![2]);
    assert_eq!(delegated.containers, 3);
    assert_eq!(
        *calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        Calls {
            bound: vec![0, 1, 2],
            layouts: vec![1, 2],
            draws: vec![1, 2],
            updates: vec![1, 2],
        }
    );
}
