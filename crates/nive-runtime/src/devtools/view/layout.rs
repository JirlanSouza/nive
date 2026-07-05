use iced::{
    widget::{column, container, row, scrollable, Space},
    Alignment, Length, Padding,
};

use nive_ui::theme::{self, GapRole, PaddingRole, SurfaceRole};
use nive_ui::widgets::containers::Panel;
use nive_ui::widgets::controls::{input, SegmentedControl, SegmentedItem};
use nive_ui::Element;

use crate::devtools::{
    DevtoolStateSnapshot, DevtoolsPanelMessage, DevtoolsPanelState, DevtoolsPanelTab,
};

use super::{operations::operations_body, resources::resources_body, shared::SEARCH_FIELD_WIDTH};

pub fn devtools_window<'a, Message>(
    state: &'a DevtoolsPanelState,
    snapshot: &'a DevtoolStateSnapshot,
    map: impl Fn(DevtoolsPanelMessage) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a + 'static,
{
    container(devtools_panel(state, snapshot, map))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn devtools_window_padding() -> Padding {
    theme::padding(PaddingRole::Panel).top(theme::space(theme::SpaceStep::Xxxl))
}

fn devtools_panel<'a, Message>(
    state: &'a DevtoolsPanelState,
    snapshot: &'a DevtoolStateSnapshot,
    map: impl Fn(DevtoolsPanelMessage) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a + 'static,
{
    let resource_count = snapshot.entries.iter().filter(|e| e.is_resource()).count();
    let operation_count = snapshot.entries.iter().filter(|e| !e.is_resource()).count();
    let search = container(
        input::default("Search devtools", state.query())
            .xs()
            .on_input(move |query| map(DevtoolsPanelMessage::SearchChanged(query))),
    )
    .width(Length::Fixed(SEARCH_FIELD_WIDTH));

    let header = row![
        nive_ui::widgets::primitives::text::label_strong("Devtools"),
        nive_ui::widgets::primitives::text::caption(format!(
            "{resource_count} resources · {operation_count} operations"
        )),
        Space::new().width(Length::Fill),
        search,
    ]
    .spacing(theme::gap(GapRole::Related))
    .align_y(Alignment::Center);

    let tabs = devtools_tabs(state, map);
    let toolbar = tab_toolbar(state, map);

    let resources = snapshot.entries.iter().filter(|e| e.is_resource());
    let operations = snapshot.entries.iter().filter(|e| !e.is_resource());

    let body = match state.active_tab {
        DevtoolsPanelTab::Resources => resources_body(state, resources, map),
        DevtoolsPanelTab::Operations => operations_body(state, operations, &snapshot.registry, map),
    };

    let body = scrollable(body).height(Length::Fill).width(Length::Fill);

    Panel::new(
        column![tabs, toolbar, body]
            .spacing(theme::gap(GapRole::Content))
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .header(header)
    .role(SurfaceRole::App)
    .padding(devtools_window_padding())
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn devtools_tabs<'a, Message>(
    state: &'a DevtoolsPanelState,
    map: impl Fn(DevtoolsPanelMessage) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    SegmentedControl::new()
        .item(
            SegmentedItem::new("Resources")
                .selected(state.active_tab == DevtoolsPanelTab::Resources)
                .on_press(map(DevtoolsPanelMessage::SelectTab(
                    DevtoolsPanelTab::Resources,
                ))),
        )
        .item(
            SegmentedItem::new("Operations")
                .selected(state.active_tab == DevtoolsPanelTab::Operations)
                .on_press(map(DevtoolsPanelMessage::SelectTab(
                    DevtoolsPanelTab::Operations,
                ))),
        )
        .xs()
        .fill()
        .into()
}

fn tab_toolbar<'a, Message>(
    state: &'a DevtoolsPanelState,
    _map: impl Fn(DevtoolsPanelMessage) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a + 'static,
{
    let toolbar = row![
        nive_ui::widgets::primitives::text::caption(match state.active_tab {
            DevtoolsPanelTab::Resources => "Inspect and simulate Resource state",
            DevtoolsPanelTab::Operations => "Drive Operation transitions",
        }),
        Space::new().width(Length::Fill),
    ]
    .spacing(theme::gap(GapRole::Related))
    .align_y(Alignment::Center);

    toolbar.into()
}
