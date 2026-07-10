use iced::{
    border::Radius,
    widget::checkbox::Status,
    widget::{checkbox as iced_checkbox, text, Checkbox as IcedCheckbox},
    Background, Border, Color, Length,
};

use crate::theme::{
    self, control_metrics, ControlRole, ControlSize, ControlState, SpaceStep, TextRole, ToneRole,
};
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq)]
struct CheckboxMetrics {
    size: f32,
    radius: f32,
    spacing: f32,
    font_size: f32,
}

pub struct Checkbox<'a, Message> {
    label: &'a str,
    checked: bool,
    size: ControlSize,
    width: Option<Length>,
    disabled: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
}

impl<'a, Message> Checkbox<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(label: &'a str, checked: bool) -> Self {
        Self {
            label,
            checked,
            size: ControlSize::Sm,
            width: None,
            disabled: false,
            on_toggle: None,
        }
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn xs(self) -> Self {
        self.size(ControlSize::Xs)
    }

    pub fn sm(self) -> Self {
        self.size(ControlSize::Sm)
    }

    pub fn md(self) -> Self {
        self.size(ControlSize::Md)
    }

    pub fn lg(self) -> Self {
        self.size(ControlSize::Lg)
    }

    crate::impl_layout_builders!(width_opt, fill_width_opt, shrink_width_opt);

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_toggle(mut self, on_toggle: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(on_toggle));
        self
    }

    pub fn on_toggle_maybe(mut self, on_toggle: Option<impl Fn(bool) -> Message + 'a>) -> Self {
        self.on_toggle = on_toggle.map(|on_toggle| Box::new(on_toggle) as _);
        self
    }

    fn into_checkbox(self) -> IcedCheckbox<'a, Message, crate::theme::Theme> {
        let metrics = metrics(self.size);
        let mut checkbox = iced_checkbox(self.checked)
            .label(self.label)
            .size(metrics.size)
            .spacing(metrics.spacing)
            .text_size(metrics.font_size)
            .text_shaping(text::Shaping::Auto)
            .style(style(metrics.radius.into()));

        if let Some(width) = self.width {
            checkbox = checkbox.width(width);
        }

        checkbox.on_toggle_maybe(if self.disabled { None } else { self.on_toggle })
    }
}

impl<'a, Message> From<Checkbox<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(checkbox: Checkbox<'a, Message>) -> Self {
        checkbox.into_checkbox().into()
    }
}

fn metrics(size: ControlSize) -> CheckboxMetrics {
    let control = control_metrics(size);

    CheckboxMetrics {
        size: match size {
            ControlSize::Xs => 14.0,
            ControlSize::Sm => 16.0,
            ControlSize::Md => 18.0,
            ControlSize::Lg => 20.0,
        },
        radius: control.radius.min(5.0),
        spacing: theme::space(SpaceStep::Md).max(control.gap),
        font_size: control.font_size,
    }
}

fn style(radius: Radius) -> impl Fn(&crate::theme::Theme, Status) -> iced_checkbox::Style {
    move |theme: &crate::theme::Theme, status: Status| {
        let theme = *theme;
        let is_checked = match status {
            Status::Active { is_checked }
            | Status::Hovered { is_checked }
            | Status::Disabled { is_checked } => is_checked,
        };
        let state = match status {
            Status::Active { .. } => ControlState::ENABLED,
            Status::Hovered { .. } => ControlState::HOVERED,
            Status::Disabled { .. } => ControlState::DISABLED,
        };
        let control = theme.control(ControlRole::Standard, state);
        let primary = theme.tone(ToneRole::Accent);
        let disabled = matches!(status, Status::Disabled { .. });
        let alpha = if disabled { 0.55 } else { 1.0 };

        iced_checkbox::Style {
            background: Background::Color(if is_checked {
                primary.color.scale_alpha(alpha)
            } else {
                control.background
            }),
            icon_color: if is_checked {
                theme.tone(ToneRole::Accent).on_color.scale_alpha(alpha)
            } else {
                Color::TRANSPARENT
            },
            border: Border {
                color: if is_checked {
                    primary.border.color.scale_alpha(alpha)
                } else {
                    control.border.color
                },
                width: if is_checked {
                    primary.border.width
                } else {
                    control.border.width
                },
                radius,
            },
            text_color: Some(if disabled {
                theme.text(TextRole::Muted).color.scale_alpha(0.65)
            } else {
                theme.text(TextRole::Secondary).color
            }),
        }
    }
}

