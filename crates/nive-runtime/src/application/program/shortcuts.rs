use iced::keyboard;
use iced::{Subscription, Task};

use crate::application::program::{
    CoreMessage, NiveMessage, ProbeCatalogEntry, Program, RuntimeMessage,
};
use crate::application::{Application, MessageSource};
use crate::input::{action_message_for_event, shortcut_message_for_event};
use crate::{ActionMap, KeyboardNavigation, ShortcutMap};

impl<A, P> Program<A, P>
where
    A: Application,
    P: ProbeCatalogEntry,
{
    pub(super) fn shortcut_subscription(&self) -> Subscription<RuntimeMessage<A, P>> {
        if self.app.is_none() {
            return Subscription::none();
        }

        keyboard::listen().map(|event| NiveMessage::Core(CoreMessage::KeyboardEvent(event)))
    }

    pub(super) fn handle_keyboard_event(
        &mut self,
        event: keyboard::Event,
    ) -> Task<RuntimeMessage<A, P>> {
        let Some(app) = self.app.as_ref() else {
            return Task::none();
        };
        let context = self.core.context();
        let actions = app.actions(context);
        let shortcuts = app.shortcuts(context);

        shortcut_message_from_event::<A, P>(&actions, &shortcuts, event)
            .map(Task::done)
            .unwrap_or_else(Task::none)
    }
}

pub(super) fn shortcut_message_from_event<A, P>(
    actions: &ActionMap<A::Message>,
    shortcuts: &ShortcutMap<A::Message>,
    event: keyboard::Event,
) -> Option<RuntimeMessage<A, P>>
where
    A: Application,
    P: ProbeCatalogEntry,
{
    if let Some(navigation) = keyboard_navigation_from_event(&event) {
        return Some(NiveMessage::Core(CoreMessage::KeyboardNavigation(
            navigation,
        )));
    }
    if is_escape_key_event(&event) {
        return None;
    }

    #[cfg(feature = "devtools")]
    if let Some(message) = devtools_toggle_from_event(event.clone()) {
        return Some(message);
    }

    action_message_for_event(actions, &event)
        .or_else(|| shortcut_message_for_event(shortcuts, &event))
        .map(|message| NiveMessage::App {
            window_id: None,
            source: MessageSource::Action,
            message,
        })
}

pub(super) fn keyboard_navigation_from_event(
    event: &keyboard::Event,
) -> Option<KeyboardNavigation> {
    crate::direction_from_keyboard_event(event).map(KeyboardNavigation::from)
}

pub(super) fn is_escape_key_event(event: &keyboard::Event) -> bool {
    matches!(
        event,
        keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Escape),
            modifiers,
            repeat: false,
            ..
        } if modifiers.is_empty()
    )
}

#[cfg(feature = "devtools")]
pub(super) fn devtools_toggle_from_event<K, M, B, P>(
    event: keyboard::Event,
) -> Option<NiveMessage<K, M, B, P>> {
    match event {
        keyboard::Event::KeyPressed { key, modifiers, .. } => {
            let is_devtools_key = matches!(
                key,
                keyboard::Key::Character(c) if c.eq_ignore_ascii_case("i")
            );
            let is_devtools_modifier = if cfg!(target_os = "macos") {
                modifiers.command() && modifiers.alt()
            } else {
                modifiers.control() && modifiers.alt()
            };
            if is_devtools_key && is_devtools_modifier {
                Some(NiveMessage::Core(CoreMessage::ToggleDevtools))
            } else {
                None
            }
        }
        _ => None,
    }
}
