use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Renderer as _, Shell, Widget,
    },
    border::Radius,
    keyboard::{self, key},
    touch, Background, Border, Color, Event, Length, Point, Rectangle, Shadow, Size, Vector,
};

use super::{
    inset_radius, segment_radius, SegmentedControl, SegmentedControlVariant, SegmentedFocus,
    SegmentedState,
};
use crate::theme::{
    choice::{self, ChoiceMetrics, ChoicePersistentState, ChoiceStateInput},
    BorderRole, ControlRole, FieldValidation, TextRole, Theme, TypographyRole,
};
use crate::widgets::display::measured_text::{EllipsisStrategy, MeasuredText};
use crate::widgets::primitives::icon;
use crate::Element;

impl<'a, T, Message> SegmentedControl<'a, T, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    pub(super) fn values_are_unique(&self) -> bool {
        self.options.iter().enumerate().all(|(index, option)| {
            self.options[index + 1..]
                .iter()
                .all(|peer| peer.value != option.value)
        })
    }

    pub(super) fn selected_index(&self) -> Option<usize> {
        self.options
            .iter()
            .position(|option| option.value == self.selected)
    }

    pub(super) fn model_valid(&self) -> bool {
        !self.semantic_name.trim().is_empty()
            && (2..=5).contains(&self.options.len())
            && self.values_are_unique()
            && self.selected_index().is_some()
    }

    pub(super) fn interactive(&self) -> bool {
        self.model_valid()
            && !self.disabled
            && self.on_select.is_some()
            && self.options.iter().any(|option| !option.disabled)
    }

    pub(super) fn reserve_icon(&self) -> bool {
        self.options.iter().any(|option| option.icon.is_some())
    }

    pub(super) fn metrics(&self, theme: Theme) -> ChoiceMetrics {
        ChoiceMetrics::for_theme(theme, self.size)
    }

    pub(super) fn item_content(&self, index: usize, maximum_width: f32) -> Element<'a, Message> {
        let option = &self.options[index];
        let metrics = self.metrics(crate::theme::active());
        let reserve_icon = self.reserve_icon();
        let icon_width = if reserve_icon {
            metrics.form.icon_size + metrics.form.gap
        } else {
            0.0
        };
        let label_width = (maximum_width - icon_width).max(0.0);
        let label = MeasuredText::new_inherited(
            option.label.clone(),
            EllipsisStrategy::End,
            TypographyRole::ControlStrong,
        )
        .max_width(label_width);
        let mut row = iced::widget::Row::new()
            .spacing(metrics.form.gap)
            .align_y(iced::Alignment::Center)
            .height(Length::Fixed(metrics.form.height));

        if reserve_icon {
            let icon: Element<'a, Message> = if let Some(role) = option.icon {
                icon::role(role).custom_size(metrics.form.icon_size).into()
            } else {
                iced::widget::Space::new()
                    .width(metrics.form.icon_size)
                    .height(metrics.form.icon_size)
                    .into()
            };
            row = row.push(
                iced::widget::Container::new(icon)
                    .width(metrics.form.icon_size)
                    .height(metrics.form.icon_size),
            );
        }

        row.push(label).into()
    }

    pub(super) fn reconciled_focus(&self, state: &SegmentedState) -> Option<usize> {
        state
            .focused_index
            .filter(|index| {
                self.options
                    .get(*index)
                    .is_some_and(|option| !option.disabled)
            })
            .or_else(|| {
                self.selected_index()
                    .filter(|index| !self.options[*index].disabled)
            })
            .or_else(|| self.options.iter().position(|option| !option.disabled))
    }

    pub(super) fn item_at(
        &self,
        state: &SegmentedState,
        layout: Layout<'_>,
        point: Point,
    ) -> Option<usize> {
        let origin = layout.bounds().position();
        state.item_bounds.iter().position(|bounds| {
            Rectangle {
                x: bounds.x + origin.x,
                y: bounds.y + origin.y,
                ..*bounds
            }
            .contains(point)
        })
    }

    pub(super) fn publish_if_changed(&self, index: usize, shell: &mut Shell<'_, Message>) {
        let Some(option) = self.options.get(index) else {
            return;
        };
        if option.disabled || option.value == self.selected {
            return;
        }
        if let Some(on_select) = &self.on_select {
            shell.publish(on_select(option.value.clone()));
        }
    }

    pub(super) fn move_bounded(
        &self,
        state: &mut SegmentedState,
        direction: isize,
    ) -> Option<usize> {
        let current = self.reconciled_focus(state)? as isize;
        let mut next = current + direction;

        while let Some(option) = usize::try_from(next)
            .ok()
            .and_then(|index| self.options.get(index))
        {
            if !option.disabled {
                let index = next as usize;
                state.focused_index = Some(index);
                return Some(index);
            }
            next += direction;
        }

        None
    }
}

