use std::rc::Rc;

use nive_ui::widgets::{RailSide, VerticalRail, VerticalRailItem};
use nive_ui::Element;

use super::model::{PanelRail, PanelRailItem};
use crate::layout::WorkbenchRegion;

impl<'a, PanelId, Message> PanelRail<'a, PanelId, Message>
where
    PanelId: Clone + 'a,
    Message: Clone + 'a,
{
    /// Builds a compact side rail.
    pub fn new(
        region: WorkbenchRegion,
        items: impl IntoIterator<Item = PanelRailItem<'a, PanelId>>,
    ) -> Self {
        Self {
            region,
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
        let side = match self.region {
            WorkbenchRegion::Right => RailSide::Right,
            _ => RailSide::Left,
        };
        let mut rail = VerticalRail::new(side).sm();
        let mapper: Option<Rc<dyn Fn(PanelId) -> Message + 'a>> = self.on_select.map(Rc::from);

        for item in self.items {
            let mut control = VerticalRailItem::new(item.label)
                .icon(item.icon)
                .selected(item.selected)
                .disabled(item.disabled);
            if let Some(status) = item.status {
                control = control.status(status);
            }
            if let Some(badge) = item.badge {
                control = control.badge(badge);
            }
            if let Some(mapper) = mapper.as_ref() {
                control = control.on_press(mapper(item.id.clone()));
            }
            rail = rail.push(control);
        }

        rail.into()
    }
}
