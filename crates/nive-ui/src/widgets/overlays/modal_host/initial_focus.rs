use iced::advanced::{layout::Layout, widget::Tree};

use crate::{Element, Renderer};

/// Resolves a modal's initial focus exactly once per open session. The
/// kernel invokes this after best-effort layout and again after `operate()`
/// has guaranteed any shared focus coordinator is bound (see
/// `ModalOverlay::operate`), so the closure must be idempotent to call and
/// safe to call twice with the same layout.
///
/// Dialog supplies a closure that resolves its `DialogInitialFocus` policy
/// (an explicit `Target` falling back to the first safe non-terminal
/// focusable). `CommandPalette` supplies a closure that focuses its search
/// `Input` unconditionally. Keeping this a plain function value (rather than
/// a shared `DialogInitialFocus`-shaped enum) is what lets the kernel stay
/// ignorant of Dialog's terminal-action concept.
pub(crate) type InitialFocusFn<'a, Message> =
    Box<dyn Fn(&mut Element<'a, Message>, &mut Tree, Layout<'_>, &Renderer) + 'a>;
