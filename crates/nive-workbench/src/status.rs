use std::borrow::Cow;

use iced::widget::{container, row, rule, stack, text};
use iced::{Alignment, Length, Padding};
use nive_ui::theme::{
    self, BorderRole, ControlSize, SurfaceRole, TextRole, Theme, ToneRole, TypographyRole,
};
use nive_ui::widgets::{ProgressBar, Spinner, ToneDot};
use nive_ui::{Element, IconRole};

use crate::layout_probe;

#[derive(Debug, Clone, Copy, PartialEq)]
struct StatusBarMetrics {
    height: f32,
    horizontal_padding: f32,
    item_gap: f32,
    inline_gap: f32,
    progress_gap: f32,
}

fn metrics(size: ControlSize) -> StatusBarMetrics {
    metrics_for_theme(theme::active(), size)
}

fn metrics_for_theme(theme: Theme, size: ControlSize) -> StatusBarMetrics {
    let control = theme.control_metrics(size);
    let spacing = theme.spacing();

    StatusBarMetrics {
        height: control.height,
        horizontal_padding: spacing.sm,
        item_gap: spacing.md,
        inline_gap: spacing.xxs,
        progress_gap: spacing.xs,
    }
}

/// One composable status-bar item.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum StatusItem<'a> {
    /// Plain text item.
    Text(Cow<'a, str>),
    /// Muted optional environment or application context.
    Context(Cow<'a, str>),
    /// Icon plus text item.
    IconText {
        /// Icon role.
        icon: IconRole,
        /// Visible label.
        label: Cow<'a, str>,
    },
    /// Severity/status item.
    Severity {
        /// Tone.
        tone: ToneRole,
        /// Visible label.
        label: Cow<'a, str>,
    },
    /// Progress item.
    Progress {
        /// Visible label.
        label: Cow<'a, str>,
        /// Current progress in the `0.0..=1.0` range.
        fraction: f32,
    },
    /// Operation summary item.
    OperationSummary {
        /// Active operation count.
        active: usize,
        /// Optional label.
        label: Cow<'a, str>,
    },
}

/// Compact workbench status bar.
#[derive(Debug, Clone, PartialEq)]
pub struct StatusBar<'a> {
    leading_items: Vec<StatusItem<'a>>,
    trailing_items: Vec<StatusItem<'a>>,
}

impl<'a> StatusItem<'a> {
    /// Creates a text item.
    pub fn text(label: impl Into<Cow<'a, str>>) -> Self {
        Self::Text(label.into())
    }

    /// Creates muted optional context.
    pub fn context(label: impl Into<Cow<'a, str>>) -> Self {
        Self::Context(label.into())
    }

    /// Creates an icon/text item.
    pub fn icon_text(icon: IconRole, label: impl Into<Cow<'a, str>>) -> Self {
        Self::IconText {
            icon,
            label: label.into(),
        }
    }

    /// Creates a severity item.
    pub fn severity(tone: ToneRole, label: impl Into<Cow<'a, str>>) -> Self {
        Self::Severity {
            tone,
            label: label.into(),
        }
    }

    /// Creates a progress item.
    pub fn progress(label: impl Into<Cow<'a, str>>, fraction: f32) -> Self {
        Self::Progress {
            label: label.into(),
            fraction: fraction.clamp(0.0, 1.0),
        }
    }

    /// Creates an operation summary item.
    pub fn operation_summary(active: usize, label: impl Into<Cow<'a, str>>) -> Self {
        Self::OperationSummary {
            active,
            label: label.into(),
        }
    }
}

impl<'a> StatusBar<'a> {
    /// Builds an empty status bar.
    pub fn new() -> Self {
        Self {
            leading_items: Vec::new(),
            trailing_items: Vec::new(),
        }
    }

    /// Builds a status bar from items.
    pub fn with_items(items: impl IntoIterator<Item = StatusItem<'a>>) -> Self {
        Self {
            leading_items: items.into_iter().collect(),
            trailing_items: Vec::new(),
        }
    }

