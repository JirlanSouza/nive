use super::helpers::{label_from_field_name, placeholder_for_field_name};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DevtoolStateSnapshot {
    pub resources: Vec<DevtoolResourceView>,
    pub operations: Vec<DevtoolOperationView>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevtoolResourceView {
    pub path: String,
    pub label: String,
    pub status: DevtoolAsyncStatus,
    pub fixtures: Vec<DevtoolFixtureView>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevtoolAsyncStatus {
    Idle,
    Loading { has_value: bool },
    Loaded,
    Failed { has_value: bool, summary: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevtoolOperationView {
    pub path: String,
    pub label: String,
    pub status: DevtoolOperationStatus,
    pub fields: Vec<DevtoolFieldSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevtoolOperationStatus {
    Idle,
    Running,
    Failed { summary: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevtoolFixtureView {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevtoolFixture<T> {
    pub id: String,
    pub label: String,
    pub value: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevtoolFieldSchema {
    pub name: String,
    pub label: String,
    pub placeholder: String,
    pub required: bool,
}

impl DevtoolFieldSchema {
    pub fn text(name: &'static str, required: bool) -> Self {
        Self {
            name: name.to_string(),
            label: label_from_field_name(name),
            placeholder: placeholder_for_field_name(name),
            required,
        }
    }
}
