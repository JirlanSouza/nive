use nive::prelude::*;
use nive::ui::theme::ToneRole;
use nive::ui::widgets::primitives::text as ntext;
use nive::widget::{column, row};

use crate::app::{Message, WidgetGallery};
use crate::catalog::PageId;
use crate::layout::{example_cell, section, variant_grid, variant_row};

const ICONS: &[IconRole] = &[
    IconRole::DialogError,
    IconRole::DialogWarning,
    IconRole::GoPrevious,
    IconRole::GoNext,
    IconRole::ActionConfirm,
    IconRole::DialogSuccess,
    IconRole::NiveDisclosureDown,
    IconRole::NiveDisclosureLeft,
    IconRole::NiveDisclosureRight,
    IconRole::NiveDisclosureUp,
    IconRole::WindowClose,
    IconRole::EditCopy,
    IconRole::EditModify,
    IconRole::ViewReveal,
    IconRole::ViewConceal,
    IconRole::Folder,
    IconRole::MailInbox,
    IconRole::DialogInformation,
    IconRole::OpenMenu,
    IconRole::ListRemove,
    IconRole::ViewMore,
    IconRole::ListAdd,
    IconRole::ViewRefresh,
    IconRole::EditFind,
    IconRole::PreferencesSystem,
    IconRole::EditDelete,
];

pub fn view(_app: &WidgetGallery) -> Element<'_, Message> {
    crate::app::page_shell(
        PageId::Icons,
        column![
            section("Sizes and colors", sizes()),
            section("Dense grid", dense_grid()),
        ]
        .spacing(18),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GalleryIconSymbol {
    BrandMark,
}

impl IconSource for GalleryIconSymbol {
    fn svg_bytes(self) -> &'static [u8] {
        match self {
            Self::BrandMark => include_bytes!("../../assets/icons/gallery-symbol.svg"),
        }
    }

    fn provider_slug(self) -> &'static str {
        match self {
            Self::BrandMark => "custom:gallery-symbol",
        }
    }
}

fn sizes() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Sizes",
            variant_row([
                Icon::role(IconRole::EditFind).xs().into(),
                Icon::role(IconRole::EditFind).sm().into(),
                Icon::role(IconRole::EditFind).md().into(),
                Icon::role(IconRole::EditFind).lg().into(),
                Icon::role(IconRole::EditFind).custom_size(28.0).into(),
            ]),
        ),
        example_cell(
            "Colors",
            variant_row([
                Icon::role(IconRole::DialogSuccess)
                    .color(theme::active().tone(ToneRole::Success).color)
                    .lg()
                    .into(),
                Icon::role(IconRole::DialogWarning)
                    .color(theme::active().tone(ToneRole::Warning).color)
                    .lg()
                    .into(),
                Icon::role(IconRole::EditDelete)
                    .color(theme::active().tone(ToneRole::Danger).color)
                    .lg()
                    .into(),
            ]),
        ),
        example_cell(
            "Symbol",
            variant_row([
                Icon::symbol(GalleryIconSymbol::BrandMark).lg().into(),
                Icon::role(IconRole::WindowClose).lg().into(),
            ]),
        ),
    ])
}

fn dense_grid() -> Element<'static, Message> {
    let mut rows = column![].spacing(12);

    for chunk in ICONS.chunks(7) {
        let mut row = row![].spacing(14).align_y(Alignment::Center);
        for icon in chunk {
            row = row.push(
                column![Icon::role(*icon).md(), ntext::caption(icon.canonical_name())]
                    .spacing(4)
                    .align_x(Alignment::Center)
                    .width(72),
            );
        }
        rows = rows.push(row);
    }

    rows.into()
}
