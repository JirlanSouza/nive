use std::borrow::Cow;
use std::rc::Rc;

use iced::{widget::container, Length, Padding};
use nive_ui::theme::{self, ControlSize, ToneRole};
use nive_ui::widgets::{SectionHeader, SectionHeaderAction, SectionHeaderStatus};
use nive_ui::{Element, IconRole};

use super::model::PanelHeaderBar;
use crate::layout::WorkbenchRegion;
use crate::panels::model::WorkbenchPanelEvent;

#[derive(Debug, Clone, Copy)]
pub(super) struct TrailingControlFlags {
    pub(super) restorable: bool,
    pub(super) maximizable: bool,
    pub(super) collapsible: bool,
    pub(super) closable: bool,
}

impl<'a, PanelId, ActionId> PanelHeaderBar<'a, PanelId, ActionId> {
    /// Builds a header bar directly from metadata.
    pub fn new(region: WorkbenchRegion, panel_id: PanelId, title: impl Into<Cow<'a, str>>) -> Self {
        Self {
            region,
            panel_id,
            title: title.into(),
            tooltip: None,
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
        let mut header = SectionHeader::new(self.title)
            .title_tooltip_maybe(self.tooltip)
            .size(size);
        if let Some(icon) = self.icon {
            header = header.icon(icon);
        }
        if let Some(badge) = self.badge {
            header = header.badge(badge);
        }
        if let Some(status) = self.status {
            header = header.status(SectionHeaderStatus::icon(
                tone_icon(status),
                status,
                tone_label(status),
            ));
        }
        header = header.trailing(trailing_controls(
            self.region,
            self.panel_id,
            self.actions,
            TrailingControlFlags {
                restorable: self.restorable,
                maximizable: self.maximizable,
                collapsible: self.collapsible,
                closable: self.closable,
            },
            mapper,
            size,
        ));

        container(header)
            .padding(Padding::ZERO.horizontal(theme::spacing().sm))
            .width(Length::Fill)
            .height(Length::Fixed(theme::control_metrics(size).height))
            .clip(true)
            .into()
    }
}

pub(super) fn trailing_controls<'a, PanelId, ActionId, Message>(
    region: WorkbenchRegion,
    panel_id: PanelId,
    actions: Vec<super::model::PanelAction<'a, ActionId>>,
    flags: TrailingControlFlags,
    mapper: Rc<dyn Fn(WorkbenchPanelEvent<PanelId, ActionId>) -> Message + 'a>,
    size: ControlSize,
) -> Element<'a, Message>
where
    PanelId: Clone + 'a,
    ActionId: Clone + 'a,
    Message: Clone + 'a,
{
    let has_app_actions = !actions.is_empty();
    let has_builtin = flags.restorable || flags.maximizable || flags.collapsible || flags.closable;
    let mut controls = Vec::new();

    for action in actions {
        let has_label = action.label.is_some();
        let label = action
            .label
            .clone()
            .unwrap_or_else(|| action.accessible_label.clone());
        let event = WorkbenchPanelEvent::Action {
            region,
            panel_id: panel_id.clone(),
            action_id: action.id,
        };
        let control = match action.icon {
            Some(icon) if has_label => SectionHeaderAction::icon_text(icon, label),
            Some(icon) => SectionHeaderAction::icon(icon).tooltip(action.accessible_label),
            None => SectionHeaderAction::text(label),
        }
        .disabled(action.disabled)
        .on_press_maybe((!action.disabled).then(|| mapper(event)));
        controls.push(control);
    }
    if has_app_actions && has_builtin {
        controls.push(SectionHeaderAction::separator());
    }
    if flags.restorable {
        controls.push(header_button(
            IconRole::ViewReveal,
            "Restore panel",
            mapper(WorkbenchPanelEvent::PanelRestoreRequested {
                region,
                panel_id: panel_id.clone(),
            }),
        ));
    }
    if flags.maximizable {
        controls.push(header_button(
            IconRole::NiveDisclosureUp,
            "Maximize panel",
            mapper(WorkbenchPanelEvent::MaximizeRequested {
                region,
                panel_id: panel_id.clone(),
            }),
        ));
    }
    if flags.collapsible {
        controls.push(header_button(
            IconRole::ViewConceal,
            "Collapse panel",
            mapper(WorkbenchPanelEvent::CollapseRequested {
                region,
                panel_id: panel_id.clone(),
            }),
        ));
    }
    if flags.closable {
        controls.push(header_button(
            IconRole::WindowClose,
            "Close panel",
            mapper(WorkbenchPanelEvent::CloseRequested { region, panel_id }),
        ));
    }

    SectionHeaderAction::group(controls, size)
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
