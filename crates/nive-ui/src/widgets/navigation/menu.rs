use std::{borrow::Cow, cell::Cell, rc::Rc};

use iced::{
    widget::{container, mouse_area, text, Column, Row, Space},
    Alignment, Length, Padding,
};
use nive_core::{Action, ActionId, ShortcutBinding};

use super::command_palette::format_shortcut;
mod relay;
pub(crate) mod style;
mod widget;

use self::relay::MessageRelay;
use self::style as menu_style;
use self::widget::{
    MenuBranch, MenuBranchHandle, MenuLevelContext, MenuList, MenuSlot, MenuTrailingMeasure,
    MenuTrailingTrack,
};
use crate::theme::{choice::ChoicePersistentState, TypographyRole};
use crate::widgets::controls::CheckboxState;
use crate::widgets::display::measured_text::{EllipsisStrategy, MeasuredText};
use crate::widgets::overlays::popover;
use crate::widgets::overlays::{
    Popover, PopoverCollision, PopoverFocusPolicy, PopoverInset, PopoverPlacement,
};
use crate::widgets::primitives::{icon as icon_widget, text as text_widget, IconRole};
use crate::Element;

const MENU_MIN_WIDTH: f32 = 180.0;
const MENU_MAX_WIDTH: f32 = 320.0;
pub(crate) const MENU_LIST_INSET: f32 = 4.0;
pub(crate) const MENU_ROW_HEIGHT: f32 = 28.0;
pub(crate) const MENU_ROW_PADDING_H: f32 = 8.0;
pub(crate) const MENU_ROW_RADIUS: f32 = 4.0;
pub(crate) const MENU_ICON_SIZE: f32 = 16.0;
const MENU_SEPARATOR_MARGIN: f32 = 4.0;
pub(crate) const MENU_COLUMN_GAP: f32 = 8.0;

/// Whether activating a Menu leaf requests closure of the complete chain.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuDismissPolicy {
    #[default]
    DismissAll,
    KeepOpen,
}

/// A canonical anchored Menu with fluent, category-specific entries.
pub struct Menu<'a, Message> {
    trigger: Element<'a, Message>,
    entries: Vec<MenuEntry<'a, Message>>,
    open: bool,
    on_dismiss: Option<Message>,
    placement: PopoverPlacement,
    collision: PopoverCollision,
    match_anchor_width: bool,
}

/// A leaf command in a [`Menu`].
pub struct MenuCommand<'a, Message> {
    id: Option<ActionId>,
    label: Cow<'a, str>,
    icon: Option<IconRole>,
    shortcut: Option<ShortcutBinding>,
    destructive: bool,
    disabled: bool,
    source_disabled: bool,
    on_press: Option<Message>,
    dismiss_policy: MenuDismissPolicy,
}

/// A controlled tri-state checkbox leaf in a [`Menu`].
pub struct MenuCheckbox<'a, Message> {
    label: Cow<'a, str>,
    state: CheckboxState,
    shortcut: Option<ShortcutBinding>,
    disabled: bool,
    on_toggle: Option<Box<dyn Fn(CheckboxState) -> Message + 'a>>,
    dismiss_policy: MenuDismissPolicy,
}

/// One application-valued option in a [`MenuRadioGroup`].
pub struct MenuRadioOption<'a, T> {
    value: T,
    label: Cow<'a, str>,
    icon: Option<IconRole>,
    annotation: Option<Cow<'a, str>>,
    disabled: bool,
}

/// A controlled application-valued radio group in a [`Menu`].
pub struct MenuRadioGroup<'a, T, Message> {
    selected: Option<T>,
    options: Vec<MenuRadioOption<'a, T>>,
    on_select: Option<Box<dyn Fn(T) -> Message + 'a>>,
    dismiss_policy: MenuDismissPolicy,
}

/// A branch entry whose child is another canonical Menu.
pub struct MenuSubmenu<'a, Message> {
    label: Cow<'a, str>,
    icon: Option<IconRole>,
    disabled: bool,
    child: Box<Menu<'a, Message>>,
}

