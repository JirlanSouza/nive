use std::{borrow::Cow, cell::Cell, rc::Rc};

use iced::{
    advanced::mouse,
    widget::{container, mouse_area, text, Column, Row, Space},
    Alignment, Length, Padding,
};

use super::style as menu_style;
use super::widget::{
    MenuBranch, MenuBranchHandle, MenuLevelContext, MenuList, MenuRowSpec, MenuSlot,
    MenuTrailingMeasure, MenuTrailingTrack,
};
use super::{
    MenuEntry, MenuEvent, MenuTrailing, MENU_COLUMN_GAP, MENU_ICON_SIZE, MENU_LIST_INSET,
    MENU_MAX_WIDTH, MENU_ROW_HEIGHT, MENU_ROW_PADDING_H, MENU_ROW_RADIUS, MENU_SEPARATOR_MARGIN,
};
use crate::theme::{choice::ChoicePersistentState, TypographyRole};
use crate::widgets::controls::CheckboxState;
use crate::widgets::display::measured_text::{EllipsisStrategy, MeasuredText};
use crate::widgets::navigation::command_palette::format_shortcut;
use crate::widgets::overlays::popover;
use crate::widgets::overlays::{PopoverInset, PopoverWidth};
use crate::widgets::primitives::{icon as icon_widget, text as text_widget, IconRef, IconRole};
use crate::Element;

