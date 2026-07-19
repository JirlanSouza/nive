use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, Operation as _, Tree},
        Clipboard, Layout, Shell,
    },
    widget::container,
    Event, Point, Size, Vector,
};

use super::initial_focus::{DialogInitialFocus, FocusFirstSafeTarget};
use crate::{
    focus::{contains_focus_target, FocusTarget, FocusTargetContext},
    focus_trap,
    theme::{surface as theme_surface, SurfaceRole},
    Element, Renderer, Theme,
};

/// Viewport safe margin applied on every edge, matching the anchored-overlay
/// contract Dialog reuses rather than inventing a second geometry model.
const SAFE_VIEWPORT_MARGIN: f32 = 16.0;
/// Maximum share of viewport height a Dialog frame may request.
const MAX_VIEWPORT_HEIGHT_SHARE: f32 = 0.80;

pub(super) struct DialogOverlay<'a, 'b, Message> {
    dialog: &'b mut Element<'a, Message>,
    state: &'b mut Tree,
    on_backdrop: Option<Message>,
    on_escape: Option<Message>,
    initial_focus: DialogInitialFocus,
    focus_context: &'b FocusTargetContext,
    expected_target: &'b mut Option<FocusTarget>,
    needs_initial_focus: &'b mut bool,
}

impl<'a, 'b, Message> DialogOverlay<'a, 'b, Message> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        dialog: &'b mut Element<'a, Message>,
        state: &'b mut Tree,
        on_backdrop: Option<Message>,
        on_escape: Option<Message>,
        initial_focus: DialogInitialFocus,
        focus_context: &'b FocusTargetContext,
        expected_target: &'b mut Option<FocusTarget>,
        needs_initial_focus: &'b mut bool,
    ) -> Self {
        Self {
            dialog,
            state,
            on_backdrop,
            on_escape,
            initial_focus,
            focus_context,
            expected_target,
            needs_initial_focus,
        }
    }
}

