mod style;

use std::borrow::Cow;

use iced::{
    widget::{container, Row},
    Alignment, Length, Padding,
};

use crate::advanced::control_group::{position_for_index, radius_for_position, SlotPosition};
use crate::theme::{ControlSize, FieldValidation, TextRole, TypographyRole};
use crate::widgets::controls::button::{Button, GroupedItemKind, GroupedItemSpec};
use crate::widgets::controls::form_frame::{FormControlFrame, FormFrameAppearance};
use crate::widgets::controls::input::Input;
use crate::widgets::feedback::Spinner;
use crate::widgets::primitives::{icon as icon_widget, IconRole, StatusIndicator, ToneDot};
use crate::Element;

use self::style as theme_input_group;

pub use style::InputGroupVariant;

/// One framed input with typed intrinsic adornments and actions.
///
/// The group inherits its Input size at construction; a later group size is
/// authoritative. Prefixes and semantic icons are secondary content, units
/// are auxiliary, and clear/activity share one stable square. Input focus
/// identifies the complete frame while an action retains local focus.
///
/// Arbitrary leading/trailing slots are rectangular escape hatches. Their
/// internal paint, rounded masking, semantics, and state propagation remain
/// caller-owned.
pub struct InputGroup<'a, Message> {
    input: Input<'a, Message>,
    leading_slots: Vec<InputGroupSlot<'a, Message>>,
    trailing_slots: Vec<InputGroupSlot<'a, Message>>,
    size: ControlSize,
    width: Length,
    variant: InputGroupVariant,
    validation: Option<FieldValidation>,
    disabled: bool,
    clear_action: Option<Button<'a, Message>>,
    activity: bool,
    reserve_activity: bool,
}