pub(super) fn menu_level_content<'a, Message: Clone + 'a>(
    entries: Vec<MenuEntry<'a, Message>>,
    context: MenuLevelContext,
) -> Element<'a, MenuEvent<Message>> {
    let reserve_choice = entries
        .iter()
        .any(|entry| matches!(entry, MenuEntry::Checkbox(_) | MenuEntry::Radio(_)));
    let reserve_icon = entries.iter().any(|entry| match entry {
        MenuEntry::Command(command) => command.icon.is_some(),
        MenuEntry::Radio(radio) => radio.icon.is_some(),
        MenuEntry::Submenu(submenu) => submenu.icon.is_some(),
        MenuEntry::Checkbox(_) | MenuEntry::Separator => false,
    });
    let reserve_trailing = entries.iter().any(|entry| match entry {
        MenuEntry::Command(command) => command.shortcut.is_some(),
        MenuEntry::Checkbox(checkbox) => checkbox.shortcut.is_some(),
        MenuEntry::Radio(radio) => radio.annotation.is_some(),
        MenuEntry::Submenu(_) => true,
        MenuEntry::Separator => false,
    });
    let trailing_width = Rc::new(Cell::new(0.0));
    let mut content = Column::new().padding(MENU_LIST_INSET).width(Length::Fill);
    let mut slots = Vec::with_capacity(entries.len());

    let tracks = MenuRowTracks {
        choice: reserve_choice,
        icon: reserve_icon,
        trailing_width: reserve_trailing.then(|| trailing_width.clone()),
    };

    for entry in entries {
        let (spec, activation, label, trailing_measure) = match &entry {
            MenuEntry::Command(command) => {
                let activation = (!command.is_disabled())
                    .then_some(command.on_press.clone())
                    .flatten()
                    .map(|message| MenuEvent::Activate(message, command.dismiss_policy));
                (
                    MenuRowSpec {
                        persistent: ChoicePersistentState::Unselected,
                        eligible: activation.is_some(),
                        disabled: command.is_disabled(),
                        destructive: command.destructive,
                    },
                    activation,
                    command.label.to_string(),
                    command.shortcut.map(|shortcut| {
                        MenuTrailingMeasure::Text(
                            format_shortcut(&shortcut).into_owned(),
                            TypographyRole::CodeSmall,
                        )
                    }),
                )
            }
            MenuEntry::Checkbox(checkbox) => {
                let activation = (!checkbox.disabled)
                    .then(|| {
                        checkbox
                            .on_toggle
                            .as_ref()
                            .map(|callback| callback(checkbox.state.next()))
                    })
                    .flatten()
                    .map(|message| MenuEvent::Activate(message, checkbox.dismiss_policy));
                (
                    MenuRowSpec {
                        persistent: match checkbox.state {
                            CheckboxState::Unchecked => ChoicePersistentState::Unselected,
                            CheckboxState::Checked => ChoicePersistentState::Selected,
                            CheckboxState::Mixed => ChoicePersistentState::Mixed,
                        },
                        eligible: activation.is_some(),
                        disabled: checkbox.disabled,
                        destructive: false,
                    },
                    activation,
                    checkbox.label.to_string(),
                    checkbox.shortcut.map(|shortcut| {
                        MenuTrailingMeasure::Text(
                            format_shortcut(&shortcut).into_owned(),
                            TypographyRole::CodeSmall,
                        )
                    }),
                )
            }
            MenuEntry::Radio(radio) => {
                let activation = (!radio.disabled)
                    .then_some(radio.on_press.clone())
                    .flatten()
                    .map(|message| MenuEvent::Activate(message, radio.dismiss_policy));
                (
                    MenuRowSpec {
                        persistent: if radio.selected {
                            ChoicePersistentState::Selected
                        } else {
                            ChoicePersistentState::Unselected
                        },
                        eligible: activation.is_some(),
                        disabled: radio.disabled,
                        destructive: false,
                    },
                    activation,
                    radio.label.to_string(),
                    radio.annotation.as_ref().map(|annotation| {
                        MenuTrailingMeasure::Text(annotation.to_string(), TypographyRole::BodySmall)
                    }),
                )
            }
            MenuEntry::Submenu(submenu) => (
                MenuRowSpec {
                    persistent: ChoicePersistentState::Unselected,
                    eligible: !submenu.disabled,
                    disabled: submenu.disabled,
                    destructive: false,
                },
                None,
                submenu.label.to_string(),
                Some(MenuTrailingMeasure::Icon),
            ),
            MenuEntry::Separator => (
                MenuRowSpec {
                    persistent: ChoicePersistentState::Unselected,
                    eligible: false,
                    disabled: false,
                    destructive: false,
                },
                None,
                String::new(),
                None,
            ),
        };
        let logical_focus = Rc::new(Cell::new(false));
        let branch = matches!(&entry, MenuEntry::Submenu(_)).then(MenuBranchHandle::new);
        slots.push(if matches!(&entry, MenuEntry::Separator) {
            MenuSlot::separator()
        } else {
            MenuSlot::row(
                spec,
                activation.clone(),
                label,
                trailing_measure,
                logical_focus.clone(),
                branch.clone(),
            )
        });
        content = content.push(match entry {
            MenuEntry::Command(command) => menu_row(
                MenuRowContent {
                    choice_mark: None,
                    icon: command.icon,
                    label: command.label,
                    trailing: command
                        .shortcut
                        .map(|shortcut| MenuTrailing::Shortcut(format_shortcut(&shortcut))),
                },
                spec,
                activation,
                tracks.clone(),
                logical_focus,
            ),
            MenuEntry::Checkbox(checkbox) => {
                let mark = match checkbox.state {
                    CheckboxState::Unchecked => None,
                    CheckboxState::Checked => Some("✓"),
                    CheckboxState::Mixed => Some("−"),
                };
                menu_row(
                    MenuRowContent {
                        choice_mark: mark,
                        icon: None,
                        label: checkbox.label,
                        trailing: checkbox
                            .shortcut
                            .map(|shortcut| MenuTrailing::Shortcut(format_shortcut(&shortcut))),
                    },
                    spec,
                    activation,
                    tracks.clone(),
                    logical_focus,
                )
            }
            MenuEntry::Radio(radio) => menu_row(
                MenuRowContent {
                    choice_mark: radio.selected.then_some("●"),
                    icon: radio.icon,
                    label: radio.label,
                    trailing: radio.annotation.map(MenuTrailing::Annotation),
                },
                spec,
                activation,
                tracks.clone(),
                logical_focus,
            ),
            MenuEntry::Submenu(submenu) => {
                let branch = branch.expect("submenu branch handle");
                let row = menu_row(
                    MenuRowContent {
                        choice_mark: None,
                        icon: submenu.icon,
                        label: submenu.label,
                        trailing: Some(MenuTrailing::Submenu),
                    },
                    spec,
                    None,
                    tracks.clone(),
                    logical_focus,
                );
                let mut child = *submenu.child;
                if matches!(child.entries.last(), Some(MenuEntry::Separator)) {
                    child.entries.pop();
                }
                let child_context = context.child(&branch);
                let ensure_visible = child_context.ensure_visible();
                let child_content =
                    menu_level_content(std::mem::take(&mut child.entries), child_context);
                MenuBranch::new(
                    row,
                    popover::surface_with_ensure_visible(
                        child_content,
                        PopoverInset::EdgeToEdge,
                        Some(&ensure_visible),
                        PopoverWidth::Content,
                    ),
                    branch,
                    ensure_visible,
                )
                .into()
            }
            MenuEntry::Separator => menu_separator(),
        });
    }

    MenuList::new(
        container(content)
            .width(Length::Fill)
            .max_width(MENU_MAX_WIDTH),
        slots,
        reserve_choice,
        reserve_icon,
        trailing_width,
        context,
    )
    .into()
}