    /// Adds one item to the leading lane.
    #[deprecated(since = "0.1.0", note = "use StatusBar::leading")]
    pub fn item(mut self, item: StatusItem<'a>) -> Self {
        self.leading_items.push(item);
        self
    }

    /// Adds one item to the flexible leading lane.
    pub fn leading(mut self, item: StatusItem<'a>) -> Self {
        self.leading_items.push(item);
        self
    }

    /// Adds one item to the protected trailing lane.
    pub fn trailing(mut self, item: StatusItem<'a>) -> Self {
        self.trailing_items.push(item);
        self
    }

    /// Returns the leading lane items.
    pub fn leading_items(&self) -> &[StatusItem<'a>] {
        &self.leading_items
    }

    /// Returns the trailing lane items.
    pub fn trailing_items(&self) -> &[StatusItem<'a>] {
        &self.trailing_items
    }

    /// Renders the status bar with the standalone [`ControlSize::Sm`] default.
    ///
    /// [`WorkbenchShell`](crate::WorkbenchShell) uses a crate-private sized
    /// path so its shared chrome size takes precedence. `StatusBar` intentionally
    /// has no public independent size builder.
    pub fn view<Message>(self) -> Element<'a, Message>
    where
        Message: Clone + 'a,
    {
        self.view_with_size(ControlSize::Sm)
    }

    pub(crate) fn view_with_size<Message>(self, size: ControlSize) -> Element<'a, Message>
    where
        Message: Clone + 'a,
    {
        let metrics = metrics(size);
        let mut leading = row![]
            .spacing(metrics.item_gap)
            .align_y(Alignment::Center)
            .height(Length::Fixed(metrics.height));
        for item in self.leading_items {
            if let Some(item) = status_item(item, size, metrics) {
                leading = leading.push(item);
            }
        }
        let leading = container(leading).width(Length::Fill).clip(true);

        let mut trailing = row![]
            .spacing(metrics.item_gap)
            .align_y(Alignment::Center)
            .height(Length::Fixed(metrics.height));
        for item in self.trailing_items {
            if let Some(item) = status_item(item, size, metrics) {
                trailing = trailing.push(item);
            }
        }

        let content = row![leading, trailing]
            .spacing(metrics.item_gap)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fixed(metrics.height));

        let content = layout_probe::probe("status_content", content);

        let bar = container(content)
            .padding(Padding::ZERO.horizontal(metrics.horizontal_padding))
            .width(Length::Fill)
            .height(Length::Fixed(metrics.height))
            .clip(true)
            .style(theme::surface::style(SurfaceRole::Chrome));

        let edge = rule::horizontal(1).style(top_edge_style);
        let edge = container(edge)
            .height(Length::Fill)
            .align_y(Alignment::Start);

        stack![bar, edge]
            .width(Length::Fill)
            .height(Length::Fixed(metrics.height))
            .into()
    }
}

fn top_edge_style(theme: &Theme) -> rule::Style {
    rule::Style {
        color: theme.border(BorderRole::Subtle).color,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}

impl Default for StatusBar<'_> {
    fn default() -> Self {
        Self::new()
    }
}

