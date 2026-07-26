use std::borrow::Cow;

use nive_core::{Action, ActionId, ShortcutBinding};

use super::{
    Menu, MenuCheckbox, MenuCommand, MenuDismissPolicy, MenuRadioGroup, MenuRadioOption,
    MenuSubmenu,
};
use crate::widgets::controls::CheckboxState;
use crate::IconRef;

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

    pub fn icon(mut self, icon: impl Into<IconRef>) -> Self {
        self.icon = Some(icon.into());
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

    pub fn icon(mut self, icon: impl Into<IconRef>) -> Self {
        self.icon = Some(icon.into());
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

    pub fn icon(mut self, icon: impl Into<IconRef>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
