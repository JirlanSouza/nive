use std::borrow::Cow;

use iced::widget::{container, row, text, Space};
use iced::{Alignment, Length, Padding};
use nive_ui::theme::{self, SurfaceRole, TextRole, ToneRole};
use nive_ui::widgets::{OperationStatusLine, ProgressBar, ToneDot};
use nive_ui::{Element, IconRole};

/// One composable status-bar item.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum StatusItem<'a> {
    /// Plain text item.
    Text(Cow<'a, str>),
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
    /// Flexible spacer.
    Spacer,
}

/// Compact workbench status bar.
#[derive(Debug, Clone, PartialEq)]
pub struct StatusBar<'a> {
    items: Vec<StatusItem<'a>>,
}

impl<'a> StatusItem<'a> {
    /// Creates a text item.
    pub fn text(label: impl Into<Cow<'a, str>>) -> Self {
        Self::Text(label.into())
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
        Self { items: Vec::new() }
    }

    /// Builds a status bar from items.
    pub fn with_items(items: impl IntoIterator<Item = StatusItem<'a>>) -> Self {
        Self {
            items: items.into_iter().collect(),
        }
    }

    /// Adds one item.
    pub fn item(mut self, item: StatusItem<'a>) -> Self {
        self.items.push(item);
        self
    }

    /// Returns items.
    pub fn items(&self) -> &[StatusItem<'a>] {
        &self.items
    }

    /// Renders the status bar.
    pub fn view<Message>(self) -> Element<'a, Message>
    where
        Message: Clone + 'a,
    {
        let spacing = theme::spacing();
        let mut content = row![]
            .spacing(spacing.md)
            .align_y(Alignment::Center)
            .width(Length::Fill);

        for item in self.items {
            content = content.push(status_item(item));
        }

        container(content)
            .padding(Padding::ZERO.horizontal(spacing.sm).vertical(spacing.xs))
            .width(Length::Fill)
            .style(theme::surface::style(SurfaceRole::Chrome))
            .into()
    }
}

impl Default for StatusBar<'_> {
    fn default() -> Self {
        Self::new()
    }
}

fn status_item<'a, Message>(item: StatusItem<'a>) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    match item {
        StatusItem::Text(label) => text(label)
            .style(theme::text::style(TextRole::Muted))
            .into(),
        StatusItem::IconText { icon, label } => row![
            nive_ui::widgets::icon::role(icon),
            text(label).style(theme::text::style(TextRole::Muted))
        ]
        .spacing(theme::spacing().xxs)
        .align_y(Alignment::Center)
        .into(),
        StatusItem::Severity { tone, label } => row![
            ToneDot::new(tone).sm(),
            text(label).style(theme::text::style(TextRole::Muted))
        ]
        .spacing(theme::spacing().xxs)
        .align_y(Alignment::Center)
        .into(),
        StatusItem::Progress { label, fraction } => row![
            text(label).style(theme::text::style(TextRole::Muted)),
            ProgressBar::percent(fraction)
                .tone(ToneRole::Accent)
                .xs()
                .width(Length::Fixed(72.0))
        ]
        .spacing(theme::spacing().xs)
        .align_y(Alignment::Center)
        .into(),
        StatusItem::OperationSummary { active, label } => {
            let status: Element<'a, Message> = if active == 0 {
                OperationStatusLine::idle().into()
            } else {
                OperationStatusLine::running(format!("{label}: {active} active")).into()
            };
            status
        }
        StatusItem::Spacer => Space::new().width(Length::Fill).into(),
    }
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
            .item(StatusItem::text("Ready"))
            .item(StatusItem::Spacer)
            .item(StatusItem::severity(ToneRole::Warning, "3 warnings"));

        assert_eq!(status.items().len(), 3);
    }
}
