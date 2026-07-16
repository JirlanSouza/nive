use std::borrow::Cow;

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, Tree},
        Clipboard, Layout, Renderer as _, Shell, Widget,
    },
    Event, Length, Point, Rectangle, Size, Vector,
};

use crate::{
    theme::{ControlSize, TextRole, ToneRole, TypographyRole},
    widgets::ToneDot,
    Element,
};

use super::{
    super::measured_text::{measure_width, EllipsisStrategy, MeasuredText},
    style as theme_metadata,
};

/// Static, surface-neutral metadata row with protected peer slots.
///
/// Whole-row interaction belongs to a selectable host. Prefer a one-item
/// `ActionGroup` containing `ContentAction` for a trailing peer action. The
/// row uses density-resolved ControlSize minimum height, measured end ellipsis,
/// and exact-value disclosure only when constrained. A tone is valid only when
/// the visible principal/secondary cluster completely states its meaning.
pub struct DataRow<'a, Message> {
    label: Cow<'a, str>,
    value: Option<Cow<'a, str>>,
    tone: Option<ToneRole>,
    reserve_indicator: bool,
    leading: Option<Element<'a, Message>>,
    trailing: Option<Element<'a, Message>>,
    size: ControlSize,
    width: Option<Length>,
}

impl<'a, Message> DataRow<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            value: None,
            tone: None,
            reserve_indicator: false,
            leading: None,
            trailing: None,
            size: ControlSize::Sm,
            width: None,
        }
    }

    /// Adds secondary text clustered beside the principal label.
    pub fn value(mut self, value: impl Into<Cow<'a, str>>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Adds a status dot when the visible label/value cluster states the status in text.
    pub fn tone(mut self, tone: ToneRole) -> Self {
        self.tone = Some(tone);
        self
    }

    /// Reserves the same status slot used by toned sibling rows.
    pub fn reserve_indicator(mut self) -> Self {
        self.reserve_indicator = true;
        self
    }

    pub fn neutral(self) -> Self {
        self.tone(ToneRole::Neutral)
    }
    pub fn accent(self) -> Self {
        self.tone(ToneRole::Accent)
    }
    pub fn info(self) -> Self {
        self.tone(ToneRole::Info)
    }
    pub fn success(self) -> Self {
        self.tone(ToneRole::Success)
    }
    pub fn warning(self) -> Self {
        self.tone(ToneRole::Warning)
    }
    pub fn danger(self) -> Self {
        self.tone(ToneRole::Danger)
    }

    pub fn leading(mut self, leading: impl Into<Element<'a, Message>>) -> Self {
        self.leading = Some(leading.into());
        self
    }

    pub fn trailing(mut self, trailing: impl Into<Element<'a, Message>>) -> Self {
        self.trailing = Some(trailing.into());
        self
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

    fn into_element(self) -> Element<'a, Message> {
        let metrics = theme_metadata::metrics(self.size);
        let cluster_has_text = !self.label.trim().is_empty()
            || self
                .value
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty());
        let has_indicator_slot = self.reserve_indicator || self.tone.is_some();
        let indicator: Element<'a, Message> = self.tone.filter(|_| cluster_has_text).map_or_else(
            || {
                if has_indicator_slot {
                    let diameter = crate::widgets::primitives::tone_dot::dot_size(self.size);
                    iced::widget::Space::new()
                        .width(Length::Fixed(diameter))
                        .height(Length::Fixed(diameter))
                        .into()
                } else {
                    iced::widget::Space::new().into()
                }
            },
            |tone| ToneDot::new(tone).size(self.size).into(),
        );
        let leading_present = self.leading.is_some();
        let trailing_present = self.trailing.is_some();
        let secondary_present = self.value.is_some();
        let leading = self
            .leading
            .unwrap_or_else(|| iced::widget::Space::new().into());
        let trailing = self
            .trailing
            .unwrap_or_else(|| iced::widget::Space::new().into());
        let principal_text = self.label;
        let secondary_text = self.value.unwrap_or_default();
        let principal = MeasuredText::new(
            principal_text.clone(),
            EllipsisStrategy::End,
            TypographyRole::Body,
            TextRole::Primary,
        )
        .into();
        let secondary = MeasuredText::new(
            secondary_text.clone(),
            EllipsisStrategy::End,
            TypographyRole::Body,
            TextRole::Secondary,
        )
        .into();

        DataRowLayout {
            children: [indicator, leading, principal, secondary, trailing],
            principal_text,
            secondary_text,
            has_indicator_slot,
            leading_present,
            secondary_present,
            trailing_present,
            indicator_width: crate::widgets::primitives::tone_dot::dot_size(self.size),
            gap: metrics.slot_gap,
            minimum_height: metrics.minimum_height,
            width: self.width.unwrap_or(Length::Shrink),
        }
        .into()
    }
}

