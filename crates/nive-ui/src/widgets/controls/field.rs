mod label_focus;
mod layout;

use std::borrow::Cow;

use iced::{
    widget::{column, container, row, text, Id, Space},
    Alignment, Border, Length, Shadow,
};

use crate::theme::{
    self, text as theme_text, ControlSize, FieldValidation, SpaceStep, TextRole, ToneRole,
    TypographyRole,
};
use crate::widgets::controls::{Input, InputGroup};
use crate::widgets::primitives::{icon, IconRole};
use crate::Element;

use self::label_focus::LabelFocus;
use self::layout::FieldGrid;

#[derive(Debug, Clone, Copy, PartialEq)]
struct FieldMetrics {
    label_to_control_gap: f32,
    control_to_support_gap: f32,
    requirement_gap: f32,
    support_line_height: f32,
}

/// A visible label, typed form control, and shared hint/error support slot.
///
/// Labels and other semantic metadata are retained for a future accessibility
/// bridge. Nive does not currently claim native AccessKit label, description,
/// error, or group relationship emission.
pub struct Field<'a, Message> {
    label: Cow<'a, str>,
    control: FieldControl<'a, Message>,
    hint: Option<Cow<'a, str>>,
    error: Option<Cow<'a, str>>,
    requirement: Option<FieldRequirement<'a>>,
    reserve_support_line: bool,
    size: ControlSize,
    disabled: bool,
    width: Length,
    #[cfg(test)]
    probe_name: Option<&'static str>,
}

/// Opaque typed boundary for controls supported by [`Field`].
///
/// Its variants are private so future controls can be added without making
/// downstream code exhaustively match framework internals.
pub struct FieldControl<'a, Message> {
    kind: FieldControlKind<'a, Message>,
}

enum FieldControlKind<'a, Message> {
    Input(Input<'a, Message>),
    InputGroup(Box<InputGroup<'a, Message>>),
    Custom(Element<'a, Message>),
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
/// Visible, app-localized requirement metadata rendered beside a field label.
pub enum FieldRequirement<'a> {
    /// The app-provided text communicates that the value is required.
    Required(Cow<'a, str>),
    /// The app-provided text communicates that the value is optional.
    Optional(Cow<'a, str>),
}

impl<'a, Message> From<Input<'a, Message>> for FieldControl<'a, Message> {
    fn from(input: Input<'a, Message>) -> Self {
        Self {
            kind: FieldControlKind::Input(input),
        }
    }
}

impl<'a, Message> From<InputGroup<'a, Message>> for FieldControl<'a, Message> {
    fn from(group: InputGroup<'a, Message>) -> Self {
        Self {
            kind: FieldControlKind::InputGroup(Box::new(group)),
        }
    }
}

impl<'a, Message> Field<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(
        label: impl Into<Cow<'a, str>>,
        control: impl Into<FieldControl<'a, Message>>,
    ) -> Self {
        Self {
            label: label.into(),
            control: control.into(),
            hint: None,
            error: None,
            requirement: None,
            reserve_support_line: false,
            size: ControlSize::Sm,
            disabled: false,
            width: Length::Fill,
            #[cfg(test)]
            probe_name: None,
        }
    }

    /// Creates a labelled field around an unsupported arbitrary widget.
    ///
    /// The caller owns focus targeting, size, validation, disabled state, and
    /// semantic association for this deliberately limited escape hatch.
    pub fn custom(
        label: impl Into<Cow<'a, str>>,
        control: impl Into<Element<'a, Message>>,
    ) -> Self {
        Self::new(
            label,
            FieldControl {
                kind: FieldControlKind::Custom(control.into()),
            },
        )
    }

    pub fn hint(mut self, hint: impl Into<Cow<'a, str>>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn error(mut self, error: impl Into<Cow<'a, str>>) -> Self {
        self.error = Some(error.into());
        self
    }

    pub fn requirement(mut self, requirement: FieldRequirement<'a>) -> Self {
        self.requirement = Some(requirement);
        self
    }

    pub fn required(self, label: impl Into<Cow<'a, str>>) -> Self {
        self.requirement(FieldRequirement::Required(label.into()))
    }

    pub fn optional(self, label: impl Into<Cow<'a, str>>) -> Self {
        self.requirement(FieldRequirement::Optional(label.into()))
    }

    pub fn reserve_support_line(mut self, reserve: bool) -> Self {
        self.reserve_support_line = reserve;
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn apply_group_context(mut self, size: ControlSize, disabled: bool) -> Self {
        self.size = size;
        self.disabled |= disabled;
        self
    }

    #[cfg(test)]
    fn probe_name(mut self, name: &'static str) -> Self {
        self.probe_name = Some(name);
        self
    }

    crate::impl_layout_builders!(width_direct, fill_width_direct, shrink_width_direct);

    fn into_element(self) -> Element<'a, Message> {
        let metrics = metrics();
        let error = normalized_error(self.error);
        let validation = if error.is_some() {
            FieldValidation::Invalid
        } else {
            FieldValidation::Valid
        };
        let (control, focus_target) =
            self.control
                .into_element(self.label.clone(), self.size, validation, self.disabled);
        let label = field_label(self.label, self.requirement, metrics.requirement_gap);
        let support = match error {
            Some(error) => Some(FieldError::new(error).into_element()),
            None => self.hint.map(|hint| FieldHint::new(hint).into_element()),
        };
        let support = if self.reserve_support_line {
            Some(support.unwrap_or_else(|| {
                Space::new()
                    .height(Length::Fixed(metrics.support_line_height))
                    .into()
            }))
        } else {
            support
        };
        let mut control_and_support = column![control].width(Length::Fill);
        if let Some(support) = support {
            control_and_support = control_and_support
                .push(support)
                .spacing(metrics.control_to_support_gap);
        }
        let content = column![label, control_and_support]
            .spacing(metrics.label_to_control_gap)
            .width(Length::Fill);

        let content: Element<'a, Message> =
            container(content).style(style).width(self.width).into();

        let field: Element<'a, Message> =
            LabelFocus::new(content, focus_target, self.disabled).into();

        #[cfg(test)]
        if let Some(name) = self.probe_name {
            return crate::test_support::named_probe(name, field);
        }

        field
    }
}

