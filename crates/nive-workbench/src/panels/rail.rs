use nive_ui::theme::ControlSize;
use nive_ui::widgets::{RailSide, VerticalRail, VerticalRailItem};
use nive_ui::Element;

use super::model::{PanelRail, PanelRailItem};

impl<'a, PanelId, Message> PanelRail<'a, PanelId, Message>
where
    PanelId: Clone + 'a,
    Message: Clone + 'a,
{
    /// Builds a compact side rail.
    pub fn new(
        side: RailSide,
        items: impl IntoIterator<Item = PanelRailItem<'a, PanelId>>,
    ) -> Self {
        Self {
            side,
            items: items.into_iter().collect(),
            on_select: None,
        }
    }

    /// Maps item activation into app messages.
    pub fn on_select(mut self, mapper: impl Fn(PanelId) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(mapper));
        self
    }

    /// Renders the side rail.
    pub fn view(self) -> Element<'a, Message> {
        self.view_with_size(ControlSize::Sm)
    }

    pub(crate) fn view_with_size(self, size: ControlSize) -> Element<'a, Message> {
        let mut rail = VerticalRail::new(self.side)
            .size(size)
            .on_select_maybe(self.on_select);

        for item in self.items {
            let mut control = VerticalRailItem::new(item.id, item.label)
                .icon(item.icon)
                .selected(item.selected)
                .disabled(item.disabled);
            if let Some(badge) = item.badge {
                control = control.badge(badge);
            }
            rail = rail.push(control);
        }

        rail.into()
    }
}