enum InputGroupSlot<'a, Message> {
    Prefix(Cow<'a, str>),
    Unit(Cow<'a, str>),
    Icon(IconRole),
    Status(StatusIndicator<'a>),
    Action(Button<'a, Message>),
    Visual(Element<'a, Message>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputGroupSlotSide {
    Leading,
    Trailing,
}

impl<'a, Message> InputGroup<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(input: Input<'a, Message>) -> Self {
        let size = input.control_size();

        Self {
            input,
            leading_slots: Vec::new(),
            trailing_slots: Vec::new(),
            size,
            width: Length::Fill,
            variant: InputGroupVariant::Default,
            validation: None,
            disabled: false,
            clear_action: None,
            activity: false,
            reserve_activity: false,
        }
    }

    pub fn prefix(self, prefix: impl Into<Cow<'a, str>>) -> Self {
        self.push_slot(
            InputGroupSlotSide::Leading,
            InputGroupSlot::Prefix(prefix.into()),
        )
    }

    pub fn unit(self, unit: impl Into<Cow<'a, str>>) -> Self {
        self.push_slot(
            InputGroupSlotSide::Trailing,
            InputGroupSlot::Unit(unit.into()),
        )
    }

    #[deprecated(note = "use InputGroup::prefix")]
    pub fn leading_text(self, leading: &'a str) -> Self {
        self.prefix(leading)
    }

    #[deprecated(note = "use InputGroup::unit")]
    pub fn trailing_text(self, trailing: &'a str) -> Self {
        self.unit(trailing)
    }

    pub fn leading_icon(self, leading: IconRole) -> Self {
        self.push_slot(InputGroupSlotSide::Leading, InputGroupSlot::Icon(leading))
    }

    pub fn semantic_icon(self, icon: IconRole) -> Self {
        self.leading_icon(icon)
    }

    pub fn leading_action(self, leading: Button<'a, Message>) -> Self {
        self.push_slot(InputGroupSlotSide::Leading, InputGroupSlot::Action(leading))
    }

    /// Adds an arbitrary rectangular leading slot.
    ///
    /// Rounded clipping, internal paint, semantics, and state propagation are
    /// owned by the caller for this escape hatch.
    pub fn leading_slot(self, leading: impl Into<Element<'a, Message>>) -> Self {
        self.push_slot(
            InputGroupSlotSide::Leading,
            InputGroupSlot::Visual(leading.into()),
        )
    }

    pub fn trailing_icon(self, trailing: IconRole) -> Self {
        self.push_slot(InputGroupSlotSide::Trailing, InputGroupSlot::Icon(trailing))
    }

    pub fn status(self, status: StatusIndicator<'a>) -> Self {
        self.push_slot(InputGroupSlotSide::Trailing, InputGroupSlot::Status(status))
    }

    pub fn trailing_action(self, trailing: Button<'a, Message>) -> Self {
        self.push_slot(
            InputGroupSlotSide::Trailing,
            InputGroupSlot::Action(trailing),
        )
    }

    /// Adds an arbitrary rectangular trailing slot.
    ///
    /// Rounded clipping, internal paint, semantics, and state propagation are
    /// owned by the caller for this escape hatch.
    pub fn trailing_slot(self, trailing: impl Into<Element<'a, Message>>) -> Self {
        self.push_slot(
            InputGroupSlotSide::Trailing,
            InputGroupSlot::Visual(trailing.into()),
        )
    }

    pub fn clear_action(mut self, action: Button<'a, Message>) -> Self {
        self.clear_action = Some(action);
        self.reserve_activity = true;
        self
    }

    pub fn activity(mut self, active: bool) -> Self {
        self.activity = active;
        self.reserve_activity = true;
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

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    crate::impl_layout_builders!(width_direct, fill_width_direct, shrink_width_direct);

    pub fn variant(mut self, variant: InputGroupVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn ghost(self) -> Self {
        self.variant(InputGroupVariant::Ghost)
    }

    pub fn validation(mut self, validation: FieldValidation) -> Self {
        self.validation = Some(validation);
        self
    }

    pub fn invalid(self, invalid: bool) -> Self {
        self.validation(if invalid {
            FieldValidation::Invalid
        } else {
            FieldValidation::Valid
        })
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub(crate) fn apply_field_context(
        mut self,
        label: Cow<'a, str>,
        size: ControlSize,
        validation: FieldValidation,
        disabled: bool,
    ) -> (Self, iced::widget::Id) {
        let (input, id) =
            self.input
                .apply_field_context(label, size, validation, disabled || self.disabled);
        self.input = input;
        self.size = size;
        self.validation = Some(validation);
        self.disabled |= disabled;
        (self, id)
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = theme_input_group::metrics(self.size);
        let effective_disabled = self.disabled || self.input.is_disabled();
        let validation = self
            .validation
            .unwrap_or_else(|| self.input.field_validation());
        let has_activity_cell = self.reserve_activity || self.clear_action.is_some();
        let slot_count = self.leading_slots.len()
            + 1
            + self.trailing_slots.len()
            + usize::from(has_activity_cell);
        let fill_input = !matches!(self.width, Length::Shrink);
        let mut row = Row::new()
            .spacing(0)
            .align_y(Alignment::Center)
            .height(Length::Fill);
        let mut index = 0;

        for slot in self.leading_slots {
            row = row.push(slot.into_element(
                self.size,
                effective_disabled,
                metrics,
                position_for_index(index, slot_count),
            ));
            index += 1;
        }

        let input_radius =
            radius_for_position(position_for_index(index, slot_count), metrics.radius);
        row = row.push(
            self.input
                .size(self.size)
                .validation(validation)
                .disabled(effective_disabled)
                .into_group_element(
                    fill_input,
                    input_radius,
                    metrics.font_size,
                    metrics.input_padding_v,
                    metrics.input_padding_h,
                ),
        );
        index += 1;

        for slot in self.trailing_slots {
            row = row.push(slot.into_element(
                self.size,
                effective_disabled,
                metrics,
                position_for_index(index, slot_count),
            ));
            index += 1;
        }

        if has_activity_cell {
            let radius = radius_for_position(position_for_index(index, slot_count), metrics.radius);
            let cell: Element<'a, Message> = if self.activity {
                container(Spinner::new().size(self.size))
                    .width(Length::Fixed(metrics.height))
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .into()
            } else if let Some(action) = self.clear_action {
                action
                    .disabled(effective_disabled)
                    .into_grouped_item_inset(GroupedItemSpec {
                        size: self.size,
                        radius,
                        height: metrics.height,
                        padding_h: 0.0,
                        selected: false,
                        destructive: false,
                        kind: GroupedItemKind::Embedded,
                    })
            } else {
                iced::widget::Space::new()
                    .width(Length::Fixed(metrics.height))
                    .height(Length::Fill)
                    .into()
            };
            row = row.push(cell);
        }

        let group = container(row)
            .height(Length::Fixed(metrics.height))
            .width(self.width);

        Element::new(FormControlFrame {
            content: group.into(),
            appearance: match self.variant {
                InputGroupVariant::Default => FormFrameAppearance::Default,
                InputGroupVariant::Ghost => FormFrameAppearance::Ghost,
            },
            validation,
            metrics: crate::theme::form_control_metrics(self.size),
            disabled: effective_disabled,
            interactive: true,
        })
    }

    fn push_slot(mut self, side: InputGroupSlotSide, slot: InputGroupSlot<'a, Message>) -> Self {
        match side {
            InputGroupSlotSide::Leading => self.leading_slots.push(slot),
            InputGroupSlotSide::Trailing => self.trailing_slots.push(slot),
        }

        self
    }
}

impl<'a, Message> InputGroupSlot<'a, Message>
where
    Message: Clone + 'a,
{
    fn into_element(
        self,
        size: ControlSize,
        disabled: bool,
        metrics: theme_input_group::InputGroupMetrics,
        position: SlotPosition,
    ) -> Element<'a, Message> {
        let radius = radius_for_position(position, metrics.radius);

        match self {
            InputGroupSlot::Prefix(label) => slot_container(
                form_text(label, TextRole::Secondary),
                radius,
                disabled,
                TextRole::Secondary,
                metrics,
            ),
            InputGroupSlot::Unit(label) => slot_container(
                form_text(label, TextRole::Muted),
                radius,
                disabled,
                TextRole::Muted,
                metrics,
            ),
            InputGroupSlot::Icon(icon) => slot_container(
                icon_widget::role(icon)
                    .custom_size(metrics.icon_size)
                    .into(),
                radius,
                disabled,
                TextRole::Secondary,
                metrics,
            ),
            InputGroupSlot::Status(status) => {
                let (tone, label) = status.into_parts();
                slot_container(
                    iced::widget::row![
                        ToneDot::new(tone).size(size).disabled(disabled),
                        form_text(label, TextRole::Secondary),
                    ]
                    .spacing(crate::theme::form_control_metrics(size).gap)
                    .align_y(Alignment::Center)
                    .into(),
                    radius,
                    disabled,
                    TextRole::Secondary,
                    metrics,
                )
            }
            InputGroupSlot::Action(action) => {
                action
                    .disabled(disabled)
                    .into_grouped_item_inset(GroupedItemSpec {
                        size,
                        radius,
                        height: metrics.height,
                        padding_h: metrics.slot_padding_h,
                        selected: false,
                        destructive: false,
                        kind: GroupedItemKind::Embedded,
                    })
            }
            InputGroupSlot::Visual(content) => {
                slot_container(content, radius, disabled, TextRole::Muted, metrics)
            }
        }
    }
}

fn slot_container<'a, Message>(
    content: Element<'a, Message>,
    radius: iced::border::Radius,
    disabled: bool,
    text_role: TextRole,
    metrics: theme_input_group::InputGroupMetrics,
) -> Element<'a, Message>
where
    Message: 'a,
{
    container(content)
        .style(theme_input_group::slot_style(radius, disabled, text_role))
        .padding(Padding::ZERO.horizontal(metrics.slot_padding_h))
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into()
}

fn form_text<'a, Message>(label: Cow<'a, str>, role: TextRole) -> Element<'a, Message>
where
    Message: 'a,
{
    let typography = crate::theme::typography(TypographyRole::Control);
    iced::widget::text(label)
        .font(typography.font)
        .size(typography.size)
        .line_height(iced::widget::text::LineHeight::Relative(
            typography.line_height,
        ))
        .style(crate::theme::text::style(role))
        .shaping(iced::widget::text::Shaping::Auto)
        .into()
}

impl<'a, Message> From<InputGroup<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(group: InputGroup<'a, Message>) -> Self {
        group.into_element()
    }
}

