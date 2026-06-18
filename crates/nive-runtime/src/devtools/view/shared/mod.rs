use iced::{
    alignment,
    widget::{column, container, row, text, Space},
    Alignment, Length,
};

use nive_ui::theme::{self, GapRole};
use nive_ui::widgets::{button, input, Badge, InputGroup, Separator};
use nive_ui::{Element, Renderer, Theme};

use crate::devtools::{
    DevtoolAsyncStatus, DevtoolFieldSchema, DevtoolOperationStatus, DevtoolsPanelMessage,
    DevtoolsPanelState, DevtoolsRowId,
};

#[cfg(test)]
mod tests;

pub(super) const STATE_TITLE_WIDTH: f32 = 260.0;
pub(super) const STATUS_WIDTH: f32 = 112.0;
pub(super) const ROW_ACTION_WIDTH: f32 = 56.0;
pub(super) const COMMAND_BUTTON_WIDTH: f32 = 76.0;
pub(super) const SECONDARY_COMMAND_BUTTON_WIDTH: f32 = 96.0;
pub(super) const FIXTURE_BUTTON_WIDTH: f32 = 132.0;
pub(super) const FIELD_WIDTH: f32 = 150.0;
pub(super) const ERROR_FIELD_WIDTH: f32 = 280.0;
pub(super) const SEARCH_FIELD_WIDTH: f32 = 220.0;
pub(super) const STATE_TITLE_LABEL_CHARS: usize = 32;
pub(super) const STATE_TITLE_PATH_CHARS: usize = 38;

pub(super) fn state_list_with_error<'a, P, Message>(
    state: &'a DevtoolsPanelState<P>,
    map: impl Fn(DevtoolsPanelMessage<P>) -> Message + Copy + 'a,
) -> iced::widget::Column<'a, Message, Theme, Renderer>
where
    P: Copy + Eq,
    Message: Clone + 'a + 'static,
{
    let mut list = column![]
        .spacing(theme::gap(GapRole::Related))
        .width(Length::Fill);

    if let Some(error) = state.last_command_error() {
        let alert = row![
            nive_ui::widgets::text::caption(error.to_string()),
            Space::new().width(Length::Fill),
            button::ghost("Dismiss")
                .xs()
                .on_press(map(DevtoolsPanelMessage::ClearCommandError)),
        ]
        .spacing(theme::gap(GapRole::Related))
        .align_y(Alignment::Center);
        list = list.push(alert).push(Separator::horizontal());
    }

    list
}

pub(super) fn scroll_footer<'a, Message>() -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    Space::new()
        .height(Length::Fixed(theme::space(theme::SpaceStep::Xl)))
        .into()
}

pub(super) fn command_button<'a, Message>(
    button: button::Button<'a, Message>,
) -> button::Button<'a, Message>
where
    Message: Clone + 'a,
{
    button.xs().width(Length::Fixed(COMMAND_BUTTON_WIDTH))
}

pub(super) fn secondary_command_button<'a, Message>(
    button: button::Button<'a, Message>,
) -> button::Button<'a, Message>
where
    Message: Clone + 'a,
{
    button
        .xs()
        .width(Length::Fixed(SECONDARY_COMMAND_BUTTON_WIDTH))
}

pub(super) fn row_action_button<'a, Message>(
    button: button::Button<'a, Message>,
) -> button::Button<'a, Message>
where
    Message: Clone + 'a,
{
    button.xs().width(Length::Fixed(ROW_ACTION_WIDTH))
}

pub(super) fn fixture_action<'a, Message>(label: String, on_press: Message) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    row![
        container(nive_ui::widgets::text::caption(label).width(Length::Fill))
            .width(Length::Fixed(120.0)),
        button::secondary("Load")
            .xs()
            .width(Length::Fixed(FIXTURE_BUTTON_WIDTH))
            .on_press(on_press),
    ]
    .spacing(theme::gap(GapRole::Tight))
    .align_y(Alignment::Center)
    .into()
}

pub(super) fn row_error<'a, P, Message>(
    row_id: DevtoolsRowId,
    error: &'a str,
    map: impl Fn(DevtoolsPanelMessage<P>) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    P: Copy + Eq,
    Message: Clone + 'a,
{
    row![
        Space::new().width(Length::Fixed(state_control_indent())),
        Badge::danger("Command failed").xs(),
        nive_ui::widgets::text::caption(error),
        Space::new().width(Length::Fill),
        button::ghost("Dismiss")
            .xs()
            .on_press(map(DevtoolsPanelMessage::ClearRowCommandError(row_id))),
    ]
    .spacing(theme::gap(GapRole::Related))
    .align_y(Alignment::Center)
    .into()
}

pub(super) fn state_control_indent() -> f32 {
    STATE_TITLE_WIDTH + STATUS_WIDTH + theme::gap(GapRole::Related) * 2.0
}

pub(super) fn state_title<'a, Message>(label: String, path: String) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    column![
        clamped_label_strong(label, STATE_TITLE_LABEL_CHARS),
        clamped_code_small(path, STATE_TITLE_PATH_CHARS),
    ]
    .spacing(theme::gap(GapRole::Tight))
    .width(Length::Fill)
    .into()
}

pub(super) fn clamped_label_strong<'a>(
    content: impl Into<String>,
    max_chars: usize,
) -> iced::widget::text::Text<'a, Theme> {
    nive_ui::widgets::text::label_strong(ellipsize_end(content.into(), max_chars))
        .width(Length::Fill)
        .wrapping(text::Wrapping::None)
}

