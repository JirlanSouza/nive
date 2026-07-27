use iced::{
    border::Radius,
    widget::button::Status,
    widget::{button, container, text, Row},
    Alignment, Background, Border, Color, Length, Shadow,
};

use crate::theme::{
    self, ControlRole, ControlSize, ControlState, TextRole, ToneRole, TypographyRole,
};
use crate::Element;

use super::button::ButtonFocusRing;
use crate::advanced::pressable::Pressable;
use crate::widgets::display::measured_text::{EllipsisStrategy, MeasuredText};
use crate::widgets::primitives::{icon, IconRef};
use crate::widgets::{StatusIndicator, ToneDot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectableItemVariant {
    Default,
    Selected,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SelectableItemMetrics {
    font_size: f32,
    padding_v: f32,
    padding_h: f32,
    radius: f32,
    leading_size: f32,
    icon_size: f32,
    gap: f32,
    height: f32,
}

/// Form-compatible selectable list row with app-owned selection state.
///
/// The row defaults to [`ControlSize::Sm`] and fill width. Selection uses a
/// subtle whole-row surface; focus is inset and independent from selection.
pub struct SelectableItem<'a, Message> {
    label: &'a str,
    selected: bool,
    leading_icon: Option<IconRef>,
    leading_color: Option<Color>,
    status_indicator: Option<StatusIndicator<'a>>,
    reserve_status_indicator: bool,
    trailing_text: Option<&'a str>,
    trailing: Option<Element<'a, Message>>,
    size: ControlSize,
    width: Length,
    on_press: Option<Message>,
    disabled: bool,
    tooltip_label: Option<&'a str>,
}

impl<'a, Message> SelectableItem<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            selected: false,
            leading_icon: None,
            leading_color: None,
            status_indicator: None,
            reserve_status_indicator: false,
            trailing_text: None,
            trailing: None,
            size: ControlSize::Sm,
            width: Length::Fill,
            on_press: None,
            disabled: false,
            tooltip_label: None,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn leading_icon(mut self, icon: impl Into<IconRef>) -> Self {
        self.leading_icon = Some(icon.into());
        self
    }

    pub fn leading_color(mut self, color: Color) -> Self {
        self.leading_color = Some(color);
        self
    }

    pub fn status_indicator(mut self, status: StatusIndicator<'a>) -> Self {
        self.status_indicator = Some(status);
        self.reserve_status_indicator = true;
        self
    }

    pub fn status_text(self, tone: ToneRole, label: impl Into<std::borrow::Cow<'a, str>>) -> Self {
        self.status_indicator(StatusIndicator::new(tone, label))
    }

    pub fn reserve_status_indicator(mut self) -> Self {
        self.reserve_status_indicator = true;
        self
    }

    /// Adds operational trailing text that follows the row interaction state.
    ///
    /// Use [`SelectableItem::trailing`] with an explicitly styled element for
    /// semantic status or muted optional metadata.
    pub fn trailing_text(mut self, trailing: &'a str) -> Self {
        self.trailing_text = Some(trailing);
        self
    }

    /// Adds caller-styled trailing content without overriding its semantic tone.
    pub fn trailing(mut self, trailing: impl Into<Element<'a, Message>>) -> Self {
        self.trailing = Some(trailing.into());
        self
    }

    pub fn xs(mut self) -> Self {
        self.size = ControlSize::Xs;
        self
    }

    pub fn sm(mut self) -> Self {
        self.size = ControlSize::Sm;
        self
    }

    pub fn md(mut self) -> Self {
        self.size = ControlSize::Md;
        self
    }

    pub fn lg(mut self) -> Self {
        self.size = ControlSize::Lg;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    crate::impl_layout_builders!(fill_width_direct, shrink_width_direct);

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    pub fn tooltip(mut self, tooltip: &'a str) -> Self {
        self.tooltip_label = Some(tooltip);
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = metrics(self.size);
        let variant = if self.selected {
            SelectableItemVariant::Selected
        } else {
            SelectableItemVariant::Default
        };
        let width = self.width;
        let activation = if self.disabled {
            None
        } else {
            self.on_press.clone()
        };
        let tooltip_label = self.tooltip_label;
        let content = self.content(metrics);

        let mut item = button::Button::new(content)
            .style(style(variant, metrics.radius))
            .padding([metrics.padding_v, metrics.padding_h]);

        item = item.width(width);

        item = item.height(metrics.height);
        let item = item.on_press_maybe(activation.clone());

        let item = Pressable::maybe_inset(
            item,
            activation,
            metrics.radius.into(),
            ButtonFocusRing::Default,
        );

        if let Some(label) = tooltip_label {
            crate::widgets::overlays::tooltip::Tooltip::new(item, label).into()
        } else {
            item
        }
    }

    fn content(self, metrics: SelectableItemMetrics) -> Element<'a, Message> {
        let typography = label_typography(self.size);
        let label = container(MeasuredText::new_inherited(
            self.label,
            EllipsisStrategy::End,
            typography,
        ))
        .width(Length::Fill)
        .clip(true);

        let mut content = Row::new()
            .spacing(metrics.gap)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill);

        if let Some(color) = self.leading_color {
            content = content.push(color_square(color, metrics.leading_size));
        }

        if self.reserve_status_indicator {
            content = match self
                .status_indicator
                .as_ref()
                .filter(|status| !status.is_empty())
            {
                Some(status) => content.push(
                    ToneDot::new(status.tone())
                        .size(self.size)
                        .disabled(self.disabled),
                ),
                None => content.push(iced::widget::Space::new().width(Length::Fixed(
                    crate::widgets::primitives::tone_dot::dot_size(self.size),
                ))),
            };
        }

        if let Some(icon) = self.leading_icon {
            content = content.push(icon::reference(icon).custom_size(metrics.icon_size));
        }

        content = content.push(label);

        if let Some(status) = self.status_indicator.filter(|status| !status.is_empty()) {
            let (_, label) = status.into_parts();
            content = content.push(text(label).size(metrics.font_size));
        }

        if let Some(trailing) = self.trailing_text {
            content = content.push(
                text(trailing)
                    .size(metrics.font_size)
                    .shaping(text::Shaping::Auto),
            );
        }

        if let Some(trailing) = self.trailing {
            content = content.push(trailing);
        }

        content.into()
    }
}

