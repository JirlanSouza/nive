use std::borrow::Cow;
use std::rc::Rc;

use iced::{widget::container, Length, Padding};
use nive_ui::theme::{self, ControlSize, ToneRole};
use nive_ui::widgets::BadgeContent;
use nive_ui::widgets::{SectionHeader, SectionHeaderAction, SectionHeaderStatus};
use nive_ui::{Element, IconRef, IconRole};

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
        let status_badge_present = self.badge.as_ref().is_some_and(
            |badge| matches!(badge, BadgeContent::Status(label) if !label.trim().is_empty()),
        );
        if let Some(badge) = self.badge {
            header = header.badge_content(badge);
        }
        if let Some(status) = self
            .status
            .filter(|status| !status.is_empty() && !status_badge_present)
        {
            let (tone, label) = status.into_parts();
            header = header.status(SectionHeaderStatus::icon_label(
                tone_icon(tone),
                label,
                tone,
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

/// The single maximize control a panel header shows.
///
/// One button carries both directions: a maximized host offers restore, and any
/// other maximizable host offers maximize.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MaximizeToggle {
    Maximize,
    Restore,
}

impl MaximizeToggle {
    pub(super) fn resolve(restorable: bool, maximizable: bool) -> Option<Self> {
        if restorable {
            Some(Self::Restore)
        } else if maximizable {
            Some(Self::Maximize)
        } else {
            None
        }
    }

    fn icon(self) -> IconRole {
        match self {
            Self::Restore => IconRole::ViewRestore,
            Self::Maximize => IconRole::ViewMaximize,
        }
    }

    fn tooltip(self) -> &'static str {
        match self {
            Self::Restore => "Restore panel",
            Self::Maximize => "Maximize panel",
        }
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
    if let Some(toggle) = MaximizeToggle::resolve(flags.restorable, flags.maximizable) {
        let event = match toggle {
            MaximizeToggle::Restore => WorkbenchPanelEvent::PanelRestoreRequested {
                region,
                panel_id: panel_id.clone(),
            },
            MaximizeToggle::Maximize => WorkbenchPanelEvent::MaximizeRequested {
                region,
                panel_id: panel_id.clone(),
            },
        };
        controls.push(header_button(
            toggle.icon(),
            toggle.tooltip(),
            mapper(event),
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
    icon: impl Into<IconRef>,
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

fn tone_icon(tone: ToneRole) -> IconRole {
    match tone {
        ToneRole::Danger => IconRole::DialogError,
        ToneRole::Warning => IconRole::DialogWarning,
        ToneRole::Success => IconRole::DialogInformation,
        ToneRole::Accent | ToneRole::Info | ToneRole::Neutral => IconRole::DialogInformation,
    }
}

#[cfg(test)]
mod header_icon_tests {
    use super::*;

    #[test]
    fn one_control_carries_both_maximize_directions() {
        // `restorable` is only set while the host is maximized.
        assert_eq!(
            MaximizeToggle::resolve(true, true),
            Some(MaximizeToggle::Restore)
        );
        assert_eq!(
            MaximizeToggle::resolve(false, true),
            Some(MaximizeToggle::Maximize)
        );
        assert_eq!(MaximizeToggle::resolve(false, false), None);
        // A maximized host restores even if the panel opted out of maximizing.
        assert_eq!(
            MaximizeToggle::resolve(true, false),
            Some(MaximizeToggle::Restore)
        );
    }

    #[test]
    fn each_direction_carries_its_own_icon_and_name() {
        assert_eq!(MaximizeToggle::Maximize.icon(), IconRole::ViewMaximize);
        assert_eq!(MaximizeToggle::Restore.icon(), IconRole::ViewRestore);
        assert_ne!(
            MaximizeToggle::Maximize.tooltip(),
            MaximizeToggle::Restore.tooltip()
        );
    }
}
