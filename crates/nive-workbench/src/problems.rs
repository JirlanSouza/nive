use std::borrow::Cow;

use iced::widget::column;
use nive_ui::theme::ToneRole;
use nive_ui::widgets::{DataRow, EmptyState};
use nive_ui::{Element, IconRole};

use crate::panels::WorkbenchPanel;

/// Neutral diagnostic severity for the problems panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ProblemSeverity {
    /// Informational diagnostic.
    Info,
    /// Warning diagnostic.
    Warning,
    /// Error diagnostic.
    Error,
}

/// Optional neutral problem location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemLocation<'a> {
    /// Source or file label.
    pub source: Cow<'a, str>,
    /// Optional one-based line number.
    pub line: Option<u32>,
    /// Optional one-based column number.
    pub column: Option<u32>,
}

/// Neutral problem/diagnostic model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem<'a> {
    /// Diagnostic severity.
    pub severity: ProblemSeverity,
    /// Diagnostic source, such as a subsystem or tool name.
    pub source: Cow<'a, str>,
    /// User-visible message.
    pub message: Cow<'a, str>,
    /// Optional location.
    pub location: Option<ProblemLocation<'a>>,
}

/// Problems panel builder that produces a generic workbench panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemsPanel<'a> {
    title: Cow<'a, str>,
    problems: Vec<Problem<'a>>,
}

impl ProblemSeverity {
    /// Returns the matching workbench status tone.
    pub const fn tone(self) -> ToneRole {
        match self {
            Self::Info => ToneRole::Neutral,
            Self::Warning => ToneRole::Warning,
            Self::Error => ToneRole::Danger,
        }
    }

    /// Returns complete visible severity text for compact status composition.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Warning => "Warning",
            Self::Error => "Error",
        }
    }
}

impl<'a> ProblemLocation<'a> {
    /// Builds a location.
    pub fn new(source: impl Into<Cow<'a, str>>) -> Self {
        Self {
            source: source.into(),
            line: None,
            column: None,
        }
    }

    /// Sets line and column.
    pub const fn position(mut self, line: u32, column: u32) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    fn label(&self) -> String {
        match (self.line, self.column) {
            (Some(line), Some(column)) => format!("{}:{line}:{column}", self.source),
            (Some(line), None) => format!("{}:{line}", self.source),
            _ => self.source.to_string(),
        }
    }
}

impl<'a> Problem<'a> {
    /// Builds a neutral problem.
    pub fn new(
        severity: ProblemSeverity,
        source: impl Into<Cow<'a, str>>,
        message: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            severity,
            source: source.into(),
            message: message.into(),
            location: None,
        }
    }

    /// Sets an optional location.
    pub fn location(mut self, location: ProblemLocation<'a>) -> Self {
        self.location = Some(location);
        self
    }
}

impl<'a> ProblemsPanel<'a> {
    /// Builds a problems panel helper.
    pub fn new(problems: impl IntoIterator<Item = Problem<'a>>) -> Self {
        Self {
            title: Cow::Borrowed("Problems"),
            problems: problems.into_iter().collect(),
        }
    }

    /// Sets the panel title.
    pub fn title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = title.into();
        self
    }

    /// Returns the problems.
    pub fn problems(&self) -> &[Problem<'a>] {
        &self.problems
    }

    /// Converts this helper into a generic workbench panel.
    ///
    /// A non-zero count keeps its warning tone but states what the count
    /// means rather than restating the title or the count badge, which
    /// already carry that the panel has problems and how many.
    pub fn into_panel<PanelId, ActionId, Message>(
        self,
        id: PanelId,
    ) -> WorkbenchPanel<'a, PanelId, ActionId, Message>
    where
        Message: Clone + 'a,
    {
        let count = self.problems.len();
        let worst = self.problems.iter().map(|problem| problem.severity).max();
        let content = problems_view(self.problems);
        WorkbenchPanel::new(id, self.title, content)
            .icon(IconRole::DialogWarning)
            .count_badge(count as u64)
            .status_text(
                if count == 0 {
                    ToneRole::Success
                } else {
                    ToneRole::Warning
                },
                if count == 0 {
                    Cow::Borrowed("No problems")
                } else {
                    Cow::Borrowed(worst.expect("non-zero count has a worst severity").label())
                },
            )
    }
}

fn problems_view<'a, Message>(problems: Vec<Problem<'a>>) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    if problems.is_empty() {
        return EmptyState::new("No problems")
            .description("Diagnostics will appear here when available.")
            .icon(IconRole::DialogWarning)
            .into();
    }

    let spacing = nive_ui::theme::spacing();
    let mut list = column![].spacing(spacing.xs).padding(spacing.sm);
    for problem in problems {
        list = list.push(problem_row(problem));
    }

    list.into()
}

fn problem_row<'a, Message>(problem: Problem<'a>) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let source = problem.location.as_ref().map_or_else(
        || problem.source.to_string(),
        |location| format!("{} · {}", problem.source, location.label()),
    );
    let metadata = format!("{} · {source}", problem.severity.label());

    DataRow::new(problem.message)
        .tone(problem.severity.tone())
        .value(metadata)
        .fill_width()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn problem_model_is_domain_neutral() {
        let problem = Problem::new(ProblemSeverity::Warning, "parser", "Unused node")
            .location(ProblemLocation::new("graph.flow").position(12, 4));

        assert_eq!(problem.source, "parser");
        assert_eq!(
            problem.location.as_ref().map(ProblemLocation::label),
            Some("graph.flow:12:4".to_string())
        );
    }

    #[test]
    fn non_zero_count_status_neither_equals_the_title_nor_restates_the_count() {
        let panel = ProblemsPanel::new([Problem::new(
            ProblemSeverity::Warning,
            "parser",
            "Unused node",
        )])
        .into_panel::<&str, &str, ()>("problems");

        let status = panel
            .status_indicator_value()
            .expect("non-zero count keeps a status");

        assert_eq!(status.tone(), ToneRole::Warning);
        assert_ne!(status.label(), "Problems");
        assert!(!status.label().contains('1'));
    }

    #[test]
    fn zero_count_status_stays_informative_and_success_toned() {
        let panel = ProblemsPanel::new(Vec::new()).into_panel::<&str, &str, ()>("problems");

        let status = panel
            .status_indicator_value()
            .expect("zero count keeps an informative status");

        assert_eq!(status.tone(), ToneRole::Success);
        assert_eq!(status.label(), "No problems");
    }
}
