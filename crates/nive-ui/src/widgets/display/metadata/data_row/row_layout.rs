use iced::{
    advanced::{layout, widget::Tree},
    Length, Point, Size,
};

use crate::widgets::display::measured_text::measure_width;
use crate::widgets::display::metadata::data_row::{DataRowLayout, TextAllocation};
use crate::{theme::TypographyRole, Element};

pub(super) fn allocate_text(
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

pub(super) fn protected(length: Length) -> bool {
    matches!(length, Length::Shrink | Length::Fixed(_))
}

pub(super) fn child_node<Message>(
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

impl<'a, Message> DataRowLayout<'a, Message> {
    pub(super) fn layout_impl(
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

        self.place_children(
            tree,
            renderer,
            width,
            indicator_width,
            leading_width,
            trailing_width,
            text,
            (indicator_gap, leading_gap, trailing_gap),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn place_children(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        width: f32,
        indicator_width: f32,
        leading_width: f32,
        trailing_width: f32,
        text: TextAllocation,
        gaps: (f32, f32, f32),
    ) -> layout::Node {
        let (indicator_gap, leading_gap, trailing_gap) = gaps;
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
}