pub(super) fn clamped_caption<'a>(
    content: impl Into<String>,
    max_chars: usize,
) -> iced::widget::text::Text<'a, Theme> {
    nive_ui::widgets::text::caption(ellipsize_end(content.into(), max_chars))
        .width(Length::Fill)
        .wrapping(text::Wrapping::None)
}

fn clamped_code_small<'a>(
    content: impl Into<String>,
    max_chars: usize,
) -> iced::widget::text::Text<'a, Theme> {
    nive_ui::widgets::text::code_small(ellipsize_end(content.into(), max_chars))
        .width(Length::Fill)
        .wrapping(text::Wrapping::None)
}

fn ellipsize_end(mut value: String, max_chars: usize) -> String {
    if max_chars == 0 {
        value.clear();
        return value;
    }

    if value.chars().count() <= max_chars {
        return value;
    }

    let keep = max_chars.saturating_sub(3);
    value = value.chars().take(keep).collect();
    value.push_str("...");
    value
}

pub(super) fn async_status_has_value(status: &DevtoolAsyncStatus) -> bool {
    match status {
        DevtoolAsyncStatus::Idle => false,
        DevtoolAsyncStatus::Loading { has_value }
        | DevtoolAsyncStatus::Failed { has_value, .. } => *has_value,
        DevtoolAsyncStatus::Loaded => true,
    }
}

pub(super) fn resource_status_has_error(status: &DevtoolAsyncStatus) -> bool {
    matches!(status, DevtoolAsyncStatus::Failed { .. })
}

pub(super) fn normalized_query(query: &str) -> String {
    query.trim().to_ascii_lowercase()
}

pub(super) fn text_matches_query<'a>(
    query: &str,
    values: impl IntoIterator<Item = &'a str>,
) -> bool {
    query.is_empty()
        || values
            .into_iter()
            .any(|value| value.to_ascii_lowercase().contains(query))
}

pub(super) fn empty_message<'a>(query: &str, empty: &'a str, filtered: &'a str) -> &'a str {
    if query.trim().is_empty() {
        empty
    } else {
        filtered
    }
}

pub(super) fn async_status_view<'a, Message>(status: &DevtoolAsyncStatus) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let detail = match status {
        DevtoolAsyncStatus::Idle => None,
        DevtoolAsyncStatus::Loading { has_value } => has_value.then_some("preserving cached value"),
        DevtoolAsyncStatus::Loaded => None,
        DevtoolAsyncStatus::Failed { has_value, summary } => {
            return column![
                Badge::danger("Failed").xs(),
                nive_ui::widgets::text::caption(if *has_value {
                    format!("{summary} · cached")
                } else {
                    summary.clone()
                }),
            ]
            .spacing(theme::gap(GapRole::Tight))
            .into();
        }
    };

    let badge: Element<'a, Message> = match status {
        DevtoolAsyncStatus::Idle => Badge::neutral("Idle").xs().into(),
        DevtoolAsyncStatus::Loading { .. } => Badge::info("Loading").xs().into(),
        DevtoolAsyncStatus::Loaded => Badge::success("Loaded").xs().into(),
        DevtoolAsyncStatus::Failed { .. } => unreachable!(),
    };

    match detail {
        Some(detail) => column![badge, nive_ui::widgets::text::caption(detail)]
            .spacing(theme::gap(GapRole::Tight))
            .into(),
        None => badge,
    }
}

pub(super) fn operation_status_view<'a, Message>(
    status: &DevtoolOperationStatus,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    match status {
        DevtoolOperationStatus::Idle => Badge::neutral("Idle").xs().into(),
        DevtoolOperationStatus::Running => Badge::info("Running").xs().into(),
        DevtoolOperationStatus::Failed { summary } => column![
            Badge::danger("Failed").xs(),
            nive_ui::widgets::text::caption(summary.clone()),
        ]
        .spacing(theme::gap(GapRole::Tight))
        .into(),
    }
}

pub(super) fn command_field<'a, P, Message>(
    state: &'a DevtoolsPanelState<P>,
    path: &str,
    field: DevtoolFieldSchema,
    map: impl Fn(DevtoolsPanelMessage<P>) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    P: Copy + Eq,
    Message: Clone + 'a + 'static,
{
    let value = state.command_value(path, &field.name);
    let path = path.to_string();
    let field_name = field.name.clone();
    let input = input::default_owned(field.placeholder.clone(), value)
        .xs()
        .on_input(move |value| {
            map(DevtoolsPanelMessage::CommandInputChanged {
                path: path.clone(),
                field_name: field_name.clone(),
                value,
            })
        });

    column![nive_ui::widgets::text::caption(field.label), input]
        .spacing(theme::gap(GapRole::Tight))
        .width(Length::Fixed(FIELD_WIDTH))
        .into()
}

pub(super) fn command_error_field<'a, P, Message>(
    state: &'a DevtoolsPanelState<P>,
    path: &str,
    map: impl Fn(DevtoolsPanelMessage<P>) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    P: Copy + Eq,
    Message: Clone + 'a + 'static,
{
    let value = state.command_error_value(path);
    let path = path.to_string();
    let input = input::default_owned("Failure message", value)
        .xs()
        .on_input(move |value| {
            map(DevtoolsPanelMessage::CommandErrorChanged {
                path: path.clone(),
                value,
            })
        });

    container(InputGroup::new(input).leading_text("err").xs().fill())
        .width(Length::Fixed(ERROR_FIELD_WIDTH))
        .align_y(alignment::Vertical::Center)
        .into()
}
