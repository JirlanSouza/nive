use iced::{
    widget::{column, container, row, scrollable, Space},
    Alignment, Length, Padding,
};

use nive_ui::theme::{self, GapRole, PaddingRole, SurfaceRole};
use nive_ui::widgets::{button, input, Checkbox, Panel, SegmentedControl, SegmentedItem};
use nive_ui::Element;

use crate::devtools::probe::{ProbeCatalogEntry, ProbePanelMessage};
use crate::devtools::{
    DevtoolStateSnapshot, DevtoolsPanelMessage, DevtoolsPanelState, DevtoolsPanelTab,
};

use super::{
    operations::operations_body, probes::probes_body, resources::resources_body,
    shared::SEARCH_FIELD_WIDTH,
};

pub fn devtools_window<'a, P, Message>(
    state: &'a DevtoolsPanelState<P>,
    snapshot: DevtoolStateSnapshot,
    map: impl Fn(DevtoolsPanelMessage<P>) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    P: ProbeCatalogEntry,
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

fn devtools_panel<'a, P, Message>(
    state: &'a DevtoolsPanelState<P>,
    snapshot: DevtoolStateSnapshot,
    map: impl Fn(DevtoolsPanelMessage<P>) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    P: ProbeCatalogEntry,
    Message: Clone + 'a + 'static,
{
    let resource_count = snapshot.resources.len();
    let operation_count = snapshot.operations.len();
    let search = container(
        input::default("Search devtools", state.query())
            .xs()
            .on_input(move |query| map(DevtoolsPanelMessage::SearchChanged(query))),
    )
    .width(Length::Fixed(SEARCH_FIELD_WIDTH));

    let header = row![
        nive_ui::widgets::text::label_strong("Devtools"),
        nive_ui::widgets::text::caption(format!(
            "{resource_count} resources · {operation_count} operations · {} active probes",
            state.active_probe_count()
        )),
        Space::new().width(Length::Fill),
        search,
    ]
    .spacing(theme::gap(GapRole::Related))
    .align_y(Alignment::Center);

    let tabs = devtools_tabs(state, map);
    let toolbar = tab_toolbar(state, map);

    let body = match state.active_tab {
        DevtoolsPanelTab::Resources => resources_body(state, snapshot.resources, map),
        DevtoolsPanelTab::Operations => operations_body(state, snapshot.operations, map),
        DevtoolsPanelTab::Probes => probes_body(state, map),
    };

    let body = scrollable(body).height(Length::Fill).width(Length::Fill);

    Panel::new(
        column![header, tabs, toolbar, body]
            .spacing(theme::gap(GapRole::Content))
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .role(SurfaceRole::App)
    .padding(devtools_window_padding())
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn devtools_tabs<'a, P, Message>(
    state: &'a DevtoolsPanelState<P>,
    map: impl Fn(DevtoolsPanelMessage<P>) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    P: ProbeCatalogEntry,
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
        .item(
            SegmentedItem::new("Probes")
                .selected(state.active_tab == DevtoolsPanelTab::Probes)
                .on_press(map(DevtoolsPanelMessage::SelectTab(
                    DevtoolsPanelTab::Probes,
                ))),
        )
        .xs()
        .fill()
        .into()
}

fn tab_toolbar<'a, P, Message>(
    state: &'a DevtoolsPanelState<P>,
    map: impl Fn(DevtoolsPanelMessage<P>) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    P: ProbeCatalogEntry,
    Message: Clone + 'a + 'static,
{
    let mut toolbar = row![
        nive_ui::widgets::text::caption(match state.active_tab {
            DevtoolsPanelTab::Resources => "Inspect and force AsyncState resources",
            DevtoolsPanelTab::Operations => "Drive OperationState transitions",
            DevtoolsPanelTab::Probes => "Inject failures or delays at async boundaries",
        }),
        Space::new().width(Length::Fill),
    ]
    .spacing(theme::gap(GapRole::Related))
    .align_y(Alignment::Center);

    if state.active_tab == DevtoolsPanelTab::Probes {
        toolbar = toolbar
            .push(
                Checkbox::new("Active only", state.active_only())
                    .xs()
                    .on_toggle(move |active_only| {
                        map(DevtoolsPanelMessage::ToggleActiveOnly(active_only))
                    }),
            )
            .push(
                button::secondary("Clear probes")
                    .xs()
                    .width(Length::Fixed(128.0))
                    .on_press(map(DevtoolsPanelMessage::Probe(
                        ProbePanelMessage::ClearAll,
                    ))),
            );
    }

    toolbar.into()
}
