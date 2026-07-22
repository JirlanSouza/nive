use std::{cell::Cell, rc::Rc};

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Renderer as _, Shell, Widget,
    },
    keyboard::{self, key::Named},
    widget::{column, container, row, text},
    window, Alignment, Background, Border, Color, Event, Length, Padding, Rectangle, Shadow, Size,
    Vector,
};

use super::helpers::{
    first_enabled, is_enabled, is_primary_press, is_release, last_enabled, move_highlight,
    option_at, option_bounds, pointer_position, primary_position, release_position, set_highlight,
    typeahead_match,
};
use super::{SelectEvent, SelectList, SelectListState, TYPEAHEAD_TIMEOUT};
use crate::{
    theme::{
        choice::{self, ChoicePersistentState, ChoiceStateInput},
        BorderRole, ControlRole, FieldValidation, TypographyRole,
    },
    widgets::{
        controls::select::SelectOption,
        display::measured_text::{EllipsisStrategy, MeasuredText},
        navigation::menu::{
            self, MENU_COLUMN_GAP, MENU_ICON_SIZE, MENU_LIST_INSET, MENU_ROW_HEIGHT,
            MENU_ROW_PADDING_H, MENU_ROW_RADIUS,
        },
        overlays::anchored_overlay::scroll::EnsureVisibleHandle,
    },
};

impl SelectListState {
    pub(in crate::widgets::controls::select) fn reset(
        &mut self,
        highlight: Option<usize>,
        highlighted_label: Option<String>,
    ) {
        self.highlight = highlight;
        self.highlighted_label = highlighted_label;
        self.pressed = None;
        self.typeahead.clear();
        self.typeahead_deadline = None;
        self.ensure_pending = highlight.is_some();
    }
}

impl<'a, T> SelectList<'a, T>
where
    T: Clone + Eq + 'a,
{
    pub(in crate::widgets::controls::select) fn new(
        options: Vec<SelectOption<'a, T>>,
        selected: Option<T>,
        ensure_visible: EnsureVisibleHandle,
        focus_visible: Rc<Cell<bool>>,
        selection_capable: bool,
    ) -> Self {
        let mut content = column![].padding(MENU_LIST_INSET).width(Length::Fill);
        if options.is_empty() {
            content = content.push(
                container(MeasuredText::new_inherited(
                    "No options available",
                    EllipsisStrategy::End,
                    TypographyRole::Control,
                ))
                .style(menu::style::row_style(false, false, false, MENU_ROW_RADIUS))
                .padding(Padding::ZERO.horizontal(MENU_ROW_PADDING_H))
                .center_y(Length::Fixed(MENU_ROW_HEIGHT))
                .width(Length::Fill),
            );
        }
        for option in &options {
            let selected_row = selected.as_ref() == Some(option.value());
            let mark = if selected_row { "✓" } else { "" };
            let row = row![
                container(text(mark))
                    .width(Length::Fixed(MENU_ICON_SIZE))
                    .center_y(Length::Fill),
                container(MeasuredText::new_inherited(
                    option.label.clone(),
                    EllipsisStrategy::End,
                    TypographyRole::Control,
                ))
                .width(Length::Fill)
                .clip(true),
            ]
            .spacing(MENU_COLUMN_GAP)
            .align_y(Alignment::Center)
            .height(Length::Fill)
            .width(Length::Fill);
            content = content.push(
                container(row)
                    .style(menu::style::row_style(
                        selected_row,
                        false,
                        option.is_disabled(),
                        MENU_ROW_RADIUS,
                    ))
                    .padding(Padding::ZERO.horizontal(MENU_ROW_PADDING_H))
                    .height(Length::Fixed(MENU_ROW_HEIGHT))
                    .width(Length::Fill),
            );
        }

        Self {
            content: container(content).width(Length::Fill).into(),
            options,
            selected,
            ensure_visible,
            focus_visible,
            selection_capable,
        }
    }

    pub(in crate::widgets::controls::select) fn request_highlight_visible(
        &self,
        state: &mut SelectListState,
        layout: Layout<'_>,
    ) {
        if !state.ensure_pending {
            return;
        }
        if let Some(bounds) = state
            .highlight
            .and_then(|index| option_bounds(layout.bounds(), index, self.options.len()))
        {
            self.ensure_visible.request(bounds);
        }
        state.ensure_pending = false;
    }
}

