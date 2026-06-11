use crate::{AsyncState, OperationState, RequestId, UserFacingError};

use super::{
    command_input::DevtoolOperationContext, DevtoolAsyncStatus, DevtoolCommand,
    DevtoolCommandResult, DevtoolFixture, DevtoolFixtureView, DevtoolInputValues,
    DevtoolOperationStatus, DevtoolOperationView, DevtoolResourceView, DevtoolStateSnapshot,
};

const DEVTOOLS_DEFAULT_ERROR: &str = "Devtools injected failure";

pub trait DevtoolStateCatalog {
    fn devtool_collect(&self, _scope: &str, _snapshot: &mut DevtoolStateSnapshot) {}

    fn devtool_apply(&mut self, _scope: &str, _command: &DevtoolCommand) -> DevtoolCommandResult {
        DevtoolCommandResult::not_handled()
    }

    fn devtool_snapshot(&self, scope: &str) -> DevtoolStateSnapshot {
        let mut snapshot = DevtoolStateSnapshot::default();
        self.devtool_collect(scope, &mut snapshot);
        snapshot
    }
}

pub trait DevtoolStateHost {
    type State: DevtoolStateCatalog;

    fn devtool_state(&self) -> &Self::State;

    fn devtool_state_mut(&mut self) -> &mut Self::State;

    fn devtool_snapshot(&self, scope: &str) -> DevtoolStateSnapshot {
        self.devtool_state().devtool_snapshot(scope)
    }

    fn devtool_apply(&mut self, scope: &str, command: &DevtoolCommand) -> DevtoolCommandResult {
        self.devtool_state_mut().devtool_apply(scope, command)
    }
}

pub trait DevtoolsApp {
    fn devtools_snapshot(&self) -> DevtoolStateSnapshot;

    fn apply_devtools_command(&mut self, command: &DevtoolCommand) -> DevtoolCommandResult;
}

pub trait DevtoolValue: Clone {
    fn devtool_fixtures(path: &str) -> Vec<DevtoolFixture<Self>>;
}

pub trait DevtoolStateField {
    fn devtool_collect_field(&self, path: &str, label: &str, snapshot: &mut DevtoolStateSnapshot);

    fn devtool_apply_field(&mut self, path: &str, command: &DevtoolCommand)
        -> DevtoolCommandResult;
}

impl<T> DevtoolStateField for AsyncState<T>
where
    T: DevtoolValue,
{
    fn devtool_collect_field(&self, path: &str, label: &str, snapshot: &mut DevtoolStateSnapshot) {
        snapshot.resources.push(DevtoolResourceView {
            path: path.to_string(),
            label: label.to_string(),
            status: async_status(self),
            fixtures: T::devtool_fixtures(path)
                .into_iter()
                .map(|fixture| DevtoolFixtureView {
                    id: fixture.id,
                    label: fixture.label,
                })
                .collect(),
        });
    }

    fn devtool_apply_field(
        &mut self,
        path: &str,
        command: &DevtoolCommand,
    ) -> DevtoolCommandResult {
        if command.path() != path {
            return DevtoolCommandResult::not_handled();
        }

        match command {
            DevtoolCommand::SetResourceIdle { .. } => {
                self.set_idle();
                DevtoolCommandResult::changed()
            }
            DevtoolCommand::SetResourceLoading { preserve_value, .. } => {
                if *preserve_value {
                    self.set_refreshing();
                } else {
                    self.set_loading();
                }
                DevtoolCommandResult::changed()
            }
            DevtoolCommand::SetResourceFailed {
                message,
                preserve_value,
                ..
            } => {
                let error = devtool_error(message);
                if *preserve_value {
                    self.set_failed(error);
                } else {
                    self.set_failed_empty(error);
                }
                DevtoolCommandResult::changed()
            }
            DevtoolCommand::SetResourceLoadedFixture { fixture_id, .. } => {
                let Some(fixture) = T::devtool_fixtures(path)
                    .into_iter()
                    .find(|fixture| fixture.id == *fixture_id)
                else {
                    return DevtoolCommandResult::failed(format!(
                        "Unknown fixture `{fixture_id}` for `{path}`"
                    ));
                };

                self.set_loaded(fixture.value);
                DevtoolCommandResult::changed()
            }
            DevtoolCommand::DismissResourceError { .. } => {
                self.dismiss_error();
                DevtoolCommandResult::changed()
            }
            DevtoolCommand::SetOperationIdle { .. }
            | DevtoolCommand::SetOperationRunning { .. }
            | DevtoolCommand::SetOperationFailed { .. } => {
                DevtoolCommandResult::failed(format!("`{path}` is a resource, not an operation"))
            }
        }
    }
}