impl<'a, Message> FieldControl<'a, Message>
where
    Message: Clone + 'a,
{
    fn into_element(
        self,
        label: Cow<'a, str>,
        size: ControlSize,
        validation: FieldValidation,
        disabled: bool,
    ) -> (Element<'a, Message>, Option<Id>) {
        match self.kind {
            FieldControlKind::Input(input) => {
                let (input, id) = input.apply_field_context(label, size, validation, disabled);
                (input.into(), Some(id))
            }
            FieldControlKind::InputGroup(group) => {
                let (group, id) = group.apply_field_context(label, size, validation, disabled);
                (group.into(), Some(id))
            }
            FieldControlKind::Custom(control) => (control, None),
        }
    }
}

impl<'a, Message> From<Field<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(field: Field<'a, Message>) -> Self {
        field.into_element()
    }
}

/// A surface-neutral labelled collection of typed [`Field`] values.
///
/// Construction requires a nonempty visible legend. The group propagates its
/// size and disabled state monotonically, while local Field errors remain next
/// to their controls. Description and group error are concise heading rows;
/// the group never paints a Card-like surface or aggregates descendant errors.
/// Native AccessKit group relationships are not emitted yet.
pub struct FieldGroup<'a, Message> {
    legend: Cow<'a, str>,
    fields: Vec<Field<'a, Message>>,
    description: Option<Cow<'a, str>>,
    error: Option<Cow<'a, str>>,
    layout: FieldGroupLayout,
    size: ControlSize,
    disabled: bool,
    width: Length,
}

#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
/// Responsive layout modes for a typed [`FieldGroup`].
pub enum FieldGroupLayout {
    /// Places each field on its own row.
    #[default]
    Vertical,
    /// Uses equal finite tracks while preserving the requested minimum width.
    ///
    /// Invalid minima normalize to 240 logical pixels. An unbounded host falls
    /// back to vertical layout.
    Wrap { min_field_width: f32 },
}