impl<'a, Message> From<SelectableItem<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(item: SelectableItem<'a, Message>) -> Self {
        item.into_element()
    }
}

fn color_square<'a, Message>(color: Color, size: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    container(text(""))
        .style(color_square_style(color, size))
        .width(size)
        .height(size)
        .into()
}

fn metrics(size: ControlSize) -> SelectableItemMetrics {
    metrics_for_theme(theme::active(), size)
}

const fn label_typography(size: ControlSize) -> TypographyRole {
    match size {
        ControlSize::Xs => TypographyRole::BodySmall,
        ControlSize::Sm | ControlSize::Md => TypographyRole::Body,
        ControlSize::Lg => TypographyRole::Heading,
    }
}

fn metrics_for_theme(theme: crate::theme::Theme, size: ControlSize) -> SelectableItemMetrics {
    let control = theme.control_metrics(size);
    let spacing = theme.spacing();

    SelectableItemMetrics {
        font_size: control.font_size,
        padding_v: match size {
            ControlSize::Xs => spacing.xxs,
            ControlSize::Sm => spacing.xs,
            ControlSize::Md => spacing.xs + 1.0,
            ControlSize::Lg => spacing.md,
        },
        padding_h: match size {
            ControlSize::Xs => spacing.sm,
            ControlSize::Sm => spacing.md,
            ControlSize::Md => spacing.md + spacing.xxs,
            ControlSize::Lg => spacing.xl,
        },
        radius: control.radius,
        leading_size: match size {
            ControlSize::Xs => 8.0,
            ControlSize::Sm | ControlSize::Md => 10.0,
            ControlSize::Lg => 12.0,
        },
        icon_size: control.icon_size,
        gap: match size {
            ControlSize::Lg => spacing.sm,
            _ => spacing.xs,
        },
        height: control.height,
    }
}

/// Resolves the combined selected×hover/pressed×disabled×focus state once,
/// centrally, so `content_color`/`default_style`/`selected_style` share it
/// instead of each reimplementing an alpha ladder.
fn resolved_control(
    theme: crate::theme::Theme,
    variant: SelectableItemVariant,
    status: Status,
) -> crate::theme::color_scheme::ControlSpec {
    let mut state = button_control_state(status);
    if matches!(variant, SelectableItemVariant::Selected) {
        state = state.selected();
    }
    // Embedded: a list item paints on the surface hosting the list and owns no
    // chrome, so untouched and disabled resolve transparent without a local
    // guard, and hover/pressed composite over whichever surface that is.
    // Selection is resolved before the role, so it is unaffected.
    theme.control(ControlRole::Embedded, state)
}

fn content_color(
    theme: &crate::theme::Theme,
    variant: SelectableItemVariant,
    status: Status,
) -> Color {
    let theme = *theme;
    let control = resolved_control(theme, variant, status);

    match (variant, status) {
        (SelectableItemVariant::Selected, _) => control.foreground,
        (SelectableItemVariant::Default, Status::Hovered | Status::Pressed) => {
            theme.text(TextRole::Primary).color
        }
        (SelectableItemVariant::Default, Status::Disabled) => control.foreground,
        (SelectableItemVariant::Default, Status::Active) => theme.text(TextRole::Secondary).color,
    }
}

fn style(
    variant: SelectableItemVariant,
    radius: f32,
) -> impl Fn(&crate::theme::Theme, Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: Status| {
        let mut style = match variant {
            SelectableItemVariant::Default => default_style(theme, status),
            SelectableItemVariant::Selected => selected_style(theme, status),
        };
        style.border.radius = radius.into();
        style
    }
}

fn color_square_style(
    color: Color,
    size: f32,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |_theme: &crate::theme::Theme| container::Style {
        background: Some(Background::Color(color)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::new(size / 2.6),
        },
        ..container::Style::default()
    }
}

