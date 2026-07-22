use std::{borrow::Cow, cell::Cell, rc::Rc};

use iced::{
    widget::{container, mouse_area, text, Column, Row, Space},
    Alignment, Length, Padding,
};

use super::style as menu_style;
use super::widget::{
    MenuBranch, MenuBranchHandle, MenuLevelContext, MenuList, MenuSlot, MenuTrailingMeasure,
    MenuTrailingTrack,
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
use crate::widgets::primitives::{icon as icon_widget, text as text_widget, IconRole};
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

    for entry in entries {
        let (eligible, activation, label, trailing_measure, persistent, explicitly_disabled) =
            match &entry {
                MenuEntry::Command(command) => {
                    let activation = (!command.is_disabled())
                        .then_some(command.on_press.clone())
                        .flatten()
                        .map(|message| MenuEvent::Activate(message, command.dismiss_policy));
                    (
                        activation.is_some(),
                        activation,
                        command.label.to_string(),
                        command.shortcut.map(|shortcut| {
                            MenuTrailingMeasure::Text(
                                format_shortcut(&shortcut).into_owned(),
                                TypographyRole::CodeSmall,
                            )
                        }),
                        ChoicePersistentState::Unselected,
                        command.is_disabled(),
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
                        activation.is_some(),
                        activation,
                        checkbox.label.to_string(),
                        checkbox.shortcut.map(|shortcut| {
                            MenuTrailingMeasure::Text(
                                format_shortcut(&shortcut).into_owned(),
                                TypographyRole::CodeSmall,
                            )
                        }),
                        match checkbox.state {
                            CheckboxState::Unchecked => ChoicePersistentState::Unselected,
                            CheckboxState::Checked => ChoicePersistentState::Selected,
                            CheckboxState::Mixed => ChoicePersistentState::Mixed,
                        },
                        checkbox.disabled,
                    )
                }
                MenuEntry::Radio(radio) => {
                    let activation = (!radio.disabled)
                        .then_some(radio.on_press.clone())
                        .flatten()
                        .map(|message| MenuEvent::Activate(message, radio.dismiss_policy));
                    (
                        activation.is_some(),
                        activation,
                        radio.label.to_string(),
                        radio.annotation.as_ref().map(|annotation| {
                            MenuTrailingMeasure::Text(
                                annotation.to_string(),
                                TypographyRole::BodySmall,
                            )
                        }),
                        if radio.selected {
                            ChoicePersistentState::Selected
                        } else {
                            ChoicePersistentState::Unselected
                        },
                        radio.disabled,
                    )
                }
                MenuEntry::Submenu(submenu) => (
                    !submenu.disabled,
                    None,
                    submenu.label.to_string(),
                    Some(MenuTrailingMeasure::Icon),
                    ChoicePersistentState::Unselected,
                    submenu.disabled,
                ),
                MenuEntry::Separator => (
                    false,
                    None,
                    String::new(),
                    None,
                    ChoicePersistentState::Unselected,
                    false,
                ),
            };
        let logical_focus = Rc::new(Cell::new(false));
        let branch = matches!(&entry, MenuEntry::Submenu(_)).then(MenuBranchHandle::new);
        slots.push(if matches!(&entry, MenuEntry::Separator) {
            MenuSlot::separator()
        } else {
            MenuSlot::row(
                eligible,
                activation.clone(),
                label,
                trailing_measure,
                persistent,
                explicitly_disabled,
                logical_focus.clone(),
                branch.clone(),
            )
        });
        content = content.push(match entry {
            MenuEntry::Command(command) => {
                let disabled = command.is_disabled();
                menu_row(
                    None,
                    command.icon,
                    command.label,
                    command
                        .shortcut
                        .map(|shortcut| MenuTrailing::Shortcut(format_shortcut(&shortcut))),
                    false,
                    command.destructive,
                    disabled,
                    activation,
                    reserve_choice,
                    reserve_icon,
                    logical_focus,
                    reserve_trailing.then(|| trailing_width.clone()),
                )
            }
            MenuEntry::Checkbox(checkbox) => {
                let mark = match checkbox.state {
                    CheckboxState::Unchecked => None,
                    CheckboxState::Checked => Some("✓"),
                    CheckboxState::Mixed => Some("−"),
                };
                menu_row(
                    mark,
                    None,
                    checkbox.label,
                    checkbox
                        .shortcut
                        .map(|shortcut| MenuTrailing::Shortcut(format_shortcut(&shortcut))),
                    checkbox.state != CheckboxState::Unchecked,
                    false,
                    checkbox.disabled,
                    activation,
                    reserve_choice,
                    reserve_icon,
                    logical_focus,
                    reserve_trailing.then(|| trailing_width.clone()),
                )
            }
            MenuEntry::Radio(radio) => menu_row(
                radio.selected.then_some("●"),
                radio.icon,
                radio.label,
                radio.annotation.map(MenuTrailing::Annotation),
                radio.selected,
                false,
                radio.disabled,
                activation,
                reserve_choice,
                reserve_icon,
                logical_focus,
                reserve_trailing.then(|| trailing_width.clone()),
            ),
            MenuEntry::Submenu(submenu) => {
                let branch = branch.expect("submenu branch handle");
                let row = menu_row(
                    None,
                    submenu.icon,
                    submenu.label,
                    Some(MenuTrailing::Submenu),
                    false,
                    false,
                    submenu.disabled,
                    None,
                    reserve_choice,
                    reserve_icon,
                    logical_focus,
                    reserve_trailing.then(|| trailing_width.clone()),
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

#[allow(clippy::too_many_arguments)]
pub(super) fn menu_row<'a, Message: Clone + 'a>(
    choice_mark: Option<&'static str>,
    icon: Option<IconRole>,
    label: Cow<'a, str>,
    trailing: Option<MenuTrailing<'a>>,
    selected: bool,
    destructive: bool,
    disabled: bool,
    activation: Option<MenuEvent<Message>>,
    reserve_choice: bool,
    reserve_icon: bool,
    logical_focus: Rc<Cell<bool>>,
    trailing_width: Option<Rc<Cell<f32>>>,
) -> Element<'a, MenuEvent<Message>> {
    let mut content: Row<'a, MenuEvent<Message>, crate::theme::Theme, iced::Renderer> =
        Row::new().spacing(MENU_COLUMN_GAP);
    if reserve_choice {
        content = content.push(
            container(text(choice_mark.unwrap_or("")))
                .width(Length::Fixed(MENU_ICON_SIZE))
                .center_y(Length::Fill),
        );
    }
    if reserve_icon {
        let leading: Element<'a, MenuEvent<Message>> = match icon {
            Some(icon) => container(icon_widget::role(icon).custom_size(MENU_ICON_SIZE))
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
    if let Some(width) = trailing_width {
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

    let row = mouse_area(
        container(content)
            .style(menu_style::row_style(
                selected,
                destructive,
                disabled,
                MENU_ROW_RADIUS,
            ))
            .padding(Padding::ZERO.horizontal(MENU_ROW_PADDING_H))
            .height(Length::Fixed(MENU_ROW_HEIGHT))
            .width(Length::Fill),
    );
    match (!disabled).then_some(activation).flatten() {
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
