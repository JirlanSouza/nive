use nive::prelude::ui::{Operation, Resource};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DemoProject {
    pub name: String,
    pub owner: String,
}

#[derive(Debug, Clone, Default)]
pub struct SaveInput;

#[derive(nive::Inspect)]
pub struct DevState {
    #[inspect(sample = sample_projects)]
    pub projects: Resource<Vec<DemoProject>>,
    #[inspect(input = sample_save_input)]
    pub save_operation: Operation<SaveInput>,
}

impl DevState {
    pub fn new() -> Self {
        Self {
            projects: Resource::idle(),
            save_operation: Operation::idle(),
        }
    }
}

pub fn sample_projects() -> Vec<DemoProject> {
    vec![
        DemoProject {
            name: "Atlas".to_owned(),
            owner: "Design Systems".to_owned(),
        },
        DemoProject {
            name: "Beacon".to_owned(),
            owner: "Runtime".to_owned(),
        },
        DemoProject {
            name: "Canvas".to_owned(),
            owner: "Examples".to_owned(),
        },
    ]
}

pub fn sample_save_input() -> SaveInput {
    SaveInput
}