impl<'a, 'b, Message> overlay::Overlay<Message, Theme, Renderer> for DialogOverlay<'a, 'b, Message>
where
    Message: Clone + 'a,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, safe_max_size(bounds));
        let dialog_node = self
            .dialog
            .as_widget_mut()
            .layout(self.state, renderer, &limits);

        if *self.needs_initial_focus {
            // Best-effort early attempt: correct immediately when no shared
            // focus coordinator is bound yet (the common case for a
            // standalone `DialogHost` in tests, or before any `FocusRoot`
            // reaches this subtree). Deliberately does not clear the flag —
            // see `operate()`, which re-resolves and clears it after
            // binding is guaranteed to have happened.
            self.resolve_initial_focus(Layout::new(&dialog_node), renderer);
        }

        let position = centered_position(dialog_node.size(), bounds);

        layout::Node::with_children(bounds, vec![dialog_node.move_to(position)])
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let Some(dialog_layout) = layout.children().next() else {
            return;
        };

        self.dialog
            .as_widget_mut()
            .operate(self.state, dialog_layout, renderer, operation);
        self.remember_dialog_target(dialog_layout, renderer);

        if *self.needs_initial_focus {
            // A `FocusRoot` ancestor (if any) binds each descendant's
            // shared-coordinator target as a side effect of the *outer*
            // operate() call it wraps — a target's own `FocusState` only
            // gains `self.coordinator` on this same pass, not before. Re-run
            // resolution now that binding (if applicable) has happened, so
            // `state.focus()` correctly activates through the coordinator
            // instead of a local flag the next wrapped pass would discard.
            //
            // A Dialog-owned nested overlay (Select, Popover, Menu, ...) may
            // already be open the first time this ever runs (for example a
            // Popover opened in the same render as the Dialog itself). Such
            // an overlay manages its own initial-entry focus and may have
            // already claimed it via real interaction before this deferred
            // resolution got a chance to run; forcing the Dialog's own
            // fallback target here would silently steal focus back. Only
            // resolve when nothing else is already managing the nested
            // scope.
            let viewport = dialog_layout.bounds();
            let nested_overlay_open = self
                .dialog
                .as_widget_mut()
                .overlay(self.state, dialog_layout, renderer, &viewport, Vector::ZERO)
                .is_some();
            if !nested_overlay_open {
                self.resolve_initial_focus(dialog_layout, renderer);
            }
            *self.needs_initial_focus = false;
        }
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let Some(dialog_layout) = layout.children().next() else {
            shell.capture_event();
            return;
        };

        if let Some(direction) = focus_trap::direction_from_event(event) {
            direction.operate(|operation| {
                self.dialog
                    .as_widget_mut()
                    .operate(self.state, dialog_layout, renderer, operation);
            });
            self.remember_dialog_target(dialog_layout, renderer);
            shell.capture_event();
            shell.invalidate_layout();
            shell.request_redraw();
            return;
        }

        let viewport = layout.bounds();
        self.dialog.as_widget_mut().update(
            self.state,
            event,
            dialog_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &viewport,
        );
        self.remember_dialog_target(dialog_layout, renderer);

        if shell.is_event_captured() {
            return;
        }

        // A Dialog-owned nested overlay (Select, Popover, Menu, ...) has
        // already had first priority via Iced's own nested-overlay dispatch
        // order (its `overlay::Element` sits above this one in the stack).
        // Reaching here means no nested overlay and no Dialog descendant
        // consumed the event, so only outer backdrop/Escape dismissal
        // remains to evaluate.
        if is_primary_outside_press(event, cursor, dialog_layout.bounds()) {
            if let Some(message) = self.on_backdrop.clone() {
                shell.publish(message);
            }
            shell.capture_event();
            return;
        }

        if is_non_repeated_escape(event) {
            if let Some(message) = self.on_escape.clone() {
                shell.publish(message);
            }
            shell.capture_event();
            return;
        }

        shell.capture_event();
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let Some(dialog_layout) = layout.children().next() else {
            return mouse::Interaction::Idle;
        };

        if !cursor.is_over(dialog_layout.bounds()) {
            return mouse::Interaction::Idle;
        }

        let viewport = layout.bounds();
        self.dialog.as_widget().mouse_interaction(
            self.state,
            dialog_layout,
            cursor,
            &viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let style = theme_surface::style(SurfaceRole::Scrim)(theme);
        container::draw_background(renderer, &style, layout.bounds());

        let Some(dialog_layout) = layout.children().next() else {
            return;
        };

        let viewport = layout.bounds();
        self.dialog.as_widget().draw(
            self.state,
            renderer,
            theme,
            inherited_style,
            dialog_layout,
            cursor,
            &viewport,
        );
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'c>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
        let dialog_layout = layout.children().next()?;
        let viewport = layout.bounds();

        self.dialog.as_widget_mut().overlay(
            self.state,
            dialog_layout,
            renderer,
            &viewport,
            Vector::ZERO,
        )
    }
}

impl<Message> DialogOverlay<'_, '_, Message> {
    /// Resolves this Dialog's declared [`DialogInitialFocus`] policy exactly
    /// once per open session (gated by `needs_initial_focus`). `Target`
    /// falls back to `First` when the explicit target is missing, disabled,
    /// or otherwise not focusable. `First` never selects the action
    /// footer's terminal (Primary or Destructive) action.
    fn resolve_initial_focus(&mut self, layout: Layout<'_>, renderer: &Renderer) {
        if let DialogInitialFocus::Target(id) = &self.initial_focus {
            let mut focus_target = operation::focusable::focus::<()>(id.clone());
            self.dialog
                .as_widget_mut()
                .operate(self.state, layout, renderer, &mut focus_target);

            let mut count = operation::focusable::count();
            self.dialog.as_widget_mut().operate(
                self.state,
                layout,
                renderer,
                &mut operation::black_box(&mut count),
            );
            let explicit_target_focused =
                matches!(count.finish(), operation::Outcome::Some(c) if c.focused.is_some());

            if explicit_target_focused {
                return;
            }
        }

        let mut fallback = FocusFirstSafeTarget::new();
        self.dialog
            .as_widget_mut()
            .operate(self.state, layout, renderer, &mut fallback);
    }

