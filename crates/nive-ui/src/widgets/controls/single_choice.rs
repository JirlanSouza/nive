use std::borrow::Cow;

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        text::Renderer as _,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Renderer as _, Shell, Widget,
    },
    alignment,
    border::Radius,
    keyboard::{self, key},
    touch, widget, Background, Border, Color, Event, Font, Length, Padding, Pixels, Rectangle,
    Shadow, Size, Vector,
};

use crate::theme::{
    self,
    choice::{self, ChoiceMetrics, ChoicePersistentState, ChoiceStateInput, ResolvedChoiceState},
    ControlSize, FieldValidation, TextRole, TypographyRole,
};
use crate::widgets::text;
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SingleChoiceKind {
    Checkbox,
    Radio,
    Switch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SingleChoiceLayout {
    Leading,
    Setting,
}

pub(super) struct SingleChoice<'a, Message> {
    kind: SingleChoiceKind,
    layout: SingleChoiceLayout,
    label: Cow<'a, str>,
    description: Option<Cow<'a, str>>,
    persistent: ChoicePersistentState,
    validation: FieldValidation,
    size: ControlSize,
    width: Length,
    disabled: bool,
    id: Option<widget::Id>,
    on_activate: Option<Message>,
    register_focus: bool,
    focused_override: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PressSource {
    Pointer,
    Touch(touch::Finger),
    Space,
}

#[derive(Debug, Default)]
struct SingleChoiceState {
    focused: bool,
    focus_visible: bool,
    press: Option<PressSource>,
}

impl<'a, Message> SingleChoice<'a, Message> {
    pub(super) fn new(
        kind: SingleChoiceKind,
        layout: SingleChoiceLayout,
        label: Cow<'a, str>,
        persistent: ChoicePersistentState,
    ) -> Self {
        Self {
            kind,
            layout,
            label,
            description: None,
            persistent,
            validation: FieldValidation::Valid,
            size: ControlSize::Sm,
            width: Length::Shrink,
            disabled: false,
            id: None,
            on_activate: None,
            register_focus: true,
            focused_override: false,
        }
    }

    pub(super) fn description(mut self, description: Option<Cow<'a, str>>) -> Self {
        self.description = description;
        self
    }

    pub(super) fn validation(mut self, validation: FieldValidation) -> Self {
        self.validation = validation;
        self
    }

    pub(super) fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub(super) fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub(super) fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub(super) fn id(mut self, id: Option<widget::Id>) -> Self {
        self.id = id;
        self
    }

    pub(super) fn on_activate(mut self, on_activate: Option<Message>) -> Self {
        self.on_activate = on_activate;
        self
    }

    pub(super) fn register_focus(mut self, register_focus: bool) -> Self {
        self.register_focus = register_focus;
        self
    }

    pub(super) fn focused(mut self, focused: bool) -> Self {
        self.focused_override = focused;
        self
    }

    fn metrics(&self, theme: crate::theme::Theme) -> ChoiceMetrics {
        ChoiceMetrics::for_theme(theme, self.size)
    }

    fn content(&self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let metrics = self.metrics(theme::active());
        let visual_width = match self.kind {
            SingleChoiceKind::Checkbox | SingleChoiceKind::Radio => metrics.indicator_size,
            SingleChoiceKind::Switch => metrics.switch_track.width,
        };
        let anchor = widget::Space::new()
            .width(Length::Fixed(
                visual_width + metrics.focus_stroke_width * 2.0,
            ))
            .height(Length::Fixed(metrics.form.height));
        let label_role = if self.disabled {
            TextRole::Disabled
        } else {
            TextRole::Primary
        };
        let description_role = if self.disabled {
            TextRole::Disabled
        } else {
            TextRole::Secondary
        };
        let mut copy = widget::Column::new()
            .push(
                text::with_role(self.label.clone(), TypographyRole::Control, label_role)
                    .wrapping(widget::text::Wrapping::WordOrGlyph),
            )
            .spacing(metrics.support_gap);

        if let Some(description) = &self.description {
            copy = copy.push(
                text::with_role(
                    description.clone(),
                    TypographyRole::BodySmall,
                    description_role,
                )
                .wrapping(widget::text::Wrapping::WordOrGlyph),
            );
        }

        let copy_width = if matches!(self.width, Length::Shrink) {
            Length::Shrink
        } else {
            Length::Fill
        };
        let copy = widget::Container::new(copy.width(copy_width))
            .padding(Padding {
                top: metrics.form.content_inset,
                right: 0.0,
                bottom: metrics.form.content_inset,
                left: 0.0,
            })
            .width(copy_width)
            .height(Length::Shrink);
        let row = match self.layout {
            SingleChoiceLayout::Leading => widget::Row::new()
                .push(anchor)
                .push(copy)
                .spacing(metrics.form.gap),
            SingleChoiceLayout::Setting => widget::Row::new()
                .push(copy)
                .push(anchor)
                .spacing(metrics.form.gap),
        };

        row.align_y(iced::Alignment::Start)
            .width(self.width)
            .height(Length::Shrink)
            .into()
    }

    fn resolved_state(
        &self,
        state: &SingleChoiceState,
        cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> ResolvedChoiceState {
        choice::resolve_state(ChoiceStateInput {
            persistent: self.persistent,
            validation: self.validation,
            callback_present: self.on_activate.is_some(),
            disabled: self.disabled,
            hovered: cursor.is_over(bounds),
            pressed: state.press.is_some(),
            focused: (state.focused && state.focus_visible) || self.focused_override,
        })
    }

    fn anchor_bounds(&self, layout: Layout<'_>, metrics: ChoiceMetrics) -> Rectangle {
        let anchor_slot = match self.layout {
            SingleChoiceLayout::Leading => layout.children().next(),
            SingleChoiceLayout::Setting => layout.children().nth(1),
        }
        .map_or(layout.bounds(), |layout| layout.bounds());
        let size = match self.kind {
            SingleChoiceKind::Checkbox | SingleChoiceKind::Radio => {
                Size::new(metrics.indicator_size, metrics.indicator_size)
            }
            SingleChoiceKind::Switch => metrics.switch_track,
        };

        Rectangle {
            x: anchor_slot.x + (anchor_slot.width - size.width) / 2.0,
            y: anchor_slot.y + (metrics.form.height - size.height) / 2.0,
            width: size.width,
            height: size.height,
        }
    }
}

impl<Message> Widget<Message, crate::theme::Theme, iced::Renderer> for SingleChoice<'_, Message>
where
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SingleChoiceState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(SingleChoiceState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(self.content())]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content().as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn size_hint(&self) -> Size<Length> {
        self.size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let mut content = self.content();
        content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &limits.width(self.width))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<SingleChoiceState>();

        if self.register_focus && self.on_activate.is_some() && !self.disabled {
            operation.focusable(self.id.as_ref(), layout.bounds(), state);
        }

        let mut content = self.content();
        content
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
        let mut content = self.content();
        content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let interactive = self.on_activate.is_some() && !self.disabled;
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<SingleChoiceState>();

        if !interactive {
            if state.focused || state.press.is_some() {
                state.focused = false;
                state.focus_visible = false;
                state.press = None;
                shell.request_redraw();
            }
            return;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if cursor.is_over(bounds) {
                    if self.register_focus {
                        state.focused = true;
                        state.focus_visible = false;
                    }
                    state.press = Some(PressSource::Pointer);
                    shell.capture_event();
                } else {
                    if self.register_focus {
                        state.focused = false;
                        state.focus_visible = false;
                    }
                    state.press = None;
                }
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let activates = state.press == Some(PressSource::Pointer) && cursor.is_over(bounds);
                state.press = None;
                if activates {
                    shell.publish(
                        self.on_activate
                            .clone()
                            .expect("interactive choice message"),
                    );
                    shell.capture_event();
                }
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::CursorLeft) if state.press == Some(PressSource::Pointer) => {
                state.press = None;
                shell.request_redraw();
            }
            Event::Touch(touch::Event::FingerPressed { id, position }) => {
                if bounds.contains(*position) {
                    if self.register_focus {
                        state.focused = true;
                        state.focus_visible = false;
                    }
                    state.press = Some(PressSource::Touch(*id));
                    shell.capture_event();
                    shell.request_redraw();
                } else {
                    if self.register_focus {
                        state.focused = false;
                        state.focus_visible = false;
                    }
                }
            }
            Event::Touch(touch::Event::FingerLifted { id, position }) => {
                let activates =
                    state.press == Some(PressSource::Touch(*id)) && bounds.contains(*position);
                if state.press == Some(PressSource::Touch(*id)) {
                    state.press = None;
                }
                if activates {
                    shell.publish(
                        self.on_activate
                            .clone()
                            .expect("interactive choice message"),
                    );
                    shell.capture_event();
                }
                shell.request_redraw();
            }
            Event::Touch(touch::Event::FingerLost { id, .. })
                if state.press == Some(PressSource::Touch(*id)) =>
            {
                state.press = None;
                shell.request_redraw();
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Space),
                repeat: false,
                ..
            }) if state.focused => {
                state.focus_visible = true;
                state.press = Some(PressSource::Space);
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Keyboard(keyboard::Event::KeyReleased {
                key: keyboard::Key::Named(key::Named::Space),
                ..
            }) if state.focused && state.press == Some(PressSource::Space) => {
                state.press = None;
                shell.publish(
                    self.on_activate
                        .clone()
                        .expect("interactive choice message"),
                );
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Window(iced::window::Event::Unfocused) => {
                state.press = None;
                state.focused = false;
                state.focus_visible = false;
                shell.request_redraw();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if self.on_activate.is_some() && !self.disabled && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let metrics = self.metrics(*theme);
        let state = tree.state.downcast_ref::<SingleChoiceState>();
        let resolved = self.resolved_state(state, cursor, layout.bounds());
        let palette = choice::palette(*theme, resolved);
        let anchor = self.anchor_bounds(layout, metrics);
        let radius = match self.kind {
            SingleChoiceKind::Checkbox => metrics.checkbox_radius,
            SingleChoiceKind::Radio | SingleChoiceKind::Switch => anchor.height / 2.0,
        };

        let anchor_background = if self.kind == SingleChoiceKind::Radio
            && resolved.control.selected
            && resolved.control.enabled
        {
            theme
                .control(
                    crate::theme::ControlRole::Selectable,
                    crate::theme::ControlState::ENABLED,
                )
                .background
        } else {
            palette.background
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: anchor,
                border: Border {
                    color: palette.perimeter,
                    width: metrics.perimeter_width,
                    radius: Radius::new(radius),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(anchor_background),
        );

        match self.kind {
            SingleChoiceKind::Checkbox if resolved.mixed => {
                let mark = Rectangle {
                    x: anchor.x + anchor.width * 0.25,
                    y: anchor.center_y() - 1.0,
                    width: anchor.width * 0.5,
                    height: 2.0,
                };
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: mark,
                        border: Border {
                            radius: Radius::new(1.0),
                            ..Border::default()
                        },
                        ..renderer::Quad::default()
                    },
                    palette.mark,
                );
            }
            SingleChoiceKind::Checkbox if resolved.control.selected => {
                renderer.fill_text(
                    iced::advanced::text::Text {
                        content: "✓".to_owned(),
                        bounds: anchor.size(),
                        size: Pixels(anchor.height * 0.72),
                        line_height: widget::text::LineHeight::default(),
                        font: Font::DEFAULT,
                        align_x: widget::text::Alignment::Center,
                        align_y: alignment::Vertical::Center,
                        shaping: widget::text::Shaping::Advanced,
                        wrapping: widget::text::Wrapping::None,
                    },
                    anchor.center(),
                    palette.mark,
                    *viewport,
                );
            }
            SingleChoiceKind::Radio if resolved.control.selected => {
                let dot_size = metrics.radio_dot_size();
                let dot = Rectangle {
                    x: anchor.center_x() - dot_size / 2.0,
                    y: anchor.center_y() - dot_size / 2.0,
                    width: dot_size,
                    height: dot_size,
                };
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: dot,
                        border: Border {
                            radius: Radius::new(dot_size / 2.0),
                            ..Border::default()
                        },
                        ..renderer::Quad::default()
                    },
                    if resolved.control.enabled {
                        theme.tone(crate::theme::ToneRole::Accent).color
                    } else {
                        palette.mark
                    },
                );
            }
            SingleChoiceKind::Switch => {
                let thumb_size = metrics.switch_thumb_size;
                let thumb = Rectangle {
                    x: if resolved.control.selected {
                        anchor.x + anchor.width - metrics.switch_thumb_inset - thumb_size
                    } else {
                        anchor.x + metrics.switch_thumb_inset
                    },
                    y: anchor.y + metrics.switch_thumb_inset,
                    width: thumb_size,
                    height: thumb_size,
                };
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: thumb,
                        border: Border {
                            radius: Radius::new(thumb_size / 2.0),
                            ..Border::default()
                        },
                        ..renderer::Quad::default()
                    },
                    palette.mark,
                );
            }
            _ => {}
        }

        let content = self.content();
        content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );

        if resolved.control.interaction.focused {
            let (bounds, focus_radius) = match self.kind {
                SingleChoiceKind::Checkbox => (
                    metrics.indicator_focus_bounds(anchor),
                    metrics.checkbox_focus_radius(),
                ),
                SingleChoiceKind::Radio => (
                    metrics.indicator_focus_bounds(anchor),
                    metrics.radio_focus_radius(),
                ),
                SingleChoiceKind::Switch => (
                    metrics.track_focus_bounds(anchor),
                    metrics.switch_focus_radius(),
                ),
            };
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        color: palette.focus,
                        width: metrics.focus_stroke_width,
                        radius: Radius::new(focus_radius),
                    },
                    ..renderer::Quad::default()
                },
                Color::TRANSPARENT,
            );
        }
    }

    fn overlay<'a>(
        &'a mut self,
        _tree: &'a mut Tree,
        _layout: Layout<'a>,
        _renderer: &iced::Renderer,
        _viewport: &Rectangle,
        _translation: Vector,
    ) -> Option<overlay::Element<'a, Message, crate::theme::Theme, iced::Renderer>> {
        None
    }
}