impl<'a, T, Message> Widget<Message, Theme, iced::Renderer> for SegmentedControl<'a, T, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SegmentedState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(SegmentedState::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.contents.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(
            &self
                .contents
                .iter()
                .map(Element::as_widget)
                .collect::<Vec<_>>(),
        );

        let state = tree.state.downcast_mut::<SegmentedState>();
        if state.focus.is_active() {
            state.focused_index = self
                .selected_index()
                .filter(|index| !self.options[*index].disabled)
                .or_else(|| self.options.iter().position(|option| !option.disabled));
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(
            self.width,
            Length::Fixed(self.metrics(crate::theme::active()).form.height),
        )
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let metrics = self.metrics(crate::theme::active());
        let limits = limits
            .width(self.width)
            .height(Length::Fixed(metrics.form.height));
        let border = metrics.perimeter_width;
        let inset = if self.variant == SegmentedControlVariant::Default {
            2.0
        } else {
            0.0
        };
        let padding = metrics.form.padding.left.max(0.0);
        let mut intrinsic_items = Vec::with_capacity(self.options.len());

        for index in 0..self.options.len() {
            let mut content = self.item_content(index, f32::INFINITY);
            tree.children[index].diff(content.as_widget());
            let node = content.as_widget_mut().layout(
                &mut tree.children[index],
                renderer,
                &layout::Limits::NONE,
            );
            intrinsic_items.push(node.size().width + padding * 2.0);
        }

        let intrinsic_width = intrinsic_items.iter().sum::<f32>() + (border + inset) * 2.0;
        let resolved = limits.resolve(
            self.width,
            Length::Fixed(metrics.form.height),
            Size::new(intrinsic_width, metrics.form.height),
        );
        let inner_width = (resolved.width - (border + inset) * 2.0).max(0.0);
        let item_width = if self.options.is_empty() {
            0.0
        } else {
            inner_width / self.options.len() as f32
        };
        let item_height = (resolved.height - (border + inset) * 2.0).max(0.0);
        let mut nodes = Vec::with_capacity(self.options.len());
        let mut bounds = Vec::with_capacity(self.options.len());
        let left = border + inset;
        let right = left + inner_width;

        for index in 0..self.options.len() {
            let x = left + item_width * index as f32;
            let next_x = if index + 1 == self.options.len() {
                right
            } else {
                left + item_width * (index + 1) as f32
            };
            let item = Rectangle {
                x,
                y: border + inset,
                width: (next_x - x).max(0.0),
                height: item_height,
            };
            let content_width = (item.width - padding * 2.0).max(0.0);
            let content = self.item_content(index, content_width);
            self.contents[index] = content;
            tree.children[index].diff(self.contents[index].as_widget());
            let node = self.contents[index].as_widget_mut().layout(
                &mut tree.children[index],
                renderer,
                &layout::Limits::new(Size::ZERO, Size::new(content_width, item.height)),
            );
            let content_y = item.y + (item.height - node.size().height).max(0.0) / 2.0;
            nodes.push(node.move_to(Point::new(item.x + padding, content_y)));
            bounds.push(item);
        }

        tree.state.downcast_mut::<SegmentedState>().item_bounds = bounds;
        layout::Node::with_children(resolved, nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<SegmentedState>();
        if self.interactive() {
            let SegmentedState {
                focus,
                focused_index,
                pressed_index,
                touch,
                ..
            } = state;
            focus.expose(operation, self.id.as_ref(), layout.bounds());
            operation.focusable(
                self.id.as_ref(),
                layout.bounds(),
                &mut SegmentedFocus {
                    focus,
                    focused_index,
                    pressed_index,
                    touch,
                },
            );
        } else {
            state.focus.clear();
        }
        for (index, (tree, child_layout)) in
            tree.children.iter_mut().zip(layout.children()).enumerate()
        {
            self.contents[index]
                .as_widget_mut()
                .operate(tree, child_layout, renderer, operation);
        }
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
        for (index, (tree, child_layout)) in
            tree.children.iter_mut().zip(layout.children()).enumerate()
        {
            self.contents[index].as_widget_mut().update(
                tree,
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        let interactive = self.interactive();
        let state = tree.state.downcast_mut::<SegmentedState>();
        if !interactive {
            state.focus.clear();
            state.focused_index = None;
            state.pressed_index = None;
            state.touch = None;
            return;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let hit = cursor
                    .position()
                    .and_then(|point| self.item_at(state, layout, point));
                if let Some(index) = hit.filter(|index| !self.options[*index].disabled) {
                    state.focus.focus_from_pointer();
                    state.focused_index = Some(index);
                    state.pressed_index = Some(index);
                    shell.capture_event();
                } else {
                    state.focus.deactivate();
                    state.pressed_index = None;
                }
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let hit = cursor
                    .position()
                    .and_then(|point| self.item_at(state, layout, point));
                let activates = state.pressed_index.filter(|index| Some(*index) == hit);
                state.pressed_index = None;
                if let Some(index) = activates {
                    self.publish_if_changed(index, shell);
                    shell.capture_event();
                }
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::CursorLeft) => {
                state.pressed_index = None;
                shell.request_redraw();
            }
            Event::Touch(touch::Event::FingerPressed { id, position }) => {
                if let Some(index) = self
                    .item_at(state, layout, *position)
                    .filter(|index| !self.options[*index].disabled)
                {
                    state.focus.focus_from_pointer();
                    state.focused_index = Some(index);
                    state.touch = Some((*id, index));
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Touch(touch::Event::FingerLifted { id, position }) => {
                let hit = self.item_at(state, layout, *position);
                let activates = state
                    .touch
                    .filter(|(finger, index)| finger == id && Some(*index) == hit)
                    .map(|(_, index)| index);
                if state.touch.is_some_and(|(finger, _)| finger == *id) {
                    state.touch = None;
                }
                if let Some(index) = activates {
                    self.publish_if_changed(index, shell);
                    shell.capture_event();
                }
                shell.request_redraw();
            }
            Event::Touch(touch::Event::FingerLost { id, .. })
                if state.touch.is_some_and(|(finger, _)| finger == *id) =>
            {
                state.touch = None;
                shell.request_redraw();
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(named),
                repeat: false,
                ..
            }) if state.focus.is_active() => {
                let focus_key = matches!(
                    named,
                    key::Named::ArrowLeft
                        | key::Named::ArrowRight
                        | key::Named::Home
                        | key::Named::End
                        | key::Named::Space
                        | key::Named::Enter
                );
                if focus_key {
                    state.focus.focus_from_keyboard();
                }
                let target = match named {
                    key::Named::ArrowLeft => self.move_bounded(state, -1),
                    key::Named::ArrowRight => self.move_bounded(state, 1),
                    key::Named::Home => self.options.iter().position(|option| !option.disabled),
                    key::Named::End => self.options.iter().rposition(|option| !option.disabled),
                    key::Named::Space | key::Named::Enter => self.reconciled_focus(state),
                    _ => None,
                };
                if let Some(index) = target {
                    state.focused_index = Some(index);
                    self.publish_if_changed(index, shell);
                    shell.capture_event();
                    shell.request_redraw();
                } else if focus_key {
                    shell.request_redraw();
                }
            }
            Event::Window(iced::window::Event::Unfocused) => {
                state.focus.deactivate();
                state.pressed_index = None;
                state.touch = None;
                shell.request_redraw();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if !self.interactive() {
            return mouse::Interaction::None;
        }
        let state = tree.state.downcast_ref::<SegmentedState>();
        if cursor
            .position()
            .and_then(|point| self.item_at(state, layout, point))
            .is_some_and(|index| !self.options[index].disabled)
        {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let metrics = self.metrics(*theme);
        let state = tree.state.downcast_ref::<SegmentedState>();
        let bounds = layout.bounds();
        let track = theme.control(ControlRole::Standard, crate::theme::ControlState::ENABLED);

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: theme.border(BorderRole::Default).color,
                    width: metrics.perimeter_width,
                    radius: Radius::new(metrics.form.radius),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(track.background),
        );

        let origin = bounds.position();
        for (index, item) in state.item_bounds.iter().enumerate() {
            let item = Rectangle {
                x: item.x + origin.x,
                y: item.y + origin.y,
                ..*item
            };
            let selected = self.options[index].value == self.selected;
            let hovered = cursor.is_over(item);
            let pressed = state.pressed_index == Some(index)
                || state.touch.is_some_and(|(_, pressed)| pressed == index);
            let focused =
                state.focus.is_focus_visible() && self.reconciled_focus(state) == Some(index);
            let resolved = choice::resolve_state(ChoiceStateInput {
                persistent: if selected {
                    ChoicePersistentState::Selected
                } else {
                    ChoicePersistentState::Unselected
                },
                validation: FieldValidation::Valid,
                callback_present: self.on_select.is_some() && self.model_valid(),
                disabled: self.disabled || self.options[index].disabled,
                hovered,
                pressed,
                focused,
            });
            let palette = choice::segment_palette(*theme, resolved);
            let fill = match self.variant {
                SegmentedControlVariant::Default if !selected && !hovered && !pressed => {
                    Color::TRANSPARENT
                }
                _ => palette.background,
            };
            let radius = segment_radius(
                self.variant,
                index,
                self.options.len(),
                metrics.form.radius.max(0.0),
            );
            renderer.fill_quad(
                renderer::Quad {
                    bounds: item,
                    border: Border {
                        color: if self.variant == SegmentedControlVariant::Linked {
                            palette.perimeter
                        } else {
                            Color::TRANSPARENT
                        },
                        width: if self.variant == SegmentedControlVariant::Linked {
                            metrics.perimeter_width
                        } else {
                            0.0
                        },
                        radius,
                    },
                    shadow: Shadow::default(),
                    snap: true,
                },
                fill,
            );

            if focused {
                let focus_bounds = metrics.segment_focus_bounds(item);
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: focus_bounds,
                        border: Border {
                            color: palette.focus,
                            width: metrics.focus_stroke_width,
                            radius: inset_radius(radius, metrics.form.focus_inset),
                        },
                        ..renderer::Quad::default()
                    },
                    Color::TRANSPARENT,
                );
            }

            let child_layout = layout.children().nth(index);
            if let Some(child_layout) = child_layout {
                let child_style = renderer::Style {
                    text_color: if self.disabled || self.options[index].disabled {
                        theme.text(TextRole::Disabled).color
                    } else {
                        palette.foreground
                    },
                };
                self.contents[index].as_widget().draw(
                    &tree.children[index],
                    renderer,
                    theme,
                    &child_style,
                    child_layout,
                    cursor,
                    viewport,
                );
            }
        }

        let _ = inherited_style;
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        let overlays = self
            .contents
            .iter_mut()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
            .filter_map(|((content, tree), layout)| {
                content
                    .as_widget_mut()
                    .overlay(tree, layout, renderer, viewport, translation)
            })
            .collect::<Vec<_>>();

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

impl operation::Focusable for SegmentedFocus<'_> {
    fn is_focused(&self) -> bool {
        operation::Focusable::is_focused(self.focus)
    }

    fn focus(&mut self) {
        operation::Focusable::focus(self.focus);
        *self.focused_index = None;
    }

    fn unfocus(&mut self) {
        operation::Focusable::unfocus(self.focus);
        *self.focused_index = None;
        *self.pressed_index = None;
        *self.touch = None;
    }
}
