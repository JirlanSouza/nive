use std::borrow::Cow;
use std::rc::Rc;

use nive_ui::theme::{ControlSize, ToneRole};
use nive_ui::widgets::{SectionHeader, SectionHeaderAction, SectionHeaderStatus};
use nive_ui::{Element, IconRole};

use super::model::PanelHeaderBar;
use crate::layout::WorkbenchRegion;
use crate::panels::model::WorkbenchPanelEvent;

impl<'a, PanelId, ActionId> PanelHeaderBar<'a, PanelId, ActionId> {
    /// Builds a header bar directly from metadata.
    pub fn new(region: WorkbenchRegion, panel_id: PanelId, title: impl Into<Cow<'a, str>>) -> Self {
        Self {
            region,
            panel_id,
            title: title.into(),
            icon: None,
            badge: None,
            status: None,
            actions: Vec::new(),
            collapsible: true,
            restorable: false,
            maximizable: true,
            closable: false,
        }
    }
}

impl<'a, PanelId, ActionId> PanelHeaderBar<'a, PanelId, ActionId>
where
    PanelId: Clone + 'a,
    ActionId: Clone + 'a,
{
    /// Renders the header bar.
    pub fn view<Message>(
        self,
        mapper: Rc<dyn Fn(WorkbenchPanelEvent<PanelId, ActionId>) -> Message + 'a>,
    ) -> Element<'a, Message>
    where
        Message: Clone + 'a,
    {
        self.view_with_size(mapper, ControlSize::Sm)
    }

    pub(crate) fn view_with_size<Message>(
        self,
        mapper: Rc<dyn Fn(WorkbenchPanelEvent<PanelId, ActionId>) -> Message + 'a>,
        size: ControlSize,
    ) -> Element<'a, Message>
    where
        Message: Clone + 'a,
    {
        let mut header = SectionHeader::new(self.title).size(size);
        if let Some(icon) = self.icon {
            header = header.icon(icon);
        }
        if let Some(badge) = self.badge {
            header = header.badge(badge);
        }
        if let Some(status) = self.status {
            header = header.status(SectionHeaderStatus::icon_label(
                tone_icon(status),
                tone_label(status),
                status,
            ));
        }

        for action in self.actions {
            let has_label = action.label.is_some();
            let label = action
                .label
                .unwrap_or_else(|| action.accessible_label.clone());
            let event = WorkbenchPanelEvent::Action {
                region: self.region,
                panel_id: self.panel_id.clone(),
                action_id: action.id.clone(),
            };
            let control = match action.icon {
                Some(icon) if has_label => SectionHeaderAction::icon_text(icon, label),
                Some(icon) => SectionHeaderAction::icon(icon).tooltip(action.accessible_label),
                None => SectionHeaderAction::text(label),
            };
            header = header.action(
                control
                    .disabled(action.disabled)
                    .on_press_maybe((!action.disabled).then(|| mapper(event))),
            );
        }

        if self.collapsible {
            header = header.action(header_button(
                IconRole::ViewConceal,
                "Collapse panel",
                mapper(WorkbenchPanelEvent::CollapseRequested {
                    region: self.region,
                    panel_id: self.panel_id.clone(),
                }),
            ));
        }
        if self.restorable {
            header = header.action(header_button(
                IconRole::ViewReveal,
                "Restore panel",
                mapper(WorkbenchPanelEvent::PanelRestoreRequested {
                    region: self.region,
                    panel_id: self.panel_id.clone(),
                }),
            ));
        }
        if self.maximizable {
            header = header.action(header_button(
                IconRole::NiveDisclosureUp,
                "Maximize panel",
                mapper(WorkbenchPanelEvent::MaximizeRequested {
                    region: self.region,
                    panel_id: self.panel_id.clone(),
                }),
            ));
        }
        if self.closable {
            header = header.action(header_button(
                IconRole::WindowClose,
                "Close panel",
                mapper(WorkbenchPanelEvent::CloseRequested {
                    region: self.region,
                    panel_id: self.panel_id,
                }),
            ));
        }

        header.into()
    }
}

fn header_button<'a, Message>(
    icon: IconRole,
    tooltip: &'a str,
    message: Message,
) -> SectionHeaderAction<'a, Message>
where
    Message: Clone + 'a,
{
    SectionHeaderAction::icon(icon)
        .tooltip(tooltip)
        .on_press(message)
}

fn tone_label(tone: ToneRole) -> &'static str {
    match tone {
        ToneRole::Neutral => "Info",
        ToneRole::Accent => "Active",
        ToneRole::Info => "Info",
        ToneRole::Success => "Ok",
        ToneRole::Warning => "Warning",
        ToneRole::Danger => "Error",
    }
}

fn tone_icon(tone: ToneRole) -> IconRole {
    match tone {
        ToneRole::Danger => IconRole::DialogError,
        ToneRole::Warning => IconRole::DialogWarning,
        ToneRole::Success => IconRole::DialogInformation,
        ToneRole::Accent | ToneRole::Info | ToneRole::Neutral => IconRole::DialogInformation,
    }
}