enum MenuEntry<'a, Message> {
    Command(MenuCommand<'a, Message>),
    Checkbox(MenuCheckbox<'a, Message>),
    Radio(MenuRadioRow<'a, Message>),
    Submenu(MenuSubmenu<'a, Message>),
    Separator,
}

#[derive(Clone)]
enum MenuEvent<Message> {
    Activate(Message, MenuDismissPolicy),
}

struct MenuRadioRow<'a, Message> {
    label: Cow<'a, str>,
    icon: Option<IconRole>,
    annotation: Option<Cow<'a, str>>,
    selected: bool,
    disabled: bool,
    on_press: Option<Message>,
    dismiss_policy: MenuDismissPolicy,
}

enum MenuTrailing<'a> {
    Shortcut(Cow<'a, str>),
    Annotation(Cow<'a, str>),
    Submenu,
}

impl<'a, Message: Clone + 'a> Menu<'a, Message> {
    pub fn new(trigger: impl Into<Element<'a, Message>>) -> Self {
        Self {
            trigger: trigger.into(),
            entries: Vec::new(),
            open: false,
            on_dismiss: None,
            placement: PopoverPlacement::BottomStart,
            collision: PopoverCollision::FlipAndShift,
            match_anchor_width: false,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.on_dismiss = Some(message);
        self
    }

    pub fn on_dismiss_maybe(mut self, message: Option<Message>) -> Self {
        self.on_dismiss = message;
        self
    }

    pub fn placement(mut self, placement: PopoverPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn collision(mut self, collision: PopoverCollision) -> Self {
        self.collision = collision;
        self
    }

    pub fn match_anchor_width(mut self) -> Self {
        self.match_anchor_width = true;
        self
    }

    pub fn command(mut self, command: MenuCommand<'a, Message>) -> Self {
        self.entries.push(MenuEntry::Command(command));
        self
    }

    pub fn checkbox(mut self, checkbox: MenuCheckbox<'a, Message>) -> Self {
        self.entries.push(MenuEntry::Checkbox(checkbox));
        self
    }

    pub fn radio_group<T>(mut self, group: MenuRadioGroup<'a, T, Message>) -> Self
    where
        T: Clone + Eq,
    {
        let valid = group.has_unique_values();
        let MenuRadioGroup {
            selected,
            options,
            on_select,
            dismiss_policy,
        } = group;

        for option in options {
            let is_selected = selected.as_ref() == Some(&option.value);
            let on_press = (valid && !option.disabled && !is_selected)
                .then(|| {
                    on_select
                        .as_ref()
                        .map(|callback| callback(option.value.clone()))
                })
                .flatten();
            self.entries.push(MenuEntry::Radio(MenuRadioRow {
                label: option.label,
                icon: option.icon,
                annotation: option.annotation,
                selected: is_selected,
                disabled: option.disabled,
                on_press,
                dismiss_policy,
            }));
        }

        self
    }

    pub fn submenu(mut self, submenu: MenuSubmenu<'a, Message>) -> Self {
        self.entries.push(MenuEntry::Submenu(submenu));
        self
    }

    pub fn separator(mut self) -> Self {
        if !self.entries.is_empty() && !matches!(self.entries.last(), Some(MenuEntry::Separator)) {
            self.entries.push(MenuEntry::Separator);
        }
        self
    }

    fn into_element(mut self) -> Element<'a, Message> {
        let context = MenuLevelContext::root();
        let ensure_visible = context.ensure_visible();
        let content = self.take_content_with_context(context);
        let mut popover = Popover::new(self.trigger)
            .content(content)
            .open(self.open)
            .placement(self.placement)
            .collision(self.collision)
            .inset(PopoverInset::EdgeToEdge)
            .focus_policy(PopoverFocusPolicy::FocusFirst)
            .on_dismiss_maybe(self.on_dismiss)
            .ensure_visible(ensure_visible);

        popover = if self.match_anchor_width {
            popover.match_anchor_width()
        } else {
            popover.content_width()
        };

        popover.into()
    }

    pub(super) fn into_content(mut self) -> Element<'a, Message> {
        self.take_content()
    }

    fn take_content(&mut self) -> Element<'a, Message> {
        self.take_content_with_context(MenuLevelContext::root())
    }

    fn take_content_with_context(&mut self, context: MenuLevelContext) -> Element<'a, Message> {
        if matches!(self.entries.last(), Some(MenuEntry::Separator)) {
            self.entries.pop();
        }
        let dismiss = self.on_dismiss.clone();
        MessageRelay::new(
            menu_level_content(std::mem::take(&mut self.entries), context),
            move |event, shell| match event {
                MenuEvent::Activate(message, policy) => {
                    shell.publish(message);
                    if policy == MenuDismissPolicy::DismissAll {
                        if let Some(dismiss) = dismiss.clone() {
                            shell.publish(dismiss);
                        }
                    }
                }
            },
        )
        .into()
    }
}

impl<'a, Message: Clone + 'a> From<Menu<'a, Message>> for Element<'a, Message> {
    fn from(menu: Menu<'a, Message>) -> Self {
        menu.into_element()
    }
}

impl<'a, Message: Clone> MenuCommand<'a, Message> {
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id: None,
            label: label.into(),
            icon: None,
            shortcut: None,
            destructive: false,
            disabled: false,
            source_disabled: false,
            on_press: None,
            dismiss_policy: MenuDismissPolicy::default(),
        }
    }

    /// Projects the canonical command semantics from a shared action.
    pub fn from_action(action: &Action<Message>) -> Self {
        Self {
            id: Some(action.id()),
            label: Cow::Owned(action.label().to_owned()),
            icon: None,
            shortcut: action.shortcut_binding().copied(),
            destructive: false,
            disabled: false,
            source_disabled: !action.is_enabled(),
            on_press: action.activate(),
            dismiss_policy: MenuDismissPolicy::default(),
        }
    }

    pub fn id(&self) -> Option<ActionId> {
        self.id
    }

    pub fn label(&self) -> &str {
        self.label.as_ref()
    }

    pub fn shortcut_binding(&self) -> Option<ShortcutBinding> {
        self.shortcut
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled || self.source_disabled
    }

    pub fn icon(mut self, icon: IconRole) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn shortcut(mut self, shortcut: ShortcutBinding) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    pub fn dismiss_policy(mut self, policy: MenuDismissPolicy) -> Self {
        self.dismiss_policy = policy;
        self
    }
}

