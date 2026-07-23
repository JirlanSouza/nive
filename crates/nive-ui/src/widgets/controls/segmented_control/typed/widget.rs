mod compose;
mod draw;
mod update;

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Event, Length, Point, Rectangle, Size, Vector,
};

use super::{SegmentedControl, SegmentedFocus, SegmentedState};
use crate::theme::{choice::ChoiceMetrics, Theme, TypographyRole};
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
        self.layout_impl(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        self.operate_impl(tree, layout, renderer, operation);
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
        self.update_impl(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.mouse_interaction_impl(tree, layout, cursor, viewport, renderer)
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
        self.draw_impl(
            tree,
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
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        self.overlay_impl(tree, layout, renderer, viewport, translation)
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