fn default_style(theme: &crate::theme::Theme, status: Status) -> button::Style {
    let theme = *theme;
    let control = resolved_control(theme, SelectableItemVariant::Default, status);

    button::Style {
        background: Some(Background::Color(control.background)),
        text_color: content_color(&theme, SelectableItemVariant::Default, status),
        border: transparent_border(),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

fn selected_style(theme: &crate::theme::Theme, status: Status) -> button::Style {
    let theme = *theme;
    let control = resolved_control(theme, SelectableItemVariant::Selected, status);

    button::Style {
        background: Some(Background::Color(control.background)),
        text_color: content_color(&theme, SelectableItemVariant::Selected, status),
        border: transparent_border(),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

fn button_control_state(status: Status) -> ControlState {
    match status {
        Status::Active => ControlState::ENABLED,
        Status::Hovered => ControlState::HOVERED,
        Status::Pressed => ControlState::PRESSED,
        Status::Disabled => ControlState::DISABLED,
    }
}

fn transparent_border() -> Border {
    Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: 0.0.into(),
    }
}

#[cfg(test)]
mod selectable_item_tests {
    use super::*;
    use crate::theme::Theme;

    #[derive(Clone)]
    enum TestMessage {}

    #[test]
    fn content_fills_fixed_button_height() {
        let metrics = metrics(ControlSize::Sm);
        let content = SelectableItem::<TestMessage>::new("Project").content(metrics);

        assert_eq!(content.as_widget().size().height, Length::Fill);
        assert_eq!(
            metrics.icon_size,
            crate::theme::control_metrics(ControlSize::Sm).icon_size
        );
    }

    #[test]
    fn selected_item_uses_app_selected_control_spec() {
        let theme = Theme::Dark;
        let style = style(SelectableItemVariant::Selected, 6.0)(&theme, Status::Active);
        let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);

        assert_eq!(background_color(&style), selected.background);
        assert_eq!(style.text_color, selected.foreground);
    }

    #[test]
    fn disabled_selected_item_uses_the_shared_resolver_not_a_local_alpha() {
        let theme = Theme::Dark;
        let style = style(SelectableItemVariant::Selected, 6.0)(&theme, Status::Disabled);
        let disabled_selected = theme.control(
            ControlRole::Selectable,
            ControlState::new().selected().disabled(),
        );

        assert_eq!(background_color(&style), disabled_selected.background);
        assert_eq!(style.text_color, disabled_selected.foreground);
        // Same canonical dimming button/style.rs uses — no widget-local 0.55.
        let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);
        assert_eq!(
            background_color(&style),
            selected.background.scale_alpha(0.60)
        );
    }

    #[test]
    fn untouched_and_disabled_fills_come_from_the_theme_not_a_local_guard() {
        let theme = Theme::Dark;

        for status in [Status::Active, Status::Disabled] {
            let style = style(SelectableItemVariant::Default, 6.0)(&theme, status);

            assert_eq!(
                background_color(&style),
                theme
                    .control(ControlRole::Embedded, button_control_state(status))
                    .background,
                "{status:?} must resolve its fill through Embedded rather than a local branch"
            );
            assert_eq!(background_color(&style).a, 0.0);
        }

        // And hover still paints, so the assertion above is not describing a
        // widget that simply never fills.
        assert_ne!(
            background_color(&style(SelectableItemVariant::Default, 6.0)(
                &theme,
                Status::Hovered
            ))
            .a,
            0.0
        );
    }

    #[test]
    fn idle_content_uses_secondary_emphasis_without_border() {
        let theme = Theme::Dark;
        let style = style(SelectableItemVariant::Default, 6.0)(&theme, Status::Active);

        assert_eq!(style.text_color, theme.text(TextRole::Secondary).color);
        assert_eq!(style.border.width, 0.0);
        assert_eq!(background_color(&style), Color::TRANSPARENT);
    }

    #[test]
    fn every_size_and_density_uses_control_height_and_radius() {
        for density in crate::theme::ThemeDensity::ALL {
            let theme = Theme::builder("SelectableItem metrics", crate::theme::ThemeMode::Dark)
                .density(density)
                .build();
            for size in [
                ControlSize::Xs,
                ControlSize::Sm,
                ControlSize::Md,
                ControlSize::Lg,
            ] {
                let metrics = metrics_for_theme(theme, size);
                let control = theme.control_metrics(size);
                assert_eq!(metrics.height, control.height);
                assert_eq!(metrics.radius, control.radius);
                assert_eq!(metrics.icon_size, control.icon_size);
            }
        }
    }

    #[test]
    fn label_typography_matches_the_canonical_control_scale() {
        for size in [
            ControlSize::Xs,
            ControlSize::Sm,
            ControlSize::Md,
            ControlSize::Lg,
        ] {
            assert_eq!(
                crate::theme::typography(label_typography(size)),
                crate::theme::control_metrics(size).text_style,
            );
        }
    }

    fn background_color(style: &button::Style) -> Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }
}