    fn remember_dialog_target(&mut self, layout: Layout<'_>, renderer: &Renderer) {
        let Some(current) = self.focus_context.capture() else {
            return;
        };
        let mut contains = contains_focus_target(current.clone());
        self.dialog.as_widget_mut().operate(
            self.state,
            layout,
            renderer,
            &mut operation::black_box(&mut contains),
        );
        let mut found = matches!(contains.finish(), operation::Outcome::Some(true));

        // The current target may instead live inside a Dialog-owned nested
        // overlay (Select, Popover, Menu, ...), which is only reachable
        // through `overlay()`/`overlay::Nested`, not the base `operate()`
        // reach above. Without this, focus moving inside such a nested
        // overlay would leave `expected_target` stale, causing the eventual
        // outer-close anchor restoration to be refused as inconsistent.
        if !found {
            let viewport = layout.bounds();
            if let Some(overlay) = self.dialog.as_widget_mut().overlay(
                self.state,
                layout,
                renderer,
                &viewport,
                Vector::ZERO,
            ) {
                let mut nested = overlay::Nested::new(overlay);
                let node = nested.layout(renderer, viewport.size());
                let mut nested_contains = contains_focus_target(current.clone());
                nested.operate(
                    Layout::new(&node),
                    renderer,
                    &mut operation::black_box(&mut nested_contains),
                );
                found = matches!(nested_contains.finish(), operation::Outcome::Some(true));
            }
        }

        if found {
            *self.expected_target = Some(current);
        }
    }
}

/// Clamps the requested viewport bounds to the safe-margin/`80vh` cap Dialog
/// resolves its semantic size against. `Dialog::layout()` receives these as
/// its `Limits.max` and clamps its own requested `DialogSize` width to fit.
fn safe_max_size(viewport: Size) -> Size {
    let width = (viewport.width - 2.0 * SAFE_VIEWPORT_MARGIN).max(0.0);
    let capped_height = (viewport.height - 2.0 * SAFE_VIEWPORT_MARGIN).max(0.0);
    let height = (MAX_VIEWPORT_HEIGHT_SHARE * viewport.height).min(capped_height);

    Size::new(width, height)
}

fn centered_position(content: Size, bounds: Size) -> Point {
    Point::new(
        center_axis(content.width, bounds.width),
        center_axis(content.height, bounds.height),
    )
}

fn center_axis(length: f32, limit: f32) -> f32 {
    let max = (limit - length).max(0.0);
    ((limit - length) / 2.0).clamp(0.0, max)
}

/// Backdrop activation is only a primary (left) mouse press or a touch
/// press whose resolved cursor position is concrete and outside the Dialog
/// frame. A `Cursor::Unavailable`/`Levitating` position — including one
/// masked by a Dialog-owned nested overlay — is never treated as outside.
fn is_primary_outside_press(
    event: &Event,
    cursor: mouse::Cursor,
    dialog_bounds: iced::Rectangle,
) -> bool {
    let Some(position) = cursor.position() else {
        return false;
    };

    if dialog_bounds.contains(position) {
        return false;
    }

    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(iced::touch::Event::FingerPressed { .. })
    )
}

fn is_non_repeated_escape(event: &Event) -> bool {
    matches!(
        event,
        Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            repeat: false,
            ..
        })
    )
}

#[cfg(test)]
mod dialog_host_tests {
    use super::*;

    #[test]
    fn centers_dialog_inside_available_bounds() {
        let position = centered_position(Size::new(40.0, 20.0), Size::new(100.0, 80.0));

        assert_eq!(position, Point::new(30.0, 30.0));
    }

    #[test]
    fn clamps_oversized_dialog_to_origin() {
        let position = centered_position(Size::new(120.0, 90.0), Size::new(100.0, 80.0));

        assert_eq!(position, Point::ORIGIN);
    }

    #[test]
    fn safe_max_size_subtracts_the_margin_from_width() {
        let max = safe_max_size(Size::new(500.0, 1000.0));

        assert_eq!(max.width, 500.0 - 2.0 * SAFE_VIEWPORT_MARGIN);
    }

    #[test]
    fn safe_max_size_caps_height_at_eighty_percent_of_a_tall_viewport() {
        let max = safe_max_size(Size::new(500.0, 1000.0));

        assert_eq!(max.height, 800.0);
    }

