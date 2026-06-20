use iced::{
    alignment,
    widget::{column, container, row, Space},
    Alignment, Length, Padding,
};

use nive_ui::theme::{self, GapRole};
use nive_ui::widgets::{
    button, input, Checkbox, InputGroup, SegmentedControl, SegmentedItem, Separator, Switch,
};
use nive_ui::Element;

use crate::devtools::probe::{ProbeCatalogEntry, ProbeDraft, ProbeEffect, ProbePanelMessage};
use crate::devtools::{DevtoolsPanelMessage, DevtoolsPanelState, DevtoolsRowId};

use super::super::shared::{
    clamped_caption, clamped_label_strong, empty_message, normalized_query, scroll_footer,
    STATE_TITLE_LABEL_CHARS, STATE_TITLE_PATH_CHARS, STATE_TITLE_WIDTH,
};

const PROBE_SWITCH_WIDTH: f32 = 36.0;
const PROBE_ACTIONS_WIDTH: f32 = 208.0;
const PROBE_FIRE_BUTTON_WIDTH: f32 = 84.0;
const PROBE_CLEAR_BUTTON_WIDTH: f32 = 52.0;

pub fn probes_body<'a, P, Message>(
    state: &'a DevtoolsPanelState<P>,
    map: impl Fn(DevtoolsPanelMessage<P>) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    P: ProbeCatalogEntry,
    Message: Clone + 'a + 'static,
{
    let query = normalized_query(state.query());
    let mut list = column![]
        .spacing(theme::gap(GapRole::Related))
        .width(Length::Fill);

    let header = row![
        Space::new().width(Length::Fixed(PROBE_SWITCH_WIDTH)),
        container(nive_ui::widgets::text::caption("Probe")).width(Length::Fixed(STATE_TITLE_WIDTH)),
        container(nive_ui::widgets::text::caption("Config summary")).width(Length::Fill),
        container(nive_ui::widgets::text::caption("Actions"))
            .width(Length::Fixed(PROBE_ACTIONS_WIDTH)),
    ]
    .spacing(theme::gap(GapRole::Related))
    .align_y(Alignment::Center);
    list = list.push(header).push(Separator::horizontal());

    let mut rendered = 0usize;
    for draft in state
        .probes
        .drafts
        .iter()
        .filter(|draft| draft.matches_filter(state.active_only(), &query))
    {
        if rendered > 0 {
            list = list.push(Separator::horizontal());
        }
        list = list.push(probe_row(state, draft, map));
        rendered += 1;
    }

    if rendered == 0 {
        list = list.push(nive_ui::widgets::text::caption(empty_message(
            state.query(),
            "No probes are available",
            "No probes match the current filters",
        )));
    }

    list = list.push(scroll_footer());

    list.into()
}