impl<'a, Message> MenuCheckbox<'a, Message> {
    pub fn new(label: impl Into<Cow<'a, str>>, state: CheckboxState) -> Self {
        Self {
            label: label.into(),
            state,
            shortcut: None,
            disabled: false,
            on_toggle: None,
            dismiss_policy: MenuDismissPolicy::default(),
        }
    }

    pub fn shortcut(mut self, shortcut: ShortcutBinding) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_toggle(mut self, callback: impl Fn(CheckboxState) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(callback));
        self
    }

    pub fn on_toggle_maybe(
        mut self,
        callback: Option<impl Fn(CheckboxState) -> Message + 'a>,
    ) -> Self {
        self.on_toggle = callback.map(|callback| Box::new(callback) as _);
        self
    }

    pub fn dismiss_policy(mut self, policy: MenuDismissPolicy) -> Self {
        self.dismiss_policy = policy;
        self
    }
}

impl<'a, T> MenuRadioOption<'a, T> {
    pub fn new(value: T, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            value,
            label: label.into(),
            icon: None,
            annotation: None,
            disabled: false,
        }
    }

    pub fn icon(mut self, icon: IconRole) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn annotation(mut self, annotation: impl Into<Cow<'a, str>>) -> Self {
        self.annotation = Some(annotation.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<'a, T: Clone + Eq, Message> MenuRadioGroup<'a, T, Message> {
    pub fn new(selected: Option<T>) -> Self {
        Self {
            selected,
            options: Vec::new(),
            on_select: None,
            dismiss_policy: MenuDismissPolicy::default(),
        }
    }

    pub fn option(mut self, option: MenuRadioOption<'a, T>) -> Self {
        self.options.push(option);
        self
    }

    pub fn on_select(mut self, callback: impl Fn(T) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(callback));
        self
    }

    pub fn on_select_maybe(mut self, callback: Option<impl Fn(T) -> Message + 'a>) -> Self {
        self.on_select = callback.map(|callback| Box::new(callback) as _);
        self
    }

    pub fn dismiss_policy(mut self, policy: MenuDismissPolicy) -> Self {
        self.dismiss_policy = policy;
        self
    }

    pub fn has_unique_values(&self) -> bool {
        self.options.iter().enumerate().all(|(index, option)| {
            self.options[..index]
                .iter()
                .all(|previous| previous.value != option.value)
        })
    }
}

impl<'a, Message> MenuSubmenu<'a, Message> {
    pub fn new(label: impl Into<Cow<'a, str>>, child: Menu<'a, Message>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            disabled: false,
            child: Box::new(child),
        }
    }

    pub fn icon(mut self, icon: IconRole) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

fn menu_level_content<'a, Message: Clone + 'a>(
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
fn menu_row<'a, Message: Clone + 'a>(
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

fn menu_separator<'a, Message: Clone + 'a>() -> Element<'a, MenuEvent<Message>> {
    container(Space::new().height(Length::Fixed(1.0)))
        .style(menu_style::separator_style())
        .height(Length::Fixed(1.0 + MENU_SEPARATOR_MARGIN * 2.0))
        .padding(Padding::ZERO.vertical(MENU_SEPARATOR_MARGIN))
        .width(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use iced::{
        advanced::mouse,
        keyboard::{self, key},
        touch, Event, Point, Size,
    };
    use nive_core::ShortcutBinding;

    use crate::test_support::{layout as widget_layout, WidgetHarness};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Message {
        Save,
        Toggle(CheckboxState),
        Select(u8),
        Dismiss,
    }

    #[test]
    fn action_projection_preserves_command_semantics_and_ui_decoration() {
        let action = Action::new("file.save", "Save", Message::Save)
            .shortcut(ShortcutBinding::primary_character('s'));
        let command = MenuCommand::from_action(&action)
            .icon(IconRole::ActionConfirm)
            .destructive()
            .dismiss_policy(MenuDismissPolicy::KeepOpen);

        assert_eq!(command.id(), Some(ActionId::new("file.save")));
        assert_eq!(command.label(), "Save");
        assert_eq!(
            command.shortcut_binding(),
            Some(ShortcutBinding::primary_character('s'))
        );
        assert!(!command.is_disabled());
        assert_eq!(command.on_press, Some(Message::Save));
        assert_eq!(command.icon, Some(IconRole::ActionConfirm));
        assert!(command.destructive);
        assert_eq!(command.dismiss_policy, MenuDismissPolicy::KeepOpen);
    }

    #[test]
    fn disabled_action_cannot_be_reenabled_by_menu_decoration() {
        let action = Action::new("file.save", "Save", Message::Save).disabled();
        let command = MenuCommand::from_action(&action).disabled(false);

        assert!(command.is_disabled());
        assert_eq!(command.on_press, None);
    }

    #[test]
    fn fluent_categories_build_one_anchored_menu() {
        let child = Menu::new(Space::new()).command(MenuCommand::new("Child"));
        let menu: Menu<'_, Message> = Menu::new(Space::new())
            .open(true)
            .on_dismiss(Message::Dismiss)
            .command(MenuCommand::new("Save").on_press(Message::Save))
            .checkbox(
                MenuCheckbox::new("Pinned", CheckboxState::Unchecked).on_toggle(Message::Toggle),
            )
            .radio_group(
                MenuRadioGroup::new(Some(1))
                    .option(MenuRadioOption::new(1, "One"))
                    .option(MenuRadioOption::new(2, "Two"))
                    .on_select(Message::Select),
            )
            .separator()
            .submenu(MenuSubmenu::new("More", child));

        assert_eq!(menu.entries.len(), 6);
        let _: Element<'_, Message> = menu.into();
    }

    #[test]
    fn separators_normalize_and_duplicate_radio_values_are_inert() {
        let duplicate_group = MenuRadioGroup::<_, Message>::new(None)
            .option(MenuRadioOption::new(1, "One"))
            .option(MenuRadioOption::new(1, "Duplicate"))
            .on_select(Message::Select);
        assert!(!duplicate_group.has_unique_values());

        let invalid = Menu::new(Space::new())
            .radio_group(duplicate_group)
            .into_content();
        let mut invalid = WidgetHarness::new(invalid, Size::new(320.0, 120.0));
        assert!(invalid.bounds().width.is_finite());
        assert!(invalid.bounds().height.is_finite());
        assert_eq!(invalid.focused_count().total, 0);
        invalid.set_cursor(Point::new(12.0, 12.0));
        invalid.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));
        let released = invalid.update(Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        )));
        assert!(released.messages.is_empty());

        let menu: Menu<'_, Message> = Menu::new(Space::new())
            .separator()
            .command(MenuCommand::new("Only"))
            .separator()
            .separator();
        assert_eq!(menu.entries.len(), 2);
    }

    #[test]
    fn natural_width_is_renderer_measured_and_clamped() {
        let short = Menu::new(Space::new())
            .command(MenuCommand::new("Save").on_press(Message::Save))
            .into_content();
        let short = WidgetHarness::new(short, Size::new(800.0, 300.0));
        assert_eq!(short.bounds().width, MENU_MIN_WIDTH);

        let long = Menu::new(Space::new())
            .command(
                MenuCommand::new(
                    "A command label deliberately wider than the maximum desktop menu width",
                )
                .on_press(Message::Save),
            )
            .into_content();
        let long = WidgetHarness::new(long, Size::new(800.0, 300.0));
        assert_eq!(long.bounds().width, MENU_MAX_WIDTH);
    }

    #[test]
    fn natural_width_respects_narrow_finite_viewports() {
        let content = Menu::new(Space::new())
            .command(MenuCommand::new("A very long command label").on_press(Message::Save))
            .into_content();
        let content = WidgetHarness::new(content, Size::new(124.0, 300.0));

        assert_eq!(content.bounds().width, 124.0);
        assert!(content.bounds().width.is_finite());
    }

    #[test]
    fn canonical_list_metrics_are_fixed() {
        let content = Menu::new(Space::new())
            .command(MenuCommand::new("Save").on_press(Message::Save))
            .separator()
            .command(MenuCommand::new("Close").on_press(Message::Save))
            .into_content();
        let content = WidgetHarness::new(content, Size::new(400.0, 300.0));

        assert_eq!(content.bounds().width, MENU_MIN_WIDTH);
        assert_eq!(
            content.bounds().height,
            MENU_LIST_INSET * 2.0 + MENU_ROW_HEIGHT * 2.0 + 1.0 + MENU_SEPARATOR_MARGIN * 2.0
        );
    }

    #[test]
    fn dismiss_all_publishes_leaf_before_dismissal() {
        let content = Menu::new(Space::new())
            .on_dismiss(Message::Dismiss)
            .command(MenuCommand::new("Save").on_press(Message::Save))
            .into_content();
        let mut harness = WidgetHarness::new(content, Size::new(320.0, 120.0));
        harness.set_cursor(Point::new(12.0, 12.0));
        harness.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));
        let released = harness.update(Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        )));

        assert_eq!(released.messages, vec![Message::Save, Message::Dismiss]);
    }

    #[test]
    fn keep_open_and_absent_dismissal_publish_only_leaf() {
        for content in [
            Menu::new(Space::new())
                .on_dismiss(Message::Dismiss)
                .command(
                    MenuCommand::new("Save")
                        .on_press(Message::Save)
                        .dismiss_policy(MenuDismissPolicy::KeepOpen),
                )
                .into_content(),
            Menu::new(Space::new())
                .command(MenuCommand::new("Save").on_press(Message::Save))
                .into_content(),
        ] {
            let mut harness = WidgetHarness::new(content, Size::new(320.0, 120.0));
            harness.set_cursor(Point::new(12.0, 12.0));
            harness.update(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            )));
            let released = harness.update(Event::Mouse(mouse::Event::ButtonReleased(
                mouse::Button::Left,
            )));

            assert_eq!(released.messages, vec![Message::Save]);
        }
    }

    #[test]
    fn persistent_choice_and_transient_interaction_keep_geometry_stable() {
        let content = Menu::new(Space::new())
            .checkbox(
                MenuCheckbox::new("Pinned", CheckboxState::Checked)
                    .on_toggle(Message::Toggle)
                    .dismiss_policy(MenuDismissPolicy::KeepOpen),
            )
            .into_content();
        let mut harness = WidgetHarness::new(content, Size::new(320.0, 120.0));
        let initial = harness.bounds();
        harness.set_cursor(Point::new(12.0, 12.0));
        harness.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));
        assert_eq!(harness.bounds(), initial);
        harness.update(Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        )));
        harness.focus_next();
        assert_eq!(harness.bounds(), initial);
    }

    #[test]
    fn trailing_track_uses_the_widest_peer_measurement_for_every_row() {
        let content = Menu::new(Space::new())
            .command(
                MenuCommand::new("Save")
                    .shortcut(ShortcutBinding::primary_character('s'))
                    .on_press(Message::Save),
            )
            .command(MenuCommand::new("Close").on_press(Message::Dismiss))
            .into_content();
        let node = widget_layout(content, Size::new(320.0, 120.0));
        let column = &node.children()[0];
        let first_row = &column.children()[0].children()[0];
        let second_row = &column.children()[1].children()[0];
        let first_trailing = first_row.children().last().expect("first trailing track");
        let second_trailing = second_row.children().last().expect("second trailing track");

        assert!(first_trailing.size().width > 0.0);
        assert_eq!(first_trailing.size().width, second_trailing.size().width);
    }

    #[test]
    fn persistent_choice_and_leading_icon_use_separate_stable_tracks() {
        let content = Menu::new(Space::new())
            .radio_group(
                MenuRadioGroup::new(Some(1))
                    .option(MenuRadioOption::new(1, "Selected").icon(IconRole::ActionConfirm))
                    .option(MenuRadioOption::new(2, "Peer"))
                    .on_select(Message::Select),
            )
            .into_content();
        let node = widget_layout(content, Size::new(320.0, 120.0));
        let column = &node.children()[0];

        for row in column.children() {
            let tracks = &row.children()[0].children();
            assert_eq!(tracks[0].size().width, MENU_ICON_SIZE);
            assert_eq!(tracks[1].size().width, MENU_ICON_SIZE);
        }
    }

    #[test]
    fn truncated_highlight_forwards_private_focus_to_tooltip_only() {
        let content = Menu::new(Space::new())
            .command(
                MenuCommand::new(
                    "A renderer-measured command label that must truncate inside the menu",
                )
                .on_press(Message::Save),
            )
            .command(MenuCommand::new("Close").on_press(Message::Dismiss))
            .into_content();
        let mut harness = WidgetHarness::new(content, Size::new(220.0, 120.0));

        assert_eq!(harness.focused_count().total, 1);
        harness.focus_next();
        harness.update(Event::Window(iced::window::Event::RedrawRequested(
            iced::time::Instant::now(),
        )));
        assert!(harness.has_overlay());
        assert_eq!(harness.focused_count().total, 1);

        harness.update(key_pressed(key::Named::ArrowDown, key::Code::ArrowDown));
        harness.update(Event::Window(iced::window::Event::RedrawRequested(
            iced::time::Instant::now(),
        )));
        assert!(!harness.has_overlay());
    }

    #[test]
    fn root_composite_is_one_focus_target_with_bounded_navigation() {
        let content = Menu::new(Space::new())
            .command(MenuCommand::new("Display only"))
            .command(MenuCommand::new("Save").on_press(Message::Save))
            .separator()
            .checkbox(
                MenuCheckbox::new("Pinned", CheckboxState::Unchecked).on_toggle(Message::Toggle),
            )
            .command(
                MenuCommand::new("Disabled")
                    .disabled(true)
                    .on_press(Message::Save),
            )
            .into_content();
        let mut harness = WidgetHarness::new(content, Size::new(320.0, 240.0));

        assert_eq!(harness.focused_count().total, 1);
        harness.focus_next();
        assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(1));

        harness.update(key_pressed(key::Named::ArrowDown, key::Code::ArrowDown));
        assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(3));
        harness.update(key_pressed(key::Named::ArrowDown, key::Code::ArrowDown));
        assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(3));

        harness.update(key_pressed(key::Named::Home, key::Code::Home));
        assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(1));
        harness.update(key_pressed(key::Named::End, key::Code::End));
        assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(3));
    }

    #[test]
    fn highlighted_row_stays_visible_in_the_popover_owned_scroll_viewport() {
        let mut menu = Menu::new(Space::new().width(40).height(24)).open(true);
        for index in 0..20 {
            menu = menu
                .command(MenuCommand::new(format!("Command {index:02}")).on_press(Message::Save));
        }
        let mut harness = WidgetHarness::new(menu.into(), Size::new(260.0, 120.0));

        assert_eq!(harness.overlay_scroll_offsets(), vec![iced::Vector::ZERO]);

        let end = harness
            .update_overlay(key_pressed(key::Named::End, key::Code::End))
            .expect("open Menu overlay");
        assert!(end.captured);
        let end_offsets = harness.overlay_scroll_offsets();
        assert_eq!(end_offsets.len(), 1);
        assert!(end_offsets[0].y > 0.0);

        let home = harness
            .update_overlay(key_pressed(key::Named::Home, key::Code::Home))
            .expect("open Menu overlay");
        assert!(home.captured);
        let home_offsets = harness.overlay_scroll_offsets();
        assert_eq!(home_offsets.len(), 1);
        assert!(home_offsets[0].y <= MENU_LIST_INSET);
    }

    #[test]
    fn highlighted_capable_row_reconciles_by_label_after_reorder() {
        let content = Menu::new(Space::new())
            .command(MenuCommand::new("Save").on_press(Message::Save))
            .command(MenuCommand::new("Close").on_press(Message::Dismiss))
            .into_content();
        let mut harness = WidgetHarness::new(content, Size::new(320.0, 120.0));
        harness.focus_next();
        harness.update(key_pressed(key::Named::ArrowDown, key::Code::ArrowDown));
        assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(1));

        harness.replace(
            Menu::new(Space::new())
                .command(MenuCommand::new("Close").on_press(Message::Dismiss))
                .command(MenuCommand::new("Save").on_press(Message::Save))
                .into_content(),
        );

        assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(0));
    }

    #[test]
    fn parked_pointer_does_not_reset_keyboard_navigation() {
        let content = Menu::new(Space::new())
            .command(MenuCommand::new("One").on_press(Message::Save))
            .command(MenuCommand::new("Two").on_press(Message::Save))
            .command(MenuCommand::new("Three").on_press(Message::Save))
            .into_content();
        let mut harness = WidgetHarness::new(content, Size::new(320.0, 140.0));
        harness.focus_next();
        let parked = Point::new(12.0, 12.0);
        harness.set_cursor(parked);
        harness.update(Event::Mouse(mouse::Event::CursorMoved { position: parked }));

        harness.update(key_pressed(key::Named::ArrowDown, key::Code::ArrowDown));
        harness.update(key_pressed(key::Named::ArrowDown, key::Code::ArrowDown));

        assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(2));
    }

    #[test]
    fn right_opens_child_without_adding_focus_and_activates_its_leaf() {
        let child =
            Menu::new(Space::new()).command(MenuCommand::new("Save child").on_press(Message::Save));
        let content = Menu::new(Space::new())
            .on_dismiss(Message::Dismiss)
            .submenu(MenuSubmenu::new("More", child))
            .into_content();
        let mut harness = WidgetHarness::new(content, Size::new(640.0, 320.0));
        harness.focus_next();

        harness.update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight));

        assert!(harness.has_overlay());
        assert_eq!(harness.focused_count().total, 1);
        assert_eq!(
            harness
                .focused_overlay_count()
                .expect("child overlay")
                .total,
            0
        );
        let activated = harness
            .update_overlay(key_pressed(key::Named::Enter, key::Code::Enter))
            .expect("child overlay update");
        assert_eq!(activated.messages, vec![Message::Save, Message::Dismiss]);
    }

    #[test]
    fn nested_escape_unwinds_only_the_innermost_level() {
        let grandchild =
            Menu::new(Space::new()).command(MenuCommand::new("Save child").on_press(Message::Save));
        let child = Menu::new(Space::new()).submenu(MenuSubmenu::new("Advanced", grandchild));
        let content = Menu::new(Space::new())
            .on_dismiss(Message::Dismiss)
            .submenu(MenuSubmenu::new("More", child))
            .into_content();
        let mut harness = WidgetHarness::new(content, Size::new(800.0, 400.0));
        harness.focus_next();
        harness.update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight));
        harness
            .update_nested_overlay(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight))
            .expect("first child overlay");
        assert!(harness.nested_overlay_bounds().len() >= 2);

        let escaped = harness
            .update_nested_overlay(key_pressed(key::Named::Escape, key::Code::Escape))
            .expect("nested child overlay");

        assert!(escaped.messages.is_empty());
        assert_eq!(harness.nested_overlay_bounds().len(), 1);
        assert_eq!(harness.focused_count().total, 1);
    }

    #[test]
    fn left_closes_child_and_right_can_reopen_the_same_branch() {
        let child =
            Menu::new(Space::new()).command(MenuCommand::new("Save child").on_press(Message::Save));
        let content = Menu::new(Space::new())
            .submenu(MenuSubmenu::new("More", child))
            .into_content();
        let mut harness = WidgetHarness::new(content, Size::new(640.0, 320.0));
        harness.focus_next();
        harness.update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight));
        assert!(harness.has_overlay());

        let closed = harness
            .update_overlay(key_pressed(key::Named::ArrowLeft, key::Code::ArrowLeft))
            .expect("child overlay");
        assert!(closed.messages.is_empty());
        assert!(!harness.has_overlay());

        harness.update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight));
        assert!(harness.has_overlay());
        assert_eq!(harness.focused_count().total, 1);
    }

    #[test]
    fn pointer_intent_uses_open_delay_and_transfer_grace() {
        let child =
            Menu::new(Space::new()).command(MenuCommand::new("Save child").on_press(Message::Save));
        let content = Menu::new(Space::new())
            .submenu(MenuSubmenu::new("More", child))
            .into_content();
        let mut harness = WidgetHarness::new(content, Size::new(640.0, 320.0));
        let start = iced::time::Instant::now();
        harness.update(Event::Window(iced::window::Event::RedrawRequested(start)));
        let row = Point::new(12.0, 12.0);
        harness.set_cursor(row);
        harness.update(Event::Mouse(mouse::Event::CursorMoved { position: row }));
        harness.update(Event::Window(iced::window::Event::RedrawRequested(
            start + Duration::from_millis(199),
        )));
        assert!(!harness.has_overlay());

        harness.update(Event::Window(iced::window::Event::RedrawRequested(
            start + Duration::from_millis(200),
        )));
        assert!(harness.has_overlay());

        let child = harness.overlay_bounds().expect("child bounds");
        let child_center = child.center();
        harness.set_cursor(child_center);
        let transfer = Event::Mouse(mouse::Event::CursorMoved {
            position: child_center,
        });
        harness.update(transfer.clone());
        harness
            .update_nested_overlay(transfer)
            .expect("child receives transfer");
        assert_eq!(
            harness.state::<widget::MenuListState>().open_submenu,
            Some(0)
        );
        harness.update(Event::Window(iced::window::Event::RedrawRequested(
            start + Duration::from_millis(500),
        )));
        assert!(harness.has_overlay());

        let outside = Point::new(400.0, 200.0);
        harness.set_cursor(outside);
        harness.update(Event::Mouse(mouse::Event::CursorMoved {
            position: outside,
        }));
        harness.update(Event::Window(iced::window::Event::RedrawRequested(
            start + Duration::from_millis(799),
        )));
        assert!(harness.has_overlay());
        harness.update(Event::Window(iced::window::Event::RedrawRequested(
            start + Duration::from_millis(800),
        )));
        assert!(!harness.has_overlay());
    }

    #[test]
    fn root_composite_activation_preserves_leaf_then_dismiss_order() {
        let content = Menu::new(Space::new())
            .on_dismiss(Message::Dismiss)
            .command(MenuCommand::new("Display only"))
            .command(MenuCommand::new("Save").on_press(Message::Save))
            .into_content();
        let mut harness = WidgetHarness::new(content, Size::new(320.0, 120.0));
        harness.focus_next();

        let activated = harness.update(key_pressed(key::Named::Enter, key::Code::Enter));

        assert_eq!(activated.messages, vec![Message::Save, Message::Dismiss]);
    }

    #[test]
    fn keyboard_activation_publishes_controlled_checkbox_and_radio_values() {
        let checkbox = Menu::new(Space::new())
            .checkbox(
                MenuCheckbox::new("Pinned", CheckboxState::Unchecked)
                    .on_toggle(Message::Toggle)
                    .dismiss_policy(MenuDismissPolicy::KeepOpen),
            )
            .into_content();
        let mut checkbox = WidgetHarness::new(checkbox, Size::new(320.0, 120.0));
        checkbox.focus_next();
        let toggled = checkbox.update(key_pressed(key::Named::Enter, key::Code::Enter));
        assert_eq!(
            toggled.messages,
            vec![Message::Toggle(CheckboxState::Checked)]
        );

        let radio = Menu::new(Space::new())
            .on_dismiss(Message::Dismiss)
            .radio_group(
                MenuRadioGroup::new(Some(1))
                    .option(MenuRadioOption::new(1, "One"))
                    .option(MenuRadioOption::new(2, "Two"))
                    .on_select(Message::Select),
            )
            .into_content();
        let mut radio = WidgetHarness::new(radio, Size::new(320.0, 120.0));
        radio.focus_next();
        assert_eq!(radio.state::<widget::MenuListState>().highlight, Some(1));
        let selected = radio.update(key_pressed(key::Named::Space, key::Code::Space));
        assert_eq!(
            selected.messages,
            vec![Message::Select(2), Message::Dismiss]
        );
    }

    #[test]
    fn touch_activation_publishes_once() {
        let content = Menu::new(Space::new())
            .on_dismiss(Message::Dismiss)
            .command(MenuCommand::new("Save").on_press(Message::Save))
            .into_content();
        let mut harness = WidgetHarness::new(content, Size::new(320.0, 120.0));
        let position = Point::new(12.0, 12.0);
        let finger = touch::Finger(7);
        let pressed = harness.update(Event::Touch(touch::Event::FingerPressed {
            id: finger,
            position,
        }));
        assert!(pressed.messages.is_empty());
        let released = harness.update(Event::Touch(touch::Event::FingerLifted {
            id: finger,
            position,
        }));

        assert_eq!(released.messages, vec![Message::Save, Message::Dismiss]);
    }

    #[test]
    fn outside_press_requests_exactly_one_root_dismissal() {
        let menu = Menu::new(Space::new().width(40).height(24))
            .open(true)
            .on_dismiss(Message::Dismiss)
            .command(MenuCommand::new("Save").on_press(Message::Save));
        let mut harness = WidgetHarness::new(menu.into(), Size::new(320.0, 200.0));
        harness.set_cursor(Point::new(319.0, 199.0));

        let dismissed = harness
            .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            )))
            .expect("open Menu overlay");

        assert_eq!(dismissed.messages, vec![Message::Dismiss]);
        assert!(dismissed.captured);
    }

    #[test]
    fn typeahead_wraps_search_and_skips_ineligible_rows() {
        let content = Menu::new(Space::new())
            .command(MenuCommand::new("Placeholder"))
            .command(MenuCommand::new("Save").on_press(Message::Save))
            .separator()
            .checkbox(
                MenuCheckbox::new("Pinned", CheckboxState::Unchecked).on_toggle(Message::Toggle),
            )
            .into_content();
        let mut harness = WidgetHarness::new(content, Size::new(320.0, 160.0));
        harness.focus_next();
        harness.update(Event::Window(iced::window::Event::RedrawRequested(
            iced::time::Instant::now(),
        )));

        harness.update(text_key("p", key::Code::KeyP));

        assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(3));
    }

    #[test]
    fn typeahead_keeps_the_prefix_through_700ms_and_resets_afterward() {
        let content = Menu::new(Space::new())
            .command(MenuCommand::new("Save").on_press(Message::Save))
            .command(MenuCommand::new("Print").on_press(Message::Save))
            .command(MenuCommand::new("Pin").on_press(Message::Save))
            .into_content();
        let mut harness = WidgetHarness::new(content, Size::new(320.0, 140.0));
        harness.focus_next();
        let start = iced::time::Instant::now();
        harness.update(Event::Window(iced::window::Event::RedrawRequested(start)));

        harness.update(text_key("p", key::Code::KeyP));
        assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(1));
        harness.update(Event::Window(iced::window::Event::RedrawRequested(
            start + Duration::from_millis(700),
        )));
        harness.update(text_key("i", key::Code::KeyI));
        assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(2));

        harness.update(Event::Window(iced::window::Event::RedrawRequested(
            start + Duration::from_millis(1_401),
        )));
        harness.update(text_key("s", key::Code::KeyS));
        assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(0));
    }

    fn key_pressed(named: key::Named, code: key::Code) -> Event {
        let key = keyboard::Key::Named(named);
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: key::Physical::Code(code),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::NONE,
            text: None,
            repeat: false,
        })
    }

    fn text_key(value: &str, code: key::Code) -> Event {
        let key = keyboard::Key::Character(value.into());
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: key::Physical::Code(code),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::NONE,
            text: Some(value.into()),
            repeat: false,
        })
    }
}