impl operation::Focusable for SingleChoiceState {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
        self.focus_visible = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
        self.focus_visible = false;
        self.press = None;
    }
}

impl<'a, Message> From<SingleChoice<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(choice: SingleChoice<'a, Message>) -> Self {
        Element::new(choice)
    }
}

#[cfg(test)]
mod tests {
    use iced::{keyboard::key, touch, Point};

    use super::*;
    use crate::test_support::WidgetHarness;
    use crate::widgets::controls::choice_test_support::{
        key_pressed, key_released, pointer_click, pointer_move, pointer_press, pointer_release,
        touch_lift, touch_press,
    };

    fn checkbox(message: Option<&'static str>) -> Element<'static, &'static str> {
        SingleChoice::new(
            SingleChoiceKind::Checkbox,
            SingleChoiceLayout::Leading,
            Cow::Borrowed("Choice"),
            ChoicePersistentState::Unselected,
        )
        .on_activate(message)
        .into()
    }

    #[test]
    fn pointer_touch_and_space_activate_once() {
        let mut pointer = WidgetHarness::new(checkbox(Some("toggle")), Size::new(240.0, 80.0));
        assert_eq!(
            pointer_click(&mut pointer, Point::new(8.0, 8.0)),
            ["toggle"]
        );
        assert!(pointer.state::<SingleChoiceState>().focused);
        assert!(!pointer.state::<SingleChoiceState>().focus_visible);

        let mut touch = WidgetHarness::new(checkbox(Some("toggle")), Size::new(240.0, 80.0));
        assert!(touch.update(touch_press(1, Point::new(8.0, 8.0))).captured);
        assert_eq!(
            touch.update(touch_lift(1, Point::new(8.0, 8.0))).messages,
            ["toggle"]
        );

        let id = widget::Id::new("choice");
        let choice: Element<'_, &'static str> = SingleChoice::new(
            SingleChoiceKind::Checkbox,
            SingleChoiceLayout::Leading,
            Cow::Borrowed("Choice"),
            ChoicePersistentState::Unselected,
        )
        .id(Some(id.clone()))
        .on_activate(Some("toggle"))
        .into();
        let mut keyboard = WidgetHarness::new(choice, Size::new(240.0, 80.0));
        keyboard.focus(id);
        assert!(keyboard.state::<SingleChoiceState>().focus_visible);
        assert!(keyboard
            .update(key_pressed(key::Named::Space, key::Code::Space))
            .messages
            .is_empty());
        assert_eq!(
            keyboard
                .update(key_released(key::Named::Space, key::Code::Space))
                .messages,
            ["toggle"]
        );
    }

