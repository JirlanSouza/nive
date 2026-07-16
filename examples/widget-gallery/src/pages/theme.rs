use nive::prelude::*;
use nive::ui::theme::{ControlSize, SurfaceRole, ToneRole};
use nive::ui::widgets::controls::button as nbutton;
use nive::ui::widgets::primitives::text as ntext;
use nive::widget::{column, row};

use crate::app::{Message, WidgetGallery};
use crate::catalog::PageId;
use crate::layout::{example_cell, section, swatch, variant_grid, variant_row};

pub fn view(app: &WidgetGallery) -> Element<'_, Message> {
    crate::app::page_shell(
        PageId::Theme,
        column![
            section("Tone roles", tones()),
            section("Surface roles", surfaces()),
            section("Typography", typography()),
            section("Control sizes", control_sizes(app)),
            section("Spacing and shape", spacing_shape()),
        ]
        .spacing(18),
    )
}

fn tones() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Role swatches",
            column![
                swatch(theme::active().tone(ToneRole::Accent).color, "Accent"),
                swatch(theme::active().tone(ToneRole::Info).color, "Info"),
                swatch(theme::active().tone(ToneRole::Success).color, "Success"),
                swatch(theme::active().tone(ToneRole::Warning).color, "Warning"),
                swatch(theme::active().tone(ToneRole::Danger).color, "Danger"),
            ]
            .spacing(8),
        ),
        example_cell(
            "Tone badges",
            variant_row([
                Badge::status("Accent").accent().into(),
                Badge::status("Info").info().into(),
                Badge::status("Success").success().into(),
                Badge::status("Warning").warning().into(),
                Badge::status("Danger").danger().into(),
            ]),
        ),
    ])
}

fn surfaces() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Canvas",
            Panel::new(ntext::body("Canvas"))
                .role(SurfaceRole::Canvas)
                .body_padding(16),
        ),
        example_cell(
            "Panel",
            Panel::new(ntext::body("Panel"))
                .role(SurfaceRole::Panel)
                .body_padding(16),
        ),
        example_cell(
            "Elevated",
            Panel::new(ntext::body("Elevated"))
                .role(SurfaceRole::Elevated)
                .body_padding(16),
        ),
        example_cell(
            "Popover",
            Panel::new(ntext::body("Popover"))
                .role(SurfaceRole::Popover)
                .body_padding(16),
        ),
        example_cell(
            "Dialog",
            Panel::new(ntext::body("Dialog"))
                .role(SurfaceRole::Dialog)
                .body_padding(16),
        ),
    ])
}

fn typography() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Type scale",
            column![
                ntext::heading("Heading"),
                ntext::title("Title"),
                ntext::section_label("Section label"),
                ntext::body("Body"),
                ntext::body_small("Body small"),
                ntext::label("Label"),
                ntext::caption("Caption"),
                ntext::code("Code"),
            ]
            .spacing(6),
        ),
        example_cell(
            "Muted and secondary",
            column![
                ntext::body("Primary text"),
                ntext::body_small("Secondary text"),
                ntext::caption("Muted caption"),
            ]
            .spacing(6),
        ),
    ])
}

fn control_sizes(app: &WidgetGallery) -> Element<'_, Message> {
    variant_grid([
        example_cell("XS", controls(ControlSize::Xs, app)),
        example_cell("SM", controls(ControlSize::Sm, app)),
        example_cell("MD", controls(ControlSize::Md, app)),
        example_cell("LG", controls(ControlSize::Lg, app)),
    ])
}

fn controls(size: ControlSize, app: &WidgetGallery) -> Element<'_, Message> {
    column![
        nbutton::primary("Button")
            .size(size)
            .on_press(Message::Noop),
        Input::new("Input", &app.form.name)
            .size(size)
            .on_change(Message::NameChanged),
        Badge::status("Badge").accent(),
        ProgressBar::percent(0.55).size(size),
    ]
    .spacing(8)
    .into()
}

fn spacing_shape() -> Element<'static, Message> {
    let spacing = theme::spacing();
    variant_grid([
        example_cell(
            "Spacing",
            column![
                spacing_bar("xs", spacing.xs),
                spacing_bar("sm", spacing.sm),
                spacing_bar("md", spacing.md),
                spacing_bar("lg", spacing.lg),
                spacing_bar("xl", spacing.xl),
            ]
            .spacing(8),
        ),
        example_cell(
            "Shape",
            row![
                Card::new(ntext::caption("md")).shape_md().padding(12),
                Card::new(ntext::caption("lg")).padding(12),
            ]
            .spacing(10),
        ),
    ])
}

fn spacing_bar(label: &'static str, width: f32) -> Element<'static, Message> {
    row![
        ntext::caption(label).width(40),
        container(text(""))
            .style(theme::surface::style(SurfaceRole::Elevated))
            .width(width)
            .height(14),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}
