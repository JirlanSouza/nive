use iced::{
    alignment,
    widget::{container, row, Space},
    Alignment, Length, Padding,
};

use nive_ui::theme::{self, GapRole};
use nive_ui::widgets::controls::button;
use nive_ui::widgets::display::Badge;
use nive_ui::widgets::primitives::Separator;
use nive_ui::Element;

use crate::devtools::types::INPUT_CAPABILITY_HINT;
use crate::devtools::{
    DevtoolsPanelMessage, DevtoolsPanelState, DevtoolsRowId, RegistryEntry, RegistryStatus,
    SimulateAction, SimulatorEntry,
};

use super::shared::{
    command_button, command_error_field, empty_message, normalized_query, row_action_button,
    row_error, scroll_footer, simulable_status_view, snapshot_has_error, state_control_indent,
    state_list_with_error, state_title, text_matches_query, ROW_ACTION_WIDTH, STATE_TITLE_WIDTH,
    STATUS_WIDTH,
};

pub(super) fn operations_body<'a, Message>(
    state: &'a DevtoolsPanelState,
    entries: impl Iterator<Item = &'a SimulatorEntry>,
    registry: &'a [RegistryEntry],
    map: impl Fn(DevtoolsPanelMessage) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a + 'static,
{
    let mut list = state_list_with_error(state, map);

    let query = normalized_query(state.query());
    let entries: Vec<_> = entries
        .filter(|e| text_matches_query(&query, [e.label.as_str(), e.path.as_str()]))
        .collect();

    if !entries.is_empty() {
        let header = row![
            container(nive_ui::widgets::primitives::text::caption("Operation"))
                .width(Length::Fixed(STATE_TITLE_WIDTH)),
            container(nive_ui::widgets::primitives::text::caption("Status"))
                .width(Length::Fixed(STATUS_WIDTH)),
            container(nive_ui::widgets::primitives::text::caption("Controls")).width(Length::Fill),
            container(nive_ui::widgets::primitives::text::caption("Actions"))
                .width(Length::Fixed(ROW_ACTION_WIDTH)),
        ]
        .spacing(theme::gap(GapRole::Related))
        .align_y(Alignment::Center);
        list = list.push(header).push(Separator::horizontal());

        for (index, entry) in entries.into_iter().enumerate() {
            if index > 0 {
                list = list.push(Separator::horizontal());
            }
            list = list.push(operation_row(state, entry, map));
        }
    } else {
        list = list.push(nive_ui::widgets::primitives::text::caption(empty_message(
            state.query(),
            "No Operation fields for this screen",
            "No operations match the current search",
        )));
    }

    // Read-only operation registry section (task 9.3)
    if !registry.is_empty() {
        list =
            list.push(Separator::horizontal())
                .push(nive_ui::widgets::primitives::text::caption(
                    "Active operations (read-only)",
                ));
        for entry in registry {
            list = list.push(registry_row(entry));
        }
    }

    list = list.push(scroll_footer());
    list.into()
}

fn operation_row<'a, Message>(
    state: &'a DevtoolsPanelState,
    entry: &'a SimulatorEntry,
    map: impl Fn(DevtoolsPanelMessage) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a + 'static,
{
    let path = entry.path.clone();
    let title = state_title(entry.label.clone(), entry.path.clone());
    let status = simulable_status_view(&entry.snapshot);
    let has_error = snapshot_has_error(&entry.snapshot);
    let has_input = entry.capabilities.input;
    let row_id = DevtoolsRowId::Operation(path.clone());
    let expanded = state.is_row_expanded(&row_id);

    let controls = row![
        command_button(
            button::secondary("Start")
                .disabled(!has_input)
                .tooltip_maybe((!has_input).then_some(INPUT_CAPABILITY_HINT))
                .on_press_maybe(has_input.then(|| {
                    map(DevtoolsPanelMessage::Simulate {
                        path: path.clone(),
                        action: SimulateAction::Start,
                    })
                }))
        ),
        command_button(
            button::destructive("Fail")
                .disabled(!has_input)
                .tooltip_maybe((!has_input).then_some(INPUT_CAPABILITY_HINT))
                .on_press_maybe(has_input.then(|| {
                    map(DevtoolsPanelMessage::Simulate {
                        path: path.clone(),
                        action: SimulateAction::Error {
                            message: state.error_input(&path),
                        },
                    })
                }))
        ),
        command_button(
            button::secondary("Reset").on_press(map(DevtoolsPanelMessage::Simulate {
                path: path.clone(),
                action: SimulateAction::Idle,
            }))
        ),
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
        let mut details = row![
            Space::new().width(Length::Fixed(state_control_indent())),
            command_error_field(state, &path, map),
            Space::new().width(Length::Fill),
        ]
        .spacing(theme::gap(GapRole::Related))
        .align_y(Alignment::Center);

        if has_error {
            details = details.push(command_button(button::ghost("Dismiss")).on_press(map(
                DevtoolsPanelMessage::Simulate {
                    path: path.clone(),
                    action: SimulateAction::DismissError,
                },
            )));
        }

        content = content.push(details);
    }

    if let Some(error) = state.row_error(&row_id) {
        content = content.push(row_error(row_id, error, map));
    }

    content.into()
}

fn registry_row<'a, Message>(entry: &'a RegistryEntry) -> Element<'a, Message>
where
    Message: Clone + 'a + 'static,
{
    let status_badge: Element<'a, Message> = match &entry.status {
        RegistryStatus::Running => Badge::info("Running").xs().into(),
        RegistryStatus::Completed => Badge::success("Completed").xs().into(),
        RegistryStatus::Failed(msg) => iced::widget::column![
            Badge::danger("Failed").xs(),
            nive_ui::widgets::primitives::text::caption(msg.clone()),
        ]
        .spacing(theme::gap(GapRole::Tight))
        .into(),
        RegistryStatus::Cancelled => Badge::neutral("Cancelled").xs().into(),
    };

    row![
        container(nive_ui::widgets::primitives::text::caption(
            entry.label.clone()
        ))
        .width(Length::Fixed(STATE_TITLE_WIDTH)),
        container(status_badge).width(Length::Fixed(STATUS_WIDTH)),
        Space::new().width(Length::Fill),
    ]
    .spacing(theme::gap(GapRole::Related))
    .align_y(Alignment::Center)
    .padding(Padding::ZERO.vertical(theme::space(theme::SpaceStep::Xs)))
    .into()
}
