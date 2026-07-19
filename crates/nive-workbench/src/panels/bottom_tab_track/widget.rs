use super::*;

impl<'a, Message> Widget<Message, nive_ui::Theme, nive_ui::Renderer>
    for BottomPanelTabTrack<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TrackState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TrackState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(self.metrics().height))
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &nive_ui::Renderer,
        limits: &layout::Limits,
    ) -> Node {
        let truncated = measured_truncation(&self.items, self.size, self.active_index, renderer);
        self.content = nive_ui::widgets::TooltipScope::new(build_content(
            &self.items,
            self.size,
            self.active_index,
            TrackBuild::Actual(&truncated),
        ))
        .into();
        tree.children[0].diff(self.content.as_widget());

        let state = tree.state.downcast_ref::<TrackState>();
        let node = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let (content_width, viewport_width, item_bounds, translated) =
            translate_track(node, state.offset);
        let state = tree.state.downcast_mut::<TrackState>();
        state.viewport_width = viewport_width;
        state.max_offset = (content_width - viewport_width).max(0.0);
        state.offset = state.offset.clamp(0.0, state.max_offset);
        state.item_bounds = item_bounds;
        self.reconcile_focus(state);

        if state.last_active_index != Some(self.active_index) {
            state.last_active_index = Some(self.active_index);
            reveal_index(state, self.active_index);
        }
        translated
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &nive_ui::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        tree.state
            .downcast_mut::<TrackState>()
            .focus
            .register(operation, None, layout.bounds());
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
        renderer: &nive_ui::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
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
        if shell.is_event_captured() {
            return;
        }

        let item_bounds = current_item_bounds(layout);
        let state = tree.state.downcast_mut::<TrackState>();
        state.hovered_index = item_bounds
            .iter()
            .position(|bounds| cursor.is_over(*bounds));

        if let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
            if state.max_offset > 0.0 && cursor.is_over(layout.bounds()) {
                let delta = horizontal_wheel(*delta);
                state.offset = (state.offset - delta).clamp(0.0, state.max_offset);
                if delta != 0.0 {
                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            return;
        }

        if state.focus.is_active() {
            if let Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(named),
                repeat: false,
                ..
            }) = event
            {
                let enabled = self.enabled_indices();
                let current = state
                    .focused_index
                    .and_then(|focused| enabled.iter().position(|index| *index == focused))
                    .unwrap_or(0);
                let target = match named {
                    keyboard::key::Named::ArrowLeft => Some(current.saturating_sub(1)),
                    keyboard::key::Named::ArrowRight => {
                        Some((current + 1).min(enabled.len().saturating_sub(1)))
                    }
                    keyboard::key::Named::Home => Some(0),
                    keyboard::key::Named::End => Some(enabled.len().saturating_sub(1)),
                    _ => None,
                };
                if let Some(target) = target.and_then(|target| enabled.get(target).copied()) {
                    operation::Focusable::focus(&mut state.focus);
                    state.focused_index = Some(target);
                    reveal_index(state, target);
                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();
                    return;
                }
                if matches!(
                    named,
                    keyboard::key::Named::Enter | keyboard::key::Named::Space
                ) {
                    operation::Focusable::focus(&mut state.focus);
                    if let Some(message) = state
                        .focused_index
                        .and_then(|index| self.items.get(index))
                        .and_then(|item| item.message.clone())
                    {
                        shell.publish(message);
                        shell.capture_event();
                    }
                    return;
                }
            }
        }

        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. })
        ) {
            let pressed_index = match event {
                Event::Mouse(_) => state.hovered_index,
                Event::Touch(touch::Event::FingerPressed { position, .. }) => item_bounds
                    .iter()
                    .position(|bounds| bounds.contains(*position)),
                _ => None,
            }
            .filter(|index| !self.items[*index].metadata.disabled);

            if let Some(index) = pressed_index {
                state.focus.focus_from_pointer();
                state.focused_index = Some(index);
                shell.request_redraw();
            } else {
                state.focus.deactivate();
            }
        }

        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        ) {
            if let Some(message) = state
                .hovered_index
                .and_then(|index| self.items.get(index))
                .and_then(|item| item.message.clone())
            {
                shell.publish(message);
                shell.capture_event();
            }
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &nive_ui::Renderer,
    ) -> mouse::Interaction {
        if current_item_bounds(layout)
            .iter()
            .any(|bounds| cursor.is_over(*bounds))
        {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut nive_ui::Renderer,
        theme: &nive_ui::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<TrackState>();
        let item_bounds = current_item_bounds(layout);
        for (index, bounds) in item_bounds.iter().enumerate() {
            let disabled = self.items[index].metadata.disabled;
            let active = index == self.active_index;
            let hovered = state.hovered_index == Some(index) && !disabled;
            let control_state = if disabled {
                ControlState::DISABLED.selected_if(active)
            } else if hovered {
                ControlState::HOVERED.selected_if(active)
            } else {
                ControlState::ENABLED.selected_if(active)
            };
            let background = if active || hovered {
                theme
                    .control(ControlRole::Selectable, control_state)
                    .background
            } else {
                Color::TRANSPARENT
            };
            renderer.fill_quad(
                renderer::Quad {
                    bounds: *bounds,
                    border: Border::default(),
                    shadow: Shadow::default(),
                    snap: true,
                },
                background,
            );
        }
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
        let metrics = self.metrics();
        if let Some(bounds) = item_bounds.get(self.active_index) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x,
                        y: bounds.y + bounds.height - 2.0,
                        width: bounds.width,
                        height: 2.0,
                    },
                    border: Border::default(),
                    shadow: Shadow::default(),
                    snap: true,
                },
                theme
                    .control(ControlRole::Selectable, ControlState::SELECTED)
                    .foreground,
            );
        }
        if state.focus.is_focus_visible() {
            if let Some(bounds) = state.focused_index.and_then(|index| item_bounds.get(index)) {
                let focus = theme.border(BorderRole::Focus);
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: *bounds,
                        border: Border {
                            color: focus.color,
                            width: focus.width,
                            radius: metrics.radius.into(),
                        },
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    Color::TRANSPARENT,
                );
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &nive_ui::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, nive_ui::Theme, nive_ui::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}