#[cfg(test)]
mod input_group_tests {
    use iced::{widget::Id, Event, Length, Point, Size};

    use super::*;
    use crate::test_support::{event_probe, named_probe, WidgetHarness};
    use crate::theme::ToneRole;
    use crate::widgets::controls::button;

    #[test]
    fn group_inherits_input_size_and_later_override_is_authoritative() {
        let inherited = InputGroup::<String>::new(Input::new("Value", "42").lg());
        assert_eq!(inherited.size, ControlSize::Lg);

        let overridden = inherited.xs();
        assert_eq!(overridden.size, ControlSize::Xs);
    }

    #[test]
    fn typed_builders_retain_owned_content_and_group_state() {
        let group = InputGroup::<String>::new(Input::new("Amount", "42"))
            .prefix(String::from("USD"))
            .unit(String::from("per month"))
            .semantic_icon(IconRole::Identity)
            .status(StatusIndicator::new(
                ToneRole::Success,
                String::from("Ready"),
            ))
            .leading_action(button::icon(IconRole::EditFind, "Search").on_press("search".into()))
            .trailing_action(button::icon(IconRole::ViewReveal, "Reveal").on_press("reveal".into()))
            .clear_action(button::icon(IconRole::WindowClose, "Clear").on_press("clear".into()))
            .activity(false)
            .validation(FieldValidation::Invalid)
            .disabled(true);

        assert_eq!(group.leading_slots.len(), 3);
        assert_eq!(group.trailing_slots.len(), 3);
        assert!(group.clear_action.is_some());
        assert_eq!(group.validation, Some(FieldValidation::Invalid));
        assert!(group.disabled);
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_text_aliases_remain_available_for_one_release() {
        let group = InputGroup::<String>::new(Input::new("Amount", "42"))
            .leading_text("USD")
            .trailing_text("kg");

        assert!(matches!(group.leading_slots[0], InputGroupSlot::Prefix(_)));
        assert!(matches!(group.trailing_slots[0], InputGroupSlot::Unit(_)));
    }

    #[test]
    fn inherited_and_overridden_sizes_drive_outer_height() {
        let inherited: Element<'_, String> = InputGroup::new(Input::new("Value", "42").lg()).into();
        let inherited = WidgetHarness::new(inherited, Size::new(320.0, 80.0));
        assert_eq!(inherited.bounds().height, 36.0);

        let overridden: Element<'_, String> =
            InputGroup::new(Input::new("Value", "42").lg()).xs().into();
        let overridden = WidgetHarness::new(overridden, Size::new(320.0, 80.0));
        assert_eq!(overridden.bounds().height, 24.0);
    }

    #[test]
    fn clear_and_activity_share_stable_shrink_geometry() {
        let clear: Element<'_, String> = InputGroup::new(Input::new("Value", "42"))
            .clear_action(button::icon(IconRole::WindowClose, "Clear").on_press("clear".into()))
            .activity(false)
            .shrink_width()
            .into();
        let loading: Element<'_, String> = InputGroup::new(Input::new("Value", "42"))
            .clear_action(button::icon(IconRole::WindowClose, "Clear").on_press("clear".into()))
            .activity(true)
            .shrink_width()
            .into();
        let clear = WidgetHarness::new(clear, Size::new(500.0, 80.0));
        let loading = WidgetHarness::new(loading, Size::new(500.0, 80.0));

        assert_eq!(clear.bounds(), loading.bounds());
        assert_eq!(clear.bounds().height, 28.0);
    }

    #[test]
    fn typed_and_custom_slots_remain_finite_under_bounded_layout() {
        let group: Element<'_, &'static str> = named_probe(
            "group",
            InputGroup::new(Input::new("Amount", "42"))
                .prefix("USD")
                .unit("kg")
                .semantic_icon(IconRole::Identity)
                .status(StatusIndicator::new(ToneRole::Success, "Ready"))
                .trailing_slot(event_probe("custom"))
                .width(Length::Fixed(240.0)),
        );
        let mut harness = WidgetHarness::new(group, Size::new(320.0, 80.0));
        let bounds = harness.named_bounds("group").expect("group bounds");

        assert_eq!(bounds.width, 240.0);
        assert_eq!(bounds.height, 28.0);
        assert!(bounds.width.is_finite());
        assert_eq!(
            harness
                .update(Event::Mouse(iced::mouse::Event::CursorEntered))
                .messages,
            vec!["custom"]
        );
    }

    #[test]
    fn bounded_layout_allocates_exact_width_after_protected_slots_and_reflows() {
        let input_id = Id::new("amount-input");
        let group: Element<'_, String> = InputGroup::new(
            Input::new("Amount", "42")
                .id(input_id.clone())
                .on_change(|value| value),
        )
        .leading_slot(
            iced::widget::Space::new()
                .width(Length::Fixed(30.0))
                .height(Length::Fill),
        )
        .trailing_slot(
            iced::widget::Space::new()
                .width(Length::Fixed(40.0))
                .height(Length::Fill),
        )
        .width(Length::Fixed(240.0))
        .into();
        let metrics = theme_input_group::metrics(ControlSize::Sm);
        let protected = 30.0 + 40.0 + 4.0 * metrics.slot_padding_h;
        let mut harness = WidgetHarness::new(group, Size::new(240.0, 80.0));
        let wide = harness
            .focusable_bounds(&input_id)
            .expect("wide input bounds");
        assert_eq!(wide.width, 240.0 - protected);

        harness.relayout(Size::new(96.0, 80.0));
        let narrow = harness
            .focusable_bounds(&input_id)
            .expect("narrow input bounds");
        assert_eq!(narrow.width, (96.0 - protected).max(0.0));

        harness.relayout(Size::new(240.0, 80.0));
        assert_eq!(
            harness
                .focusable_bounds(&input_id)
                .expect("wide input bounds again"),
            wide
        );
    }

