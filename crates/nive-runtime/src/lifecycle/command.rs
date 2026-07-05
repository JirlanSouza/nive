use iced::{window, Task};

use crate::UserFacingError;

#[derive(Debug)]
pub enum CloseDecision<M> {
    Close,
    Defer(Task<M>),
    Cancel,
}

#[derive(Debug)]
pub enum ExitDecision<M> {
    Exit,
    Defer(Task<M>),
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowCommand<K> {
    Open(K),
    Close(window::Id),
    CloseAllKind(K),
    Focus(window::Id),
    Replace { current: window::Id, next: K },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandRejected<K> {
    pub command: WindowCommand<K>,
    pub reason: CommandRejectionReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandRejectionReason {
    MissingWindowSpec,
    MissingWindow,
    InvalidState,
    Exiting,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformError {
    pub operation: &'static str,
    pub error: UserFacingError,
}