/// Which columns every row in a level reserves, so their contents line up.
///
/// Layout, not state — kept apart from [`MenuRowSpec`] for that reason, and
/// identical for every row in the level.
#[derive(Debug, Clone)]
pub(super) struct MenuRowTracks {
    pub(super) choice: bool,
    pub(super) icon: bool,
    pub(super) trailing_width: Option<Rc<Cell<f32>>>,
}

/// What a row shows, as opposed to [`MenuRowSpec`], which is what it is.
pub(super) struct MenuRowContent<'a> {
    pub(super) choice_mark: Option<&'static str>,
    pub(super) icon: Option<IconRef>,
    pub(super) label: Cow<'a, str>,
    pub(super) trailing: Option<MenuTrailing<'a>>,
}

pub(super) fn menu_row<'a, Message: Clone + 'a>(
    row: MenuRowContent<'a>,
    spec: MenuRowSpec,
    activation: Option<MenuEvent<Message>>,
    tracks: MenuRowTracks,
    logical_focus: Rc<Cell<bool>>,
) -> Element<'a, MenuEvent<Message>> {
    let MenuRowContent {
        choice_mark,
        icon,
        label,
        trailing,
    } = row;
    let mut content: Row<'a, MenuEvent<Message>, crate::theme::Theme, iced::Renderer> =
        Row::new().spacing(MENU_COLUMN_GAP);
    if tracks.choice {
        content = content.push(
            container(text(choice_mark.unwrap_or("")))
                .width(Length::Fixed(MENU_ICON_SIZE))
                .center_y(Length::Fill),
        );
    }
    if tracks.icon {
        let leading: Element<'a, MenuEvent<Message>> = match icon {
            Some(icon) => container(icon_widget::reference(icon).custom_size(MENU_ICON_SIZE))
                .width(Length::Fixed(MENU_ICON_SIZE))
                .center_y(Length::Fill)
                .into(),
            None => Space::new().width(Length::Fixed(MENU_ICON_SIZE)).into(),
        };
        content = content.push(leading);
    }
    content = content
        .push(
            container(
                MeasuredText::new_inherited(label, EllipsisStrategy::End, TypographyRole::Control)
                    .logical_focus_candidate(logical_focus),
            )
            .width(Length::Fill)
            .clip(true),
        )
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill);
    if let Some(width) = tracks.trailing_width {
        let trailing: Element<'a, MenuEvent<Message>> = match trailing {
            Some(MenuTrailing::Shortcut(label)) => text_widget::code_small(label).into(),
            Some(MenuTrailing::Annotation(annotation)) => {
                text_widget::body_small(annotation).into()
            }
            Some(MenuTrailing::Submenu) => icon_widget::role(IconRole::NiveDisclosureRight)
                .custom_size(MENU_ICON_SIZE)
                .into(),
            None => Space::new().into(),
        };
        content = content.push(MenuTrailingTrack::new(trailing, width));
    }

    // A row that can be activated or opened owns its cursor, the way tabs and
    // the overflow chevrons own theirs. Ineligible rows — explicitly disabled,
    // display-only, or the already committed choice — keep the default cursor,
    // matching the fact that they take no highlight either.
    let row = mouse_area(
        container(content)
            .style(menu_style::row_style(
                spec.selected(),
                spec.destructive,
                spec.disabled,
                MENU_ROW_RADIUS,
            ))
            .padding(Padding::ZERO.horizontal(MENU_ROW_PADDING_H))
            .height(Length::Fixed(MENU_ROW_HEIGHT))
            .width(Length::Fill),
    )
    .interaction(if spec.eligible {
        mouse::Interaction::Pointer
    } else {
        mouse::Interaction::None
    });
    match (!spec.disabled).then_some(activation).flatten() {
        Some(activation) => row.on_release(activation).into(),
        None => row.into(),
    }
}

pub(super) fn menu_separator<'a, Message: Clone + 'a>() -> Element<'a, MenuEvent<Message>> {
    container(Space::new().height(Length::Fixed(1.0)))
        .style(menu_style::separator_style())
        .height(Length::Fixed(1.0 + MENU_SEPARATOR_MARGIN * 2.0))
        .padding(Padding::ZERO.vertical(MENU_SEPARATOR_MARGIN))
        .width(Length::Fill)
        .into()
}
