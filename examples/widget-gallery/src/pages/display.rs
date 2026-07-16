use nive::prelude::*;
use nive::ui::theme::{SurfaceRole, ToneRole};
use nive::ui::widgets::primitives::text as ntext;
use nive::widget::{column, container, row};

use crate::app::{Message, WidgetGallery};
use crate::catalog::PageId;
use crate::layout::{example_cell, section, variant_grid, variant_row};

pub fn view(_app: &WidgetGallery) -> Element<'_, Message> {
    crate::app::page_shell(
        PageId::Display,
        column![
            section("Text and icon", text_and_icon()),
            section("Badges and semantic indicators", badges()),
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
            "Status badges",
            variant_row([
                Badge::status("Neutral").neutral().into(),
                Badge::status("Info").info().into(),
                Badge::status("Success").success().into(),
                Badge::status("Warning").warning().into(),
                Badge::status("Danger").danger().into(),
            ]),
        ),
        example_cell(
            "Counts · fixed geometry",
            variant_row([
                Badge::count(0).into(),
                Badge::count(1).into(),
                Badge::count(2).info().into(),
                Badge::count(99).warning().into(),
                Badge::count(100).danger().into(),
            ]),
        ),
        example_cell(
            "Labelled status · disabled · activity",
            variant_row([
                StatusIndicator::new(ToneRole::Success, "Healthy").into(),
                row![
                    Badge::count(12),
                    StatusIndicator::new(ToneRole::Warning, "Needs review")
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .into(),
                Badge::status("Disabled").warning().disabled(true).into(),
                row![Spinner::new(), ntext::body("Loading")]
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .into(),
            ]),
        ),
        example_cell(
            "Long, empty, and clipped hosts",
            variant_row([
                Badge::status("A long status that reveals the complete value").info().into(),
                Badge::status("   ").warning().into(),
                container(Badge::status("12px host").danger())
                    .width(Length::Fixed(12.0))
                    .clip(true)
                    .into(),
                container(Badge::count(8))
                    .width(Length::Fixed(8.0))
                    .clip(true)
                    .into(),
            ]),
        ),
        example_cell(
            "ToneDot 6/8 px labelled slots",
            variant_row([
                row![ToneDot::new(ToneRole::Neutral).xs(), ntext::body("Neutral · 6 px")]
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .into(),
                row![ToneDot::new(ToneRole::Accent).sm(), ntext::body("Accent · 6 px")]
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .into(),
                row![ToneDot::new(ToneRole::Info).xs(), ntext::body("Xs · 6 px")]
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .into(),
                row![ToneDot::new(ToneRole::Success).sm(), ntext::body("Sm · 6 px")]
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .into(),
                row![ToneDot::new(ToneRole::Warning).md(), ntext::body("Md · 8 px")]
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .into(),
                row![ToneDot::new(ToneRole::Danger).lg(), ntext::body("Lg · 8 px")]
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .into(),
            ]),
        ),
        example_cell(
            "Warning dot over opaque surfaces",
            variant_row([
                indicator_surface(SurfaceRole::App, "App"),
                indicator_surface(SurfaceRole::Chrome, "Chrome"),
                indicator_surface(SurfaceRole::Sidebar, "Sidebar"),
                indicator_surface(SurfaceRole::Panel, "Panel"),
                indicator_surface(SurfaceRole::Elevated, "Elevated"),
                indicator_surface(SurfaceRole::Canvas, "Canvas"),
                indicator_surface(SurfaceRole::Dialog, "Dialog"),
                indicator_surface(SurfaceRole::Popover, "Popover"),
            ]),
        ),
    ])
}

fn indicator_surface(role: SurfaceRole, label: &'static str) -> Element<'static, Message> {
    Panel::new(StatusIndicator::new(ToneRole::Warning, label))
        .role(role)
        .body_padding(6)
        .into()
}

fn surfaces() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Card",
            Card::new(
                column![
                    ntext::body_strong("Card"),
                    ntext::body("Compact content")
                ]
                .spacing(4),
            ),
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
            .body_padding(14)
            .width(Length::Fill),
        ),
        example_cell(
            "MetricCard · surface-free",
            MetricCard::new("Open issues", 128)
                .status(Badge::status("review").warning())
                .trend(ntext::body_small("+12 this week")),
        ),
    ])
}

