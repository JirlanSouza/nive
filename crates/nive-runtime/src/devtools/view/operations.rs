use iced::{
    alignment,
    widget::{container, row, Space},
    Alignment, Length, Padding,
};

use nive_ui::theme::{self, GapRole};
use nive_ui::widgets::{button, Separator};
use nive_ui::Element;

use crate::devtools::{
    DevtoolOperationView, DevtoolsPanelMessage, DevtoolsPanelState, DevtoolsRowId,
};
use crate::probe::ProbeCatalogEntry;

use super::shared::{
    command_button, command_error_field, command_field, empty_message, normalized_query,
    operation_status_view, row_action_button, row_error, scroll_footer, secondary_command_button,
    state_control_indent, state_list_with_error, state_title, text_matches_query, ROW_ACTION_WIDTH,
    STATE_TITLE_WIDTH, STATUS_WIDTH,
};

pub(super) fn operations_body<'a, P, Message>(
    state: &'a DevtoolsPanelState<P>,
    operations: Vec<DevtoolOperationView>,
    map: impl Fn(DevtoolsPanelMessage<P>) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    P: ProbeCatalogEntry,
    Message: Clone + 'a + 'static,
{
    let mut list = state_list_with_error(state, map);

    let query = normalized_query(state.query());
    let operations: Vec<_> = operations
        .into_iter()
        .filter(|operation| {
            text_matches_query(&query, [operation.label.as_str(), operation.path.as_str()])
        })
        .collect();

    if operations.is_empty() {
        return list
            .push(nive_ui::widgets::text::caption(empty_message(
                state.query(),
                "No OperationState entries for this screen",
                "No operations match the current search",
            )))
            .into();
    }

    let header = row![
        container(nive_ui::widgets::text::caption("Operation"))
            .width(Length::Fixed(STATE_TITLE_WIDTH)),
        container(nive_ui::widgets::text::caption("Status")).width(Length::Fixed(STATUS_WIDTH)),
        container(nive_ui::widgets::text::caption("Controls")).width(Length::Fill),
        container(nive_ui::widgets::text::caption("Actions"))
            .width(Length::Fixed(ROW_ACTION_WIDTH)),
    ]
    .spacing(theme::gap(GapRole::Related))
    .align_y(Alignment::Center);
    list = list.push(header).push(Separator::horizontal());

    for (index, operation) in operations.into_iter().enumerate() {
        if index > 0 {
            list = list.push(Separator::horizontal());
        }
        list = list.push(operation_row(state, operation, map));
    }

    list = list.push(scroll_footer());

    list.into()
}

fn operation_row<'a, P, Message>(
    state: &'a DevtoolsPanelState<P>,
    operation: DevtoolOperationView,
    map: impl Fn(DevtoolsPanelMessage<P>) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    P: ProbeCatalogEntry,
    Message: Clone + 'a + 'static,
{
    let path = operation.path.clone();
    let title = state_title(operation.label, operation.path.clone());
    let status = operation_status_view(&operation.status);
    let row_id = DevtoolsRowId::Operation(path.clone());
    let expanded = state.is_row_expanded(&row_id);
    let mut fields = row![]
        .spacing(theme::gap(GapRole::Related))
        .align_y(Alignment::Center);
    let mut visible_fields = 0usize;

    for field in operation.fields {
        visible_fields += 1;
        fields = fields.push(command_field(state, &path, field, map));
    }

    if visible_fields == 0 {
        fields = fields.push(nive_ui::widgets::text::caption("No inputs"));
    }

    let controls = row![
        command_button(button::secondary("Start"))
            .on_press(map(DevtoolsPanelMessage::SetOperationRunning(path.clone()))),
        command_button(button::destructive("Fail"))
            .on_press(map(DevtoolsPanelMessage::SetOperationFailed(path.clone()))),
        command_button(button::secondary("Reset"))
            .on_press(map(DevtoolsPanelMessage::SetOperationIdle(path.clone()))),
    ]
    .spacing(theme::gap(GapRole::Tight))
    .align_y(Alignment::Center);
    let action = row_action_button(button::ghost(if expanded { "Hide" } else { "Edit" }))
        .on_press(map(DevtoolsPanelMessage::ToggleRowExpanded(row_id.clone())));

    let main = row![
        container(title).width(Length::Fixed(STATE_TITLE_WIDTH)),
        container(status).width(Length::Fixed(STATUS_WIDTH)),
        container(controls).width(Length::Fill),
        container(action)
            .width(Length::Fixed(ROW_ACTION_WIDTH))
            .align_x(alignment::Horizontal::Right),
    ]
    .spacing(theme::gap(GapRole::Related))
    .align_y(Alignment::Center);

    let mut content = iced::widget::column![main]
        .spacing(theme::gap(GapRole::Related))
        .width(Length::Fill)
        .padding(Padding::ZERO.vertical(theme::space(theme::SpaceStep::Xs)));

    if expanded {
        content = content.push(
            row![
                Space::new().width(Length::Fixed(state_control_indent())),
                container(fields).width(Length::Fill),
            ]
            .spacing(theme::gap(GapRole::Related))
            .align_y(Alignment::Center),
        );
        content = content.push(
            row![
                Space::new().width(Length::Fixed(state_control_indent())),
                command_error_field(state, &path, map),
                secondary_command_button(button::ghost("Clear inputs"))
                    .on_press(map(DevtoolsPanelMessage::ClearCommandInputs(path.clone()))),
                Space::new().width(Length::Fill),
            ]
            .spacing(theme::gap(GapRole::Related))
            .align_y(Alignment::Center),
        );
    }

    if let Some(error) = state.row_command_error(&row_id) {
        content = content.push(row_error(row_id, error, map));
    }

    content.into()
}