impl<'a, Message> FieldGroup<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(
        legend: impl Into<Cow<'a, str>>,
        fields: impl IntoIterator<Item = Field<'a, Message>>,
    ) -> Self {
        let legend = legend.into();
        assert!(
            !legend.trim().is_empty(),
            "FieldGroup requires a nonempty visible legend"
        );

        Self {
            legend,
            fields: fields.into_iter().collect(),
            description: None,
            error: None,
            layout: FieldGroupLayout::Vertical,
            size: ControlSize::Sm,
            disabled: false,
            width: Length::Fill,
        }
    }

    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn error(mut self, error: impl Into<Cow<'a, str>>) -> Self {
        self.error = Some(error.into());
        self
    }

    pub fn layout(mut self, layout: FieldGroupLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn vertical(self) -> Self {
        self.layout(FieldGroupLayout::Vertical)
    }

    pub fn wrap(self, min_field_width: f32) -> Self {
        self.layout(FieldGroupLayout::Wrap { min_field_width })
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    crate::impl_layout_builders!(width_direct, fill_width_direct, shrink_width_direct);

    fn into_element(self) -> Element<'a, Message> {
        let field_gap = match self.size {
            ControlSize::Xs | ControlSize::Sm => theme::space(SpaceStep::Lg),
            ControlSize::Md | ControlSize::Lg => theme::space(SpaceStep::Xl),
        };
        let fields = self
            .fields
            .into_iter()
            .map(|field| field.apply_group_context(self.size, self.disabled).into())
            .collect();
        let minimum = match self.layout {
            FieldGroupLayout::Vertical => None,
            FieldGroupLayout::Wrap { min_field_width } => Some(sanitize_minimum(min_field_width)),
        };
        let fields: Element<'a, Message> = FieldGrid::new(fields, field_gap, minimum).into();
        let mut heading = column![FieldLabel::new(self.legend)]
            .spacing(theme::space(SpaceStep::Xs))
            .width(Length::Fill);
        if let Some(description) = self.description {
            heading = heading.push(FieldHint::new(description));
        }
        if let Some(error) = normalized_error(self.error) {
            heading = heading.push(FieldError::new(error));
        }

        column![heading, fields]
            .spacing(theme::space(SpaceStep::Lg))
            .width(self.width)
            .into()
    }
}

impl<'a, Message> From<FieldGroup<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(group: FieldGroup<'a, Message>) -> Self {
        group.into_element()
    }
}

pub struct FieldLabel<'a> {
    label: Cow<'a, str>,
}

impl<'a> FieldLabel<'a> {
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
        }
    }

    pub(super) fn into_element<Message>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        text(self.label)
            .font(theme::typography(TypographyRole::BodyStrong).font)
            .size(theme::typography(TypographyRole::BodyStrong).size)
            .line_height(text::LineHeight::Relative(
                theme::typography(TypographyRole::BodyStrong).line_height,
            ))
            .wrapping(text::Wrapping::WordOrGlyph)
            .style(label_style())
            .into()
    }
}

impl<'a, Message> From<FieldLabel<'a>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(label: FieldLabel<'a>) -> Self {
        label.into_element()
    }
}

pub struct FieldHint<'a> {
    hint: Cow<'a, str>,
}

impl<'a> FieldHint<'a> {
    pub fn new(hint: impl Into<Cow<'a, str>>) -> Self {
        Self { hint: hint.into() }
    }

    fn into_element<Message>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        text(self.hint)
            .font(theme::typography(TypographyRole::BodySmall).font)
            .size(theme::typography(TypographyRole::BodySmall).size)
            .line_height(text::LineHeight::Relative(
                theme::typography(TypographyRole::BodySmall).line_height,
            ))
            .wrapping(text::Wrapping::WordOrGlyph)
            .style(hint_style())
            .into()
    }
}

impl<'a, Message> From<FieldHint<'a>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(hint: FieldHint<'a>) -> Self {
        hint.into_element()
    }
}

pub struct FieldError<'a> {
    error: Cow<'a, str>,
}

impl<'a> FieldError<'a> {
    pub fn new(error: impl Into<Cow<'a, str>>) -> Self {
        Self {
            error: error.into(),
        }
    }

