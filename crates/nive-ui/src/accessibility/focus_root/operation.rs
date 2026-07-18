use std::any::Any;

use iced::{
    advanced::widget::{operation, Id},
    Rectangle, Vector,
};

use super::FocusRootState;
use crate::{
    advanced::focus::FocusState,
    focus::{FocusGeneration, FocusTargetContext, SharedFocusCoordinator},
};

#[derive(Debug, Default)]
pub(super) struct BindingPass;

impl operation::Operation for BindingPass {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
        operate(self);
    }
}

pub(super) struct FocusOperation<'a> {
    inner: &'a mut dyn operation::Operation,
    coordinator: &'a SharedFocusCoordinator,
    generation: FocusGeneration,
}

impl<'a> FocusOperation<'a> {
    pub(super) fn new(
        inner: &'a mut dyn operation::Operation,
        coordinator: &'a SharedFocusCoordinator,
        generation: FocusGeneration,
    ) -> Self {
        Self {
            inner,
            coordinator,
            generation,
        }
    }
}

impl operation::Operation for FocusOperation<'_> {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
        self.inner.traverse(&mut |inner| {
            operate(&mut FocusOperation::new(
                inner,
                self.coordinator,
                self.generation,
            ));
        });
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        self.inner.container(id, bounds);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        state: &mut dyn operation::Scrollable,
    ) {
        self.inner
            .scrollable(id, bounds, content_bounds, translation, state);
    }

    fn focusable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        state: &mut dyn operation::Focusable,
    ) {
        self.inner.focusable(id, bounds, state);
    }

    fn text_input(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        state: &mut dyn operation::TextInput,
    ) {
        self.inner.text_input(id, bounds, state);
    }

    fn text(&mut self, id: Option<&Id>, bounds: Rectangle, text: &str) {
        self.inner.text(id, bounds, text);
    }

    fn custom(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn Any) {
        if let Some(focus) = state.downcast_mut::<FocusState>() {
            focus.bind(self.coordinator, self.generation);
        } else if let Some(context) = state.downcast_mut::<FocusTargetContext>() {
            context.bind(self.coordinator);
        } else if let Some(root) = state.downcast_mut::<FocusRootState>() {
            if !std::sync::Arc::ptr_eq(&root.coordinator, self.coordinator) {
                root.nested_root_detected = true;
            }
        }
        self.inner.custom(id, bounds, state);
    }

    fn finish(&self) -> operation::Outcome<()> {
        match self.inner.finish() {
            operation::Outcome::None => operation::Outcome::None,
            operation::Outcome::Some(output) => operation::Outcome::Some(output),
            operation::Outcome::Chain(next) => {
                operation::Outcome::Chain(Box::new(OwnedFocusOperation {
                    inner: next,
                    coordinator: self.coordinator.clone(),
                    generation: self.generation,
                }))
            }
        }
    }
}

struct OwnedFocusOperation {
    inner: Box<dyn operation::Operation>,
    coordinator: SharedFocusCoordinator,
    generation: FocusGeneration,
}

impl operation::Operation for OwnedFocusOperation {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
        self.inner.traverse(&mut |inner| {
            operate(&mut FocusOperation::new(
                inner,
                &self.coordinator,
                self.generation,
            ));
        });
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        self.inner.container(id, bounds);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        state: &mut dyn operation::Scrollable,
    ) {
        self.inner
            .scrollable(id, bounds, content_bounds, translation, state);
    }

    fn focusable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        state: &mut dyn operation::Focusable,
    ) {
        self.inner.focusable(id, bounds, state);
    }

    fn text_input(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        state: &mut dyn operation::TextInput,
    ) {
        self.inner.text_input(id, bounds, state);
    }

    fn text(&mut self, id: Option<&Id>, bounds: Rectangle, text: &str) {
        self.inner.text(id, bounds, text);
    }

    fn custom(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn Any) {
        let mut wrapper =
            FocusOperation::new(self.inner.as_mut(), &self.coordinator, self.generation);
        wrapper.custom(id, bounds, state);
    }

    fn finish(&self) -> operation::Outcome<()> {
        match self.inner.finish() {
            operation::Outcome::None => operation::Outcome::None,
            operation::Outcome::Some(output) => operation::Outcome::Some(output),
            operation::Outcome::Chain(next) => operation::Outcome::Chain(Box::new(Self {
                inner: next,
                coordinator: self.coordinator.clone(),
                generation: self.generation,
            })),
        }
    }
}
