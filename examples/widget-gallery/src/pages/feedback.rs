use nive::prelude::*;
use nive::ui::widgets::controls::button as nbutton;
use nive::ui::widgets::primitives::text as ntext;
use nive::widget::{column, row};

use crate::app::{FeedbackMode, Message, WidgetGallery};
use crate::catalog::PageId;
use crate::layout::{example_cell, section, variant_grid, variant_row};

pub fn view(app: &WidgetGallery) -> Element<'_, Message> {
    crate::app::page_shell(
        PageId::Feedback,
        column![
            section("Alerts and callouts", alerts()),
            section("Progress and loading", loading()),
            section("Empty and error states", states()),
            section("Resource and operation status", status_lines(app)),
            section("Local state controls", controls(app)),
        ]
        .spacing(18),
    )
}

fn alerts() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "InlineAlert",
            InlineAlert::new("Project indexed")
                .body("Search and navigation are up to date.")
                .success()
                .action(nbutton::secondary("View").shrink_width().on_press(Message::Noop)),
        ),
        example_cell(
            "Warning",
            InlineAlert::new("Configuration warning")
                .body("Review pending settings before deploying.")
                .warning(),
        ),
        example_cell(
            "Danger",
            InlineAlert::new("Sync failed")
                .body("Retry keeps the current feedback action layout visible.")
                .danger()
                .action(
                    nbutton::destructive("Retry")
                        .shrink_width()
                        .on_press(Message::Noop),
                ),
        ),
    ])
}

fn loading() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "ProgressBar",
            column![
                ProgressBar::percent(0.32).info(),
                ProgressBar::percent(0.68).success().md(),
                ProgressBar::percent(0.9).warning().lg(),
            ]
            .spacing(10),
        ),
        example_cell(
            "Spinner",
            variant_row([
                Spinner::new().xs().label("Fetching").into(),
                Spinner::new().md().label("Saving").into(),
                Spinner::new().danger().lg().label("Retrying").into(),
            ]),
        ),
        example_cell(
            "Skeleton",
            column![
                nive::ui::widgets::feedback::skeleton::text_row().width(Length::Fill),
                nive::ui::widgets::feedback::skeleton::block().height(Length::Fixed(42.0)),
                nive::ui::widgets::feedback::skeleton::control(row![
                    nive::ui::widgets::feedback::skeleton::rounded(),
                    nive::ui::widgets::feedback::skeleton::text_row().width(Length::Fill),
                ]),
            ]
            .spacing(10),
        ),
    ])
}

fn states() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "EmptyState",
            container(
                EmptyState::new("No projects")
                    .description("Create a project to populate this list.")
                    .icon(IconRole::MailInbox)
                    .action(nbutton::primary("Create").on_press(Message::Noop)),
            )
            .height(180),
        ),
        example_cell(
            "ErrorFeedback",
            ErrorFeedback::new("Could not load projects")
                .description("The local service returned a user-facing error.")
                .message("Connection refused")
                .action(ErrorFeedbackAction::primary_retry("Retry", Message::Noop))
                .action(ErrorFeedbackAction::details(Message::ShowDialog(
                    crate::app::DialogKind::LongContent,
                ))),
        ),
        example_cell(
            "ErrorEmptyState",
            container(
                ErrorEmptyState::new("Import failed")
                    .description("The selected file could not be parsed.")
                    .icon(IconRole::DialogWarning)
                    .action(ErrorFeedbackAction::retry("Try again", Message::Noop)),
            )
            .height(180),
        ),
        example_cell(
            "ErrorDetailsDialog",
            ErrorDetailsDialog::new("stack frame: widget_gallery::feedback::details")
                .on_close(Message::Noop),
        ),
    ])
}

fn status_lines(app: &WidgetGallery) -> Element<'_, Message> {
    variant_grid([
        example_cell(
            "ResourceStatusLine",
            column![
                ResourceStatusLine::<Message>::idle(),
                ResourceStatusLine::refreshing("Refreshing cached data"),
                ResourceStatusLine::stale_error("Showing stale data after refresh failed")
                    .action(ErrorFeedbackAction::retry("Retry", Message::Noop)),
            ]
            .spacing(8),
        ),
        example_cell(
            "OperationStatusLine",
            column![
                OperationStatusLine::<Message>::idle(),
                OperationStatusLine::running("Saving changes"),
                OperationStatusLine::error("Save failed").action(ErrorFeedbackAction::details(
                    Message::ShowDialog(crate::app::DialogKind::LongContent)
                )),
            ]
            .spacing(8),
        ),
        example_cell(
            "OperationActionGroup",
            OperationActionGroup::new()
                .status(match app.feedback {
                    FeedbackMode::Running => OperationStatusLine::running("Publishing"),
                    FeedbackMode::Error => OperationStatusLine::error("Publish failed"),
                    _ => OperationStatusLine::idle(),
                })
                .action(
                    nbutton::primary("Publish")
                        .on_press(Message::FeedbackModeChanged(FeedbackMode::Running)),
                )
                .action(
                    nbutton::secondary("Reset")
                        .on_press(Message::FeedbackModeChanged(FeedbackMode::Idle)),
                ),
        ),
    ])
}

fn controls(app: &WidgetGallery) -> Element<'_, Message> {
    let current = match app.feedback {
        FeedbackMode::Idle => "Idle",
        FeedbackMode::Loading => "Loading",
        FeedbackMode::Loaded => "Loaded",
        FeedbackMode::Refreshing => "Refreshing",
        FeedbackMode::Error => "Error",
        FeedbackMode::Empty => "Empty",
        FeedbackMode::Running => "Running",
    };

    column![
        ntext::body_small(format!("Current local feedback mode: {current}")),
        Select::new(FeedbackMode::ALL.to_vec(), Some(app.feedback))
            .on_select(Message::FeedbackModeChanged)
            .fill_width(),
        InlineAlert::new("Devtools optional")
            .body("Run the explicit devtools command only when inspecting simulator integration.")
            .info(),
    ]
    .spacing(12)
    .into()
}