impl<'a, Message> From<DataRow<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(row: DataRow<'a, Message>) -> Self {
        row.into_element()
    }
}

struct DataRowLayout<'a, Message> {
    children: [Element<'a, Message>; 5],
    principal_text: Cow<'a, str>,
    secondary_text: Cow<'a, str>,
    has_indicator_slot: bool,
    leading_present: bool,
    secondary_present: bool,
    trailing_present: bool,
    indicator_width: f32,
    gap: f32,
    minimum_height: f32,
    width: Length,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TextAllocation {
    principal: f32,
    secondary: f32,
    gap: f32,
}

fn allocate_text(
    remaining: f32,
    principal: f32,
    secondary: Option<f32>,
    gap: f32,
    ellipsis: f32,
) -> TextAllocation {
    let remaining = remaining.max(0.0);
    let Some(secondary) = secondary else {
        return TextAllocation {
            principal: principal.min(remaining),
            secondary: 0.0,
            gap: 0.0,
        };
    };
    let gap = gap.min(remaining);
    if principal + gap + secondary <= remaining {
        TextAllocation {
            principal,
            secondary,
            gap,
        }
    } else if principal + gap + ellipsis <= remaining {
        TextAllocation {
            principal,
            secondary: remaining - principal - gap,
            gap,
        }
    } else if gap + ellipsis * 2.0 <= remaining {
        TextAllocation {
            principal: remaining - gap - ellipsis,
            secondary: ellipsis,
            gap,
        }
    } else {
        TextAllocation {
            principal: remaining,
            secondary: 0.0,
            gap: 0.0,
        }
    }
}

fn protected(length: Length) -> bool {
    matches!(length, Length::Shrink | Length::Fixed(_))
}

fn child_node<Message>(
    child: &mut Element<'_, Message>,
    tree: &mut Tree,
    renderer: &iced::Renderer,
    width: f32,
) -> layout::Node {
    child.as_widget_mut().layout(
        tree,
        renderer,
        &layout::Limits::new(Size::ZERO, Size::new(width.max(0.0), f32::INFINITY))
            .width(Length::Fixed(width.max(0.0))),
    )
}

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for DataRowLayout<'a, Message>
where
    Message: Clone + 'a,
{
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(
            &self
                .children
                .iter()
                .map(Element::as_widget)
                .collect::<Vec<_>>(),
        );
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let maximum = if limits.max().width.is_finite() {
            limits.max().width.max(0.0)
        } else {
            10_000.0
        };
        let principal_complete = measure_width(
            renderer,
            &self.principal_text,
            crate::theme::typography(TypographyRole::Body),
        );
        let secondary_complete = self.secondary_present.then(|| {
            measure_width(
                renderer,
                &self.secondary_text,
                crate::theme::typography(TypographyRole::Body),
            )
        });
        let ellipsis = measure_width(
            renderer,
            "…",
            crate::theme::typography(TypographyRole::Body),
        );

        let leading_length = self.children[1].as_widget().size().width;
        let trailing_length = self.children[4].as_widget().size().width;
        let measure_slot = |child: &mut Element<'_, Message>, tree: &mut Tree, cap: f32| {
            child
                .as_widget_mut()
                .layout(
                    tree,
                    renderer,
                    &layout::Limits::new(Size::ZERO, Size::new(cap, f32::INFINITY)),
                )
                .size()
                .width
                .min(cap)
        };
        let mut cap = maximum;
        let trailing_protected = if self.trailing_present && protected(trailing_length) {
            let value = measure_slot(&mut self.children[4], &mut tree.children[4], cap);
            cap = (cap - value).max(0.0);
            value
        } else {
            0.0
        };
        let leading_protected = if self.leading_present && protected(leading_length) {
            measure_slot(&mut self.children[1], &mut tree.children[1], cap)
        } else {
            0.0
        };
        let gap_count = usize::from(self.has_indicator_slot)
            + usize::from(self.leading_present)
            + usize::from(self.trailing_present)
            + usize::from(self.secondary_present);
        let intrinsic = self.indicator_width * f32::from(self.has_indicator_slot)
            + self.gap * gap_count as f32
            + leading_protected
            + trailing_protected
            + principal_complete
            + secondary_complete.unwrap_or(0.0);
        let width = limits
            .resolve(
                self.width,
                Length::Shrink,
                Size::new(intrinsic, self.minimum_height),
            )
            .width;

        let mut remaining = width;
        let indicator_width = if self.has_indicator_slot {
            self.indicator_width.min(remaining)
        } else {
            0.0
        };
        remaining -= indicator_width;
        let (indicator_gap, leading_gap, trailing_gap) = {
            let mut take_gap = |present: bool| {
                let value = if present {
                    self.gap.min(remaining)
                } else {
                    0.0
                };
                remaining -= value;
                value
            };
            (
                take_gap(self.has_indicator_slot),
                take_gap(self.leading_present),
                take_gap(self.trailing_present),
            )
        };
        let trailing_width = trailing_protected.min(remaining);
        remaining -= trailing_width;
        let leading_width = leading_protected.min(remaining);
        remaining -= leading_width;

        let complete_cluster =
            principal_complete + secondary_complete.map_or(0.0, |secondary| self.gap + secondary);
        let surplus = (remaining - complete_cluster).max(0.0);
        let leading_factor = if protected(leading_length) {
            0
        } else {
            leading_length.fill_factor()
        };
        let trailing_factor = if protected(trailing_length) {
            0
        } else {
            trailing_length.fill_factor()
        };
        let total_factor = u32::from(leading_factor) + u32::from(trailing_factor);
        let flexible_width = |factor: u16| {
            if total_factor == 0 {
                0.0
            } else {
                surplus * f32::from(factor) / total_factor as f32
            }
        };
        let leading_width = leading_width + flexible_width(leading_factor);
        let trailing_width = trailing_width + flexible_width(trailing_factor);
        let text_remaining =
            (remaining - flexible_width(leading_factor) - flexible_width(trailing_factor)).max(0.0);
        let text = allocate_text(
            text_remaining,
            principal_complete,
            secondary_complete,
            self.gap,
            ellipsis,
        );

        let nodes = [
            child_node(
                &mut self.children[0],
                &mut tree.children[0],
                renderer,
                indicator_width,
            ),
            child_node(
                &mut self.children[1],
                &mut tree.children[1],
                renderer,
                leading_width,
            ),
            child_node(
                &mut self.children[2],
                &mut tree.children[2],
                renderer,
                text.principal,
            ),
            child_node(
                &mut self.children[3],
                &mut tree.children[3],
                renderer,
                text.secondary,
            ),
            child_node(
                &mut self.children[4],
                &mut tree.children[4],
                renderer,
                trailing_width,
            ),
        ];
        let height = nodes.iter().fold(self.minimum_height, |height, node| {
            height.max(node.size().height)
        });
        let mut x = 0.0;
        let mut place = |node: layout::Node, gap_after: f32| {
            let y = (height - node.size().height) / 2.0;
            let placed = node.move_to(Point::new(x, y));
            x += placed.size().width + gap_after;
            placed
        };
        let indicator = place(nodes[0].clone(), indicator_gap);
        let leading = place(nodes[1].clone(), leading_gap);
        let principal = place(nodes[2].clone(), text.gap);
        let secondary = place(nodes[3].clone(), 0.0);
        let trailing_x = (width - trailing_width).max(x + trailing_gap);
        let trailing = nodes[4].clone().move_to(Point::new(
            trailing_x.min(width),
            (height - nodes[4].size().height) / 2.0,
        ));
        layout::Node::with_children(
            Size::new(width, height),
            vec![indicator, leading, principal, secondary, trailing],
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        for ((child, tree), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child
                .as_widget_mut()
                .operate(tree, layout, renderer, operation);
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
        for ((child, tree), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
            if shell.is_event_captured() {
                break;
            }
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
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let clip = layout.bounds().intersection(viewport).unwrap_or_default();
        renderer.with_layer(clip, |renderer| {
            for ((child, tree), layout) in self
                .children
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
            {
                child
                    .as_widget()
                    .draw(tree, renderer, theme, style, layout, cursor, &clip);
            }
        });
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        let children = self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|((child, tree), layout)| {
                child
                    .as_widget_mut()
                    .overlay(tree, layout, renderer, viewport, translation)
            })
            .collect::<Vec<_>>();
        (!children.is_empty()).then(|| overlay::Group::with_children(children).overlay())
    }
}

impl<'a, Message> From<DataRowLayout<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(row: DataRowLayout<'a, Message>) -> Self {
        Element::new(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::{mouse, widget::Space, Event, Size};

    #[test]
    fn allocation_preserves_principal_then_both_markers() {
        assert_eq!(
            allocate_text(80.0, 40.0, Some(60.0), 8.0, 10.0),
            TextAllocation {
                principal: 40.0,
                secondary: 32.0,
                gap: 8.0
            }
        );
        assert_eq!(
            allocate_text(35.0, 40.0, Some(60.0), 8.0, 10.0),
            TextAllocation {
                principal: 17.0,
                secondary: 10.0,
                gap: 8.0
            }
        );
        assert_eq!(
            allocate_text(15.0, 40.0, Some(60.0), 8.0, 10.0),
            TextAllocation {
                principal: 15.0,
                secondary: 0.0,
                gap: 0.0
            }
        );
    }

    #[test]
    fn allocation_handles_complete_exact_and_principal_only_cases() {
        assert_eq!(
            allocate_text(108.0, 40.0, Some(60.0), 8.0, 10.0),
            TextAllocation {
                principal: 40.0,
                secondary: 60.0,
                gap: 8.0,
            }
        );
        assert_eq!(
            allocate_text(24.0, 40.0, None, 8.0, 10.0),
            TextAllocation {
                principal: 24.0,
                secondary: 0.0,
                gap: 0.0,
            }
        );
    }

    #[test]
    fn only_shrink_and_fixed_slots_are_protected() {
        assert!(protected(Length::Shrink));
        assert!(protected(Length::Fixed(40.0)));
        assert!(!protected(Length::Fill));
        assert!(!protected(Length::FillPortion(3)));
    }

    #[test]
    fn real_layout_protects_fixed_trailing_content_and_stays_finite() {
        let node = crate::test_support::layout(
            DataRow::<()>::new("Principal content that must remain visible")
                .value("Secondary metadata that yields first")
                .tone(ToneRole::Warning)
                .trailing(Space::new().width(Length::Fixed(40.0)))
                .fill_width()
                .into(),
            Size::new(140.0, 100.0),
        );
        let children = node.children();
        let trailing = children.last().expect("trailing slot");
        assert_eq!(node.size().width, 140.0);
        assert_eq!(trailing.size().width, 40.0);
        assert!(trailing.bounds().x + trailing.bounds().width <= node.size().width);
        assert!(children
            .iter()
            .all(|child| child.bounds().width.is_finite() && child.bounds().x.is_finite()));
    }

    #[test]
    fn real_layout_keeps_indicator_reservation_and_minimum_height_stable() {
        let reserved = crate::test_support::layout(
            DataRow::<()>::new("Healthy")
                .reserve_indicator()
                .fill_width()
                .into(),
            Size::new(240.0, 100.0),
        );
        let toned = crate::test_support::layout(
            DataRow::<()>::new("Healthy").success().fill_width().into(),
            Size::new(240.0, 100.0),
        );
        assert_eq!(reserved.size(), toned.size());
        assert_eq!(reserved.children()[0].size(), toned.children()[0].size());
        assert!(reserved.size().height >= theme_metadata::metrics(ControlSize::Sm).minimum_height);
    }

    #[test]
    fn arbitrary_leading_and_trailing_events_are_delegated() {
        let messages = crate::test_support::event_messages(
            DataRow::new("Static row")
                .leading(crate::test_support::event_probe(1_u8))
                .trailing(crate::test_support::event_probe(2_u8))
                .fill_width()
                .into(),
            Size::new(240.0, 80.0),
            Event::Mouse(mouse::Event::CursorEntered),
        );
        assert_eq!(messages, vec![1, 2]);
    }
}