    #[test]
    fn safe_max_size_falls_back_to_the_margin_cap_on_a_short_viewport() {
        let max = safe_max_size(Size::new(500.0, 100.0));

        assert_eq!(max.height, 100.0 - 2.0 * SAFE_VIEWPORT_MARGIN);
    }

    #[test]
    fn safe_max_size_never_goes_negative_below_the_margin() {
        let max = safe_max_size(Size::new(20.0, 20.0));

        assert_eq!(max.width, 0.0);
        assert_eq!(max.height, 0.0);
    }

    #[test]
    fn left_press_outside_bounds_is_backdrop() {
        let bounds = iced::Rectangle::new(Point::new(100.0, 100.0), Size::new(50.0, 50.0));
        let cursor = mouse::Cursor::Available(Point::new(10.0, 10.0));
        let event = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));

        assert!(is_primary_outside_press(&event, cursor, bounds));
    }

    #[test]
    fn right_press_outside_bounds_is_not_backdrop() {
        let bounds = iced::Rectangle::new(Point::new(100.0, 100.0), Size::new(50.0, 50.0));
        let cursor = mouse::Cursor::Available(Point::new(10.0, 10.0));
        let event = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right));

        assert!(!is_primary_outside_press(&event, cursor, bounds));
    }

    #[test]
    fn left_press_inside_bounds_is_not_backdrop() {
        let bounds = iced::Rectangle::new(Point::new(100.0, 100.0), Size::new(50.0, 50.0));
        let cursor = mouse::Cursor::Available(Point::new(110.0, 110.0));
        let event = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));

        assert!(!is_primary_outside_press(&event, cursor, bounds));
    }

    #[test]
    fn unavailable_cursor_is_never_backdrop_even_outside_bounds() {
        let bounds = iced::Rectangle::new(Point::new(100.0, 100.0), Size::new(50.0, 50.0));
        let event = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));

        assert!(!is_primary_outside_press(
            &event,
            mouse::Cursor::Unavailable,
            bounds
        ));
    }

    #[test]
    fn levitating_cursor_is_never_backdrop() {
        let bounds = iced::Rectangle::new(Point::new(100.0, 100.0), Size::new(50.0, 50.0));
        let event = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));

        assert!(!is_primary_outside_press(
            &event,
            mouse::Cursor::Levitating(Point::new(10.0, 10.0)),
            bounds
        ));
    }

    #[test]
    fn touch_press_outside_bounds_is_backdrop() {
        let bounds = iced::Rectangle::new(Point::new(100.0, 100.0), Size::new(50.0, 50.0));
        let cursor = mouse::Cursor::Available(Point::new(10.0, 10.0));
        let event = Event::Touch(iced::touch::Event::FingerPressed {
            id: iced::touch::Finger(0),
            position: Point::new(10.0, 10.0),
        });

        assert!(is_primary_outside_press(&event, cursor, bounds));
    }

    #[test]
    fn non_repeated_escape_is_recognized() {
        let key = iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape);
        let event = Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: iced::keyboard::key::Physical::Code(iced::keyboard::key::Code::Escape),
            location: iced::keyboard::Location::Standard,
            modifiers: iced::keyboard::Modifiers::NONE,
            text: None,
            repeat: false,
        });

        assert!(is_non_repeated_escape(&event));
    }

    #[test]
    fn repeated_escape_is_not_recognized() {
        let key = iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape);
        let event = Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: iced::keyboard::key::Physical::Code(iced::keyboard::key::Code::Escape),
            location: iced::keyboard::Location::Standard,
            modifiers: iced::keyboard::Modifiers::NONE,
            text: None,
            repeat: true,
        });

        assert!(!is_non_repeated_escape(&event));
    }

    #[test]
    fn backdrop_style_resolves_from_the_semantic_scrim_role_in_every_built_in_mode() {
        for theme in [crate::theme::Theme::Light, crate::theme::Theme::Dark] {
            let style = theme_surface::style(SurfaceRole::Scrim)(&theme);
            let expected = theme.surface(SurfaceRole::Scrim);

            assert_eq!(
                style.background,
                Some(iced::Background::Color(expected.background))
            );
        }
    }
}
