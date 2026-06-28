use nive::prelude::*;
use nive::ui::theme::ToneRole;
use nive::ui::widgets::text as ntext;
use nive::widget::{column, row};

use crate::app::{Message, WidgetGallery};
use crate::catalog::PageId;
use crate::layout::{example_cell, section, variant_grid, variant_row};

const ICONS: &[IconName] = &[
    IconName::AlertCircle,
    IconName::AlertTriangle,
    IconName::ArrowLeft,
    IconName::ArrowRight,
    IconName::Check,
    IconName::CheckCircle,
    IconName::ChevronDown,
    IconName::ChevronLeft,
    IconName::ChevronRight,
    IconName::ChevronUp,
    IconName::Close,
    IconName::Copy,
    IconName::Edit,
    IconName::Eye,
    IconName::EyeOff,
    IconName::Folder,
    IconName::Inbox,
    IconName::Info,
    IconName::Menu,
    IconName::Minus,
    IconName::MoreHorizontal,
    IconName::Plus,
    IconName::RefreshCw,
    IconName::Search,
    IconName::Settings,
    IconName::Trash,
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

fn sizes() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Sizes",
            variant_row([
                Icon::new(IconName::Search).xs().into(),
                Icon::new(IconName::Search).sm().into(),
                Icon::new(IconName::Search).md().into(),
                Icon::new(IconName::Search).lg().into(),
                Icon::new(IconName::Search).size(28.0).into(),
            ]),
        ),
        example_cell(
            "Colors",
            variant_row([
                Icon::new(IconName::CheckCircle)
                    .color(theme::active().tone(ToneRole::Success).color)
                    .lg()
                    .into(),
                Icon::new(IconName::AlertTriangle)
                    .color(theme::active().tone(ToneRole::Warning).color)
                    .lg()
                    .into(),
                Icon::new(IconName::Trash)
                    .color(theme::active().tone(ToneRole::Danger).color)
                    .lg()
                    .into(),
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
                column![Icon::new(*icon).md(), ntext::caption(icon.provider_slug())]
                    .spacing(4)
                    .align_x(Alignment::Center)
                    .width(72),
            );
        }
        rows = rows.push(row);
    }

    rows.into()
}