fn metadata() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "KeyValueList · standalone",
            KeyValueList::new()
                .item(MetadataItem::new("Owner", "Design Systems"))
                .item(MetadataItem::new("Status", "Refreshing").status(ToneRole::Info))
                .item(MetadataItem::new("Commit", "9f31ad7").code_value("9f31ad7"))
                .item(MetadataItem::new("Version", "").custom_value(
                    MetadataTag::code("1.4.0-beta.2+build.7")
                ))
                .label_width(96.0),
        ),
        example_cell(
            "KeyValueList · host-owned surface",
            Card::new(
                column![
                    KeyValueList::new()
                        .item(MetadataItem::new(
                            "Long metadata label that is constrained",
                            "A wrapped plain value that remains readable in narrow inspectors",
                        ))
                        .label_width(72.0)
                        .fill_width(),
                    KeyValueList::new()
                        .item(MetadataItem::new("Invalid-width sanitation", "finite geometry"))
                        .item(MetadataItem::new("Empty status", " ").status(ToneRole::Warning))
                        .label_width(f32::NAN)
                        .fill_width(),
                ]
                .spacing(8),
            )
            .outlined(),
        ),
        example_cell(
            "DataRow · protected peer action",
            column![
                DataRow::new("Renderer")
                    .value("Iced")
                    .tone(ToneRole::Success)
                    .reserve_indicator(),
                DataRow::new("Long metadata key that pressures dense rows")
                    .value("Long value preserved for review")
                    .reserve_indicator()
                    .trailing(ActionGroup::new().action(
                        ContentAction::label("Inspect").on_press(Message::Noop)
                    )),
            ]
            .spacing(8),
        ),
        example_cell(
            "Static row vs selectable owner",
            column![
                DataRow::new("Static metadata").value("No row activation"),
                SelectableItem::new("Selectable owner")
                    .status_text(ToneRole::Info, "Ready")
                    .on_press(Message::Noop),
            ]
            .spacing(8),
        ),
        example_cell(
            "ControlSize rhythm · active density",
            variant_row([
                column![
                    KeyValueList::new().item(MetadataItem::new("Xs", "14 px")).xs(),
                    DataRow::new("Xs row").value("14 px").xs(),
                ]
                .spacing(4)
                .into(),
                column![
                    KeyValueList::new().item(MetadataItem::new("Sm", "14 px")).sm(),
                    DataRow::new("Sm row").value("14 px").sm(),
                ]
                .spacing(4)
                .into(),
                column![
                    KeyValueList::new().item(MetadataItem::new("Md", "14 px")).md(),
                    DataRow::new("Md row").value("14 px").md(),
                ]
                .spacing(4)
                .into(),
                column![
                    KeyValueList::new().item(MetadataItem::new("Lg", "14 px")).lg(),
                    DataRow::new("Lg row").value("14 px").lg(),
                ]
                .spacing(4)
                .into(),
            ]),
        ),
        example_cell(
            "Ultra-narrow DataRow allocation",
            container(
                column![
                    DataRow::new("Principal label that remains protected")
                        .value("Secondary yields first")
                        .leading(container(ntext::body("Fixed")).width(Length::Fixed(32.0)))
                        .trailing(ActionGroup::new().action(
                            ContentAction::icon(IconRole::ViewReveal, "Inspect")
                                .on_press(Message::Noop)
                        ))
                        .fill_width(),
                    DataRow::new("Fill peer")
                        .value("may collapse")
                        .trailing(container(ntext::body("Fill surplus")).width(Length::Fill))
                        .fill_width(),
                ]
                .spacing(4),
            )
            .width(Length::Fixed(180.0))
            .clip(true),
        ),
    ])
}

fn identity() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "InitialAvatar sizes and kinds",
            variant_row([
                row![InitialAvatar::new("Nive").entity().xs(), ntext::body("Nive")]
                    .spacing(6).align_y(Alignment::Center).into(),
                row![InitialAvatar::new("Élodie Martin").person().sm(), ntext::body("Élodie Martin")]
                    .spacing(6).align_y(Alignment::Center).into(),
                row![InitialAvatar::new("界 面").entity().accent().md(), ntext::body("界 面")]
                    .spacing(6).align_y(Alignment::Center).into(),
                row![InitialAvatar::new("Widget Gallery").person().accent().lg(), ntext::body("Widget Gallery")]
                    .spacing(6).align_y(Alignment::Center).into(),
            ]),
        ),
        example_cell(
            "Fallback and status outlines",
            variant_row([
                row![InitialAvatar::new("👩‍💻").person(), ntext::body("Emoji fallback")]
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .into(),
                row![InitialAvatar::new("").entity(), ntext::body("Empty identity fallback")]
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .into(),
                row![
                    InitialAvatar::new("Ada Lovelace")
                        .person()
                        .status(AvatarStatus::on_surface(
                            ToneRole::Success,
                            SurfaceRole::App,
                        )),
                    ntext::body("Ada Lovelace · available")
                ]
                .spacing(6)
                .align_y(Alignment::Center)
                .into(),
                row![
                    InitialAvatar::new("Interactive")
                        .status(AvatarStatus::on_interactive(ToneRole::Info)),
                    ntext::body("Interactive identity · info")
                ]
                .spacing(6)
                .align_y(Alignment::Center)
                .into(),
                row![
                    InitialAvatar::new("Static color").status(AvatarStatus::with_outline(
                        ToneRole::Warning,
                        Color::from_rgb8(246, 247, 251),
                    )),
                    ntext::body("Static host · warning")
                ]
                .spacing(6)
                .align_y(Alignment::Center)
                    .into(),
            ]),
        ),
        example_cell(
            "Technical metadata",
            variant_row([
                MetadataTag::code("1.4.0").into(),
                MetadataTag::code("1.4.0-beta.2+build.7").into(),
                container(MetadataTag::code("commit-9f31ad7c0ffee"))
                    .width(Length::Fixed(48.0))
                    .clip(true)
                    .into(),
                StatusIndicator::new(ToneRole::Info, "beta").into(),
                row![
                    MetadataTag::code("commit-9f31ad7c0ffee"),
                    ActionGroup::new().action(
                        ContentAction::icon(IconRole::EditCopy, "Copy identifier")
                            .on_press(Message::Noop)
                    )
                ]
                .spacing(6)
                .align_y(Alignment::Center)
                .into(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_review_matrices_build_with_canonical_public_apis() {
        let _: Element<'static, Message> = badges();
        let _: Element<'static, Message> = metadata();
        let _: Element<'static, Message> = identity();
    }
}