    pub(super) fn into_element<Message>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let typography = theme::typography(TypographyRole::BodySmall);
        let message = text(self.error)
            .font(typography.font)
            .size(typography.size)
            .line_height(text::LineHeight::Relative(typography.line_height))
            .wrapping(text::Wrapping::WordOrGlyph)
            .style(error_style())
            .width(Length::Fill);

        row![
            icon::role(IconRole::ValidationError)
                .custom_size(14.0)
                .color(theme::active().tone(ToneRole::Danger).color),
            message
        ]
        .spacing(theme::space(SpaceStep::Xs))
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .into()
    }
}

impl<'a, Message> From<FieldError<'a>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(error: FieldError<'a>) -> Self {
        error.into_element()
    }
}

fn metrics() -> FieldMetrics {
    let support = theme::typography(TypographyRole::BodySmall);
    FieldMetrics {
        label_to_control_gap: theme::space(SpaceStep::Sm),
        control_to_support_gap: theme::space(SpaceStep::Xs),
        requirement_gap: theme::space(SpaceStep::Xs),
        support_line_height: support.size * support.line_height,
    }
}

pub(super) fn normalized_error<'a>(error: Option<Cow<'a, str>>) -> Option<Cow<'a, str>> {
    error.filter(|value| !value.trim().is_empty())
}

fn sanitize_minimum(minimum: f32) -> f32 {
    if minimum.is_finite() && minimum > 0.0 {
        minimum
    } else {
        240.0
    }
}

fn field_label<'a, Message>(
    label: Cow<'a, str>,
    requirement: Option<FieldRequirement<'a>>,
    gap: f32,
) -> Element<'a, Message>
where
    Message: 'a,
{
    let mut content = row![FieldLabel::new(label)]
        .spacing(gap)
        .align_y(Alignment::Center)
        .width(Length::Fill);
    if let Some(requirement) = requirement {
        let requirement = match requirement {
            FieldRequirement::Required(text) | FieldRequirement::Optional(text) => text,
        };
        content = content.push(FieldHint::new(requirement));
    }

    content.wrap().into()
}

fn style(theme: &crate::theme::Theme) -> container::Style {
    container::Style {
        text_color: Some(theme.text(TextRole::Primary).color),
        background: None,
        border: Border::default(),
        shadow: Shadow::default(),
        ..container::Style::default()
    }
}

fn label_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    theme_text::style(TextRole::Primary)
}

fn hint_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    theme_text::style(TextRole::Secondary)
}

fn error_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    theme_text::tone(ToneRole::Danger)
}

#[cfg(test)]
mod field_tests {
    use std::{cell::Cell, rc::Rc};

    use super::*;
    use crate::test_support::{named_probe, WidgetHarness};
    use crate::theme::{Theme, ThemeBuilder, ThemeDensity, ThemeMode};
    use iced::{mouse, Event, Point, Size};

    #[test]
    fn style_uses_primary_text_color() {
        let theme = Theme::Dark;
        let style = style(&theme);

        assert_eq!(style.text_color, Some(theme.text(TextRole::Primary).color));
    }

    #[test]
    fn hint_and_error_styles_use_app_theme() {
        let theme = Theme::Dark;

        assert_eq!(
            hint_style()(&theme).color,
            Some(theme.text(TextRole::Secondary).color)
        );
        assert_eq!(
            error_style()(&theme).color,
            Some(theme.tone(ToneRole::Danger).color)
        );
    }

    #[test]
    fn label_style_uses_app_theme() {
        let theme = Theme::Dark;

        assert_eq!(
            label_style()(&theme).color,
            Some(theme.text(TextRole::Primary).color)
        );
    }

    #[test]
    fn typed_boundary_retains_owned_label_and_private_control_kind() {
        let field = Field::<String>::new(
            String::from("Account name"),
            Input::new("Name", "Ada").on_change(|value| value),
        )
        .required(String::from("Required"));

        assert_eq!(field.label.as_ref(), "Account name");
        assert!(matches!(field.control.kind, FieldControlKind::Input(_)));
        assert!(matches!(
            field.requirement,
            Some(FieldRequirement::Required(ref value)) if value == "Required"
        ));
    }

