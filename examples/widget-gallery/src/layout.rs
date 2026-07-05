use nive::prelude::*;
use nive::ui::theme::SurfaceRole;
use nive::ui::widgets::primitives::text as ntext;
use nive::widget::{column, row};

use crate::catalog::CatalogEntry;

pub fn page_shell<'a>(
    entry: &CatalogEntry,
    content: impl Into<Element<'a, crate::app::Message>>,
) -> Element<'a, crate::app::Message> {
    column![
        column![
            ntext::section_label(entry.family),
            ntext::heading(entry.title),
            ntext::body_small(entry.summary),
        ]
        .spacing(4),
        content.into(),
    ]
    .spacing(20)
    .width(Length::Fill)
    .into()
}

pub fn section<'a>(
    title: &'a str,
    content: impl Into<Element<'a, crate::app::Message>>,
) -> Element<'a, crate::app::Message> {
    column![ntext::title(title), content.into()]
        .spacing(12)
        .width(Length::Fill)
        .into()
}

pub fn variant_grid<'a>(
    cells: impl IntoIterator<Item = Element<'a, crate::app::Message>>,
) -> Element<'a, crate::app::Message> {
    let mut rows = column![].spacing(12).width(Length::Fill);
    let mut current = row![].spacing(12).width(Length::Fill);
    let mut count = 0;

    for cell in cells {
        current = current.push(container(cell).width(Length::FillPortion(1)));
        count += 1;

        if count % 3 == 0 {
            rows = rows.push(current);
            current = row![].spacing(12).width(Length::Fill);
        }
    }

    if count % 3 != 0 {
        rows = rows.push(current);
    }

    rows.into()
}

pub fn variant_row<'a>(
    cells: impl IntoIterator<Item = Element<'a, crate::app::Message>>,
) -> Element<'a, crate::app::Message> {
    row(cells.into_iter().collect::<Vec<_>>())
        .spacing(12)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
}

pub fn example_cell<'a>(
    label: &'a str,
    content: impl Into<Element<'a, crate::app::Message>>,
) -> Element<'a, crate::app::Message> {
    Panel::new(column![ntext::caption(label), content.into()].spacing(10))
        .role(SurfaceRole::Panel)
        .padding(14)
        .width(Length::Fill)
        .into()
}

pub fn sidebar_button<'a>(
    entry: &'static CatalogEntry,
    active: bool,
) -> Element<'a, crate::app::Message> {
    SelectableItem::new(entry.title)
        .selected(active)
        .trailing_text(entry.family)
        .on_press(crate::app::Message::Navigate(entry.id))
        .into()
}

pub fn swatch(color: Color, label: &'static str) -> Element<'static, crate::app::Message> {
    row![
        ColorSwatch::new(color).md(),
        ntext::caption(label).width(Length::Fill),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}