    #[test]
    fn release_outside_and_lost_touch_cancel_activation() {
        let mut pointer = WidgetHarness::new(checkbox(Some("toggle")), Size::new(240.0, 80.0));
        pointer.set_cursor(Point::new(8.0, 8.0));
        pointer.update(pointer_press());
        pointer.update(pointer_move(Point::new(220.0, 70.0)));
        pointer.set_cursor(Point::new(220.0, 70.0));
        assert!(pointer.update(pointer_release()).messages.is_empty());

        let mut touch = WidgetHarness::new(checkbox(Some("toggle")), Size::new(240.0, 80.0));
        touch.update(touch_press(7, Point::new(8.0, 8.0)));
        assert!(touch
            .update(Event::Touch(touch::Event::FingerLost {
                id: touch::Finger(7),
                position: Point::new(220.0, 70.0),
            }))
            .messages
            .is_empty());
    }

    #[test]
    fn display_only_and_disabled_have_no_focus_or_activation() {
        let mut display = WidgetHarness::new(checkbox(None), Size::new(240.0, 80.0));
        assert_eq!(
            display.focused_count(),
            operation::focusable::Count::default()
        );
        assert!(pointer_click(&mut display, Point::new(8.0, 8.0)).is_empty());

        let disabled: Element<'_, &'static str> = SingleChoice::new(
            SingleChoiceKind::Switch,
            SingleChoiceLayout::Leading,
            Cow::Borrowed("Switch"),
            ChoicePersistentState::Selected,
        )
        .disabled(true)
        .on_activate(Some("toggle"))
        .into();
        let mut disabled = WidgetHarness::new(disabled, Size::new(240.0, 80.0));
        assert_eq!(
            disabled.focused_count(),
            operation::focusable::Count::default()
        );
        assert!(pointer_click(&mut disabled, Point::new(8.0, 8.0)).is_empty());
    }
}