    #[test]
    fn custom_boundary_is_explicit_and_keeps_canonical_defaults() {
        let field = Field::<()>::custom("Custom", Space::new());

        assert!(matches!(field.control.kind, FieldControlKind::Custom(_)));
        assert_eq!(field.size, ControlSize::Sm);
        assert_eq!(field.width, Length::Fill);
    }

    #[test]
    fn empty_and_whitespace_errors_normalize_to_absence() {
        assert_eq!(normalized_error(None), None);
        assert_eq!(normalized_error(Some(Cow::Borrowed(""))), None);
        assert_eq!(normalized_error(Some(Cow::Borrowed("  \n"))), None);
        assert_eq!(
            normalized_error(Some(Cow::Borrowed("Required"))).as_deref(),
            Some("Required")
        );
    }

    #[test]
    fn reserved_support_adds_one_line_and_error_replaces_hint_without_double_height() {
        let plain: Element<'_, ()> = Field::new("Name", Input::new("Name", "Ada")).into();
        let reserved: Element<'_, ()> = Field::new("Name", Input::new("Name", "Ada"))
            .reserve_support_line(true)
            .into();
        let hint: Element<'_, ()> = Field::new("Name", Input::new("Name", "Ada"))
            .hint("Helpful")
            .reserve_support_line(true)
            .into();
        let error: Element<'_, ()> = Field::new("Name", Input::new("Name", "Ada"))
            .hint("Helpful")
            .error("Required")
            .reserve_support_line(true)
            .into();
        let size = Size::new(320.0, 200.0);
        let plain = WidgetHarness::new(plain, size).bounds().height;
        let reserved = WidgetHarness::new(reserved, size).bounds().height;
        let hint = WidgetHarness::new(hint, size).bounds().height;
        let error = WidgetHarness::new(error, size).bounds().height;

        assert!(reserved > plain);
        assert_eq!(hint, reserved);
        assert_eq!(error, hint);
    }