fn probe_row<'a, P, Message>(
    state: &'a DevtoolsPanelState<P>,
    draft: &'a ProbeDraft<P>,
    map: impl Fn(DevtoolsPanelMessage<P>) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    P: ProbeCatalogEntry,
    Message: Clone + 'a + 'static,
{
    let probe = draft.probe;
    let meta = probe.meta();
    let row_id = DevtoolsRowId::Probe(meta.key.to_string());
    let expanded = state.is_row_expanded(&row_id);
    let enabled = Switch::new(draft.enabled).on_toggle(move |enabled| {
        map(DevtoolsPanelMessage::Probe(
            ProbePanelMessage::SetProbeEnabled(probe, enabled),
        ))
    });
    let title = column![
        clamped_label_strong(meta.key, STATE_TITLE_LABEL_CHARS),
        clamped_caption(format!("short: {}", meta.short_key), STATE_TITLE_PATH_CHARS),
    ]
    .spacing(theme::gap(GapRole::Tight))
    .width(Length::Fill);
    let summary = nive_ui::widgets::text::caption(draft.summary());
    let actions = row![
        button::secondary("Fire next")
            .xs()
            .width(Length::Fixed(PROBE_FIRE_BUTTON_WIDTH))
            .tooltip("Fire next probe")
            .on_press(map(DevtoolsPanelMessage::Probe(
                ProbePanelMessage::FireNext(probe),
            ))),
        button::ghost("Clear")
            .xs()
            .width(Length::Fixed(PROBE_CLEAR_BUTTON_WIDTH))
            .on_press(map(DevtoolsPanelMessage::Probe(
                ProbePanelMessage::ClearProbe(probe),
            ))),
        button::ghost(if expanded { "Hide" } else { "Edit" })
            .xs()
            .width(Length::Fixed(PROBE_CLEAR_BUTTON_WIDTH))
            .on_press(map(DevtoolsPanelMessage::ToggleRowExpanded(row_id.clone()))),
    ]
    .spacing(theme::gap(GapRole::Tight))
    .align_y(Alignment::Center);

    let main = row![
        container(enabled).width(Length::Fixed(PROBE_SWITCH_WIDTH)),
        container(title).width(Length::Fixed(STATE_TITLE_WIDTH)),
        container(summary).width(Length::Fill),
        container(actions)
            .width(Length::Fixed(PROBE_ACTIONS_WIDTH))
            .align_x(alignment::Horizontal::Right),
    ]
    .spacing(theme::gap(GapRole::Related))
    .align_y(Alignment::Center);

    let mut content = column![main]
        .spacing(theme::gap(GapRole::Related))
        .width(Length::Fill)
        .padding(Padding::ZERO.vertical(theme::space(theme::SpaceStep::Xs)));

    if !expanded {
        return content.into();
    }

    let effect = probe_effect_control(probe, draft.effect, map);
    let delay = numeric_field("Delay", &draft.delay_ms, "ms", 128.0, move |value| {
        map(DevtoolsPanelMessage::Probe(
            ProbePanelMessage::DelayChanged(probe, value),
        ))
    });
    let skip = numeric_field("Skip", &draft.skip, "", 104.0, move |value| {
        map(DevtoolsPanelMessage::Probe(ProbePanelMessage::SkipChanged(
            probe, value,
        )))
    });
    let count = numeric_field("Count", &draft.count, "", 104.0, move |value| {
        map(DevtoolsPanelMessage::Probe(
            ProbePanelMessage::CountChanged(probe, value),
        ))
    });
    let repeat = Checkbox::new("Repeat", draft.repeat)
        .xs()
        .on_toggle(move |repeat| {
            map(DevtoolsPanelMessage::Probe(
                ProbePanelMessage::RepeatChanged(probe, repeat),
            ))
        });
    let message = InputGroup::new(
        input::default_owned("Failure message", draft.message.clone())
            .xs()
            .on_input(move |message| {
                map(DevtoolsPanelMessage::Probe(
                    ProbePanelMessage::MessageChanged(probe, message),
                ))
            }),
    )
    .leading_text("msg")
    .xs()
    .fill();

    content = content.push(
        row![
            Space::new().width(Length::Fixed(
                PROBE_SWITCH_WIDTH + STATE_TITLE_WIDTH + theme::gap(GapRole::Related) * 2.0
            )),
            container(effect).width(Length::Fixed(164.0)),
            delay,
            skip,
            count,
            repeat,
            container(message).width(Length::Fill),
        ]
        .spacing(theme::gap(GapRole::Related))
        .align_y(Alignment::Center),
    );

    content.into()
}

fn probe_effect_control<'a, P, Message>(
    probe: P,
    effect: ProbeEffect,
    map: impl Fn(DevtoolsPanelMessage<P>) -> Message + Copy + 'a,
) -> Element<'a, Message>
where
    P: ProbeCatalogEntry,
    Message: Clone + 'a,
{
    SegmentedControl::new()
        .item(
            SegmentedItem::new("Fail")
                .selected(effect == ProbeEffect::Fail)
                .on_press(map(DevtoolsPanelMessage::Probe(
                    ProbePanelMessage::EffectChanged(probe, ProbeEffect::Fail),
                ))),
        )
        .item(
            SegmentedItem::new("Delay")
                .selected(effect == ProbeEffect::DelayOnly)
                .on_press(map(DevtoolsPanelMessage::Probe(
                    ProbePanelMessage::EffectChanged(probe, ProbeEffect::DelayOnly),
                ))),
        )
        .xs()
        .fill()
        .into()
}

fn numeric_field<'a, Message>(
    label: &'a str,
    value: &'a str,
    trailing: &'a str,
    width: f32,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let input = input::default("", value).xs().on_input(on_input);
    let mut group = InputGroup::new(input).leading_text(label).xs().shrink();

    if !trailing.is_empty() {
        group = group.trailing_text(trailing);
    }

    container(group)
        .width(Length::Fixed(width))
        .align_y(alignment::Vertical::Center)
        .into()
}