    #[test]
    fn width_modes_resolve_finitely_for_bounded_and_unbounded_hosts() {
        for (width, expected) in [
            (Length::Fill, 300.0),
            (Length::FillPortion(2), 300.0),
            (Length::Fixed(180.0), 180.0),
        ] {
            let group: Element<'_, ()> = InputGroup::new(Input::new("Value", "42"))
                .width(width)
                .into();
            let harness = WidgetHarness::new(group, Size::new(300.0, 80.0));
            assert_eq!(harness.bounds().width, expected);
            assert!(harness.bounds().width.is_finite());
        }

        let shrink: Element<'_, ()> = InputGroup::new(Input::new("Value", "42"))
            .prefix("USD")
            .unit("kg")
            .shrink_width()
            .into();
        let bounded = WidgetHarness::new(shrink, Size::new(300.0, 80.0));
        assert!(bounded.bounds().width.is_finite());
        assert!(bounded.bounds().width < 300.0);

        let unbounded: Element<'_, ()> = InputGroup::new(Input::new("Value", "42"))
            .prefix("USD")
            .unit("kg")
            .shrink_width()
            .into();
        let unbounded = WidgetHarness::new(unbounded, Size::INFINITE);
        assert!(unbounded.bounds().width.is_finite());
    }

    #[test]
    fn focus_order_and_action_routing_follow_visual_slot_order() {
        let leading_id = Id::new("leading-action");
        let input_id = Id::new("group-value");
        let trailing_id = Id::new("trailing-action");
        let group: Element<'_, &'static str> = InputGroup::new(
            Input::new("Value", "42")
                .id(input_id.clone())
                .on_change(|_| "change"),
        )
        .leading_action(
            button::icon(IconRole::EditFind, "Search")
                .id(leading_id.clone())
                .on_press("leading"),
        )
        .trailing_action(
            button::icon(IconRole::ViewReveal, "Reveal")
                .id(trailing_id.clone())
                .on_press("trailing"),
        )
        .into();
        let mut harness = WidgetHarness::new(group, Size::new(280.0, 80.0));

        assert_eq!(
            harness.focusable_ids(),
            vec![leading_id, input_id, trailing_id.clone()]
        );

        let trailing = harness
            .focusable_bounds(&trailing_id)
            .expect("trailing action");
        harness.set_cursor(Point::new(trailing.center_x(), trailing.center_y()));
        assert!(harness
            .update(Event::Mouse(iced::mouse::Event::ButtonPressed(
                iced::mouse::Button::Left,
            )))
            .messages
            .is_empty());
        assert_eq!(
            harness
                .update(Event::Mouse(iced::mouse::Event::ButtonReleased(
                    iced::mouse::Button::Left,
                )))
                .messages,
            vec!["trailing"]
        );
    }

    #[test]
    fn arbitrary_slot_delegates_event_mouse_and_operate_paths() {
        let custom = crate::widgets::overlays::tooltip::immediate_for_test(
            named_probe("custom-slot", event_probe("custom")),
            "Custom slot tooltip",
        );
        let group: Element<'_, &'static str> = InputGroup::new(Input::new("Value", "42"))
            .trailing_slot(custom)
            .width(Length::Fixed(180.0))
            .into();
        let mut harness = WidgetHarness::new(group, Size::new(180.0, 80.0));
        let slot = harness.named_bounds("custom-slot").expect("custom slot");
        harness.set_cursor(Point::new(slot.center_x(), slot.center_y()));

        assert_eq!(
            harness.mouse_interaction(),
            iced::mouse::Interaction::Pointer
        );
        assert_eq!(
            harness
                .update(Event::Mouse(iced::mouse::Event::CursorMoved {
                    position: Point::new(slot.center_x(), slot.center_y()),
                }))
                .messages,
            vec!["custom"]
        );
        assert!(harness.has_overlay());
    }
}