    #[test]
    fn activating_an_already_focused_label_keeps_focus_without_blur() {
        let id = Id::new("name");
        let field: Element<'_, &'static str> = Field::new(
            "Name",
            Input::new("Name", "Ada")
                .id(id.clone())
                .on_change(|_| "change")
                .on_blur("blur"),
        )
        .into();
        let mut harness = WidgetHarness::new(field, Size::new(320.0, 120.0));
        harness.focus(id);
        harness.set_cursor(Point::new(4.0, 4.0));

        let result = harness.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));

        assert!(result.messages.is_empty());
        assert_eq!(harness.focused_widgets(), 1);
    }

    #[test]
    fn generated_label_target_keeps_the_input_anchor_across_view_rebuilds() {
        fn view() -> Element<'static, &'static str> {
            let content = iced::widget::column![
                Input::new("Search", "")
                    .id(Id::new("sidebar-search"))
                    .on_change(|_| "search"),
                Field::new(
                    "Empty value",
                    Input::new("Enter a value", "").on_change(|_| "changed"),
                )
                .probe_name("empty-field"),
                crate::widgets::button::primary("After")
                    .id(Id::new("after-empty-field"))
                    .on_press("after"),
            ]
            .spacing(12);

            crate::accessibility::FocusRoot::new(iced::widget::scrollable(content)).into()
        }

        let mut harness = WidgetHarness::new(view(), Size::new(400.0, 240.0));
        let field = harness.named_bounds("empty-field").expect("empty field");
        harness.set_cursor(Point::new(field.x + 20.0, field.y + field.height - 10.0));
        harness.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));
        assert_eq!(
            harness
                .managed_focus()
                .entries
                .iter()
                .filter(|entry| entry.active)
                .count(),
            1
        );

        harness.set_cursor(Point::new(380.0, 220.0));
        harness.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));
        assert_eq!(
            harness
                .managed_focus()
                .entries
                .iter()
                .filter(|entry| entry.anchor_only)
                .count(),
            1
        );

        harness.replace(view());
        assert_eq!(
            harness
                .managed_focus()
                .entries
                .iter()
                .filter(|entry| entry.anchor_only)
                .count(),
            1
        );

        harness.focus_next();
        assert_eq!(harness.focused_ids(), [Id::new("after-empty-field")]);
    }

    #[test]
    fn enabled_label_uses_default_cursor_and_still_focuses_its_input() {
        let field: Element<'_, ()> = Field::new(
            "Name",
            Input::new("Name", "Ada")
                .id(Id::new("name"))
                .on_change(|_| ()),
        )
        .into();
        let mut harness = WidgetHarness::new(field, Size::new(320.0, 120.0));
        harness.set_cursor(Point::new(4.0, 4.0));

        assert_eq!(harness.mouse_interaction(), mouse::Interaction::Idle);

        harness.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));

        assert_eq!(harness.focused_widgets(), 1);
    }

    #[test]
    fn activating_a_sibling_label_blurs_once_and_moves_the_single_focus() {
        let first_id = Id::new("first");
        let second_id = Id::new("second");
        let first_visual_focus = Rc::new(Cell::new(false));
        let second_visual_focus = Rc::new(Cell::new(false));
        let fields: Element<'_, &'static str> = iced::widget::column![
            Field::new(
                "First",
                Input::new("First", "One")
                    .id(first_id.clone())
                    .track_focus(Rc::clone(&first_visual_focus))
                    .on_change(|_| "first change")
                    .on_blur("first blur"),
            ),
            named_probe(
                "second-field",
                Field::new(
                    "Second",
                    Input::new("Second", "Two")
                        .id(second_id)
                        .track_focus(Rc::clone(&second_visual_focus))
                        .on_change(|_| "second change")
                        .on_blur("second blur"),
                ),
            ),
        ]
        .spacing(12)
        .into();
        let mut harness = WidgetHarness::new(fields, Size::new(320.0, 240.0));
        harness.focus(first_id);
        let second = harness.named_bounds("second-field").expect("second field");
        harness.set_cursor(Point::new(second.x + 4.0, second.y + 4.0));

        let result = harness.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));

        assert_eq!(result.messages, vec!["first blur"]);
        assert!(!first_visual_focus.get());
        assert!(second_visual_focus.get());
    }

    #[test]
    fn disabled_typed_field_label_does_not_focus_its_input() {
        let field: Element<'_, ()> = Field::new(
            "Name",
            Input::new("Name", "Ada")
                .id(Id::new("disabled-name"))
                .on_change(|_| ()),
        )
        .disabled(true)
        .into();
        let mut harness = WidgetHarness::new(field, Size::new(320.0, 120.0));
        harness.set_cursor(Point::new(4.0, 4.0));

        harness.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));

        assert_eq!(harness.focused_widgets(), 0);
    }

    #[test]
    #[should_panic(expected = "nonempty visible legend")]
    fn field_group_rejects_an_empty_visible_legend() {
        let _ = FieldGroup::<()>::new("  ", Vec::new());
    }

    #[test]
    fn field_group_api_retains_heading_layout_and_context() {
        let group = FieldGroup::<()>::new(
            String::from("Profile"),
            [Field::new("Name", Input::new("Name", "Ada")).error("Local error")],
        )
        .description(String::from("Account identity"))
        .error(String::from("Review this group"))
        .wrap(f32::NAN)
        .lg()
        .disabled(true);

        assert_eq!(group.legend.as_ref(), "Profile");
        assert_eq!(group.description.as_deref(), Some("Account identity"));
        assert_eq!(group.error.as_deref(), Some("Review this group"));
        assert_eq!(group.fields[0].error.as_deref(), Some("Local error"));
        assert_eq!(group.size, ControlSize::Lg);
        assert!(group.disabled);
        assert!(matches!(
            group.layout,
            FieldGroupLayout::Wrap { min_field_width } if min_field_width.is_nan()
        ));
        assert_eq!(sanitize_minimum(f32::NAN), 240.0);
        assert_eq!(sanitize_minimum(0.0), 240.0);
        assert_eq!(sanitize_minimum(180.0), 180.0);
    }

    #[test]
    fn wrap_uses_the_exact_finite_column_formula_and_narrow_fallback() {
        fn field<'a>(name: &'static str) -> Field<'a, ()> {
            Field::custom(
                name,
                named_probe(
                    name,
                    container(Space::new())
                        .width(Length::Fill)
                        .height(Length::Fixed(12.0)),
                ),
            )
        }

        let group: Element<'_, ()> = FieldGroup::new("Profile", [field("first"), field("second")])
            .wrap(240.0)
            .into();
        let mut harness = WidgetHarness::new(group, Size::new(500.0, 240.0));
        let first = harness.named_bounds("first").expect("first");
        let second = harness.named_bounds("second").expect("second");

        assert_eq!(first.width, 244.0);
        assert_eq!(second.width, 244.0);
        assert_eq!(second.x - first.x, 256.0);

        harness.relayout(Size::new(200.0, 300.0));
        let first = harness.named_bounds("first").expect("narrow first");
        let second = harness.named_bounds("second").expect("narrow second");
        assert_eq!(first.width, 200.0);
        assert_eq!(second.width, 200.0);
        assert_eq!(second.x, first.x);
        assert!(second.y > first.y);
    }

    #[test]
    fn typed_context_retains_label_metadata_and_propagates_size_validation_and_disabled() {
        let (input, _) = Input::<()>::new("Name", "Ada").apply_field_context(
            Cow::Owned(String::from("Account name")),
            ControlSize::Lg,
            FieldValidation::Invalid,
            true,
        );

        assert_eq!(input.semantic_name_value(), Some("Account name"));
        assert_eq!(input.control_size(), ControlSize::Lg);
        assert_eq!(input.field_validation(), FieldValidation::Invalid);
        assert!(input.is_disabled());

        let optional = Field::<()>::new("Reference", Input::new("Reference", "A-1"))
            .optional(String::from("Optional"));
        assert!(matches!(
            optional.requirement,
            Some(FieldRequirement::Optional(ref value)) if value == "Optional"
        ));
    }

    #[test]
    fn multiline_support_grows_beyond_the_reserved_minimum() {
        let short: Element<'_, ()> = Field::new("Name", Input::new("Name", "Ada"))
            .error("Required")
            .reserve_support_line(true)
            .into();
        let long: Element<'_, ()> = Field::new("Name", Input::new("Name", "Ada"))
            .error("This long validation message must wrap by word or glyph inside a narrow field")
            .reserve_support_line(true)
            .into();
        let short = WidgetHarness::new(short, Size::new(150.0, 300.0));
        let long = WidgetHarness::new(long, Size::new(150.0, 300.0));

        assert!(long.bounds().height > short.bounds().height);
    }

    #[test]
    fn custom_field_label_does_not_claim_or_create_a_focus_target() {
        let field: Element<'_, ()> = Field::custom("Custom", Space::new()).into();
        let mut harness = WidgetHarness::new(field, Size::new(200.0, 100.0));
        harness.set_cursor(Point::new(4.0, 4.0));
        harness.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));

        assert_eq!(harness.focused_widgets(), 0);
    }

    #[test]
    fn anatomy_gaps_follow_every_active_density_step() {
        for density in ThemeDensity::ALL {
            let theme = ThemeBuilder::new("Field density", ThemeMode::Light)
                .density(density)
                .build();
            let _guard = crate::theme::testing::ThemeTestGuard::activate(theme);
            let field: Element<'_, ()> = Field::custom(
                "Label",
                named_probe(
                    "control",
                    container(Space::new())
                        .width(Length::Fill)
                        .height(Length::Fixed(10.0)),
                ),
            )
            .into();
            let mut harness = WidgetHarness::new(field, Size::new(240.0, 120.0));
            let control = harness.named_bounds("control").expect("control");
            let label_line = theme.typography(TypographyRole::BodyStrong);

            assert_eq!(
                control.y,
                label_line.size * label_line.line_height + theme.space(SpaceStep::Sm)
            );
            assert_eq!(metrics().requirement_gap, theme.space(SpaceStep::Xs));
            assert_eq!(metrics().control_to_support_gap, theme.space(SpaceStep::Xs));
        }
    }

    #[test]
    fn field_group_wrap_crosses_every_exact_standard_threshold() {
        fn probed(name: &'static str) -> Field<'static, ()> {
            Field::custom(name, Space::new()).probe_name(name)
        }

        for (width, expected_columns) in [(491.0, 1), (492.0, 2), (743.0, 2), (744.0, 3)] {
            let group: Element<'_, ()> = FieldGroup::new(
                "Thresholds",
                [probed("one"), probed("two"), probed("three")],
            )
            .wrap(240.0)
            .into();
            let mut harness = WidgetHarness::new(group, Size::new(width, 400.0));
            let one = harness.named_bounds("one").expect("one");
            let two = harness.named_bounds("two").expect("two");
            let three = harness.named_bounds("three").expect("three");
            let first_row = [one, two, three]
                .into_iter()
                .filter(|bounds| bounds.y == one.y)
                .count();

            assert_eq!(first_row, expected_columns);
        }
    }

    #[test]
    fn field_group_vertical_gap_tracks_size_and_density_without_own_surface() {
        for density in ThemeDensity::ALL {
            let theme = ThemeBuilder::new("Group density", ThemeMode::Light)
                .density(density)
                .build();
            let _guard = crate::theme::testing::ThemeTestGuard::activate(theme);

            for (size, expected_step) in [
                (ControlSize::Sm, SpaceStep::Lg),
                (ControlSize::Md, SpaceStep::Xl),
            ] {
                let group: Element<'_, ()> = FieldGroup::new(
                    "Vertical",
                    [
                        Field::custom("One", Space::new()).probe_name("one"),
                        Field::custom("Two", Space::new()).probe_name("two"),
                    ],
                )
                .size(size)
                .into();
                let mut harness = WidgetHarness::new(group, Size::new(320.0, 300.0));
                let one = harness.named_bounds("one").expect("one");
                let two = harness.named_bounds("two").expect("two");

                assert_eq!(two.y - one.y - one.height, theme.space(expected_step));
                assert_eq!(one.x, 0.0);
                assert_eq!(one.width, 320.0);
            }
        }
    }

    #[test]
    fn typed_input_group_receives_field_size_and_label_focus() {
        let id = Id::new("group-input");
        let field: Element<'_, ()> = Field::new(
            "Amount",
            InputGroup::new(Input::new("Amount", "42").id(id.clone()).on_change(|_| ()))
                .prefix("USD"),
        )
        .lg()
        .into();
        let mut harness = WidgetHarness::new(field, Size::new(320.0, 120.0));
        let input = harness.focusable_bounds(&id).expect("typed group input");
        assert_eq!(
            input.height,
            theme::form_control_metrics(ControlSize::Lg).height
        );

        harness.set_cursor(Point::new(4.0, 4.0));
        harness.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));
        assert_eq!(harness.focused_widgets(), 1);
    }

    #[test]
    fn unbounded_wrap_falls_back_to_vertical_order() {
        let children = vec![
            named_probe(
                "first-unbounded",
                Space::new()
                    .width(Length::Fixed(80.0))
                    .height(Length::Fixed(10.0)),
            ),
            named_probe(
                "second-unbounded",
                Space::new()
                    .width(Length::Fixed(100.0))
                    .height(Length::Fixed(10.0)),
            ),
        ];
        let grid: Element<'_, ()> = FieldGrid::new(children, 12.0, Some(240.0)).into();
        let mut harness = WidgetHarness::new(grid, Size::INFINITE);
        let first = harness
            .named_bounds("first-unbounded")
            .expect("first unbounded");
        let second = harness
            .named_bounds("second-unbounded")
            .expect("second unbounded");

        assert_eq!(first.x, second.x);
        assert_eq!(second.y - first.y - first.height, 12.0);
    }

    #[test]
    fn group_context_is_monotonic_and_preserves_local_error() {
        let field = Field::<()>::new("Name", Input::new("Name", "Ada"))
            .error("Local error")
            .disabled(true)
            .apply_group_context(ControlSize::Lg, false);

        assert_eq!(field.size, ControlSize::Lg);
        assert!(field.disabled);
        assert_eq!(field.error.as_deref(), Some("Local error"));
    }
}
