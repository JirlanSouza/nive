use nive::prelude::*;
use nive::ui::widgets::{button as nbutton, text as ntext};
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
                .action(nbutton::secondary("View").shrink().on_press(Message::Noop)),
        ),
        example_cell(
            "Callout alias",
            Callout::new("Configuration warning")
                .body("The callout type aliases InlineAlert and shares behavior.")
                .warning(),
        ),
        example_cell(
            "Danger",
            InlineAlert::new("Sync failed")
                .body("Retry keeps the current feedback action layout visible.")
                .danger()
                .action(
                    nbutton::destructive("Retry")
                        .shrink()
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
            "LoadingIndicator / Spinner",
            variant_row([
                LoadingIndicator::new().xs().label("Fetching").into(),
                Spinner::new().md().label("Saving").into(),
                LoadingIndicator::new()
                    .danger()
                    .lg()
                    .label("Retrying")
                    .into(),
            ]),
        ),
        example_cell(
            "Skeleton",
            column![
                nive::ui::widgets::skeleton::text_row().width(Length::Fill),
                nive::ui::widgets::skeleton::block().height(Length::Fixed(42.0)),
                nive::ui::widgets::skeleton::control(row![
                    nive::ui::widgets::skeleton::rounded(),
                    nive::ui::widgets::skeleton::text_row().width(Length::Fill),
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
                    .icon(IconName::Inbox)
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
                    .icon(IconName::AlertTriangle)
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
        SegmentedControl::new()
            .item(mode_item("Idle", FeedbackMode::Idle, app.feedback))
            .item(mode_item("Loading", FeedbackMode::Loading, app.feedback))
            .item(mode_item("Loaded", FeedbackMode::Loaded, app.feedback))
            .item(mode_item("Refresh", FeedbackMode::Refreshing, app.feedback))
            .item(mode_item("Error", FeedbackMode::Error, app.feedback))
            .item(mode_item("Empty", FeedbackMode::Empty, app.feedback))
            .item(mode_item("Running", FeedbackMode::Running, app.feedback))
            .fill(),
        InlineAlert::new("Devtools state")
            .body("Open devtools to force the inspected Resource and Operation sample fields.")
            .info(),
    ]
    .spacing(12)
    .into()
}

fn mode_item(
    label: &'static str,
    mode: FeedbackMode,
    active: FeedbackMode,
) -> SegmentedItem<'static, Message> {
    SegmentedItem::new(label)
        .selected(mode == active)
        .on_press(Message::FeedbackModeChanged(mode))
}