#[cfg(test)]
mod checkbox_tests {
    use super::*;
    use crate::theme::Theme;
    use iced::{
        advanced::{
            layout::{Layout, Limits, Node},
            mouse,
            widget::Tree,
            Shell,
        },
        Event, Font, Pixels, Point, Rectangle, Size, Vector,
    };

    const ORIGIN: Vector = Vector::new(16.0, 16.0);

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Message {
        Toggled(bool),
    }

    struct Harness<'a> {
        element: Element<'a, Message>,
        tree: Tree,
        node: Node,
        renderer: iced::Renderer,
    }

    impl<'a> Harness<'a> {
        fn new(element: Element<'a, Message>) -> Self {
            let tree = Tree::new(&element);
            let mut harness = Self {
                element,
                tree,
                node: Node::new(Size::ZERO),
                renderer: test_renderer(),
            };
            harness.layout();
            harness
        }

        fn layout(&mut self) {
            self.element.as_widget_mut().diff(&mut self.tree);
            self.node = self.element.as_widget_mut().layout(
                &mut self.tree,
                &self.renderer,
                &Limits::new(Size::ZERO, Size::new(240.0, 80.0)),
            );
        }

        fn center(&self) -> Point {
            let bounds = Layout::with_offset(ORIGIN, &self.node).bounds();

            Point::new(
                bounds.x + bounds.width / 2.0,
                bounds.y + bounds.height / 2.0,
            )
        }

        fn click(&mut self, position: Point) -> Vec<Message> {
            let mut messages = Vec::new();
            let mut clipboard = iced::advanced::clipboard::Null;
            let viewport = Rectangle::new(Point::ORIGIN, Size::new(4096.0, 4096.0));
            let events = [
                Event::Mouse(mouse::Event::CursorMoved { position }),
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            ];

            for event in events {
                let cursor = mouse::Cursor::Available(position);
                let mut shell = Shell::new(&mut messages);
                self.element.as_widget_mut().update(
                    &mut self.tree,
                    &event,
                    Layout::with_offset(ORIGIN, &self.node),
                    cursor,
                    &self.renderer,
                    &mut clipboard,
                    &mut shell,
                    &viewport,
                );
            }

            messages
        }
    }

    fn test_renderer() -> iced::Renderer {
        iced_renderer::fallback::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::default(),
            Pixels(14.0),
        ))
    }

    #[test]
    fn checked_checkbox_uses_app_primary_background() {
        let theme = Theme::Dark;
        let checkbox = style(Radius::new(4.0))(&theme, Status::Active { is_checked: true });

        assert_eq!(
            background_color(checkbox.background),
            theme.tone(ToneRole::Accent).color
        );
    }

    #[test]
    fn unchecked_checkbox_uses_app_active_control_background() {
        let theme = Theme::Dark;
        let checkbox = style(Radius::new(4.0))(&theme, Status::Active { is_checked: false });

        assert_eq!(
            background_color(checkbox.background),
            theme
                .control(ControlRole::Standard, ControlState::ENABLED)
                .background
        );
    }

    fn background_color(background: Background) -> Color {
        match background {
            Background::Color(color) => color,
            _ => panic!("Expected color background"),
        }
    }

    #[test]
    fn enabled_checkbox_with_callback_emits_toggle() {
        let checkbox: Element<'_, Message> = Checkbox::new("Enabled", false)
            .on_toggle(Message::Toggled)
            .into();
        let mut harness = Harness::new(checkbox);
        let center = harness.center();

        let messages = harness.click(center);

        assert_eq!(messages, vec![Message::Toggled(true)]);
    }

    #[test]
    fn disabled_checkbox_ignores_present_callback() {
        let checkbox: Element<'_, Message> = Checkbox::new("Disabled", false)
            .on_toggle(Message::Toggled)
            .disabled(true)
            .into();
        let mut harness = Harness::new(checkbox);
        let center = harness.center();

        let messages = harness.click(center);

        assert!(messages.is_empty());
    }
}
