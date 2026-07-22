#![allow(refining_impl_trait_internal)]
mod support;

use std::time::Duration;

use super::shortcuts::{devtools_toggle_from_event, shortcut_message_from_event};
use super::*;
use crate::application::update::RuntimeCommand;
use crate::application::{ApplicationConfig, CommandRejectionReason, WindowCommand};
use crate::{
    Action, ActionMap, NamedShortcutKey, ShortcutBinding, ShortcutMap, ShortcutModifiers,
    ThemePreference, Toast, WindowHandle, WindowSession,
};
use nive_ui::theme::{testing::ThemeTestGuard, Theme};
use support::*;

fn program() -> Program<TestApp> {
    plain_program::<TestApp>()
        .map(|(program, _)| program)
        .unwrap_or_else(|error| panic!("test program failed: {error}"))
}

#[cfg(feature = "devtools")]
fn plain_program<A>() -> Result<ProgramBoot<A, NoProbe>>
where
    A: Application,
{
    Program::<A>::new(A::config(), None)
}

#[cfg(not(feature = "devtools"))]
fn plain_program<A>() -> Result<ProgramBoot<A, NoProbe>>
where
    A: Application,
{
    Program::<A>::new(A::config())
}

#[cfg(feature = "devtools")]
fn plain_program_with_config<A>(
    config: ApplicationConfig<A::Window, A::Bootstrap>,
) -> Result<ProgramBoot<A, NoProbe>>
where
    A: Application,
{
    Program::<A>::new(config, None)
}

#[cfg(not(feature = "devtools"))]
fn plain_program_with_config<A>(
    config: ApplicationConfig<A::Window, A::Bootstrap>,
) -> Result<ProgramBoot<A, NoProbe>>
where
    A: Application,
{
    Program::<A>::new(config)
}

#[cfg(feature = "devtools")]
fn devtools_program(config: DevtoolsConfig) -> Program<TestApp> {
    Program::new(
        TestApp::config(),
        Some(DevtoolsRuntime::<TestApp>::new(config)),
    )
    .map(|(program, _)| program)
    .unwrap_or_else(|error| panic!("test program failed: {error}"))
}

fn key_pressed(
    key: keyboard::Key,
    modifiers: keyboard::Modifiers,
    repeat: bool,
) -> keyboard::Event {
    use iced::keyboard::key::{Code, Physical};
    use iced::keyboard::Location;

    keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: Physical::Code(Code::KeyK),
        location: Location::Standard,
        modifiers,
        text: None,
        repeat,
    }
}

fn main_window_id(program: &Program<TestApp>) -> window::Id {
    program
        .core
        .registry
        .all(TestWindow::Main)
        .map(|handle| handle.id)
        .next()
        .unwrap_or_else(window::Id::unique)
}

fn open_main_window(program: &mut Program<TestApp>) -> window::Id {
    let main_id = main_window_id(program);
    let _handle = program.core.registry.mark_opened(main_id);
    main_id
}

mod focus;

mod devtools;
mod replace;
mod routing;
mod session;
mod shortcuts;
mod toast;
mod windows;
