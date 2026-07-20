use iced::{
    advanced::{
        layout::Layout,
        widget::{
            operation::{self, Focusable, Outcome},
            tree::Tree,
            Id, Operation,
        },
    },
    Rectangle,
};
use std::any::Any;

use super::super::dialog::TerminalActionTag;
use super::super::modal_host::InitialFocusFn;
use crate::{Element, Renderer};

/// Where a Dialog places focus the first time it opens. `First` resolves,
/// in order, the first enabled body focusable, then the first enabled
/// Cancel/Secondary footer action, then the safe header close affordance —
/// never the footer's terminal (Primary or Destructive) action. `Target`
/// falls back to `First` when the named target is missing, disabled, or
/// outside the Dialog. Non-exhaustive so a future policy can be added
/// without breaking downstream exhaustive matches:
///
/// ```compile_fail
/// use nive_ui::widgets::overlays::DialogInitialFocus;
///
/// fn describe(policy: DialogInitialFocus) -> &'static str {
///     match policy {
///         DialogInitialFocus::First => "first",
///         DialogInitialFocus::Target(_) => "target",
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum DialogInitialFocus {
    #[default]
    First,
    Target(Id),
}

/// Builds the shared modal kernel's [`InitialFocusFn`] hook for a Dialog's
/// declared [`DialogInitialFocus`] policy. `Target` falls back to `First`
/// when the explicit target is missing, disabled, or otherwise not
/// focusable. `First` never selects the action footer's terminal (Primary or
/// Destructive) action. Keeping this resolution in the Dialog layer (rather
/// than the kernel) is what lets the kernel stay ignorant of the
/// Dialog-specific `TerminalActionTag` concept.
pub(super) fn dialog_initial_focus_fn<'a, Message>(
    policy: DialogInitialFocus,
) -> InitialFocusFn<'a, Message>
where
    Message: 'a,
{
    Box::new(move |content, tree, layout, renderer| {
        resolve_initial_focus(&policy, content, tree, layout, renderer);
    })
}

fn resolve_initial_focus<Message>(
    policy: &DialogInitialFocus,
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
) {
    if let DialogInitialFocus::Target(id) = policy {
        let mut focus_target = operation::focusable::focus::<()>(id.clone());
        content
            .as_widget_mut()
            .operate(tree, layout, renderer, &mut focus_target);

        let mut count = operation::focusable::count();
        content.as_widget_mut().operate(
            tree,
            layout,
            renderer,
            &mut operation::black_box(&mut count),
        );
        let explicit_target_focused =
            matches!(count.finish(), Outcome::Some(c) if c.focused.is_some());

        if explicit_target_focused {
            return;
        }
    }

    let mut fallback = FocusFirstSafeTarget::new();
    content
        .as_widget_mut()
        .operate(tree, layout, renderer, &mut fallback);
}

/// Focuses the first enabled, non-terminal focusable target reached while
/// operating a Dialog's content, skipping the single focusable widget
/// immediately following a [`TerminalActionTag`] announcement (the Dialog
/// action footer's terminal action, wrapped by
/// `dialog::footer::TerminalActionMarker`). Body content is visited before
/// footer/header content because `Dialog::operate()` orders itself that way
/// for focus-related operations.
struct FocusFirstSafeTarget {
    skip_next_focusable: bool,
    resolved: bool,
}

impl FocusFirstSafeTarget {
    fn new() -> Self {
        Self {
            skip_next_focusable: false,
            resolved: false,
        }
    }
}

impl Operation<()> for FocusFirstSafeTarget {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
        operate(self);
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Any) {
        if state.downcast_mut::<TerminalActionTag>().is_some() {
            self.skip_next_focusable = true;
        }
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        if self.resolved {
            state.unfocus();
            return;
        }

        if self.skip_next_focusable {
            self.skip_next_focusable = false;
            state.unfocus();
            return;
        }

        state.focus();
        self.resolved = true;
    }

    fn finish(&self) -> Outcome<()> {
        if self.resolved {
            Outcome::Some(())
        } else {
            Outcome::None
        }
    }
}

#[cfg(test)]
mod initial_focus_tests {
    use super::*;

    #[test]
    fn first_is_the_default() {
        assert_eq!(DialogInitialFocus::default(), DialogInitialFocus::First);
    }
}