impl<C> DevtoolStateField for OperationState<C>
where
    C: DevtoolOperationContext,
{
    fn devtool_collect_field(&self, path: &str, label: &str, snapshot: &mut DevtoolStateSnapshot) {
        snapshot.operations.push(DevtoolOperationView {
            path: path.to_string(),
            label: label.to_string(),
            status: operation_status(self),
            fields: C::devtool_fields(),
        });
    }

    fn devtool_apply_field(
        &mut self,
        path: &str,
        command: &DevtoolCommand,
    ) -> DevtoolCommandResult {
        if command.path() != path {
            return DevtoolCommandResult::not_handled();
        }

        match command {
            DevtoolCommand::SetOperationIdle { .. } => {
                self.clear();
                DevtoolCommandResult::changed()
            }
            DevtoolCommand::SetOperationRunning { input, .. } => {
                let inputs = DevtoolInputValues::new(input);
                let Ok(context) = C::devtool_build(&inputs) else {
                    return DevtoolCommandResult::failed(format!("Invalid inputs for `{path}`"));
                };

                self.start(devtools_request_id(), context);
                DevtoolCommandResult::changed()
            }
            DevtoolCommand::SetOperationFailed { input, message, .. } => {
                let inputs = DevtoolInputValues::new(input);
                let Ok(context) = C::devtool_build(&inputs) else {
                    return DevtoolCommandResult::failed(format!("Invalid inputs for `{path}`"));
                };

                let request_id = devtools_request_id();
                self.start(request_id, context);
                self.fail(request_id, devtool_error(message));
                DevtoolCommandResult::changed()
            }
            DevtoolCommand::SetResourceIdle { .. }
            | DevtoolCommand::SetResourceLoading { .. }
            | DevtoolCommand::SetResourceFailed { .. }
            | DevtoolCommand::SetResourceLoadedFixture { .. }
            | DevtoolCommand::DismissResourceError { .. } => {
                DevtoolCommandResult::failed(format!("`{path}` is an operation, not a resource"))
            }
        }
    }
}

fn devtools_request_id() -> RequestId {
    RequestId::new(1_000_000)
}

fn devtool_error(message: &str) -> UserFacingError {
    let message = message.trim();
    if message.is_empty() {
        UserFacingError::devtools(DEVTOOLS_DEFAULT_ERROR)
    } else {
        UserFacingError::devtools(message)
    }
}

fn async_status<T>(state: &AsyncState<T>) -> DevtoolAsyncStatus {
    if let Some(error) = state.error() {
        return DevtoolAsyncStatus::Failed {
            has_value: state.value().is_some(),
            summary: error.summary().to_string(),
        };
    }

    if state.is_loading() {
        return DevtoolAsyncStatus::Loading {
            has_value: state.value().is_some(),
        };
    }

    if state.value().is_some() {
        DevtoolAsyncStatus::Loaded
    } else {
        DevtoolAsyncStatus::Idle
    }
}

fn operation_status<C>(state: &OperationState<C>) -> DevtoolOperationStatus {
    if let Some(error) = state.error() {
        DevtoolOperationStatus::Failed {
            summary: error.summary().to_string(),
        }
    } else if state.is_running() {
        DevtoolOperationStatus::Running
    } else {
        DevtoolOperationStatus::Idle
    }
}
