use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Renderer as _, Shell, Widget,
    },
    border::Radius,
    Background, Border, Color, Event, Length, Rectangle, Shadow, Size, Vector,
};

use crate::theme::{
    BorderRole, ControlRole, ControlState, FieldValidation, FormControlMetrics, Theme,
};
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FormFrameAppearance {
    Default,
    Ghost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct FormFrameState {
    pub(super) hovered: bool,
    pub(super) focused: bool,
    pub(super) disabled: bool,
}

impl FormFrameState {
    #[cfg(test)]
    const fn enabled() -> Self {
        Self {
            hovered: false,
            focused: false,
            disabled: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct FormFrameStyle {
    pub(super) background: Color,
    pub(super) foreground: Color,
    pub(super) perimeter: Border,
    pub(super) focus: Option<Border>,
}

pub(super) fn resolve(
    theme: Theme,
    appearance: FormFrameAppearance,
    validation: FieldValidation,
    metrics: FormControlMetrics,
    state: FormFrameState,
) -> FormFrameStyle {
    let control_state = if state.disabled {
        ControlState::DISABLED
    } else {
        ControlState::ENABLED
    };
    let control = theme.control(ControlRole::Standard, control_state);
    let neutral_border = if state.hovered && !state.disabled {
        theme.border(BorderRole::Strong)
    } else {
        theme.border(BorderRole::Default)
    };
    let perimeter = match validation {
        FieldValidation::Invalid => theme.border(BorderRole::Danger),
        FieldValidation::Valid if appearance == FormFrameAppearance::Ghost => {
            crate::theme::BorderSpec::none()
        }
        FieldValidation::Valid => neutral_border,
    };
    let background = match appearance {
        FormFrameAppearance::Default => control.background,
        FormFrameAppearance::Ghost => Color::TRANSPARENT,
    };
    let focus = (!state.disabled && state.focused).then(|| Border {
        color: theme.border(BorderRole::Focus).color,
        width: metrics.focus_stroke_width,
        radius: Radius::new(metrics.focus_radius),
    });

    FormFrameStyle {
        background,
        foreground: control.foreground,
        perimeter: Border {
            color: perimeter.color,
            width: metrics.field_border_width,
            radius: Radius::new(metrics.radius),
        },
        focus,
    }
}

pub(super) struct FormControlFrame<'a, Message> {
    pub(super) content: Element<'a, Message>,
    pub(super) appearance: FormFrameAppearance,
    pub(super) validation: FieldValidation,
    pub(super) metrics: FormControlMetrics,
    pub(super) disabled: bool,
    pub(super) interactive: bool,
}

#[derive(Debug, Default)]
struct FrameWidgetState {
    focused: bool,
    hovered: bool,
}

impl<'a, Message> Widget<Message, Theme, iced::Renderer> for FormControlFrame<'a, Message>
where
    Message: 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<FrameWidgetState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(FrameWidgetState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let hovered = self.interactive && !self.disabled && cursor.is_over(layout.bounds());
        let state = tree.state.downcast_mut::<FrameWidgetState>();

        if state.hovered != hovered {
            state.hovered = hovered;
            shell.request_redraw();
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let focused = super::input::adapter::content_has_visual_focus(
            &mut self.content,
            &mut tree.children[0],
            layout,
            renderer,
        );
        let state = tree.state.downcast_mut::<FrameWidgetState>();

        if state.focused != focused {
            state.focused = focused;
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        _inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<FrameWidgetState>();
        let style = resolve(
            *theme,
            self.appearance,
            self.validation,
            self.metrics,
            FormFrameState {
                hovered: state.hovered,
                focused: state.focused,
                disabled: self.disabled,
            },
        );
        let bounds = layout.bounds();

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.perimeter,
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(style.background),
        );

        let inherited_style = renderer::Style {
            text_color: style.foreground,
        };
        renderer.with_layer(bounds, |renderer| {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                &inherited_style,
                layout,
                cursor,
                viewport,
            );
        });

        if let Some(focus) = style.focus {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: self.metrics.focus_bounds(bounds),
                    border: focus,
                    shadow: Shadow::default(),
                    snap: true,
                },
                Background::Color(Color::TRANSPARENT),
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

#[cfg(test)]
mod form_frame_tests {
    use super::*;
    use crate::test_support::WidgetHarness;
    use crate::theme::{ControlSize, Theme, ThemeBuilder, ThemeDensity, ThemeMode};
    use crate::widgets::controls::Input;

    #[test]
    fn pointer_blur_hides_the_frame_focus_while_retaining_navigation_anchor() {
        let input: Element<'_, String> = Input::new("Name", "Ada").on_change(|value| value).into();
        let mut harness = WidgetHarness::new(input, Size::new(240.0, 80.0));

        harness.set_cursor(iced::Point::new(20.0, 14.0));
        harness.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));
        assert!(harness.state::<FrameWidgetState>().focused);

        harness.set_cursor(iced::Point::new(20.0, 60.0));
        harness.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));
        assert!(!harness.state::<FrameWidgetState>().focused);
    }

    #[test]
    fn every_built_in_density_and_size_uses_exact_frame_geometry() {
        let sizes = [
            ControlSize::Xs,
            ControlSize::Sm,
            ControlSize::Md,
            ControlSize::Lg,
        ];

        for density in ThemeDensity::ALL {
            let theme = ThemeBuilder::new("Form frame matrix", ThemeMode::Light)
                .density(density)
                .build();

            for size in sizes {
                let metrics = theme.form_control_metrics(size);
                let style = resolve(
                    theme,
                    FormFrameAppearance::Default,
                    FieldValidation::Valid,
                    metrics,
                    FormFrameState {
                        focused: true,
                        ..FormFrameState::enabled()
                    },
                );

                assert_eq!(style.perimeter.width, 1.0);
                assert_eq!(style.perimeter.radius, Radius::new(metrics.radius));
                assert_eq!(style.focus.expect("focus").width, 2.0);
                assert_eq!(
                    style.focus.expect("focus").radius,
                    Radius::new((metrics.radius - 1.0).max(0.0))
                );
                assert_eq!(
                    metrics.focus_bounds(Rectangle::new(
                        iced::Point::ORIGIN,
                        iced::Size::new(120.0, metrics.height),
                    )),
                    Rectangle::new(
                        iced::Point::new(1.0, 1.0),
                        iced::Size::new(118.0, metrics.height - 2.0),
                    )
                );
            }
        }
    }

    #[test]
    fn invalid_focus_keeps_danger_perimeter_and_independent_focus() {
        let theme = Theme::Dark;
        let metrics = theme.form_control_metrics(ControlSize::Sm);
        let style = resolve(
            theme,
            FormFrameAppearance::Default,
            FieldValidation::Invalid,
            metrics,
            FormFrameState {
                focused: true,
                ..FormFrameState::enabled()
            },
        );

        assert_eq!(
            style.perimeter.color,
            theme.border(BorderRole::Danger).color
        );
        assert_eq!(style.perimeter.width, 1.0);
        assert_eq!(style.focus.expect("focus stroke").width, 2.0);
    }

    #[test]
    fn ghost_omits_only_idle_chrome() {
        let theme = Theme::Dark;
        let metrics = theme.form_control_metrics(ControlSize::Sm);
        let idle = resolve(
            theme,
            FormFrameAppearance::Ghost,
            FieldValidation::Valid,
            metrics,
            FormFrameState::enabled(),
        );
        let invalid = resolve(
            theme,
            FormFrameAppearance::Ghost,
            FieldValidation::Invalid,
            metrics,
            FormFrameState::enabled(),
        );

        assert_eq!(idle.background, Color::TRANSPARENT);
        assert_eq!(idle.perimeter.color, Color::TRANSPARENT);
        assert_eq!(idle.perimeter.width, 1.0);
        assert_eq!(
            invalid.perimeter.color,
            theme.border(BorderRole::Danger).color
        );
    }

    #[test]
    fn disabled_suppresses_hover_and_focus_without_multiplying_alpha() {
        let theme = Theme::Dark;
        let metrics = theme.form_control_metrics(ControlSize::Sm);
        let style = resolve(
            theme,
            FormFrameAppearance::Default,
            FieldValidation::Valid,
            metrics,
            FormFrameState {
                hovered: true,
                focused: true,
                disabled: true,
            },
        );
        let disabled = theme.control(ControlRole::Standard, ControlState::DISABLED);

        assert_eq!(style.background, disabled.background);
        assert_eq!(style.foreground, disabled.foreground);
        assert_eq!(
            style.perimeter.color,
            theme.border(BorderRole::Default).color
        );
        assert_eq!(style.focus, None);
    }

    #[test]
    fn state_changes_do_not_change_outer_geometry() {
        let theme = Theme::Dark;
        let metrics = theme.form_control_metrics(ControlSize::Sm);
        let states = [
            FormFrameState::enabled(),
            FormFrameState {
                hovered: true,
                ..FormFrameState::enabled()
            },
            FormFrameState {
                focused: true,
                ..FormFrameState::enabled()
            },
            FormFrameState {
                disabled: true,
                ..FormFrameState::enabled()
            },
        ];

        for state in states {
            for validation in [FieldValidation::Valid, FieldValidation::Invalid] {
                let style = resolve(
                    theme,
                    FormFrameAppearance::Default,
                    validation,
                    metrics,
                    state,
                );

                assert_eq!(style.perimeter.width, metrics.field_border_width);
                assert_eq!(style.perimeter.radius, Radius::new(metrics.radius));
            }
        }
    }

    #[test]
    fn disabled_invalid_uses_semantic_danger_without_local_alpha() {
        let theme = Theme::Dark;
        let metrics = theme.form_control_metrics(ControlSize::Sm);
        let style = resolve(
            theme,
            FormFrameAppearance::Default,
            FieldValidation::Invalid,
            metrics,
            FormFrameState {
                disabled: true,
                ..FormFrameState::enabled()
            },
        );

        assert_eq!(
            style.perimeter.color,
            theme.border(BorderRole::Danger).color
        );
        assert_eq!(style.perimeter.width, metrics.field_border_width);
    }
}