impl<'a, T> Widget<SelectEvent<T>, crate::theme::Theme, iced::Renderer> for SelectList<'a, T>
where
    T: Clone + Eq + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SelectListState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(SelectListState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
        let state = tree.state.downcast_mut::<SelectListState>();
        let reconciled = state
            .highlighted_label
            .as_deref()
            .and_then(|label| {
                self.options.iter().enumerate().find_map(|(index, option)| {
                    (option.label() == label
                        && is_enabled(&self.options, index, self.selection_capable))
                    .then_some(index)
                })
            })
            .or_else(|| {
                state
                    .highlight
                    .filter(|index| is_enabled(&self.options, *index, self.selection_capable))
            })
            .or_else(|| {
                state
                    .highlighted_label
                    .is_some()
                    .then(|| first_enabled(&self.options, self.selection_capable))
                    .flatten()
            });
        set_highlight(&self.options, state, reconciled);
        state.pressed = None;
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
        shell: &mut Shell<'_, SelectEvent<T>>,
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

        let state = tree.state.downcast_mut::<SelectListState>();
        self.request_highlight_visible(state, layout);
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            state.now = Some(*now);
            if state
                .typeahead_deadline
                .is_some_and(|deadline| *now > deadline)
            {
                state.typeahead.clear();
                state.typeahead_deadline = None;
            }
        }

        if let Some(position) = pointer_position(event, cursor) {
            let highlight = position
                .and_then(|point| option_at(layout.bounds(), point, self.options.len()))
                .filter(|index| is_enabled(&self.options, *index, self.selection_capable));
            if state.highlight != highlight {
                set_highlight(&self.options, state, highlight);
                state.ensure_pending = highlight.is_some();
                shell.request_redraw();
            }
        }

        if is_primary_press(event) {
            state.pressed = primary_position(event, cursor)
                .and_then(|point| option_at(layout.bounds(), point, self.options.len()))
                .filter(|index| is_enabled(&self.options, *index, self.selection_capable));
            if state.pressed.is_some() {
                shell.capture_event();
            }
            shell.request_redraw();
        }

        if let Some(index) = release_position(event, cursor)
            .and_then(|point| option_at(layout.bounds(), point, self.options.len()))
            .filter(|index| Some(*index) == state.pressed)
        {
            shell.publish(SelectEvent::Commit(self.options[index].value().clone()));
            shell.capture_event();
        }
        if is_release(event) {
            state.pressed = None;
            shell.request_redraw();
        }

        if let Event::Keyboard(keyboard::Event::KeyPressed {
            text: Some(text), ..
        }) = event
        {
            if !text.is_empty() && text.chars().all(|character| !character.is_control()) {
                let now = state.now.unwrap_or_else(iced::time::Instant::now);
                if state
                    .typeahead_deadline
                    .is_none_or(|deadline| now > deadline)
                {
                    state.typeahead.clear();
                }
                state.typeahead.push_str(text);
                state.typeahead_deadline = Some(now + TYPEAHEAD_TIMEOUT);
                if let Some(index) = typeahead_match(
                    &self.options,
                    state.highlight,
                    state.typeahead.as_str(),
                    self.selection_capable,
                ) {
                    set_highlight(&self.options, state, Some(index));
                    state.ensure_pending = true;
                }
                shell.capture_event();
                shell.request_redraw();
                return;
            }
        }

        let moved = match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::ArrowDown),
                ..
            }) => move_highlight(&self.options, state.highlight, 1, self.selection_capable),
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::ArrowUp),
                ..
            }) => move_highlight(&self.options, state.highlight, -1, self.selection_capable),
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::Home),
                ..
            }) => first_enabled(&self.options, self.selection_capable),
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::End),
                ..
            }) => last_enabled(&self.options, self.selection_capable),
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::Enter | Named::Space),
                ..
            }) => {
                if let Some(index) = state.highlight {
                    shell.publish(SelectEvent::Commit(self.options[index].value().clone()));
                    shell.capture_event();
                }
                return;
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::Tab),
                ..
            }) => {
                shell.publish(SelectEvent::Close);
                return;
            }
            _ => return,
        };
        if moved != state.highlight {
            set_highlight(&self.options, state, moved);
            state.ensure_pending = moved.is_some();
            shell.request_redraw();
        }
        shell.capture_event();
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        cursor
            .position()
            .and_then(|point| option_at(layout.bounds(), point, self.options.len()))
            .filter(|index| is_enabled(&self.options, *index, self.selection_capable))
            .map_or(mouse::Interaction::None, |_| mouse::Interaction::Pointer)
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
        let state = tree.state.downcast_ref::<SelectListState>();
        for (index, option) in self.options.iter().enumerate() {
            let Some(bounds) = option_bounds(layout.bounds(), index, self.options.len()) else {
                continue;
            };
            let resolved = choice::resolve_state(ChoiceStateInput {
                persistent: if self.selected.as_ref() == Some(option.value()) {
                    ChoicePersistentState::Selected
                } else {
                    ChoicePersistentState::Unselected
                },
                validation: FieldValidation::Valid,
                callback_present: self.selection_capable && !option.is_disabled(),
                disabled: option.is_disabled(),
                hovered: state.highlight == Some(index),
                pressed: state.pressed == Some(index),
                focused: self.focus_visible.get() && state.highlight == Some(index),
            });
            let control = theme.control(ControlRole::Selectable, resolved.control);
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border::default().rounded(MENU_ROW_RADIUS),
                    shadow: Shadow::default(),
                    snap: true,
                },
                control.background,
            );
            if resolved.control.interaction.focused {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + 1.0,
                            y: bounds.y + 1.0,
                            width: (bounds.width - 2.0).max(0.0),
                            height: (bounds.height - 2.0).max(0.0),
                        },
                        border: Border {
                            color: theme.border(BorderRole::Focus).color,
                            width: 1.0,
                            radius: MENU_ROW_RADIUS.into(),
                        },
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    Background::Color(Color::TRANSPARENT),
                );
            }
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
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, SelectEvent<T>, crate::theme::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}
