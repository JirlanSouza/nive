use nive::prelude::*;
use nive::ui::theme::ToneRole;
use nive::ui::widgets::primitives::text as ntext;
use nive::widget::{column, row};

use crate::app::{Message, WidgetGallery};
use crate::catalog::PageId;
use crate::layout::{example_cell, section, variant_grid, variant_row};

pub fn view(_app: &WidgetGallery) -> Element<'_, Message> {
    crate::app::page_shell(
        PageId::Display,
        column![
            section("Text and icon", text_and_icon()),
            section("Badges and swatches", badges()),
            section("Surfaces", surfaces()),
            section("Metadata", metadata()),
            section("Identity and separators", identity()),
        ]
        .spacing(18),
    )
}

fn text_and_icon() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Typography roles",
            column![
                ntext::heading("Heading"),
                ntext::title("Title"),
                ntext::body("Primary body text"),
                ntext::body_small("Secondary body text"),
                ntext::caption("Muted caption"),
                ntext::code("code_sample")
            ]
            .spacing(6),
        ),
        example_cell(
            "Icons",
            variant_row([
                Icon::role(IconRole::EditFind).xs().into(),
                Icon::role(IconRole::PreferencesSystem).sm().into(),
                Icon::role(IconRole::ViewRefresh).md().into(),
                Icon::role(IconRole::DialogWarning)
                    .lg()
                    .color(Color::from_rgb8(196, 112, 0))
                    .into(),
            ]),
        ),
        example_cell(
            "Long text",
            ntext::body(
                "A long display string keeps the current wrapping and shaping behavior visible for review.",
            )
            .width(Length::Fill),
        ),
    ])
}

fn badges() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Tone badges",
            variant_row([
                Badge::neutral("Neutral").into(),
                Badge::info("Info").into(),
                Badge::success("Success").into(),
                Badge::warning("Warning").into(),
                Badge::danger("Danger").into(),
            ]),
        ),
        example_cell(
            "Sizes",
            variant_row([
                Badge::accent("XS").xs().into(),
                Badge::accent("SM").sm().into(),
                Badge::accent("MD").md().into(),
                Badge::accent("LG").lg().into(),
            ]),
        ),
        example_cell(
            "ColorSwatch",
            variant_row([
                ColorSwatch::new(Color::from_rgb8(64, 123, 255)).xs().into(),
                ColorSwatch::new(Color::from_rgb8(31, 172, 99))
                    .sm()
                    .selected(true)
                    .into(),
                ColorSwatch::new(Color::from_rgb8(218, 78, 78)).md().into(),
                ColorSwatch::new(Color::from_rgb8(118, 84, 186))
                    .lg()
                    .disabled(true)
                    .into(),
            ]),
        ),
    ])
}

fn surfaces() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Card",
            Card::new(
                column![
                    ntext::label_strong("Card"),
                    ntext::body_small("Compact content")
                ]
                .spacing(4),
            )
            .padding(14),
        ),
        example_cell(
            "Panel",
            Panel::new(
                column![
                    ntext::label_strong("Panel"),
                    ntext::body_small("Surface role panel")
                ]
                .spacing(4),
            )
            .padding(14)
            .width(Length::Fill),
        ),
        example_cell("MetricCard", MetricCard::new("Open issues", 128)),
    ])
}

fn metadata() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "KeyValueList",
            KeyValueList::new()
                .item(MetadataItem::new("Owner", "Design Systems"))
                .item(MetadataItem::new("Status", "Refreshing").tone(ToneRole::Info))
                .item(MetadataItem::new("Version", "0.1.0-beta").value(VersionBadge::new("0.1.0"))),
        ),
        example_cell(
            "DataRow",
            column![
                DataRow::new("Renderer")
                    .value("Iced")
                    .tone(ToneRole::Success),
                DataRow::new("Long metadata key that pressures dense rows")
                    .value("Long value preserved for review")
                    .trailing(Badge::warning("Review")),
            ]
            .spacing(8),
        ),
        example_cell(
            "Compact cards",
            row![
                Card::new(MetricCard::new("Files", 24)).sm().padding(10),
                Card::new(MetricCard::new("Errors", 0)).sm().padding(10),
            ]
            .spacing(8),
        ),
    ])
}

fn identity() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "InitialAvatar",
            variant_row([
                InitialAvatar::new("Nive").sm().into(),
                InitialAvatar::new("Widget Gallery").accent().lg().into(),
            ]),
        ),
        example_cell(
            "VersionBadge",
            variant_row([
                VersionBadge::new("0.1.0").into(),
                VersionBadge::new("beta").into(),
            ]),
        ),
        example_cell(
            "Separator",
            column![
                ntext::body_small("Horizontal"),
                Separator::horizontal(),
                row![
                    ntext::body_small("Left"),
                    Separator::vertical(),
                    ntext::body_small("Right")
                ]
                .spacing(10)
                .height(30),
            ]
            .spacing(8),
        ),
    ])
}
