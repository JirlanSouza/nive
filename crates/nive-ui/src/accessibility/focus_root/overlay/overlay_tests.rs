use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, Id},
        Layout,
    },
    Rectangle, Size,
};

use super::FocusOverlay;
use crate::{
    advanced::focus::FocusState,
    focus::{lock_coordinator, FocusCoordinator},
    Renderer, Theme,
};

#[derive(Debug, Default)]
struct UnknownState;

struct FocusAndUnknown {
    unknown_visits: usize,
}

impl operation::Operation for FocusAndUnknown {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
        operate(self);
    }

    fn focusable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        state: &mut dyn operation::Focusable,
    ) {
        state.focus();
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn std::any::Any) {
        if state.downcast_ref::<UnknownState>().is_some() {
            self.unknown_visits += 1;
        }
    }
}

struct ProbeOverlay {
    state: FocusState,
    unknown: UnknownState,
    nested: ProbeNested,
}

impl overlay::Overlay<(), Theme, Renderer> for ProbeOverlay {
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        layout::Node::new(Size::new(80.0, 40.0))
    }

    fn draw(
        &self,
        _renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        self.state.register(operation, None, layout.bounds());
        operation.custom(None, layout.bounds(), &mut self.unknown);
    }

    fn overlay<'a>(
        &'a mut self,
        _layout: Layout<'a>,
        _renderer: &Renderer,
    ) -> Option<overlay::Element<'a, (), Theme, Renderer>> {
        Some(overlay::Element::new(Box::new(ProbeNestedRef {
            inner: &mut self.nested,
        })))
    }
}

struct ProbeNested {
    state: FocusState,
    unknown: UnknownState,
}

struct ProbeNestedRef<'a> {
    inner: &'a mut ProbeNested,
}

impl overlay::Overlay<(), Theme, Renderer> for ProbeNestedRef<'_> {
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        layout::Node::new(Size::new(40.0, 20.0))
    }

    fn draw(
        &self,
        _renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        self.inner.state.register(operation, None, layout.bounds());
        operation.custom(None, layout.bounds(), &mut self.inner.unknown);
    }
}

#[test]
fn production_adapter_shares_one_generation_and_delegates_unknown_custom_state() {
    let coordinator = FocusCoordinator::shared();
    let mut base = FocusState::default();
    let parent = ProbeOverlay {
        state: FocusState::default(),
        unknown: UnknownState,
        nested: ProbeNested {
            state: FocusState::default(),
            unknown: UnknownState,
        },
    };
    let base_token = base.token();
    let parent_token = parent.state.token();
    let nested_token = parent.nested.state.token();
    let generation = lock_coordinator(&coordinator).begin_liveness();
    base.bind(&coordinator, generation);
    operation::Focusable::focus(&mut base);

    let renderer = crate::test_support::renderer();
    let mut adapter = FocusOverlay::wrap(
        overlay::Element::new(Box::new(parent)),
        coordinator.clone(),
        0,
    );
    let parent_node = adapter
        .as_overlay_mut()
        .layout(&renderer, Size::new(200.0, 120.0));
    let parent_layout = Layout::new(&parent_node);
    let mut operation = FocusAndUnknown { unknown_visits: 0 };
    adapter
        .as_overlay_mut()
        .operate(parent_layout, &renderer, &mut operation);

    assert!(lock_coordinator(&coordinator).is_live(base_token));
    assert!(lock_coordinator(&coordinator).is_live(parent_token));
    assert!(!lock_coordinator(&coordinator).is_current(base_token));
    assert!(lock_coordinator(&coordinator).is_current(parent_token));

    let mut nested = adapter
        .as_overlay_mut()
        .overlay(parent_layout, &renderer)
        .expect("nested adapter");
    let nested_node = nested
        .as_overlay_mut()
        .layout(&renderer, Size::new(200.0, 120.0));
    nested
        .as_overlay_mut()
        .operate(Layout::new(&nested_node), &renderer, &mut operation);

    assert_eq!(operation.unknown_visits, 2);
    let coordinator = lock_coordinator(&coordinator);
    assert!(coordinator.is_live(base_token));
    assert!(coordinator.is_live(parent_token));
    assert!(coordinator.is_live(nested_token));
    assert!(!coordinator.is_current(parent_token));
    assert!(coordinator.is_current(nested_token));
}
