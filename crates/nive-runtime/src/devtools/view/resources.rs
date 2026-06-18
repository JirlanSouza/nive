use iced::{
    alignment,
    widget::{container, row, Space},
    Alignment, Length, Padding,
};

use nive_ui::theme::{self, GapRole};
use nive_ui::widgets::{button, Separator};
use nive_ui::Element;

use crate::devtools::{
    DevtoolResourceView, DevtoolsPanelMessage, DevtoolsPanelState, DevtoolsRowId,
};
use crate::probe::ProbeCatalogEntry;

use super::shared::{
    async_status_has_value, async_status_view, command_button, command_error_field, empty_message,
    fixture_action, normalized_query, resource_status_has_error, row_action_button, row_error,
    scroll_footer, secondary_command_button, state_control_indent, state_list_with_error,
    state_title, text_matches_query, ROW_ACTION_WIDTH, STATE_TITLE_WIDTH, STATUS_WIDTH,
};

pub(super) fn resources_body<'a, P, Message>(
    state: &'a DevtoolsPanelState<P>,
    resources: Vec<DevtoolResourceView>,
    map: impl Fn(DevtoolsPanelMessage<P>) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    P: ProbeCatalogEntry,
    Message: Clone + 'a + 'static,
{
    let mut list = state_list_with_error(state, map);

    let query = normalized_query(state.query());
    let resources: Vec<_> = resources
        .into_iter()
        .filter(|resource| {
            text_matches_query(&query, [resource.label.as_str(), resource.path.as_str()])
        })
        .collect();

    if resources.is_empty() {
        return list
            .push(nive_ui::widgets::text::caption(empty_message(
                state.query(),
                "No AsyncState resources for this screen",
                "No resources match the current search",
            )))
            .into();
    }

    let header = row![
        container(nive_ui::widgets::text::caption("Resource"))
            .width(Length::Fixed(STATE_TITLE_WIDTH)),
        container(nive_ui::widgets::text::caption("Status")).width(Length::Fixed(STATUS_WIDTH)),
        container(nive_ui::widgets::text::caption("Controls")).width(Length::Fill),
        container(nive_ui::widgets::text::caption("Actions"))
            .width(Length::Fixed(ROW_ACTION_WIDTH)),
    ]
    .spacing(theme::gap(GapRole::Related))
    .align_y(Alignment::Center);
    list = list.push(header).push(Separator::horizontal());

    for (index, resource) in resources.into_iter().enumerate() {
        if index > 0 {
            list = list.push(Separator::horizontal());
        }
        list = list.push(resource_row(state, resource, map));
    }

    list = list.push(scroll_footer());

    list.into()
}

fn resource_row<'a, P, Message>(
    state: &'a DevtoolsPanelState<P>,
    resource: DevtoolResourceView,
    map: impl Fn(DevtoolsPanelMessage<P>) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    P: ProbeCatalogEntry,
    Message: Clone + 'a + 'static,
{
    let path = resource.path.clone();
    let title = state_title(resource.label, resource.path.clone());
    let status = async_status_view(&resource.status);
    let preserve_failed_value = async_status_has_value(&resource.status);
    let row_id = DevtoolsRowId::Resource(path.clone());
    let expanded = state.is_row_expanded(&row_id);
    let controls = row![
        command_button(button::secondary("Set idle"))
            .on_press(map(DevtoolsPanelMessage::SetResourceIdle(path.clone()))),
        command_button(button::secondary("Loading")).on_press(map(
            DevtoolsPanelMessage::SetResourceLoading {
                path: path.clone(),
                preserve_value: false,
            }
        )),
        command_button(button::secondary("Refresh")).on_press(map(
            DevtoolsPanelMessage::SetResourceLoading {
                path: path.clone(),
                preserve_value: true,
            }
        )),
        secondary_command_button(button::destructive("Fail empty")).on_press(map(
            DevtoolsPanelMessage::SetResourceFailedEmpty(path.clone())
        )),
        secondary_command_button(button::destructive("Fail cached"))
            .disabled(!preserve_failed_value)
            .on_press(map(DevtoolsPanelMessage::SetResourceFailedCached(
                path.clone()
            ))),
    ]
    .spacing(theme::gap(GapRole::Tight))
    .align_y(Alignment::Center);
    let action = row_action_button(button::ghost(if expanded { "Hide" } else { "Edit" }))
        .on_press(map(DevtoolsPanelMessage::ToggleRowExpanded(row_id.clone())));

    let error_input = command_error_field(state, &path, map);
    let mut fixtures = row![]
        .spacing(theme::gap(GapRole::Related))
        .align_y(Alignment::Center);
    for fixture in resource.fixtures {
        fixtures = fixtures.push(fixture_action(
            fixture.label,
            map(DevtoolsPanelMessage::SetResourceLoadedFixture {
                path: path.clone(),
                fixture_id: fixture.id,
            }),
        ));
    }

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
            error_input,
            fixtures,
            Space::new().width(Length::Fill),
        ]
        .spacing(theme::gap(GapRole::Related))
        .align_y(Alignment::Center);

        if resource_status_has_error(&resource.status) {
            details = details.push(command_button(button::ghost("Dismiss")).on_press(map(
                DevtoolsPanelMessage::DismissResourceError(path.clone()),
            )));
        }

        content = content.push(details);
    }

    if let Some(error) = state.row_command_error(&row_id) {
        content = content.push(row_error(row_id, error, map));
    }

    content.into()
}
