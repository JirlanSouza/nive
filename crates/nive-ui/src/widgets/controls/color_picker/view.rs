use iced::{
    widget::{column, container, row},
    Alignment, Length, Size,
};

use crate::{
    theme::{self, GapRole, PaddingRole, SpaceStep, SurfaceRole},
    widgets::{ColorSwatch, Input, InputGroup, Panel},
    Element,
};

use super::{
    controls::{alpha_slider, hue_slider, saturation_value_area},
    event::ColorPickerEvent,
    state::ColorPickerSnapshot,
};

const PREVIEW_SIZE: f32 = 22.0;
const HEX_INPUT_WIDTH: f32 = 132.0;
const ALPHA_INPUT_WIDTH: f32 = 72.0;

pub(super) fn color_picker_size() -> Size<Length> {
    Size::new(Length::Shrink, Length::Shrink)
}

pub(super) fn color_picker_content<'a>(
    snapshot: ColorPickerSnapshot,
    disabled: bool,
) -> Element<'a, ColorPickerEvent> {
    panel_surface(picker_content(snapshot, disabled))
}

fn panel_surface<'a>(content: Element<'a, ColorPickerEvent>) -> Element<'a, ColorPickerEvent> {
    Panel::new(content)
        .role(SurfaceRole::Popover)
        .padding(theme::padding(PaddingRole::Content))
        .width(Length::Shrink)
        .into()
}

fn picker_content<'a>(
    snapshot: ColorPickerSnapshot,
    disabled: bool,
) -> Element<'a, ColorPickerEvent> {
    column![
        color_controls(&snapshot, disabled),
        text_controls(&snapshot, disabled),
    ]
    .spacing(theme::gap(GapRole::Related))
    .into()
}

fn color_controls<'a>(
    snapshot: &ColorPickerSnapshot,
    disabled: bool,
) -> Element<'a, ColorPickerEvent> {
    row![
        saturation_value_area(snapshot.color, snapshot.hsva, disabled),
        hue_slider(snapshot.color, snapshot.hsva.hue(), disabled),
        alpha_slider(snapshot.color, snapshot.hsva.alpha(), disabled),
    ]
    .spacing(theme::gap(GapRole::Related))
    .align_y(Alignment::Center)
    .into()
}

fn text_controls<'a>(
    snapshot: &ColorPickerSnapshot,
    disabled: bool,
) -> Element<'a, ColorPickerEvent> {
    row![
        ColorSwatch::new(snapshot.color)
            .size(PREVIEW_SIZE)
            .radius(5.0),
        hex_field(snapshot.hex_input.clone(), disabled),
        alpha_field(snapshot.alpha_input.clone(), disabled),
    ]
    .spacing(theme::space(SpaceStep::Sm))
    .align_y(Alignment::Center)
    .into()
}

fn hex_field<'a>(value: String, disabled: bool) -> Element<'a, ColorPickerEvent> {
    let input = Input::<ColorPickerEvent>::new("", value)
        .xs()
        .disabled(disabled)
        .on_change_maybe((!disabled).then_some(ColorPickerEvent::HexInput));

    container(InputGroup::new(input).leading_text("HEX").xs().fill_width())
        .width(Length::Fixed(HEX_INPUT_WIDTH))
        .into()
}

fn alpha_field<'a>(value: String, disabled: bool) -> Element<'a, ColorPickerEvent> {
    let input = Input::<ColorPickerEvent>::new("", value)
        .xs()
        .disabled(disabled)
        .on_change_maybe((!disabled).then_some(ColorPickerEvent::AlphaInput));

    container(InputGroup::new(input).trailing_text("%").xs().fill_width())
        .width(Length::Fixed(ALPHA_INPUT_WIDTH))
        .into()
}