fn status_item<'a, Message>(
    item: StatusItem<'a>,
    size: ControlSize,
    metrics: StatusBarMetrics,
) -> Option<Element<'a, Message>>
where
    Message: Clone + 'a,
{
    match item {
        StatusItem::Text(label) => Some(status_text(label, TextRole::Secondary)),
        StatusItem::Context(label) => Some(status_text(label, TextRole::Muted)),
        StatusItem::IconText { icon, label } => Some(
            row![
                nive_ui::widgets::icon::role(icon)
                    .color(theme::active().text(TextRole::Secondary).color),
                status_text(label, TextRole::Secondary)
            ]
            .spacing(metrics.inline_gap)
            .align_y(Alignment::Center)
            .into(),
        ),
        StatusItem::Severity { tone, label } => Some(
            row![
                ToneDot::new(tone).size(size),
                status_text(label, TextRole::Secondary)
            ]
            .spacing(metrics.inline_gap)
            .align_y(Alignment::Center)
            .into(),
        ),
        StatusItem::Progress { label, fraction } => Some(
            row![
                status_text(label, TextRole::Secondary),
                ProgressBar::percent(fraction)
                    .tone(ToneRole::Accent)
                    .size(status_progress_size(size))
                    .width(Length::Fixed(72.0))
            ]
            .spacing(metrics.progress_gap)
            .align_y(Alignment::Center)
            .into(),
        ),
        StatusItem::OperationSummary { active, label } => {
            if active == 0 {
                None
            } else {
                Some(
                    container(
                        Spinner::new()
                            .neutral()
                            .size(status_operation_size(size))
                            .label(format!("{label}: {active} active")),
                    )
                    .width(Length::Shrink)
                    .into(),
                )
            }
        }
    }
}

fn status_text<'a, Message>(label: Cow<'a, str>, role: TextRole) -> Element<'a, Message>
where
    Message: 'a,
{
    let style = theme::typography(TypographyRole::BodySmall);
    text(label)
        .font(style.font)
        .size(style.size)
        .line_height(style.line_height)
        .wrapping(text::Wrapping::None)
        .style(theme::text::style(role))
        .into()
}

fn status_progress_size(size: ControlSize) -> ControlSize {
    match size {
        ControlSize::Xs | ControlSize::Sm => ControlSize::Xs,
        ControlSize::Md => ControlSize::Sm,
        ControlSize::Lg => ControlSize::Md,
    }
}

fn status_operation_size(size: ControlSize) -> ControlSize {
    status_progress_size(size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_is_clamped() {
        let item = StatusItem::progress("Indexing", 1.5);

        assert_eq!(
            item,
            StatusItem::Progress {
                label: Cow::Borrowed("Indexing"),
                fraction: 1.0
            }
        );
    }

    #[test]
    fn status_bar_composes_items() {
        let status = StatusBar::new()
            .leading(StatusItem::text("Ready"))
            .leading(StatusItem::context("main"))
            .trailing(StatusItem::severity(ToneRole::Warning, "3 warnings"));

        assert_eq!(status.leading_items().len(), 2);
        assert_eq!(status.trailing_items().len(), 1);
    }

    #[test]
    fn zero_operation_summary_is_omitted() {
        assert!(status_item::<()>(
            StatusItem::operation_summary(0, "Operations"),
            ControlSize::Sm,
            metrics(ControlSize::Sm),
        )
        .is_none());
    }

    #[test]
    fn status_typography_is_body_small_twelve_pixels() {
        assert_eq!(theme::typography(TypographyRole::BodySmall).size, 12.0);
    }

    #[test]
    fn operation_summary_uses_a_role_derived_nested_size() {
        assert_eq!(status_operation_size(ControlSize::Xs), ControlSize::Xs);
        assert_eq!(status_operation_size(ControlSize::Sm), ControlSize::Xs);
        assert_eq!(status_operation_size(ControlSize::Md), ControlSize::Sm);
        assert_eq!(status_operation_size(ControlSize::Lg), ControlSize::Md);
    }

    #[test]
    fn outer_height_matches_control_metrics_across_densities_and_sizes() {
        for density in nive_ui::theme::ThemeDensity::ALL {
            let theme = nive_ui::theme::Theme::builder(
                "StatusBar metric test",
                nive_ui::theme::ThemeMode::Dark,
            )
            .density(density)
            .build();

            for size in [
                ControlSize::Xs,
                ControlSize::Sm,
                ControlSize::Md,
                ControlSize::Lg,
            ] {
                assert_eq!(
                    metrics_for_theme(theme, size).height,
                    theme.control_metrics(size).height
                );
            }
        }
    }
}
