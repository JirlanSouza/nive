use iced::{
    alignment,
    widget::{column, container, row, text, Space},
    Alignment, Length,
};

use nive_ui::theme::{self, GapRole};
use nive_ui::widgets::{button, input, Badge, InputGroup, Separator};
use nive_ui::{Element, Renderer, Theme};

use crate::devtools::{DevtoolsPanelMessage, DevtoolsPanelState, DevtoolsRowId};
use crate::inspect::SimulableSnapshot;

#[cfg(test)]
mod tests;

pub(super) const STATE_TITLE_WIDTH: f32 = 260.0;
pub(super) const STATUS_WIDTH: f32 = 112.0;
pub(super) const ROW_ACTION_WIDTH: f32 = 56.0;
pub(super) const COMMAND_BUTTON_WIDTH: f32 = 76.0;
pub(super) const SECONDARY_COMMAND_BUTTON_WIDTH: f32 = 96.0;
pub(super) const SEARCH_FIELD_WIDTH: f32 = 220.0;
pub(super) const STATE_TITLE_LABEL_CHARS: usize = 32;
pub(super) const STATE_TITLE_PATH_CHARS: usize = 38;

pub(super) fn state_list_with_error<'a, Message>(
    state: &'a DevtoolsPanelState,
    map: impl Fn(DevtoolsPanelMessage) -> Message + Copy + 'a,
) -> iced::widget::Column<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a + 'static,
{
    let mut list = column![]
        .spacing(theme::gap(GapRole::Related))
        .width(Length::Fill);

    if let Some(error) = state.last_error() {
        let alert = row![
            nive_ui::widgets::text::caption(error.to_string()),
            Space::new().width(Length::Fill),
            button::ghost("Dismiss")
                .xs()
                .on_press(map(DevtoolsPanelMessage::ClearLastError)),
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

pub(super) fn row_error<'a, Message>(
    row_id: DevtoolsRowId,
    error: &'a str,
    map: impl Fn(DevtoolsPanelMessage) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
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

pub(super) fn snapshot_has_value(s: &SimulableSnapshot) -> bool {
    crate::devtools::host::snapshot_has_value(s)
}

pub(super) fn snapshot_has_error(s: &SimulableSnapshot) -> bool {
    crate::devtools::host::snapshot_has_error(s)
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

pub(super) fn simulable_status_view<'a, Message>(
    snapshot: &SimulableSnapshot,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    match snapshot {
        SimulableSnapshot::Idle => Badge::neutral("Idle").xs().into(),
        SimulableSnapshot::Loading { has_value } => {
            let badge: Element<'a, Message> = Badge::info("Loading").xs().into();
            if *has_value {
                column![
                    badge,
                    nive_ui::widgets::text::caption("preserving cached value")
                ]
                .spacing(theme::gap(GapRole::Tight))
                .into()
            } else {
                badge
            }
        }
        SimulableSnapshot::Loaded => Badge::success("Loaded").xs().into(),
        SimulableSnapshot::Failed { has_value, summary } => column![
            Badge::danger("Failed").xs(),
            nive_ui::widgets::text::caption(if *has_value {
                format!("{summary} · cached")
            } else {
                summary.clone()
            }),
        ]
        .spacing(theme::gap(GapRole::Tight))
        .into(),
        SimulableSnapshot::Running => Badge::info("Running").xs().into(),
        SimulableSnapshot::OperationFailed { summary } => column![
            Badge::danger("Failed").xs(),
            nive_ui::widgets::text::caption(summary.clone()),
        ]
        .spacing(theme::gap(GapRole::Tight))
        .into(),
    }
}

pub(super) fn command_error_field<'a, Message>(
    state: &'a DevtoolsPanelState,
    path: &str,
    map: impl Fn(DevtoolsPanelMessage) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a + 'static,
{
    let value = state.error_input(path);
    let path = path.to_string();
    let input = input::default_owned("Failure message", value)
        .xs()
        .on_input(move |value| {
            map(DevtoolsPanelMessage::ErrorMessageChanged {
                path: path.clone(),
                value,
            })
        });

    container(InputGroup::new(input).leading_text("err").xs().fill())
        .width(Length::Fixed(280.0))
        .align_y(alignment::Vertical::Center)
        .into()
}
