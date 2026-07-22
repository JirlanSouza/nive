use super::relay::MessageRelay;
use super::render::menu_level_content;
use super::widget::MenuLevelContext;
use super::{
    Menu, MenuCheckbox, MenuCommand, MenuDismissPolicy, MenuEntry, MenuEvent, MenuRadioGroup,
    MenuRadioRow, MenuSubmenu,
};
use crate::widgets::overlays::{
    Popover, PopoverCollision, PopoverFocusPolicy, PopoverInset, PopoverPlacement,
};
use crate::Element;

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

    pub(super) fn into_element(mut self) -> Element<'a, Message> {
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

    #[cfg(test)]
    pub(super) fn into_content(mut self) -> Element<'a, Message> {
        self.take_content()
    }

    #[cfg(test)]
    pub(super) fn take_content(&mut self) -> Element<'a, Message> {
        self.take_content_with_context(MenuLevelContext::root())
    }

    pub(super) fn take_content_with_context(
        &mut self,
        context: MenuLevelContext,
    ) -> Element<'a, Message> {
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
