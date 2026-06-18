#![allow(dead_code)]

use std::marker::PhantomData;

pub struct AsyncState<T>(pub PhantomData<T>);
pub struct OperationState<T>(pub PhantomData<T>);

pub struct DevtoolFixture<T> {
    pub value: T,
}

pub struct DevtoolStateSnapshot;
pub struct DevtoolCommand;

pub struct DevtoolCommandResult;

impl DevtoolCommandResult {
    pub fn not_handled() -> Self {
        Self
    }

    pub fn handled(&self) -> bool {
        false
    }
}

pub trait DevtoolStateCatalog {
    fn devtool_collect(&self, _scope: &str, _snapshot: &mut DevtoolStateSnapshot) {}

    fn devtool_apply(
        &mut self,
        _scope: &str,
        _command: &DevtoolCommand,
    ) -> DevtoolCommandResult {
        DevtoolCommandResult::not_handled()
    }
}

pub trait DevtoolStateHost {
    type State: DevtoolStateCatalog;

    fn devtool_state(&self) -> &Self::State;
    fn devtool_state_mut(&mut self) -> &mut Self::State;
}

pub fn join_path(scope: &str, field: &str) -> String {
    format!("{scope}.{field}")
}

pub fn collect_async_state_field<T>(
    _state: &AsyncState<T>,
    _path: &str,
    _label: &str,
    _snapshot: &mut DevtoolStateSnapshot,
    _fixtures: Vec<DevtoolFixture<T>>,
) {
}

pub fn apply_async_state_field<T>(
    _state: &mut AsyncState<T>,
    _path: &str,
    _command: &DevtoolCommand,
    _fixtures: impl FnOnce() -> Vec<DevtoolFixture<T>>,
) -> DevtoolCommandResult {
    DevtoolCommandResult::not_handled()
}

pub fn collect_operation_state_field<T>(
    _state: &OperationState<T>,
    _path: &str,
    _label: &str,
    _snapshot: &mut DevtoolStateSnapshot,
) where
    T: DevtoolOperationContext,
{
}

pub fn apply_operation_state_field<T>(
    _state: &mut OperationState<T>,
    _path: &str,
    _command: &DevtoolCommand,
) -> DevtoolCommandResult
where
    T: DevtoolOperationContext,
{
    DevtoolCommandResult::not_handled()
}

pub struct DevtoolFieldSchema;
pub struct DevtoolInputValues<'a>(pub PhantomData<&'a ()>);

pub trait DevtoolInputField: Sized {
    fn devtool_schema(_name: &str) -> DevtoolFieldSchema;
    fn devtool_build(_name: &str, _inputs: &DevtoolInputValues<'_>) -> Result<Self, String>;
}

impl DevtoolInputField for String {
    fn devtool_schema(_name: &str) -> DevtoolFieldSchema {
        DevtoolFieldSchema
    }

    fn devtool_build(_name: &str, _inputs: &DevtoolInputValues<'_>) -> Result<Self, String> {
        Ok(String::new())
    }
}

pub trait DevtoolOperationContext: Sized {
    fn devtool_fields() -> Vec<DevtoolFieldSchema>;
    fn devtool_build(inputs: &DevtoolInputValues<'_>) -> Result<Self, String>;
}

pub trait Devtools {}

#[derive(Clone, Copy)]
pub struct ProbeMeta;

impl ProbeMeta {
    pub const fn new(
        _key: &'static str,
        _short_key: &'static str,
        _summary: &'static str,
        _scope: ProbeErrorScope,
    ) -> Self {
        Self
    }
}

#[derive(Clone, Copy)]
pub enum ProbeErrorScope {
    Bootstrap,
    Custom(&'static str),
}

pub trait ProbeCatalogEntry: Sized + 'static {
    const ALL: &'static [Self];

    fn meta(self) -> ProbeMeta;
}
